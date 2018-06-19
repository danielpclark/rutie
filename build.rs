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

fn main() {
    if cfg!(feature="test") {
        println!("cargo:rustc-link-search={}", rbconfig("libdir"));
        println!("cargo:rustc-link-lib=dylib={}", rbconfig("RUBY_SO_NAME"));
    }
    link();
}
