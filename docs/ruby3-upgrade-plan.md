# Ruby 3 upgrade plan (Rutie 0.11 → 0.13)

This plan takes Rutie from Ruby 2 (0.10) to Ruby 3 in three releases. Each
Rutie minor version supports exactly three Ruby minor versions:

| Rutie | Ruby | Change from the previous line |
|---|---|---|
| 0.10 | 2.5, 2.6, 2.7 | current line (see `docs/ruby2-full-support-plan.md`) |
| 0.11 | 3.0, 3.1, 3.2 | drops Ruby 2, first Ruby 3 release |
| 0.12 | 3.1, 3.2, 3.3 | drops 3.0, adds 3.3 |
| 0.13 | 3.2, 3.3, 3.4 | drops 3.1, adds 3.4 |

It is P9 of the Ruby 2 plan: the Ruby 2 packages (P0–P8) finish first, so
0.10 is feature-complete and fully tested on Ruby 2 before any Ruby 3 work
lands. The Ruby 2 plan's ground rules (hand-maintained FFI, the
`rubysys` → `binding` → `class` → `dsl` layering, a running and asserting
doctest for every public item, unit tests via `on_ruby_thread`, rule 8 on
safe variants, rule 9 on SemVer) carry over unchanged. Rule 9 is why each
line below is a MINOR release: 0.11, 0.12 and 0.13 may break the public API
(`src/class/*`, `src/helpers/*`), while their PATCH releases may change only
`src/rubysys/*`, `src/binding/*` and `src/util.rs`.

**How the facts below were checked.** Items marked *(headers)* were compared
in the installed headers of RVM's prebuilt 3.0.6, 3.1.4, 3.2.6 and 3.3.6
(`https://rvm.io/binaries/ubuntu/{22.04,24.04}/x86_64/`) and the headers of
the `v3_4_11` tag. *(exports)* means checked against the exported symbols of
those RVM builds' `libruby.so` (3.0–3.3; RVM has no 3.4 binary, so 3.4 is
still to check). *(NEWS)* means the "C API updates" or "Compatibility issues"
sections of `NEWS-3.x.0.md`. Anything marked **verify** is expected but not
yet confirmed.

---

## 1. Release and branch strategy

- **0.10.x stays on Ruby 2.** Before the Ruby 3 work starts, branch
  `0.10-stable` from the last Ruby 2 commit. Ruby 2 fixes go there and are
  released as 0.10.x; `master` moves to Ruby 3.
- **0.11 drops Ruby 2 entirely.** `build.rs` refuses (not warns about) Ruby 2
  on the 0.11+ line. Code gated on `ruby_2_*` is deleted, not kept behind cfgs.
- **Each release drops one Ruby and adds one.** Code for the dropped version
  is deleted in that release, so `rubysys` never carries more than three
  versions' worth of gates.
- **SemVer.** 0.11, 0.12 and 0.13 are breaking releases (Cargo treats 0.x
  minors as incompatible), so they can remove APIs. Anything slated for
  removal is deprecated (`#[deprecated]`) one release earlier where possible.
- **Ruby versions.** As of this plan (September 2026) 3.0, 3.1 and 3.2 are
  past upstream end of life and 3.3 and 3.4 are maintained, so 0.11 exists to
  give users of older Rubies a migration step. 0.13 is the first line that
  covers only maintained Rubies. Ruby 4.0 would be a later line (0.14) and is
  outside this plan.

## 2. Build, cfg flags and CI (every release)

- `build.rs`: extend `SUPPORTED_*` to emit `ruby_3_0` … `ruby_3_4` (exact) and
  `ruby_gte_3_1` … `ruby_gte_3_4` (cumulative) plus `rustc-check-cfg` lines,
  for the three versions the release supports. Keep exporting
  `DEP_RUBY_VERSION_MAJOR/MINOR`. Fail with a clear message on an unsupported
  Ruby. `cfg_flags_match_linked_ruby` in `src/lib.rs` is updated the same way.
- **CI matrix:** Linux and macOS × {stable, beta} × the three Rubies ×
  {dynamic, static}, as today. Ruby 3 has prebuilt binaries in
  `ruby/setup-ruby` for Linux, macOS (including arm64) and Windows, which
  replaces building Ruby with RVM (the slowest part of CI today) and makes
  the Windows rows testable. Static builds stay best-effort until they link:
  today's static rows fail with `could not find native static library
  'ruby'`, because RVM built `libruby.so` despite `--disable-shared`.
- **OpenSSL:** Ruby 3.0's `openssl` extension needs OpenSSL 1.1 (**verify**;
  3.1 and later build against OpenSSL 3). Keep the source-built OpenSSL
  1.1.1 step only while 3.0 is in the matrix, i.e. drop it in 0.12.
- **Local testing:** RVM's prebuilt binaries cover 3.0.x/3.1.x
  (`ubuntu/22.04/x86_64`) and 3.2.6/3.3.6 (`ubuntu/24.04/x86_64`). 3.4 has no
  RVM binary; build it with ruby-build, or from the `v3_4_*` tag. One
  `CARGO_TARGET_DIR=$HOME/rt-<version>/target` per Ruby, as in the Ruby 2
  recipe. Test a 3.2 built with YJIT as well (see the next item): the
  3.2.6 binaries above have no YJIT.
- **libruby and the Rust runtime:** Ruby 3.2 built with YJIT (Rust) exports
  YJIT's copy of the Rust runtime from `libruby`: `__rust_start_panic`,
  `__rust_panic_cleanup` and, when the rustc versions match, hundreds of
  `core`/`alloc` functions (3.2.11: 1719 `_R` symbols; 3.3 and 3.4 hide
  them). Any binary that meets libruby before the Rust standard library on
  its link line binds its panic runtime to YJIT's, so the first
  `resume_unwind` aborts (seen with the same rustc; a different rustc
  can crash instead). rustc
  puts a crate's own native libraries before std and an upstream crate's
  after it, so only Rutie's own unit-test binary was affected. `build.rs`
  links libruby through a generated `#[link]` attribute
  (`$OUT_DIR/link_ruby.rs`, included by `src/lib.rs` outside `cfg(test)`)
  and gives Rutie's own targets a trailing `-l` linker argument. Check with
  `nm -m <test binary> | grep "from libruby" | grep " __R"`, which must
  print nothing.

## 3. ABI differences that affect Rutie's direct struct reads

Rutie reads some Ruby structs and bit patterns directly (`src/rubysys/value.rs`,
`string.rs`, `array.rs`, `encoding.rs`, `constant.rs`). Those reads are the
riskiest part of the upgrade: a wrong layout is silent memory corruption,
not a build error. The rule from the Ruby 2 plan applies: prefer a C
function, and gate per version only where there is none.

| Item | 2.5–2.7 | 3.0 | 3.1 | 3.2 | 3.3 | 3.4 |
|---|---|---|---|---|---|---|
| `Qnil` / `Qundef` *(headers)* | `0x08` / `0x34` | same | same | **`0x04` / `0x24`** | `0x04` / `0x24` | `0x04` / `0x24` |
| `Qfalse`, `Qtrue`, immediate/fixnum/flonum/symbol masks *(headers)* | 2.x values | same | same | same | same | same |
| `RString` *(headers)* | `as.heap.{len,ptr,aux}` or embedded, length in flag bits | same as 2.7 | same (`USE_RVARGC` is 0) | **embedded length in `as.embed.len` (`long`)** (`USE_RVARGC` 1) | **`len` moved to the top level**, `as.heap.{ptr,aux}` / `as.embed.ary` | same as 3.3 |
| `RArray` embedded length *(headers)* | `FL_USER3..4` | same | same | **`FL_USER3..9`** (`USE_RVARGC`), `as.ary` variable length | same as 3.2 | same as 3.2 (**verify** masks) |
| `RTypedData` / `rb_data_type_t` *(headers)* | `type, typed_flag, data`; `dcompact`, `reserved[1]` | same | same | same | same, plus `TYPED_DATA_EMBEDDED` (only with `RUBY_TYPED_EMBEDDABLE`, which Rutie does not set) | same as 3.3 |
| `T_*` type tags, `FL_FREEZE`, `FL_EXIVAR`, `FL_USHIFT`, encoding shift *(headers)* | 2.x values | same | same | same | same | same |
| Bit `1<<8` *(headers)* | `FL_TAINT` | **`FL_SHAREABLE`** (Ractor) | same | same | same | same |

Work this implies (0.11, R1):

- `value.rs`: per-version special constants (`0x08`/`0x34` for 3.0–3.1,
  `0x04`/`0x24` for 3.2+). Everything derived from `Qnil`, such as `RTEST`
  and `NIL_P` style checks, must follow; add unit tests comparing
  `Value::is_nil`/`is_true`/`is_undef` with what Ruby reports.
- `string.rs`: `rstring_len` / `rstring_ptr` / `rstring_end` for three
  layouts (3.0–3.1, 3.2, 3.3+). Better still, replace them with function
  calls where the cost is acceptable (`rb_str_length` is a `VALUE`;
  `RSTRING_PTR`/`RSTRING_LEN` are inline-only). **Decide** in R1 whether
  a tiny C shim, compiled with the `cc` crate and exporting
  `rutie_rstring_ptr`/`len`, is worth the build dependency.
- `array.rs`: the embedded-length mask per version (a wrong mask reads the
  wrong length of an embedded array).
- `constant.rs`: remove `FL_TAINT`, `FL_UNTRUSTED` and the `FL_DUPPED`
  derived from them (bit 8 means "shareable" in 3.x).
- `wrappable_struct!` keeps `reserved: [null; 2]`; in 3.x the struct is
  `dcompact` plus `reserved[1]`, the same size and position, so it stays
  compatible. R4 gives `dcompact` a real value.

A **prototype check** belongs in R1 as well: a script (kept in `ci/` or
`docs/`) that parses every `extern "C"` declaration in `src/rubysys/` and its
C prototype comment, then checks each name against the target Ruby's headers
and `libruby` exports. That is how this section was produced; running it in
CI catches later drift.

## 4. Removed, hidden and deprecated C APIs Rutie binds

| Symbol | Status in 3.x | Used by Rutie | Action |
|---|---|---|---|
| `rb_cData` | not exported from 3.0 *(exports)*, removed in 3.2 *(NEWS)* | `Class::data()` | 0.11: remove `Class::data()` and the declaration (a public API removal, so it waits for the MINOR release) |
| `rb_str_valid_encoding_p` | `static` in Ruby's C source, never exported | declared only | removed in 0.10.0 |
| `rb_enc_from_encoding_index`, `rb_f_eval`, `rb_str_force_encoding` | never in public headers; not exported | only in commented-out notes ("LINKER CANNOT FIND") | nothing to do |
| `rb_gc_force_recycle` | deprecated no-op since 3.1 *(headers, NEWS)*, **removed in 3.4** *(NEWS)* | `GC::force_recycle` | 0.11: document as a no-op on 3.1+; 0.12: `#[deprecated]`; 0.13: remove the method (linking fails on 3.4 otherwise) |
| `rb_thread_wait_fd`, `rb_thread_fd_writable` | deprecated since 3.1 *(NEWS)*, still declared in 3.4 *(headers)* | `Thread::wait_fd`, `Thread::wait_fd_writable` | 0.11: add `rb_io_wait`-based variants (3.0+); 0.12: deprecate the old ones; remove when Ruby does |
| `$SAFE`, taint and trust functions | removed (3.0 / 3.2) *(NEWS)* | none bound | nothing to do beyond the constants in §3 |
| `rb_newobj`, `rb_newobj_of` | removed in 3.4 *(NEWS)* | not bound | none |
| `rb_objspace_marked_object_p` | not in public headers; **not exported by 3.4** *(exports)* | `GC::is_marked` | 0.12: `#[deprecated]`; 0.13: remove |
| `rb_complex_polar` | deprecated since 3.0 *(headers)* | `Complex::polar` | 0.11: bind `rb_complex_new_polar` instead |
| `rb_postponed_job_register(_one)` | deprecated in 3.3 *(NEWS)* | not bound | bind `rb_postponed_job_preregister`/`trigger` if ever needed |
| `rb_io_t` members | hidden in 3.3 *(NEWS)* | not read | keep using `IO` methods and `rb_io_*` functions |

All other functions `rutie` binds today (about 560) are present in the
3.0–3.4 headers *(headers)* and exported by 3.0–3.3 *(exports)*. Prototypes
still need the R1 check: presence doesn't mean the signature is unchanged.

### New C APIs by version *(headers)*

| API | 2.7 | 3.0 | 3.1 | 3.2 | 3.3 | 3.4 |
|---|---|---|---|---|---|---|
| `rb_gc_mark_movable`, `rb_gc_location` | yes | yes | yes | yes | yes | yes |
| `rb_ext_ractor_safe`, `RUBY_TYPED_FROZEN_SHAREABLE` | – | yes | yes | yes | yes | yes |
| `rb_io_wait` | – | yes | yes | yes | yes | yes |
| `rb_fiber_scheduler_current` (and the `rb_fiber_scheduler_*` family) | – | – | yes | yes | yes | yes |
| `rb_fiber_new_storage`, `rb_hash_new_capa` | – | – | – | yes | yes | yes |
| `rb_data_define` | – | – | – | – | yes | yes |

## 5. Behaviour changes that affect Rutie's API, tests and doctests

- **Keyword arguments are separated from positional ones (3.0)** *(NEWS)*.
  `Object::send_with_keywords` and `VM::is_keyword_given` (2.7-only today)
  become available unconditionally. `VM::scan_args` with `:` follows
  3.0 rules (a trailing hash is no longer taken as keywords unless passed as
  keywords). `methods!` gets no implicit keyword conversion; callers use
  `VM::get_kwargs`. Tests covering 2.7's keyword warnings are replaced.
- **Frozen `Range` and `Regexp` literals (3.0)**; `Hash#each` always yields
  pairs (3.0); `TRUE`/`FALSE`/`NIL` constants removed (3.0) *(NEWS)*.
- **`Fixnum`/`Bignum` constants removed (3.2)** *(NEWS)*: doctests and Ruby
  snippets must not name them. The Rust `Fixnum` type stays.
  `Kernel#=~`, `File.exists?` and `Dir.exists?` are also gone in 3.2.
- **Message formats:** 3.3 `NoMethodError` messages no longer include the
  receiver's `#inspect`; 3.4 quotes with `'` instead of a backtick and adds
  the class name before the method name *(NEWS)*. Assertions on messages
  compare only stable parts or are gated with `#[cfg(ruby_gte_3_4)]`.
- **`Hash#inspect` (3.4)** renders `{user: 1}` and `{"user" => 1}`
  *(NEWS)*. Doctests comparing `inspect` output of hashes need gates or
  structural assertions.
- **Chilled string literals (3.4)**: mutating a literal from a file without
  a `frozen_string_literal` comment warns *(NEWS)*. Tests that mutate
  strings created by `VM::eval("'...'")` must `dup` first, or set the magic
  comment.
- **Parser:** Prism is the default parser in 3.4 (**verify** exact
  `SyntaxError` message changes); `VM::run_file` and `VM::eval` tests assert
  on the error class, not the message text.
- **Fibers:** 3.0 uses native coroutines on every platform, including
  arm64 macOS where 2.5/2.6 fell back to `ucontext`. The
  create-and-resume-under-the-same-tag barrier that made `Fiber` and
  `Enumerator` use `rb_rescue2` must be retested on each 3.x (**verify**);
  keep `rb_rescue2` unless it is shown to be unnecessary.
- **`ruby_options` once per process:** re-check that `VM::run_file`'s
  `rb_argv0` guard still holds on 3.x (**verify**).
- **Threads:** 3.3 adds M:N threads (`RUBY_MN_THREADS=1`) *(NEWS)*; the
  `on_ruby_thread` test harness and `Thread::call_without_gvl` are retested
  with it enabled.

## 6. 0.11 — Ruby 3.0, 3.1, 3.2

- [x] **R0 Branch, cfgs, CI.** §2 for 3.0/3.1/3.2; `build.rs` rejects Ruby 2
      (and 3.3+) naming the Rutie line to use; `ruby_2_*` gates and code
      paths deleted. CI uses `ruby/setup-ruby` on `ubuntu-22.04` (it has
      3.0 builds, so no OpenSSL step), with macOS and Windows best-effort
      and static-Ruby rows dropped (RVM never produced a static `libruby`).
      *Update:* 0.11 is rebased onto 0.10.2 (`0.10-windows-support`), so CI
      also runs Windows (MSVC and GNU, required) and static Rubies built
      with `ruby-build` on Linux and macOS (best-effort), with the Ruby 2
      specifics of that work (copy-stack fibers) removed.
      `build.rs` now reruns when `PATH`, `RUBY` or a version manager's
      variable changes: it printed no `rerun-if` lines, so switching Ruby
      kept the old cfgs, a silent ABI mismatch now that `Qnil` differs.
      *Left to the maintainer:* creating `0.10-stable` from the 0.10.1
      release (`dc8f8bb`), and tagging releases.
- [x] **R1 ABI audit.** §3 changes (special constants, `RString`, `RArray`,
      `FL_SHAREABLE`); `ci/check_rubysys.py` (headers, return types,
      exports; run in CI) found `rb_enc_codepoint_len` declared with the
      wrong return type and the deprecated `rb_complex_polar`; both fixed.
      Unit tests read strings of 0–700 bytes and arrays of 0–200 elements,
      made several ways, across the embed boundary, and check the special
      constants and `Value::ty` against Ruby. `rb_eql` returning `int`
      (`Object::is_eql` was always `false` on 3.x) was found here too.
      Decision: no C shim; the direct reads are version-gated and tested.
- [x] **R2 Removed and deprecated APIs.** `Class::data` removed;
      `GC::force_recycle` documented as a no-op on 3.1+;
      `Thread::wait_readable`/`wait_writable` (`rb_io_wait`, taking an `IO`
      and an optional timeout, returning `Result<bool, _>`).
- [x] **R3 Keyword arguments.** Ungated with R0. `VM::scan_args` follows
      Ruby 3 (`RB_SCAN_ARGS_PASS_CALLED_KEYWORDS`), and
      `VM::scan_args_with_keywords` keeps Ruby 2's reading (rule 8: a
      second method rather than a changed one). `methods!` docs and a test
      cover keywords arriving as a trailing `Hash`.
- [x] **R4 GC compaction.** `GC::mark_movable`, `GC::location`,
      `GC::compact`, and a `compact` clause in `wrappable_struct!` stored in
      `reserved[0]` (so `DataTypeFunction` keeps its layout). A test moves
      200 wrapped strings with `GC.verify_compaction_references`.
- [x] **R5 Ractor.** `VM::ext_ractor_safe` (unsafe) and
      `VM::ext_ractor_unsafe`. Ruby's main thread starts with C methods
      marked Ractor-safe, so `VM::init` switches that off. Tested from a
      real Ractor. *Deferred:* `RUBY_TYPED_FROZEN_SHAREABLE` for wrapped
      data, which needs `get_data_mut` to refuse frozen objects first.
- [x] **R6 Fibers, scheduler, threads.** Fiber tests pass on 3.0–3.2 with
      `rb_rescue2` kept. `Fiber::with_storage` (3.2, `rb_fiber_new_storage`).
      Scheduler hooks are not bound (no safe wrapper designed yet).
- [x] **R7 Tests and doctests.** Every unit test and doctest passes on
      3.0.6, 3.1.4 and 3.2.6 (stable Rust), and on a 3.2 built with YJIT
      (3.2.11, §2 "libruby and the Rust runtime"); doctests no longer name
      `Fixnum`/`Bignum` or use UTF-32 as a real encoding. `VM::run_file`
      has a unit test now that it is a `load`.
- [x] **R8 New APIs (optional).** `Hash::with_capacity` on every supported
      Ruby (a hint, ignored before 3.2). Memory view: skipped.
- [x] **R9 Release.** README (support table, roadmap, badge, 0.11
      migration notes), CHANGELOG, version 0.11.0.

## 7. 0.12 — Ruby 3.1, 3.2, 3.3

- [x] Drop 3.0: `build.rs` supports 3.1–3.3 and points 3.0 at 0.11; the
      only 3.0-specific code was the cfg test. CI moves to `ubuntu-latest`
      (there was no OpenSSL step left to drop).
- [x] `RString`: the 3.3 layout (top-level `len`); three layouts are now
      3.1, 3.2 and 3.3, covered by the direct-read unit tests.
- [x] `GC::force_recycle`, `GC::is_marked` and
      `Thread::wait_fd`/`wait_fd_writable` get `#[deprecated]` pointing to
      their replacements (none for `is_marked`, whose C function Ruby 3.4
      no longer exports).
- [x] 3.3 behaviour: every unit test and doctest passes on 3.3 unchanged
      (none asserts a `NoMethodError` message), and the unit and `Thread`
      doctests pass with `RUBY_MN_THREADS=1`. `rb_io_t` is not read.
- [x] Optional 3.3 APIs: `IO::is_closed` uses `rb_io_closed_p` on 3.3.
      *Deferred:* `rb_data_define` (a `DataClass` wrapper deserves its own
      design), `rb_io_path`/`rb_io_mode`, `rb_io_open_descriptor`.
- [x] Tests, doctests, README, CHANGELOG, version 0.12.0.

## 8. 0.13 — Ruby 3.2, 3.3, 3.4

- [ ] Drop 3.1: `Qnil`/`Qundef` become one set of values (`0x04`/`0x24`);
      `RString` has two layouts (3.2 and 3.3+).
- [ ] Remove `GC::force_recycle` (`rb_gc_force_recycle` is gone in 3.4) and
      `GC::is_marked` (`rb_objspace_marked_object_p` is gone in 3.4).
- [ ] Check the 3.4 prototypes and exports with the R1 script against a real
      3.4 `libruby` (only headers were checked for this plan).
- [ ] 3.4 behaviour: message quoting and class names, `Hash#inspect`,
      chilled strings, Prism `SyntaxError` messages (§5).
- [ ] Tests, doctests, README, CHANGELOG, release 0.13.0.

## 9. How to work a package

Same as §5 of the Ruby 2 plan: bind in `rubysys` with C prototype comments,
wrap in `binding`, expose in `class` with a running, asserting doctest and a
bottom-of-file unit test, verify on all three Rubies (stable and beta)
before pushing, and update the CHANGELOG in the same commit.

## 10. Open questions for the maintainer

1. *Resolved:* 0.10.0 is not released yet, so all of the Ruby 2 work,
   including the breaking `VM::at_exit` change, ships in 0.10.0 (a MINOR
   release under rule 9).
2. How long `0.10-stable` gets fixes after 0.11 ships.
3. Whether Windows becomes a supported (not best-effort) target once CI
   uses prebuilt Rubies.
4. Whether a 0.14 line for Ruby 4.0 follows the same three-version pattern.
