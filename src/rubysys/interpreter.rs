// Embedding the interpreter: the parts of `ruby/internal/interpreter.h`,
// `ruby/vm.h` and `ruby/assert.h` not in `vm.rs`. Most of these end the
// process or tear the VM down.

use crate::rubysys::types::{c_char, c_int, c_void, VmPointer};

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // void
    // rb_assert_failure(const char *file, int line, const char *name, const char *expr)
    //
    // Reports a failed `RUBY_ASSERT` (`name` is the function, or NULL) and
    // aborts the process.
    pub fn rb_assert_failure(
        file: *const c_char,
        line: c_int,
        name: *const c_char,
        expr: *const c_char,
    ) -> !;
    // void *
    // ruby_process_options(int argc, char **argv)
    //
    // Like `ruby_options`, but raises on errors instead of returning a node
    // that tells the caller to exit. Returns the compiled script for
    // `ruby_run_node`.
    pub fn ruby_process_options(argc: c_int, argv: *mut *mut c_char) -> *mut c_void;
    // void
    // ruby_show_copyright(void)
    //
    // Prints `ruby -v`'s copyright line to stdout.
    pub fn ruby_show_copyright();
    // void
    // ruby_show_version(void)
    //
    // Prints the version (`ruby -v`) to stdout.
    pub fn ruby_show_version();
    // void
    // ruby_sig_finalize(void)
    //
    // Removes Ruby's own signal handlers, for when the program goes on after
    // the VM is finished with.
    pub fn ruby_sig_finalize();
    // void
    // ruby_stop(int)
    //
    // `ruby_cleanup` and `exit(3)`.
    pub fn ruby_stop(ex: c_int) -> !;
    // int
    // ruby_vm_destruct(ruby_vm_t *vm)
    //
    // Destroys a VM (as `ruby_cleanup` does); returns 0.
    pub fn ruby_vm_destruct(vm: VmPointer) -> c_int;
}
