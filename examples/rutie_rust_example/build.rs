// When Rutie falls back to a static libruby (a Ruby built without a shared
// one), this executable must export libruby's functions to Ruby's extensions.
// Rutie publishes the linker flag for that as `DEP_RUBY_LINK_ARG`, since its
// own build script cannot pass linker arguments to crates that depend on it.
fn main() {
    if let Ok(arg) = std::env::var("DEP_RUBY_LINK_ARG") {
        println!("cargo:rustc-link-arg={}", arg);
    }
}
