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
    // rb_ext_ractor_safe(bool flag)
    //
    // Sets whether C methods defined afterwards on this thread may run in
    // any Ractor; `require` resets it for each extension it loads.
    pub fn rb_ext_ractor_safe(flag: bool);
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
    // void
    // rb_clear_constant_cache_for_id(ID id)
    //
    // Invalidates the inline caches of constant lookups for the name `id`.
    pub fn rb_clear_constant_cache_for_id(id: Id);
    // void *
    // rb_ext_resolve_symbol(const char *feature, const char *symbol)
    //
    // The address of `symbol` in the native extension loaded for `feature`
    // (such as `"json/ext/parser"`); null when the feature is not loaded, is
    // not a native extension, or has no such symbol.
    pub fn rb_ext_resolve_symbol(feature: *const c_char, symbol: *const c_char) -> *mut c_void;
}

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // Whether Ruby frees all of its memory when the VM shuts down (the
    // `RUBY_FREE_AT_EXIT` environment variable, for memory checkers). Ruby
    // 3.4+.
    //
    // bool
    // ruby_free_at_exit_p(void)
    pub fn ruby_free_at_exit_p() -> bool;
    // Raises the `NoMemoryError`-style "malloc: possible integer overflow"
    // error for `x + y`. Ruby 3.4+.
    //
    // void
    // ruby_malloc_add_size_overflow(size_t x, size_t y)
    pub fn ruby_malloc_add_size_overflow(x: size_t, y: size_t) -> !;
    // Reports a failed `RUBY_ASSERT` with a printf-style detail message and
    // aborts. Ruby 3.4+.
    //
    // void
    // rb_assert_failure_detail(const char *file, int line, const char *name,
    //                          const char *expr, const char *fmt, ...)
    pub fn rb_assert_failure_detail(
        file: *const c_char,
        line: c_int,
        name: *const c_char,
        expr: *const c_char,
        fmt: *const c_char,
        ...
    ) -> !;
}

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // void
    // rb_backtrace(void)
    //
    // Prints the current backtrace to `$stderr`.
    pub fn rb_backtrace();
    // VALUE
    // rb_f_notimplement(int argc, const VALUE *argv, VALUE obj, VALUE marker)
    //
    // Never returns (raises `NotImplementedError`). Defining a method with
    // this function as its body (arity -1) makes `respond_to?` false for it.
    pub fn rb_f_notimplement(argc: Argc, argv: *const Value, object: Value, marker: Value)
        -> Value;
    // int
    // rb_frame_method_id_and_class(ID *idp, VALUE *klassp)
    //
    // `0` outside a method; either pointer may be null.
    pub fn rb_frame_method_id_and_class(id: *mut Id, klass: *mut Value) -> c_int;
    // VALUE
    // rb_make_backtrace(void)
    pub fn rb_make_backtrace() -> Value;
    // const char *
    // rb_sourcefile(void)
    //
    // Null outside Ruby code.
    pub fn rb_sourcefile() -> *const c_char;
    // int
    // rb_sourceline(void)
    pub fn rb_sourceline() -> c_int;
    // VALUE
    // rb_call_super_kw(int argc, const VALUE *argv, int kw_splat)
    pub fn rb_call_super_kw(argc: Argc, argv: *const Value, kw_splat: c_int) -> Value;
    // VALUE
    // rb_current_receiver(void)
    //
    // Raises `RuntimeError` outside a method.
    pub fn rb_current_receiver() -> Value;
    // VALUE
    // rb_eval_string_wrap(const char *str, int *state)
    //
    // `rb_eval_string_protect` under an anonymous module, like
    // `load(file, true)`. The exception stays in `$!` on failure.
    pub fn rb_eval_string_wrap(string: *const c_char, state: *mut c_int) -> Value;
    // VALUE
    // rb_extract_keywords(VALUE *orighash)
    //
    // Returns a new hash of the Symbol-keyed entries of `*orighash` (or `0`
    // when there are none), and sets `*orighash` to a new hash of the other
    // entries (or `0` when there are none). The hash itself is unchanged.
    pub fn rb_extract_keywords(original: *mut Value) -> Value;
    // VALUE
    // rb_funcall(VALUE recv, ID mid, int n, ...)
    //
    // The variadic arguments are `n` `VALUE`s.
    pub fn rb_funcall(receiver: Value, method: Id, n: c_int, ...) -> Value;
    // VALUE
    // rb_funcall_passing_block(VALUE recv, ID mid, int argc, const VALUE *argv)
    //
    // Passes the block given to the method running.
    pub fn rb_funcall_passing_block(
        receiver: Value,
        method: Id,
        argc: Argc,
        argv: *const Value,
    ) -> Value;
    // VALUE
    // rb_funcall_passing_block_kw(VALUE recv, ID mid, int argc, const VALUE *argv, int kw_splat)
    pub fn rb_funcall_passing_block_kw(
        receiver: Value,
        method: Id,
        argc: Argc,
        argv: *const Value,
        kw_splat: c_int,
    ) -> Value;
    // VALUE
    // rb_funcall_with_block_kw(VALUE recv, ID mid, int argc, const VALUE *argv, VALUE procval, int kw_splat)
    pub fn rb_funcall_with_block_kw(
        receiver: Value,
        method: Id,
        argc: Argc,
        argv: *const Value,
        procval: Value,
        kw_splat: c_int,
    ) -> Value;
    // VALUE
    // rb_funcallv_public_kw(VALUE recv, ID mid, int argc, const VALUE *argv, int kw_splat)
    pub fn rb_funcallv_public_kw(
        receiver: Value,
        method: Id,
        argc: Argc,
        argv: *const Value,
        kw_splat: c_int,
    ) -> Value;
    // VALUE
    // rb_f_exit(int argc, const VALUE *argv)
    //
    // `Kernel#exit`: never returns (raises `SystemExit`).
    pub fn rb_f_exit(argc: Argc, argv: *const Value) -> Value;
    // ID
    // rb_frame_callee(void)
    //
    // The name the method running was called by (an alias); `0` outside a
    // method.
    pub fn rb_frame_callee() -> Id;
    // ID
    // rb_frame_this_func(void)
    //
    // The original name of the method running; `0` outside a method.
    pub fn rb_frame_this_func() -> Id;
    // VALUE
    // rb_make_exception(int argc, const VALUE *argv)
    //
    // Builds the exception `raise(*argv)` would raise; `nil` for no
    // arguments.
    pub fn rb_make_exception(argc: Argc, argv: *const Value) -> Value;
    // void
    // rb_obj_call_init_kw(VALUE, int, const VALUE*, int)
    pub fn rb_obj_call_init_kw(object: Value, argc: Argc, argv: *const Value, kw_splat: c_int);
    // VALUE
    // rb_block_call_kw(VALUE obj, ID mid, int argc, const VALUE *argv,
    //                  rb_block_call_func_t proc, VALUE data2, int kw_splat)
    pub fn rb_block_call_kw(
        obj: Value,
        method_id: Id,
        argc: Argc,
        argv: *const Value,
        block: BlockCallFunction,
        outer_scope: Value,
        kw_splat: c_int,
    ) -> Value;
    // VALUE
    // rb_each(VALUE obj)
    //
    // Calls `obj.each` with the block given to the method running.
    pub fn rb_each(object: Value) -> Value;
    // VALUE
    // rb_yield_block(RB_BLOCK_CALL_FUNC_ARGLIST(yielded_arg, callback_arg))
    //
    // A block function (for `rb_block_call`) that yields its arguments to
    // the block of the method running.
    pub fn rb_yield_block(
        yielded_arg: Value,
        callback_arg: Value,
        argc: c_int,
        argv: *const Value,
        block_arg: Value,
    ) -> Value;
    // VALUE
    // rb_yield_splat_kw(VALUE ary, int kw_splat)
    pub fn rb_yield_splat_kw(values: Value, kw_splat: c_int) -> Value;
    // VALUE
    // rb_yield_values(int n, ...)
    //
    // The variadic arguments are `n` `VALUE`s.
    pub fn rb_yield_values(n: c_int, ...) -> Value;
    // VALUE
    // rb_yield_values_kw(int n, const VALUE *argv, int kw_splat)
    pub fn rb_yield_values_kw(argc: Argc, argv: *const Value, kw_splat: c_int) -> Value;
    // VALUE
    // rb_get_argv(void)
    //
    // `ARGV`.
    pub fn rb_get_argv() -> Value;
    // void *
    // rb_load_file(const char *file)
    //
    // Parses (does not run) a script, returning the node for
    // `ruby_exec_node`, or null after printing the syntax error.
    pub fn rb_load_file(file: *const c_char) -> *mut c_void;
    // void *
    // rb_load_file_str(VALUE file)
    pub fn rb_load_file_str(file: Value) -> *mut c_void;
}
