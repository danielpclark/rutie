use rubysys::types::{CallbackPtr, c_char, c_int, c_void, Value};

extern "C" {
    pub fn ruby_init();
    pub fn rb_block_proc() -> Value;
    pub fn rb_block_given_p() -> c_int;
    pub fn rb_errinfo() -> Value;
    pub fn rb_eval_string(string: *const c_char) -> Value;
    pub fn rb_eval_string_protect(string: *const c_char, state: *mut c_int) -> Value;
    pub fn rb_exc_raise(exception: Value);
    pub fn rb_raise(exception: Value, message: *const c_char);
    pub fn rb_require(name: *const c_char) -> Value;
    pub fn rb_set_errinfo(err: Value);
    pub fn rb_protect(func: CallbackPtr, args: *const c_void, state: *mut c_int) -> Value;

    #[deprecated(since="0.2.18",
        note="Use `thread::rb_thread_call_without_gvl()` instead")]
    pub fn rb_thread_call_without_gvl(func: CallbackPtr,
                                      args: *const c_void,
                                      unblock_func: CallbackPtr,
                                      unblock_args: *const c_void)
                                      -> *mut c_void;

    #[deprecated(since="0.2.18",
        note="Use `thread::rb_thread_call_without_gvl2()` instead")]
    pub fn rb_thread_call_without_gvl2(func: CallbackPtr,
                                       args: *const c_void,
                                       unblock_func: CallbackPtr,
                                       unblock_args: *const c_void)
                                       -> *mut c_void;

    #[deprecated(since="0.2.18",
        note="Use `thread::rb_thread_call_with_gvl()` instead")]
    pub fn rb_thread_call_with_gvl(func: CallbackPtr, args: *const c_void) -> *mut c_void;
}
