extern crate pkg_config;
use std::env;
use std::ffi::{OsStr, OsString};
use std::process::Command;
use std::path::{Path};

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

fn use_libdir() {
    println!("cargo:rustc-link-search={}", rbconfig("libdir"));
}

fn transform_lib_args(rbconfig_key: &str, replacement: &str) -> String {
    rbconfig(rbconfig_key).replace("-l", replacement)
}

// returns a list of the current systems executable paths
fn path() -> Vec<String> {
    env::var_os("PATH").unwrap_or(OsString::new()).
        to_string_lossy().split(':').map(|s| s.into()).collect()
}

fn rvm_path() -> Option<String> {
    for p in path() {
        if p.contains("rvm/bin") {
            return Some(p[0..p.len()-4].to_string())
        }
    }
    return None;
}

fn rvm_libruby_static_path() -> Option<String> {
    rvm_path().map(|pth|
        format!("{}/src/ruby-{}", pth, rbconfig("RUBY_PROGRAM_VERSION"))
    )
}

fn use_static() {
    // Ruby gives back the libs in the form: `-lpthread -lgmp`
    // Cargo wants them as: `-l pthread -l gmp`
    println!("cargo:rustc-flags={}", transform_lib_args("LIBS", "-l "));

    // Ruby removed libruby-static.a by default in https://bugs.ruby-lang.org/issues/12845
    // so we'll have to check known locations based on which ruby version manager
    // is in use or default install.
    let static_ruby_location: String = rvm_libruby_static_path().
        unwrap_or(rbconfig("libdir"));
    println!("cargo:rustc-link-search={}", static_ruby_location);

    let ruby_static = rbconfig("LIBRUBY_A");
    let ruby_static = &ruby_static[3..ruby_static.len() - 2];

    println!("cargo:rustc-link-lib={}", ruby_static);
    ci_stderr_log!("Using static linker flags");
}

fn use_dylib() {
    use_libdir();
    println!("cargo:rustc-link-lib=dylib={}", rbconfig("RUBY_SO_NAME"));
    ci_stderr_log!("Using dynamic linker flags");
}

fn main() {
    // Ruby programs calling Rust don't need cc linking
    if let None = std::env::var_os("NO_LINK_RUTIE") {

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

        if rbconfig("target_os") != "mingw32" && env::var_os("RUBY_STATIC").is_some() {
            ci_stderr_log!("Not mingw && RUBY_STATIC exists");
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
