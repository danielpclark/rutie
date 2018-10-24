extern crate pkg_config;
use std::ffi::{OsStr, OsString};
use std::process::Command;
use std::path::Path;
use std::env;

#[cfg(target_os = "windows")]
use std::fs;

macro_rules! ci_stderr_log {
    () => (eprint!("\n"));
    ($($arg:tt)*) => ({
        if env::var_os("CI_STDERR_LOG").is_some() { eprintln!($($arg)*) }
    })
}

fn rbconfig(key: &str) -> String {
    let ruby = match env::var_os("RUBY") {
        Some(val) => val.to_os_string(),
        None => OsStr::new("ruby").to_os_string(),
    };
    let config = Command::new(ruby)
        .arg("-e")
        .arg(format!("print RbConfig::CONFIG['{}']", key))
        .output()
        .unwrap_or_else(|e| panic!("ruby not found: {}", e));

    String::from_utf8(config.stdout).expect("RbConfig value not UTF-8!")
}

fn ruby_name() -> OsString {
    match env::var_os("LIBRUBY_NAME") {
        Some(name) => name,
        None => OsString::from("ruby"),
    }
}

fn set_env_pkg_config() {
    if env::var_os("PKG_CONFIG_PATH").is_none() {
        let key = "PKG_CONFIG_PATH";
        let value = Path::new(&rbconfig("libdir")).join("pkgconfig");
        std::env::set_var(key, &value);
        ci_stderr_log!("Set PKG_CONFIG_PATH to {:?}", value);
    }
}

fn trim_teeny(version: &str) -> &str {
    version.rsplitn(2, '.').collect::<Vec<&str>>().last().unwrap()
}

fn ruby_version() -> String {
    rbconfig("RUBY_PROGRAM_VERSION")
}

fn transform_lib_args(rbconfig_key: &str, replacement: &str) -> String {
    rbconfig(rbconfig_key).replace("-l", replacement)
}

// returns a list of the current systems executable paths
#[cfg(not(target_os = "windows"))]
fn path() -> Vec<String> {
    env::var_os("PATH").unwrap_or(OsString::new()).
        to_string_lossy().split(':').map(|s| s.into()).collect()
}

#[cfg(not(target_os = "windows"))]
fn rvm_path() -> Option<String> {
    for p in path() {
        if p.contains("rvm/bin") {
            return Some(p[0..p.len()-4].to_string())
        }
    }
    return None;
}

#[cfg(not(target_os = "windows"))]
fn rvm_libruby_static_path() -> Option<String> {
    let pth = rvm_path();
    if pth.is_none() { return None; }

    let path = format!("{}/src/ruby-{}", pth.unwrap(), rbconfig("RUBY_PROGRAM_VERSION"));

    if !Path::new(&path).exists() { return None; }
    if !Path::new(&path).join("libruby-static.a").exists() { return None; }

    Some(path)
}

#[cfg(not(target_os = "windows"))]
fn static_ruby_file_name() -> String {
    "libruby-static.a".to_string()
}

#[cfg(target_os = "windows")]
fn static_ruby_file_name() -> String {
    let libruby_so = rbconfig("LIBRUBY_SO");
    let ruby_dll = Path::new(&libruby_so);
    let name = ruby_dll.file_stem().unwrap();
    format!("{}.lib", name.to_string_lossy())
}

#[cfg(not(target_os = "windows"))]
fn static_ruby_location() -> String {
    let location: Option<String> = env::var_os("RUBY_STATIC_PATH").map(|s|s.to_string_lossy().to_string());
    let location: String = location.unwrap_or(rvm_libruby_static_path().unwrap_or(rbconfig("libdir")));

    if !Path::new(&location).join(static_ruby_file_name()).exists() {
        panic!("{} was not found in path but static build was chosen.\n\
               Please use environment variable RUBY_STATIC_PATH to define where {} is located.",
               static_ruby_file_name(), static_ruby_file_name());
    }

    location
}

#[cfg(target_os = "windows")]
fn static_ruby_location() -> String {
    let location: Option<String> = env::var_os("RUBY_STATIC_PATH").map(|s|s.to_string_lossy().to_string());
    let location: String = location.unwrap_or(
      Path::new("target").join(env::var_os("PROFILE").unwrap()).join("deps").to_string_lossy().to_string()
    );

    location
}

#[cfg(not(target_os = "windows"))]
fn windows_static_ruby_dep() {}

// Windows needs ligmp-10.dll as gmp.lib
#[cfg(target_os = "windows")]
fn windows_static_ruby_dep() {
    Command::new("build/windows/vcbuild.cmd")
        .arg("-arch=x64")
        .arg("-host_arch=x64")
        .arg("&&")
        .arg("dumpbin")
        .arg("/exports")
        .arg("/out:exports.txt")
        .arg(format!("{}/ruby_builtin_dlls/libgmp-10.dll", rbconfig("bindir")))
        .output()
        .unwrap();

    Command::new("build/windows/exports.bat").output().unwrap();

    let deps_dir = Path::new("target").join(env::var_os("PROFILE").unwrap()).join("deps");

    Command::new("build/windows/vcbuild.cmd")
        .arg("-arch=x64")
        .arg("-host_arch=x64")
        .arg("&&")
        .arg("lib")
        .arg("/def:exports.def")
        .arg("/name:gmp")
        .arg(format!("/libpath:{}/ruby_builtin_dlls", rbconfig("bindir")))
        .arg("/machine:x64")
        .arg(format!("/out:{}/gmp.lib", deps_dir.to_string_lossy()))
        .output()
        .unwrap();

    fs::remove_file("exports.def").expect("couldn't remove exports.def");
    fs::remove_file("exports.txt").expect("couldn't remove exports.txt");
}

fn use_static() {
    // Ruby removed libruby-static.a by default in https://bugs.ruby-lang.org/issues/12845
    // so we'll have to check known locations based on which ruby version manager
    // is in use or default install.
    println!("cargo:rustc-link-search={}", static_ruby_location());
    let static_name = static_ruby_file_name();
    let static_name = Path::new(&static_name).file_stem().unwrap().to_string_lossy();
    println!("cargo:rustc-link-lib={}", static_name);

    // If Windows
    windows_static_ruby_dep();

    // Ruby gives back the libs in the form: `-lpthread -lgmp`
    // Cargo wants them as: `-l pthread -l gmp`
    // **Flags must be last in order for linking!**
    println!("cargo:rustc-flags={}", transform_lib_args("LIBS", "-l "));

    ci_stderr_log!("Using static linker flags");
}

fn use_dylib() {
    println!("cargo:rustc-link-search={}", rbconfig("libdir"));
    println!("cargo:rustc-link-lib=dylib={}", rbconfig("RUBY_SO_NAME"));
    ci_stderr_log!("Using dynamic linker flags");
}

#[cfg(target_os = "windows")]
fn delete<'a>(s: &'a str, from: &'a str) -> String {
    let mut result = String::new();
    let mut last_end = 0;
    for (start, part) in s.match_indices(from) {
        result.push_str(unsafe { s.get_unchecked(last_end..start) });
        last_end = start + part.len();
    }
    result.push_str(unsafe { s.get_unchecked(last_end..s.len()) });
    result
}

#[cfg(target_os = "windows")]
fn purge_refptr_text() {
    let buffer = fs::read_to_string("exports.def")
        .expect("Failed to read 'exports.def'");
    fs::write("exports.def", delete(&buffer, ".refptr."))
        .expect("Failed to write update to 'exports.def'");
}

#[cfg(target_os = "windows")]
fn windows_support() {
    println!("cargo:rustc-link-search={}", rbconfig("bindir"));
    let mingw_libs: OsString = env::var_os("MINGW_LIBS").unwrap_or(
        OsString::from(format!("{}/ruby_builtin_dlls", rbconfig("bindir")))
    );
    println!("cargo:rustc-link-search={}", mingw_libs.to_string_lossy());

    let deps_dir = Path::new("target").join(env::var_os("PROFILE").unwrap()).join("deps");
    let libruby_so = rbconfig("LIBRUBY_SO");
    let ruby_dll = Path::new(&libruby_so);
    let name = ruby_dll.file_stem().unwrap();
    let target = deps_dir.join(format!("{}.lib", name.to_string_lossy()));

    Command::new("build/windows/vcbuild.cmd")
        .arg("-arch=x64")
        .arg("-host_arch=x64")
        .arg("&&")
        .arg("dumpbin")
        .arg("/exports")
        .arg("/out:exports.txt")
        .arg(Path::new(&rbconfig("bindir")).join(&libruby_so))
        .output()
        .unwrap();

    Command::new("build/windows/exports.bat").output().unwrap();

    purge_refptr_text();
   
    Command::new("build/windows/vcbuild.cmd")
        .arg("-arch=x64")
        .arg("-host_arch=x64")
        .arg("&&")
        .arg("lib")
        .arg("/def:exports.def")
        .arg(format!("/name:{}", name.to_string_lossy()))
        .arg(format!("/libpath:{}", rbconfig("bindir")))
        .arg("/machine:x64")
        .arg(format!("/out:{}", target.to_string_lossy()))
        .output()
        .unwrap();

    fs::remove_file("exports.def").expect("couldn't remove exports.def");
    fs::remove_file("exports.txt").expect("couldn't remove exports.txt");
}

#[cfg(not(target_os = "windows"))]
fn windows_support() {}

fn main() {
    // Ruby programs calling Rust doesn't need cc linking
    if let None = std::env::var_os("NO_LINK_RUTIE") {

        // If windows OS do windows stuff
        windows_support();

        if env::var_os("RUTIE_NO_PKG_CONFIG").is_none() && env::var_os("RUBY_STATIC").is_none() {
            // Ruby often includes pkgconfig under their lib dir
            set_env_pkg_config();

            let ruby_version = ruby_version();
            let version = trim_teeny(&ruby_version);

            let ruby_name = ruby_name();
            let name = ruby_name.to_str().unwrap_or("ruby");

            // To disable the use of pkg-config set the environment variable `RUTIE_NO_PKG_CONFIG`
            match pkg_config::Config::new().atleast_version(version).probe(name) {
                Ok(_) => {
                    ci_stderr_log!("pkg-config is being used");
                    return;
                },
                Err(err) => ci_stderr_log!("{:?}", err),
            }
        }

        // if rbconfig("target_os") != "mingw32" && env::var_os("RUBY_STATIC").is_some() {
        if env::var_os("RUBY_STATIC").is_some() {
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
                    panic!(msg)
                }
            }
        }
    }
}
