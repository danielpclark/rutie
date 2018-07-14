use rubysys::types::{CallbackPtr, c_char, c_int, c_void, Value, Id, Argc};

extern "C" {
    // void
    // ruby_init(void)
    pub fn ruby_init();
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
    // void
    // rb_exc_raise(VALUE mesg)
    pub fn rb_exc_raise(exception: Value);
    // void
    // rb_raise(VALUE exc, const char *fmt, ...)
    pub fn rb_raise(exception: Value, message: *const c_char);
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
    pub fn rb_funcallv_public(receiver: Value, method: Id, argc: Argc, argv: *const Value) -> Value;
    // VALUE
    // rb_block_call(VALUE obj, ID mid, int argc, const VALUE * argv,
    //               VALUE (*bl_proc) (ANYARGS), VALUE data2)
    pub fn rb_block_call(obj: Value, method_id: Id, argc: Argc, argv: *const Value,
                         block: extern fn(Value, Value, Argc, *const Value) -> Value,
                         outer_scope: Value) -> Value;
}
