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
fn target_file() -> PathBuf {
    Path::new(&rbconfig("libdir")).join(&rbconfig("LIBRUBY_SONAME"))
}

#[cfg(any(unix))]
fn link() {
    use std::os::unix::fs::symlink;
    let destination = dep("debug", &rbconfig("LIBRUBY_SONAME"));
    let _ = symlink(target_file(), destination);
}

#[cfg(any(windows))]
fn link() {
    use std::os::windows::fs::symlink_file;
    let destination = dep("debug", &rbconfig("LIBRUBY_SONAME"));
    let _ = symlink_file(target_file(), destination);
}

#[cfg(not(any(unix,windows)))]
fn link() {
    use std::fs::{copy, remove_file};
    let destination = dep("debug", &rbconfig("LIBRUBY_SONAME"));
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
