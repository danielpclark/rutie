use crate::rubysys::types::{
    c_char, c_int, c_void, size_t, Argc, BlockCallFunction, CallbackMutPtr, CallbackPtr, Id, Value,
    VmPointer,
};

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // RUBY_EXTERN VALUE rb_argv0;
    //
    // The script name given to `ruby_options`; `Qfalse` (0) until it runs.
    pub static rb_argv0: Value;
    // RUBY_EXTERN const char ruby_description[];
    //
    // `RUBY_DESCRIPTION` as Ruby was built (without the ` +JIT` marker a
    // running Ruby adds when MJIT is enabled).
    pub static ruby_description: [c_char; 0];
    // void
    // ruby_init(void)
    pub fn ruby_init();
    // void
    // ruby_init_loadpath(void)
    pub fn ruby_init_loadpath();
    // void
    // ruby_vm_at_exit(void(*func)(ruby_vm_t *))
    pub fn ruby_vm_at_exit(func: VmPointer);
    // VALUE
    // rb_block_proc(void)
    pub fn rb_block_proc() -> Value;
    // int
    // rb_block_given_p(void)
    pub fn rb_block_given_p() -> c_int;
    // VALUE
    // rb_errinfo(void)
    pub fn rb_errinfo() -> Value;
    // VALUE
    // rb_eval_string(const char *str)
    pub fn rb_eval_string(string: *const c_char) -> Value;
    // VALUE
    // rb_eval_string_protect(const char *str, int *pstate)
    pub fn rb_eval_string_protect(string: *const c_char, state: *mut c_int) -> Value;
    // VALUE
    // rb_f_abort(int argc, const VALUE *argv)
    pub fn rb_f_abort(argc: Argc, argv: *const Value) -> Value;
    // //////////////// UNAVAILABLE METHOD ////////////////
    // // VALUE
    // // rb_f_eval(int argc, const VALUE *argv, VALUE self)
    // pub fn rb_f_eval(argc: c_int, argv: *const Value, self_: Value) -> Value;
    // ///////////////// ///////////////// ///////////////
    // void
    // rb_exc_raise(VALUE mesg)
    pub fn rb_exc_raise(exception: Value) -> !;
    // void
    // rb_exit(int status)
    pub fn rb_exit(status: c_int);
    // void
    // rb_raise(VALUE exc, const char *fmt, ...)
    //
    // `fmt` is a printf format; never pass untrusted text as `fmt`.
    pub fn rb_raise(exception: Value, fmt: *const c_char, ...) -> !;
    // VALUE
    // rb_require(const char *fname)
    pub fn rb_require(name: *const c_char) -> Value;
    // void
    // rb_set_errinfo(VALUE err)
    pub fn rb_set_errinfo(err: Value);
    // VALUE
    // rb_protect(VALUE (* proc) (VALUE), VALUE data, int *pstate)
    pub fn rb_protect(func: CallbackPtr, args: *const c_void, state: *mut c_int) -> Value;
    // VALUE
    // rb_funcallv(VALUE recv, ID mid, int argc, const VALUE *argv)
    pub fn rb_funcallv(receiver: Value, method: Id, argc: Argc, argv: *const Value) -> Value;
    // VALUE
    // rb_funcallv_public(VALUE recv, ID mid, int argc, const VALUE *argv)
    pub fn rb_funcallv_public(receiver: Value, method: Id, argc: Argc, argv: *const Value)
        -> Value;
    // VALUE
    // rb_block_call(VALUE obj, ID mid, int argc, const VALUE * argv,
    //               VALUE (*bl_proc) (ANYARGS), VALUE data2)
    pub fn rb_block_call(
        obj: Value,
        method_id: Id,
        argc: Argc,
        argv: *const Value,
        block: BlockCallFunction,
        outer_scope: Value,
    ) -> Value;
    // VALUE
    // rb_yield_splat(VALUE values)
    pub fn rb_yield_splat(values: Value) -> Value;
    // VALUE
    // rb_yield_values2(int n, const VALUE *argv)
    pub fn rb_yield_values2(argc: Argc, argv: *const Value) -> Value;
    // VALUE
    // rb_yield(VALUE val)
    pub fn rb_yield(value: Value) -> Value;
    // VALUE
    // rb_call_super(int argc, const VALUE *argv)
    pub fn rb_call_super(argc: Argc, argv: *const Value) -> Value;
    // VALUE
    // rb_catch(const char *tag, VALUE (*func)(ANYARGS), VALUE data)
    pub fn rb_catch(tag: *const c_char, func: BlockCallFunction, data: Value) -> Value;
    // VALUE
    // rb_catch_obj(VALUE tag, VALUE (*func)(ANYARGS), VALUE data)
    pub fn rb_catch_obj(tag: Value, func: BlockCallFunction, data: Value) -> Value;
    // VALUE
    // rb_ensure(VALUE (*b_proc)(ANYARGS), VALUE data1,
    //           VALUE (*e_proc)(ANYARGS), VALUE data2)
    pub fn rb_ensure(
        body: rutie_callback!(type fn(CallbackMutPtr) -> Value),
        body_data: CallbackMutPtr,
        ensure: rutie_callback!(type fn(CallbackMutPtr) -> Value),
        ensure_data: CallbackMutPtr,
    ) -> Value;
    // VALUE
    // rb_funcall_with_block(VALUE recv, ID mid, int argc, const VALUE *argv, VALUE pass_procval)
    pub fn rb_funcall_with_block(
        receiver: Value,
        method: Id,
        argc: Argc,
        argv: *const Value,
        procval: Value,
    ) -> Value;
    // VALUE
    // rb_funcallv_kw(VALUE recv, ID mid, int argc, const VALUE *argv, int kw_splat)
    //
    // Ruby 2.7 and later only.
    #[cfg(ruby_gte_2_7)]
    pub fn rb_funcallv_kw(
        receiver: Value,
        method: Id,
        argc: Argc,
        argv: *const Value,
        kw_splat: c_int,
    ) -> Value;
    // void
    // rb_iter_break(void)
    pub fn rb_iter_break() -> !;
    // void
    // rb_iter_break_value(VALUE val)
    pub fn rb_iter_break_value(value: Value) -> !;
    // void
    // rb_jump_tag(int state)
    pub fn rb_jump_tag(state: c_int) -> !;
    // int
    // rb_keyword_given_p(void)
    //
    // Ruby 2.7 and later only.
    #[cfg(ruby_gte_2_7)]
    pub fn rb_keyword_given_p() -> c_int;
    // void
    // rb_need_block(void)
    //
    // Raises `LocalJumpError` unless a block was given.
    pub fn rb_need_block();
    // void
    // rb_obj_call_init(VALUE obj, int argc, const VALUE *argv)
    pub fn rb_obj_call_init(object: Value, argc: Argc, argv: *const Value);
    // VALUE
    // rb_rescue(VALUE (* b_proc)(ANYARGS), VALUE data1,
    //           VALUE (* r_proc)(ANYARGS), VALUE data2)
    pub fn rb_rescue(
        body: rutie_callback!(type fn(CallbackMutPtr) -> Value),
        body_data: CallbackMutPtr,
        rescue: rutie_callback!(type fn(CallbackMutPtr, Value) -> Value),
        rescue_data: CallbackMutPtr,
    ) -> Value;
    // VALUE
    // rb_rescue2(VALUE (* b_proc)(ANYARGS), VALUE data1,
    //            VALUE (* r_proc)(ANYARGS), VALUE data2, ...)
    //
    // The variadic arguments are the exception classes to rescue,
    // terminated by a `0` (`Value::from(0)`).
    pub fn rb_rescue2(
        body: rutie_callback!(type fn(CallbackMutPtr) -> Value),
        body_data: CallbackMutPtr,
        rescue: rutie_callback!(type fn(CallbackMutPtr, Value) -> Value),
        rescue_data: CallbackMutPtr,
        ...
    ) -> Value;
    // void
    // rb_set_end_proc(void (*func)(VALUE), VALUE data)
    //
    // `data` is marked by the GC, so it must be a Ruby object or an
    // immediate value, never a raw pointer.
    pub fn rb_set_end_proc(func: rutie_callback!(type fn(Value)), data: Value);
    // void
    // rb_throw(const char *tag, VALUE val)
    pub fn rb_throw(tag: *const c_char, value: Value) -> !;
    // void
    // rb_throw_obj(VALUE tag, VALUE value)
    pub fn rb_throw_obj(tag: Value, value: Value) -> !;
    // int
    // ruby_cleanup(volatile int ex)
    pub fn ruby_cleanup(status: c_int) -> c_int;
    // int
    // ruby_setup(void)
    //
    // `ruby_init` without the `exit(EXIT_FAILURE)` on failure; returns the
    // tag state instead. Does nothing when the VM already exists.
    pub fn ruby_setup() -> c_int;
    // void
    // ruby_finalize(void)
    //
    // Runs the end procs and the finalizers of every object without freeing
    // the VM; no Ruby object may be used afterwards.
    pub fn ruby_finalize();
    // void *
    // ruby_options(int argc, char **argv)
    //
    // Processes `ruby(1)` command line options and compiles the script. Keeps
    // `argv` (for `$0=` and `Process.setproctitle`), which must therefore
    // live until the process exits. Only callable once per process: Ruby
    // 2.7 reads past its builtin table when it loads the prelude again.
    pub fn ruby_options(argc: c_int, argv: *mut *mut c_char) -> *mut c_void;
    // int
    // ruby_executable_node(void *n, int *status)
    pub fn ruby_executable_node(node: *mut c_void, status: *mut c_int) -> c_int;
    // int
    // ruby_exec_node(void *n)
    pub fn ruby_exec_node(node: *mut c_void) -> c_int;
    // int
    // ruby_run_node(void *n)
    //
    // `ruby_exec_node` followed by `ruby_cleanup`.
    pub fn ruby_run_node(node: *mut c_void) -> c_int;
    // void
    // ruby_script(const char *name)
    pub fn ruby_script(name: *const c_char);
    // void
    // ruby_set_script_name(VALUE name)
    pub fn ruby_set_script_name(name: Value);
    // void
    // ruby_set_argv(int argc, char **argv)
    pub fn ruby_set_argv(argc: c_int, argv: *mut *mut c_char);
    // void
    // ruby_prog_init(void)
    pub fn ruby_prog_init();
    // void
    // ruby_init_stack(volatile VALUE *addr)
    pub fn ruby_init_stack(addr: *mut Value);
    // void
    // ruby_sysinit(int *argc, char ***argv)
    //
    // Keeps `argv` like `ruby_options` does. On Windows it also sets up the
    // Win32 layer (standard handles, environment, Winsock) and must run
    // before `ruby_init`.
    pub fn ruby_sysinit(argc: *mut c_int, argv: *mut *mut *mut c_char);
    // int
    // ruby_native_thread_p(void)
    pub fn ruby_native_thread_p() -> c_int;
    // int
    // ruby_stack_check(void)
    pub fn ruby_stack_check() -> c_int;
    // size_t
    // ruby_stack_length(VALUE **p)
    pub fn ruby_stack_length(p: *mut *mut Value) -> size_t;
}
