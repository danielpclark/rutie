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
}
