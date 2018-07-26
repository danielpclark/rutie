extern crate pkg_config;
use std::env;
use std::ffi::OsStr;
use std::process::Command;
use std::path::{Path};

macro_rules! ci_stderr_log {
    () => (eprint!("\n"));
    ($($arg:tt)*) => ({
        if env::var("CI_STDERR_LOG").is_ok() { eprintln!($($arg)*) }
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

fn use_static() {
    // Ruby gives back the libs in the form: `-lpthread -lgmp`
    // Cargo wants them as: `-l pthread -l gmp`
    println!("cargo:rustc-flags={}", transform_lib_args("LIBS", "-l "));
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

        if let None = env::var_os("RUTIE_NO_PKG_CONFIG") {
            // Ruby often includes pkgconfig under their lib dir
            set_env_pkg_config();

            // To disable the use of pkg-config set the environment variable `RUTIE_NO_PKG_CONFIG`
            match pkg_config::Config::new().atleast_version(trim_teeny(&ruby_version())).probe("ruby") {
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
