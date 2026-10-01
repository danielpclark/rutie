// When Rutie links a static Ruby, this executable must export libruby's
// functions to Ruby's own extensions (`enc/encdb` is loaded at boot). Rutie
// publishes the linker flag for that as `DEP_RUBY_LINK_ARG`, since its build
// script cannot pass linker arguments to crates that depend on it.
fn main() {
    if let Ok(arg) = std::env::var("DEP_RUBY_LINK_ARG") {
        println!("cargo:rustc-link-arg={}", arg);
    }
}
