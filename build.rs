use std::env;
use std::ffi::OsStr;
use std::process::Command;
use std::path::{Path, PathBuf};

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

fn dep(release: &str, file: &str) -> String {
    let dest = Path::new("target");
    dest.join(release).join("deps").join(file).to_str().unwrap().to_string()
}

#[inline]
fn libruby_soname() -> String {
    // Once Rubies earler than the 2.5 series are deprecated
    // we can simply use `rbconfig("LIBRUBY_SONAME")`
    rbconfig("LIBRUBY").rsplitn(2, ".").
      skip(1).next().unwrap().to_owned()
}

#[inline]
fn target_file() -> PathBuf {
    Path::new(&rbconfig("libdir")).join(&libruby_soname())
}

#[cfg(any(unix))]
fn link() {
    use std::os::unix::fs::symlink;
    let destination = dep("debug", &libruby_soname());
    let _ = symlink(target_file(), destination);
}

#[cfg(any(windows))]
fn link() {
    use std::os::windows::fs::symlink_file;
    let destination = dep("debug", &libruby_soname());
    let _ = symlink_file(target_file(), destination);
}

#[cfg(not(any(unix,windows)))]
fn link() {
    use std::fs::{copy, remove_file};
    let destination = dep("debug", &libruby_soname());
    let _ = remove_file(&destination);
    let _ = copy(target_file(), destination);
}

fn use_libdir() {
    println!("cargo:rustc-link-search={}", rbconfig("libdir"));
}

fn transform_lib_args(rbconfig_key: &str, replacement: &str) -> String {
    rbconfig(rbconfig_key).replace("-l", replacement)
}

fn use_static() {
    // Ruby gives back the libs in the form: `-lpthread -lgmp`
    // Cargo wants them as: `-l pthread -l gmp`
    println!("cargo:rustc-flags={}", transform_lib_args("LIBS", "-l "));
}

fn use_dylib() {
    use_libdir();
    println!("cargo:rustc-link-lib=dylib={}", rbconfig("RUBY_SO_NAME"));
}

fn main() {
		if cfg!(feature="test") {
        println!("cargo:rustc-link-search={}", rbconfig("libdir"));
        println!("cargo:rustc-link-lib=dylib={}", rbconfig("RUBY_SO_NAME"));
    } else if rbconfig("target_os") != "mingw32" && env::var_os("RUBY_STATIC").is_some() {
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
                panic!(msg)
            }
        }
    }
    link();
}
