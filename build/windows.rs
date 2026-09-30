// Windows support for `build.rs`.
//
// Ruby for Windows is usually RubyInstaller's (built with MinGW), which ships
// the Ruby DLL in `bindir` and a GNU import library (`lib<so>.dll.a`) in
// `libdir`. The GNU Rust toolchain links against that import library as is.
// The MSVC toolchain needs an MSVC import library (`<so>.lib`), which is built
// here from the DLL's export table with `lib.exe`. A Ruby built with MSVC
// (mswin) ships its own `<so>.lib` and needs none of this.
//
// Everything here is chosen from the *target* (`CARGO_CFG_TARGET_*`), not from
// `#[cfg]`, which in a build script describes the host.

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

pub fn is_target() -> bool {
    env::var("CARGO_CFG_TARGET_OS").map_or(false, |os| os == "windows")
}

pub fn is_msvc_target() -> bool {
    is_target() && env::var("CARGO_CFG_TARGET_ENV").map_or(false, |target_env| target_env == "msvc")
}

// The name `src/rubysys` links the Ruby DLL by. Rust only imports variables
// such as `rb_cObject` from a DLL (through `__imp_rb_cObject`) when their
// `extern` block names the library in `#[link]`, and that name has to be
// fixed, while Ruby's is not (`x64-msvcrt-ruby270`, `x64-ucrt-ruby310`, ...).
// Without it, MSVC's linker resolves each variable to a jump thunk and Rust
// reads the thunk's code instead of the variable.
const LINK_NAME: &str = "rutie_ruby";

// Puts an import library for the Ruby DLL in `OUT_DIR` under `LINK_NAME`, and
// turns on the `#[link]` attributes in `src/rubysys` (`rutie_dllimport`).
pub fn link_ruby_dll(so_name: &str, libdir: &Path, bindir: &Path, libruby_so: &str) {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is not set"));

    if is_msvc_target() {
        let mswin_lib = libdir.join(format!("{}.lib", so_name));

        if mswin_lib.is_file() {
            // An MSVC-built (mswin) Ruby ships its own import library.
            copy(&mswin_lib, &out_dir.join(format!("{}.lib", LINK_NAME)));
        } else {
            msvc_import_library(&bindir.join(libruby_so), libruby_so, &out_dir);
        }
    } else {
        // MinGW Ruby ships a GNU import library, `lib<so>.dll.a`.
        let gnu_lib = libdir.join(format!("lib{}.dll.a", so_name));

        if !gnu_lib.is_file() {
            panic!(
                "The import library for the Ruby DLL, {}, was not found.",
                gnu_lib.display()
            );
        }

        copy(&gnu_lib, &out_dir.join(format!("lib{}.dll.a", LINK_NAME)));
    }

    // `OUT_DIR` is `<target>/[<triple>/]<profile>/build/rutie-<hash>/out`.
    if let Some(profile_dir) = out_dir.ancestors().nth(3) {
        let deps_dir = profile_dir.join("deps");

        if fs::create_dir_all(&deps_dir).is_ok() {
            copy_builtin_dlls(&bindir.join("ruby_builtin_dlls"), &deps_dir);
        }
    }

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-cfg=rutie_dllimport");
}

// The Ruby DLL depends on DLLs in `bin\ruby_builtin_dlls` (`libgmp-10.dll`,
// ...), which `ruby.exe` finds through its manifest but a Rust program does
// not. Cargo puts `target/<profile>/deps` on `PATH` for `cargo run`,
// `cargo test` and doctests, so copies there are found (as `libruby.so` is
// symlinked there on Linux). The Ruby DLL itself is not copied: Ruby finds
// its library directory from where it is loaded.
fn copy_builtin_dlls(builtin_dlls: &Path, destination: &Path) {
    let entries = match fs::read_dir(builtin_dlls) {
        Ok(entries) => entries,
        // Not a RubyInstaller Ruby.
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let from = entry.path();
        let is_dll = from
            .extension()
            .map_or(false, |extension| extension.eq_ignore_ascii_case("dll"));

        if !is_dll {
            continue;
        }

        let to = destination.join(entry.file_name());
        let up_to_date = match (fs::metadata(&from), fs::metadata(&to)) {
            (Ok(from), Ok(to)) => from.len() == to.len(),
            _ => false,
        };

        if !up_to_date {
            copy(&from, &to);
        }
    }
}

fn copy(from: &Path, to: &Path) {
    fs::copy(from, to).unwrap_or_else(|e| {
        panic!(
            "could not copy {} to {}: {}",
            from.display(),
            to.display(),
            e
        )
    });
}

// Builds an MSVC import library for a DLL (from its export table) with `lib.exe`.
fn msvc_import_library(dll: &Path, libruby_so: &str, out_dir: &Path) {
    let def = out_dir.join(format!("{}.def", LINK_NAME));
    let lib = out_dir.join(format!("{}.lib", LINK_NAME));

    let dll_bytes = fs::read(&dll)
        .unwrap_or_else(|e| panic!("could not read the Ruby DLL {}: {}", dll.display(), e));
    let exports = pe::exports(&dll_bytes)
        .unwrap_or_else(|e| panic!("could not read the exports of {}: {}", dll.display(), e));

    let machine = match exports.machine {
        pe::MACHINE_AMD64 => "X64",
        pe::MACHINE_I386 => "X86",
        pe::MACHINE_ARM64 => "ARM64",
        other => panic!(
            "{} is for an unsupported machine type 0x{:x}",
            dll.display(),
            other
        ),
    };

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let expected = match target_arch.as_str() {
        "x86_64" => "X64",
        "x86" => "X86",
        "aarch64" => "ARM64",
        _ => machine,
    };
    if machine != expected {
        panic!(
            "The Ruby found ({}) is built for {}, but the Rust target is {}. \
             Use a Ruby and a Rust toolchain for the same architecture \
             (set RUBY to the ruby.exe to build against).",
            dll.display(),
            machine,
            target_arch
        );
    }

    fs::write(&def, exports.module_definition(libruby_so))
        .unwrap_or_else(|e| panic!("could not write {}: {}", def.display(), e));

    let lib_exe = find_lib_exe(&target_arch);
    let output = Command::new(&lib_exe)
        .arg("/nologo")
        .arg(format!("/def:{}", def.display()))
        .arg(format!("/out:{}", lib.display()))
        .arg(format!("/machine:{}", machine))
        .arg(format!("/name:{}", libruby_so))
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "could not run {} to make an import library for {}: {}. \
                 Install the Visual Studio C++ build tools, or build from a \
                 Developer Command Prompt so lib.exe is on PATH.",
                lib_exe.display(),
                libruby_so,
                e
            )
        });

    if !output.status.success() {
        panic!(
            "{} failed to make an import library for {}:\n{}{}",
            lib_exe.display(),
            libruby_so,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

// Checks that a static Ruby library usable by the target toolchain exists,
// and adds `libdir` to the search path for it.
pub fn check_static_library(so_name: &str, libdir: &Path) {
    let msvc_lib = libdir.join(format!("{}-static.lib", so_name));
    let gnu_lib = libdir.join(format!("lib{}-static.a", so_name));

    let found = if is_msvc_target() {
        if gnu_lib.is_file() && !msvc_lib.is_file() {
            panic!(
                "{} is a MinGW static library, which the MSVC Rust toolchain \
                 cannot link. Build with the GNU toolchain \
                 (--target x86_64-pc-windows-gnu), or link Ruby dynamically.",
                gnu_lib.display()
            );
        }
        msvc_lib.is_file()
    } else {
        gnu_lib.is_file()
    };

    if !found && env::var_os("RUBY_STATIC_PATH").is_none() {
        panic!(
            "No static Ruby library was found in {} for a static build. \
             RubyInstaller does not ship one: unset RUBY_STATIC to link Ruby \
             dynamically, or point RUBY_STATIC_PATH at a static Ruby library.",
            libdir.display()
        );
    }

    println!("cargo:rustc-link-search=native={}", libdir.display());
}

// Finds `lib.exe` for the target architecture: from the environment of a
// Visual Studio developer prompt, through `vswhere`, or on `PATH`.
fn find_lib_exe(target_arch: &str) -> PathBuf {
    let host = if cfg!(target_arch = "x86") {
        "Hostx86"
    } else if cfg!(target_arch = "aarch64") {
        "Hostarm64"
    } else {
        "Hostx64"
    };
    let host_arch = &host[4..];
    let target = match target_arch {
        "x86" => "x86",
        "aarch64" => "arm64",
        _ => "x64",
    };

    let tool_dirs = env::var_os("VCToolsInstallDir")
        .map(PathBuf::from)
        .into_iter()
        .chain(vswhere_tools_dir());

    for tools in tool_dirs {
        // Any `lib.exe` can write an import library for any machine, so fall
        // back to the host's own if the target's is not installed.
        for arch in [target, host_arch].iter() {
            let lib_exe = tools.join("bin").join(host).join(arch).join("lib.exe");
            if lib_exe.is_file() {
                return lib_exe;
            }
        }
    }

    PathBuf::from("lib.exe")
}

// `VC\Tools\MSVC\<version>` of the newest Visual Studio with the C++ tools.
fn vswhere_tools_dir() -> Option<PathBuf> {
    let program_files = env::var_os("ProgramFiles(x86)").or_else(|| env::var_os("ProgramFiles"))?;
    let vswhere = Path::new(&program_files)
        .join("Microsoft Visual Studio")
        .join("Installer")
        .join("vswhere.exe");

    let output = Command::new(vswhere)
        .args(&[
            "-latest",
            "-products",
            "*",
            "-requires",
            "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
            "-property",
            "installationPath",
        ])
        .output()
        .ok()?;

    let installation = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if installation.is_empty() {
        return None;
    }

    let vc = Path::new(&installation).join("VC");
    let version = fs::read_to_string(
        vc.join("Auxiliary")
            .join("Build")
            .join("Microsoft.VCToolsVersion.default.txt"),
    )
    .ok()?;

    Some(vc.join("Tools").join("MSVC").join(version.trim()))
}

// Just enough of the PE format to list a DLL's exports.
mod pe {
    use std::convert::TryInto;

    pub const MACHINE_I386: u16 = 0x14c;
    pub const MACHINE_AMD64: u16 = 0x8664;
    pub const MACHINE_ARM64: u16 = 0xaa64;

    const PE32_MAGIC: u16 = 0x10b;
    const PE32_PLUS_MAGIC: u16 = 0x20b;
    const SCN_MEM_EXECUTE: u32 = 0x2000_0000;

    pub struct Export {
        pub name: String,
        // Variables (`rb_cObject`, ...) must be imported as `DATA`. Otherwise
        // `lib.exe` makes them jump thunks and reading one reads code.
        pub data: bool,
    }

    pub struct Exports {
        pub machine: u16,
        pub exports: Vec<Export>,
    }

    impl Exports {
        pub fn module_definition(&self, dll_name: &str) -> String {
            let mut def = format!("LIBRARY \"{}\"\nEXPORTS\n", dll_name);

            for export in &self.exports {
                def.push_str("    ");
                def.push_str(&export.name);
                if export.data {
                    def.push_str(" DATA");
                }
                def.push('\n');
            }

            def
        }
    }

    struct Section {
        virtual_address: u32,
        virtual_size: u32,
        raw_offset: u32,
        raw_size: u32,
        characteristics: u32,
    }

    fn u16_at(bytes: &[u8], offset: usize) -> Result<u16, String> {
        bytes
            .get(offset..offset + 2)
            .map(|b| u16::from_le_bytes(b.try_into().unwrap()))
            .ok_or_else(|| format!("truncated file at offset 0x{:x}", offset))
    }

    fn u32_at(bytes: &[u8], offset: usize) -> Result<u32, String> {
        bytes
            .get(offset..offset + 4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .ok_or_else(|| format!("truncated file at offset 0x{:x}", offset))
    }

    fn section_for(sections: &[Section], rva: u32) -> Option<&Section> {
        sections.iter().find(|s| {
            let size = s.virtual_size.max(s.raw_size);
            rva >= s.virtual_address && rva - s.virtual_address < size
        })
    }

    fn offset_of(sections: &[Section], rva: u32) -> Result<usize, String> {
        section_for(sections, rva)
            .map(|s| (rva - s.virtual_address + s.raw_offset) as usize)
            .ok_or_else(|| format!("address 0x{:x} is outside every section", rva))
    }

    fn c_string_at(bytes: &[u8], offset: usize) -> Result<String, String> {
        let tail = bytes
            .get(offset..)
            .ok_or_else(|| format!("truncated file at offset 0x{:x}", offset))?;
        let end = tail
            .iter()
            .position(|&b| b == 0)
            .ok_or_else(|| format!("unterminated name at offset 0x{:x}", offset))?;

        Ok(String::from_utf8_lossy(&tail[..end]).into_owned())
    }

    // Exported names a module definition file can list. MinGW DLLs can also
    // export internal names such as `.refptr.rb_cObject`.
    fn is_symbol_name(name: &str) -> bool {
        !name.is_empty()
            && name.bytes().all(|b| {
                b.is_ascii_alphanumeric() || b == b'_' || b == b'@' || b == b'$' || b == b'?'
            })
    }

    pub fn exports(bytes: &[u8]) -> Result<Exports, String> {
        if bytes.get(0..2) != Some(b"MZ") {
            return Err("not a PE file (no MZ header)".to_string());
        }

        let pe = u32_at(bytes, 0x3c)? as usize;
        if bytes.get(pe..pe + 4) != Some(b"PE\0\0") {
            return Err("not a PE file (no PE signature)".to_string());
        }

        let coff = pe + 4;
        let machine = u16_at(bytes, coff)?;
        let section_count = u16_at(bytes, coff + 2)? as usize;
        let optional_size = u16_at(bytes, coff + 16)? as usize;
        let optional = coff + 20;

        let data_directories = match u16_at(bytes, optional)? {
            PE32_MAGIC => optional + 96,
            PE32_PLUS_MAGIC => optional + 112,
            magic => return Err(format!("unknown optional header magic 0x{:x}", magic)),
        };
        // The export table is data directory 0.
        let export_rva = u32_at(bytes, data_directories)?;
        let export_size = u32_at(bytes, data_directories + 4)?;

        let sections = (0..section_count)
            .map(|i| {
                let header = optional + optional_size + i * 40;

                Ok(Section {
                    virtual_size: u32_at(bytes, header + 8)?,
                    virtual_address: u32_at(bytes, header + 12)?,
                    raw_size: u32_at(bytes, header + 16)?,
                    raw_offset: u32_at(bytes, header + 20)?,
                    characteristics: u32_at(bytes, header + 36)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        if export_rva == 0 {
            return Err("the DLL has no export table".to_string());
        }

        let directory = offset_of(&sections, export_rva)?;
        let function_count = u32_at(bytes, directory + 20)?;
        let name_count = u32_at(bytes, directory + 24)? as usize;
        let functions = offset_of(&sections, u32_at(bytes, directory + 28)?)?;
        let names = offset_of(&sections, u32_at(bytes, directory + 32)?)?;
        let ordinals = offset_of(&sections, u32_at(bytes, directory + 36)?)?;

        let mut exports = Vec::with_capacity(name_count);

        for i in 0..name_count {
            let name = c_string_at(bytes, offset_of(&sections, u32_at(bytes, names + i * 4)?)?)?;

            if !is_symbol_name(&name) {
                continue;
            }

            let ordinal = u16_at(bytes, ordinals + i * 2)? as u32;
            if ordinal >= function_count {
                return Err(format!("export {} has an invalid ordinal", name));
            }

            let rva = u32_at(bytes, functions + ordinal as usize * 4)?;
            let forwarded = rva >= export_rva && rva - export_rva < export_size;
            let data = !forwarded
                && section_for(&sections, rva)
                    .map_or(false, |s| s.characteristics & SCN_MEM_EXECUTE == 0);

            exports.push(Export { name, data });
        }

        Ok(Exports { machine, exports })
    }
}
