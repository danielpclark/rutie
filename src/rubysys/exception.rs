use crate::rubysys::types::{c_char, c_int, Value};

extern "C" {
    pub static rb_eArgError: Value;
    pub static rb_eException: Value;
    pub static rb_eFrozenError: Value;
    pub static rb_eLocalJumpError: Value;
    pub static rb_eNotImpError: Value;
    pub static rb_eRuntimeError: Value;
    pub static rb_eStandardError: Value;
    pub static rb_eTypeError: Value;
    pub static rb_eZeroDivError: Value;

    // void
    // rb_check_frozen(VALUE obj)
    pub fn rb_check_frozen(object: Value);
    // void
    // rb_check_type(VALUE x, int t)
    pub fn rb_check_type(object: Value, value_type: c_int);
    // void
    // rb_error_arity(int argc, int min, int max)
    pub fn rb_error_arity(argc: c_int, min: c_int, max: c_int) -> !;
    // void
    // rb_error_frozen_object(VALUE frozen_obj)
    pub fn rb_error_frozen_object(object: Value) -> !;
    // VALUE
    // rb_exc_new_str(VALUE etype, VALUE str)
    pub fn rb_exc_new_str(exception_class: Value, message: Value) -> Value;
    // void
    // rb_frozen_error_raise(VALUE frozen_obj, const char *fmt, ...)
    //
    // Ruby 2.7 and later only.
    #[cfg(ruby_gte_2_7)]
    pub fn rb_frozen_error_raise(object: Value, fmt: *const c_char, ...) -> !;
    // void
    // rb_notimplement(void)
    pub fn rb_notimplement() -> !;
    // void
    // rb_num_zerodiv(void)
    pub fn rb_num_zerodiv() -> !;
    // void
    // rb_sys_fail(const char *mesg)
    //
    // Calls `rb_bug` (aborting the process) when `errno` is `0`; prefer
    // `rb_syserr_fail` with an explicit error number.
    pub fn rb_sys_fail(message: *const c_char) -> !;
    // void
    // rb_syserr_fail(int e, const char *mesg)
    pub fn rb_syserr_fail(errno: c_int, message: *const c_char) -> !;
    // VALUE
    // rb_syserr_new(int n, const char *mesg)
    pub fn rb_syserr_new(errno: c_int, message: *const c_char) -> Value;
    // void
    // rb_warn(const char *fmt, ...)
    pub fn rb_warn(fmt: *const c_char, ...);
    // void
    // rb_warning(const char *fmt, ...)
    pub fn rb_warning(fmt: *const c_char, ...);
}
