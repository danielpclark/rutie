#!/usr/bin/env python3
"""Check src/rubysys against the Ruby found on PATH (or $RUBY).

For every `pub fn` and `pub static` in an `extern "C"` block of
src/rubysys/*.rs that is active for this Ruby (its `#[cfg(ruby_*)]`
attributes are evaluated against the Ruby version), the script checks that:

  - the name is declared in Ruby's public headers,
  - the Rust return type agrees with the C one (VALUE, int, void, ...),
  - libruby exports the symbol (Linux, with `nm`; skipped elsewhere).

A declaration with a `// LINKER CANNOT FIND` note is skipped, and one noted
`// NOT IN PUBLIC HEADERS` is only checked against the exports. It exits with
status 1 when a check fails, so CI catches drift when a Ruby is added.

Usage: python3 ci/check_rubysys.py
"""

import glob
import os
import re
import subprocess
import sys

RUBY = os.environ.get("RUBY", "ruby")


def rbconfig(key):
    return subprocess.check_output(
        [RUBY, "-e", "print RbConfig::CONFIG[ARGV[0]].to_s", key], text=True
    )


def ruby_version():
    major, minor = rbconfig("ruby_version").split(".")[:2]
    return int(major), int(minor)


def cfg_active(cfg, version):
    """Evaluate a cfg predicate made of ruby_M_m, ruby_gte_M_m, not, all, any."""
    cfg = cfg.strip()
    m = re.fullmatch(r"(not|all|any)\((.*)\)", cfg, re.S)
    if m:
        parts = split_args(m.group(2))
        values = [cfg_active(p, version) for p in parts]
        return {"not": lambda v: not v[0], "all": all, "any": any}[m.group(1)](values)
    m = re.fullmatch(r"ruby_gte_(\d+)_(\d+)", cfg)
    if m:
        return version >= (int(m.group(1)), int(m.group(2)))
    m = re.fullmatch(r"ruby_(\d+)_(\d+)", cfg)
    if m:
        return version == (int(m.group(1)), int(m.group(2)))
    # Platform cfgs, for the host the script runs on.
    windows = sys.platform.startswith("win")
    if cfg == "windows":
        return windows
    if cfg == "unix":
        return not windows
    m = re.fullmatch(r'target_os\s*=\s*"(\w+)"', cfg)
    if m:
        host = {"linux": "linux", "darwin": "macos"}.get(sys.platform, "windows" if windows else sys.platform)
        return m.group(1) == host
    # Anything else (target_pointer_width, rutie_dllimport, ...): active.
    return not cfg.startswith("rutie_dllimport")


def split_args(text):
    parts, depth, current = [], 0, ""
    for ch in text:
        if ch == "," and depth == 0:
            parts.append(current)
            current = ""
            continue
        depth += ch == "("
        depth -= ch == ")"
        current += ch
    if current.strip():
        parts.append(current)
    return parts


def rust_declarations(version):
    """Yields (file, name, kind, rust_return, internal) for active declarations."""
    for path in sorted(glob.glob("src/rubysys/*.rs")):
        text = open(path).read()
        for block in re.finditer(r'((?:#\[[^\n]*\]\n)*)extern "C" \{(.*?)\n\}', text, re.S):
            block_cfgs = re.findall(r"#\[cfg\((.*)\)\]", block.group(1))
            if not all(cfg_active(c, version) for c in block_cfgs):
                continue
            body = block.group(2)
            # Attributes and notes directly above each item.
            for m in re.finditer(
                r"((?:[ \t]*(?://[^\n]*|#\[[^\n]*\])\n)*)[ \t]*pub (fn|static) (\w+)"
                r"(?:\s*\(.*?\)\s*(?:->\s*([^;]+))?|:\s*([^;]+));",
                body,
                re.S,
            ):
                preamble, kind, name = m.group(1), m.group(2), m.group(3)
                if "LINKER CANNOT FIND" in preamble:
                    continue
                internal = "NOT IN PUBLIC HEADERS" in preamble
                cfgs = re.findall(r"#\[cfg\((.*)\)\]", preamble)
                if not all(cfg_active(c, version) for c in cfgs):
                    continue
                ret = (m.group(4) or "()").strip() if kind == "fn" else m.group(5).strip()
                yield path, name, kind, " ".join(ret.split()), internal


def header_declarations():
    hdrdir = rbconfig("rubyhdrdir")
    archdir = rbconfig("rubyarchhdrdir")
    text = ""
    for root in {hdrdir, archdir}:
        for path in glob.glob(os.path.join(root, "**", "*.h"), recursive=True):
            if "mjit_min" in path or "rb_rjit" in path:
                continue
            text += open(path, errors="ignore").read() + "\n"
    text = re.sub(r"/\*.*?\*/", " ", text, flags=re.S)
    text = re.sub(r"//[^\n]*", " ", text)
    text = re.sub(r"\\\n", " ", text)

    # Ruby 3.0 wraps some prototypes: `NORETURN(void rb_raise(...));`.
    text = re.sub(r"\bDEPRECATED_BY\s*\(\s*\w+\s*,", "(", text)
    text = re.sub(r"\b(?:NORETURN|DEPRECATED)\s*\(", "(", text)
    text = re.sub(r"\bMJIT_STATIC\b", "", text)

    functions, variables = {}, set()
    for m in re.finditer(
        r"(?:^|[;}(\n])\s*((?:[A-Za-z_]\w*[\s\*]+)+?)\b((?:rb|ruby)_\w+)\s*\(([^;{}]*?)\)\s*\)?\s*"
        r"(?:(?:RBIMPL_ATTR_\w+|__attribute__)\s*\((?:[^()]|\([^()]*\))*\)\s*)*[;{]",
        text,
    ):
        ret = " ".join(m.group(1).split())
        # A call in an inline function body, not a prototype.
        if re.search(r"\b(return|else|if|case|do|goto)\b", ret):
            continue
        functions.setdefault(m.group(2), ret)
    # Variables are `rb_*` or `ruby_*`, and a few `RUBY_*` (`RUBY_IO_BUFFER_PAGE_SIZE`);
    # some are structs (`RUBY_EXTERN const struct T name;`).
    for m in re.finditer(r"\b(?:RUBY_EXTERN|extern)\s+(?:const\s+)?(?:struct\s+)?\w+[\s\*]+((?:rb|ruby|RUBY)_\w+(?:\s*,\s*\*?\s*(?:rb|ruby|RUBY)_\w+)*)\s*(?:\[[^\]]*\])?\s*;", text):
        variables.update(n.strip(" *") for n in m.group(1).split(","))
    return functions, variables


def c_return_type(declaration):
    words = [
        w
        for w in declaration.replace("*", " * ").split()
        if not re.match(r"(RBIMPL_\w+|RUBY_EXTERN|extern|static|inline|NORETURN|PRINTF_ARGS|"
                        r"DEPRECATED\w*|RUBY_SYMBOL_EXPORT_\w+|const|volatile|struct|enum|"
                        r"RUBY_ATTR_\w+|CONSTFUNC|PUREFUNC|ERRORFUNC|WARNINGFUNC)$", w)
    ]
    pointer = "*" in words
    base = [w for w in words if w != "*"]
    return (base[-1] if base else ""), pointer


EXPECTED = {
    "Value": {"VALUE"},
    "Id": {"ID"},
    "()": {"void"},
    "!": {"void"},
    "c_int": {"int"},
    "bool": {"bool", "_Bool"},
    "c_long": {"long"},
    "c_ulong": {"long"},
    "c_uint": {"int", "unsigned"},
    "size_t": {"size_t", "st_index_t"},
    "c_double": {"double"},
    "SignedValue": {"SIGNED_VALUE", "long"},
    "EncodingIndex": {"int"},
    "Argc": {"int"},
    "RawFd": {"int"},
    "st_retval": {"int"},
}


def return_type_ok(rust, declaration):
    # Ignoring a C return value is harmless.
    if rust == "()":
        return True
    base, pointer = c_return_type(declaration)
    if rust.startswith("*") or rust in ("EncodingType", "VmPointer") or rust.startswith("Option<"):
        return pointer
    if pointer:
        return False
    return base in EXPECTED.get(rust, {base})


def exported_symbols():
    if not sys.platform.startswith("linux"):
        return None
    libdir = rbconfig("libdir")
    so = rbconfig("LIBRUBY_SO") or rbconfig("LIBRUBY")
    path = os.path.join(libdir, so)
    if not os.path.exists(path):
        return None
    out = subprocess.run(["nm", "-D", "--defined-only", path], capture_output=True, text=True).stdout
    return {line.split()[-1] for line in out.splitlines() if line.strip()}


def main():
    version = ruby_version()
    functions, variables = header_declarations()
    exports = exported_symbols()
    failures = 0
    checked = 0

    for path, name, kind, ret, internal in rust_declarations(version):
        checked += 1
        problems = []
        if internal:
            pass
        elif kind == "fn":
            if name not in functions:
                problems.append("not declared in the headers")
            elif not return_type_ok(ret, functions[name]):
                problems.append(f"returns {ret} in Rust but `{functions[name]}` in C")
        elif name not in variables and name not in functions:
            problems.append("not declared in the headers")
        if exports is not None and name not in exports:
            problems.append("not exported by libruby")
        for problem in problems:
            failures += 1
            print(f"{path}: {name}: {problem}")

    where = "" if exports is not None else " (exports not checked on this platform)"
    print(f"Ruby {version[0]}.{version[1]}: checked {checked} declarations, {failures} problems{where}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
