use std::{
    collections::{HashMap, HashSet},
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "openbsd"
))]
use std::fs;

#[path = "build/windows.rs"]
mod windows;

macro_rules! ci_stderr_log {
    () => (eprint!("\n"));
    ($($arg:tt)*) => ({
        if env::var_os("CI_STDERR_LOG").is_some() { eprintln!($($arg)*) }
    })
}

fn rbconfig(key: &str) -> String {
    try_rbconfig(key).unwrap_or_else(|e| panic!("ruby not found: {}", e))
}

fn try_rbconfig(key: &str) -> Result<String, std::io::Error> {
    let ruby = env::var_os("RUBY").unwrap_or(OsString::from("ruby"));

    let config = Command::new(ruby)
        .arg("-e")
        .arg(format!("print RbConfig::CONFIG['{}']", key))
        .output()?;

    Ok(String::from_utf8(config.stdout).expect("RbConfig value not UTF-8!"))
}

// Minor versions of Ruby 2 that Rutie supports. Each one gets an exact
// `ruby_2_N` cfg and, from 2.6 on, a cumulative `ruby_gte_2_N` cfg.
const SUPPORTED_RUBY_2_MINORS: [u32; 3] = [5, 6, 7];

// Emits `ruby_2_5` / `ruby_2_6` / `ruby_2_7` for the exact version of the
// Ruby found by `rbconfig` and `ruby_gte_2_6` / `ruby_gte_2_7` for every
// version at or above those, so bindings can be gated with
// `#[cfg(ruby_gte_2_7)]` instead of sniffing the version at runtime.
//
// The version is also exported to crates depending on Rutie as
// `DEP_RUBY_VERSION_MAJOR` / `DEP_RUBY_VERSION_MINOR` (through `links = "ruby"`).
fn ruby_version_cfgs() {
    for minor in SUPPORTED_RUBY_2_MINORS.iter() {
        println!("cargo:rustc-check-cfg=cfg(ruby_2_{})", minor);
        println!("cargo:rustc-check-cfg=cfg(ruby_gte_2_{})", minor);
    }

    let (major, minor) = match (try_rbconfig("MAJOR"), try_rbconfig("MINOR")) {
        (Ok(major), Ok(minor)) => match (major.parse::<u32>(), minor.parse::<u32>()) {
            (Ok(major), Ok(minor)) => (major, minor),
            _ => {
                println!(
                    "cargo:warning=Could not read the Ruby version from RbConfig; \
                     no Ruby version cfg flags were set."
                );
                return;
            }
        },
        _ => {
            // Without linking (the `no-link` feature or `NO_LINK_RUTIE`) a Ruby
            // is not required to build, so a missing Ruby is not an error here.
            ci_stderr_log!("ruby not found; no Ruby version cfg flags were set");
            return;
        }
    };

    println!("cargo:version_major={}", major);
    println!("cargo:version_minor={}", minor);

    if major != 2 || !SUPPORTED_RUBY_2_MINORS.contains(&minor) {
        println!(
            "cargo:warning=Rutie supports Ruby 2.5, 2.6 and 2.7; found Ruby {}.{}. \
             Ruby 3 is not supported yet.",
            major, minor
        );
    }

    for supported in SUPPORTED_RUBY_2_MINORS.iter() {
        if major == 2 && minor == *supported {
            println!("cargo:rustc-cfg=ruby_2_{}", supported);
        }

        if major > 2 || (major == 2 && minor >= *supported) {
            println!("cargo:rustc-cfg=ruby_gte_2_{}", supported);
        }
    }

    ci_stderr_log!("Ruby version cfg flags set for Ruby {}.{}", major, minor);
}

#[cfg(not(target_os = "macos"))]
fn macos_static_ruby_dep() {}

#[cfg(target_os = "macos")]
fn macos_static_ruby_dep() {
    println!("cargo:rustc-link-lib=framework=Foundation");
}

fn use_static() {
    if let Some(location) = env::var_os("RUBY_STATIC_PATH").map(|s| s.to_string_lossy().to_string())
    {
        println!("cargo:rustc-link-search={}", location);
    }

    // If Windows
    if windows::is_target() {
        windows::check_static_library(&rbconfig("RUBY_SO_NAME"), Path::new(&rbconfig("libdir")));
    }

    // If Mac OS
    macos_static_ruby_dep();

    // **Flags must be last in order for linking!**
    static_linker_args();

    ci_stderr_log!("Using static linker flags");
}

fn use_dylib() {
    println!("cargo:rustc-link-search={}", rbconfig("libdir"));

    // If Windows
    if windows::is_target() {
        windows::link_ruby_dll(
            &rbconfig("RUBY_SO_NAME"),
            Path::new(&rbconfig("libdir")),
            Path::new(&rbconfig("bindir")),
            &rbconfig("LIBRUBY_SO"),
        );
    }

    dynamic_linker_args();
    ci_stderr_log!("Using dynamic linker flags");
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "openbsd"
))]
use std::os::unix::fs::symlink;

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "openbsd"
))]
fn ruby_lib_link_name() -> String {
    // Rust with linker search paths doesn't seem to use those paths
    // but rather resorts to the systems Ruby.  So we symlink into
    // our own deps directory for it to work.
    let so_file = format!("libruby.so.{}.{}", rbconfig("MAJOR"), rbconfig("MINOR"));
    let out_dir = env::var("OUT_DIR").unwrap();
    let working_dir = out_dir.splitn(2, "/target").next().unwrap();
    let profile_dir = format!("{}/target/{}", working_dir, env::var("PROFILE").unwrap());
    let source = format!("{}/{}", rbconfig("libdir"), so_file);

    // Newer Cargo versions run test binaries with `target/<profile>` on the
    // library search path instead of `target/<profile>/deps`, so link both.
    // Cargo also forwards native search paths inside the target directory to
    // the library path used to run doctests.
    println!("cargo:rustc-link-search=native={}", profile_dir);
    for destination in [format!("{}/deps", profile_dir), profile_dir] {
        fs::create_dir_all(&destination).expect("create_dir_all fail");
        let target = format!("{}/{}", destination, so_file);

        if fs::read_link(&target).is_err() {
            symlink(&source, target).expect("symlink fail");
        }
    }

    rbconfig("RUBY_SO_NAME")
}

#[cfg(target_os = "macos")]
fn ruby_lib_link_name() -> String {
    format!(
        "{}.{}.{}",
        rbconfig("RUBY_BASE_NAME"),
        rbconfig("MAJOR"),
        rbconfig("MINOR")
    )
}

#[cfg(target_os = "windows")]
fn ruby_lib_link_name() -> String {
    rbconfig("RUBY_SO_NAME")
}

fn dynamic_linker_args() {
    // On Windows `src/rubysys` links the Ruby DLL itself, through `#[link]`
    // attributes (see `build/windows.rs`), and `LIBS` are the DLL's own
    // dependencies (`-lgmp` on Ruby 2.5), which programs using it do not need.
    if windows::is_target() {
        return;
    }

    let mut library = Library::new();
    library.parse_libs_cflags(rbconfig("LIBRUBYARG_SHARED").as_bytes(), false);
    println!("cargo:rustc-link-lib=dylib={}", ruby_lib_link_name());
    library.parse_libs_cflags(rbconfig("LIBS").as_bytes(), false);
}

fn static_linker_args() {
    let mut library = Library::new();
    // On Windows this names the import library for the Ruby DLL.
    if !windows::is_target() {
        library.parse_libs_cflags(rbconfig("LIBRUBYARG_SHARED").as_bytes(), true);
    }
    library.parse_libs_cflags(
        format!("-l{}-static", rbconfig("RUBY_SO_NAME")).as_bytes(),
        true,
    );
    library.parse_libs_cflags(rbconfig("MAINLIBS").as_bytes(), false);
}

#[derive(Debug)]
pub struct Library {
    pub libs: Vec<String>,
    pub link_paths: Vec<PathBuf>,
    pub frameworks: Vec<String>,
    pub framework_paths: Vec<PathBuf>,
    pub include_paths: Vec<PathBuf>,
    pub defines: HashMap<String, Option<String>>,
    pub version: String,
    _priv: (),
}

impl Library {
    fn new() -> Library {
        Library {
            libs: Vec::new(),
            link_paths: Vec::new(),
            include_paths: Vec::new(),
            frameworks: Vec::new(),
            framework_paths: Vec::new(),
            defines: HashMap::new(),
            version: String::new(),
            _priv: (),
        }
    }

    fn parse_libs_cflags(&mut self, output: &[u8], statik: bool) {
        let mut is_msvc = false;
        if let Ok(target) = env::var("TARGET") {
            if target.contains("msvc") {
                is_msvc = true;
            }
        }

        let words = split_flags(output);
        let parts = words
            .iter()
            .filter(|l| l.len() > 2)
            .map(|arg| (&arg[0..2], &arg[2..]))
            .collect::<HashSet<_>>();

        let mut dirs = Vec::new();
        for &(flag, val) in &parts {
            match flag {
                "-L" => {
                    let meta = format!("rustc-link-search=native={}", val);
                    println!("cargo:{}", &meta);
                    dirs.push(PathBuf::from(val));
                    self.link_paths.push(PathBuf::from(val));
                }
                "-F" => {
                    let meta = format!("rustc-link-search=framework={}", val);
                    println!("cargo:{}", &meta);
                    self.framework_paths.push(PathBuf::from(val));
                }
                "-I" => {
                    self.include_paths.push(PathBuf::from(val));
                }
                "-l" => {
                    // These are provided by the CRT with MSVC
                    if is_msvc && ["m", "c", "pthread"].contains(&val) {
                        continue;
                    }

                    if is_static() && statik {
                        let meta = format!("rustc-link-lib=static={}", val);
                        println!("cargo:{}", &meta);
                    } else {
                        let meta = format!("rustc-link-lib={}", val);
                        println!("cargo:{}", &meta);
                    }

                    self.libs.push(val.to_string());
                }
                "-D" => {
                    let mut iter = val.split("=");
                    self.defines.insert(
                        iter.next().unwrap().to_owned(),
                        iter.next().map(|s| s.to_owned()),
                    );
                }
                _ => {}
            }
        }

        let mut iter = words.iter().flat_map(|arg| {
            if arg.starts_with("-Wl,") {
                arg[4..].split(',').collect()
            } else {
                vec![arg.as_ref()]
            }
        });
        while let Some(part) = iter.next() {
            if part != "-framework" {
                continue;
            }
            if let Some(lib) = iter.next() {
                let meta = format!("rustc-link-lib=framework={}", lib);
                println!("cargo:{}", &meta);
                self.frameworks.push(lib.to_string());
            }
        }

        // A Ruby built with MSVC (mswin) names libraries by file: `user32.lib`.
        if is_msvc {
            for word in &words {
                if let Some(lib) = word.strip_suffix(".lib") {
                    if lib.is_empty() || lib.starts_with('-') || lib.contains(&['/', '\\'][..]) {
                        continue;
                    }

                    if is_static() && statik {
                        println!("cargo:rustc-link-lib=static={}", lib);
                    } else {
                        println!("cargo:rustc-link-lib={}", lib);
                    }

                    self.libs.push(lib.to_string());
                }
            }
        }
    }
}

fn split_flags(output: &[u8]) -> Vec<String> {
    let mut word = Vec::new();
    let mut words = Vec::new();

    for &b in output {
        match b {
            b' ' => {
                if !word.is_empty() {
                    words.push(String::from_utf8(word).unwrap());
                    word = Vec::new();
                }
            }
            _ => word.push(b),
        }
    }

    if !word.is_empty() {
        words.push(String::from_utf8(word).unwrap());
    }

    words
}

fn is_static() -> bool {
    env::var_os("RUBY_STATIC").is_some()
}

fn should_link() -> bool {
    std::env::var_os("NO_LINK_RUTIE").is_none()
        && std::env::var_os("CARGO_FEATURE_NO_LINK").is_none()
}

// A Windows DLL cannot leave symbols to be found when it is loaded, so a
// Ruby extension has to link to the Ruby DLL (the one already loaded into
// `ruby.exe`). The `no-link` feature and `NO_LINK_RUTIE` only apply when no
// Ruby is available, so `cargo check` and `cargo doc` still work without one.
fn should_link_windows() -> bool {
    if should_link() {
        return true;
    }

    match try_rbconfig("RUBY_SO_NAME") {
        Ok(so_name) if !so_name.is_empty() => {
            println!(
                "cargo:warning=Linking to {} although `no-link`/NO_LINK_RUTIE is set: \
                 Windows DLLs (including Ruby extensions) must link to the Ruby DLL.",
                so_name
            );
            true
        }
        _ => false,
    }
}

fn main() {
    println!("cargo:rustc-check-cfg=cfg(rutie_dllimport)");
    ruby_version_cfgs();

    let link = if windows::is_target() {
        should_link_windows()
    } else {
        should_link()
    };

    // Ruby programs calling Rust doesn't need cc linking
    if link {
        if is_static() {
            ci_stderr_log!("RUBY_STATIC is set");
            use_static()
        } else {
            match rbconfig("ENABLE_SHARED").as_str() {
                "no" => use_static(),
                "yes" => use_dylib(),
                _ => {
                    let msg = "Error! Couldn't find a valid value for \
                    RbConfig::CONFIG['ENABLE_SHARED']. \
                    This may mean that your ruby's build config is corrupted. \
                    Possible solution: build a new Ruby with the `--enable-shared` configure opt.";
                    ci_stderr_log!("{}", &msg);
                    panic!("{}", msg)
                }
            }
        }
    }
}
