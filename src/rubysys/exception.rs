use crate::rubysys::types::{c_char, c_int, c_long, Value};

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // Ruby's built-in exception classes (`RUBY_EXTERN VALUE` in `ruby.h`).
    pub static rb_eArgError: Value;
    pub static rb_eEOFError: Value;
    pub static rb_eEncCompatError: Value;
    pub static rb_eEncodingError: Value;
    pub static rb_eException: Value;
    pub static rb_eFatal: Value;
    pub static rb_eFloatDomainError: Value;
    pub static rb_eFrozenError: Value;
    pub static rb_eIOError: Value;
    pub static rb_eIndexError: Value;
    pub static rb_eInterrupt: Value;
    pub static rb_eKeyError: Value;
    pub static rb_eLoadError: Value;
    pub static rb_eLocalJumpError: Value;
    pub static rb_eMathDomainError: Value;
    pub static rb_eNameError: Value;
    pub static rb_eNoMemError: Value;
    pub static rb_eNoMatchingPatternError: Value;
    pub static rb_eNoMatchingPatternKeyError: Value;
    pub static rb_eNoMethodError: Value;
    pub static rb_eNotImpError: Value;
    pub static rb_eRangeError: Value;
    pub static rb_eRegexpError: Value;
    pub static rb_eRuntimeError: Value;
    pub static rb_eScriptError: Value;
    pub static rb_eSecurityError: Value;
    pub static rb_eSignal: Value;
    pub static rb_eStandardError: Value;
    pub static rb_eStopIteration: Value;
    pub static rb_eSyntaxError: Value;
    pub static rb_eSysStackError: Value;
    pub static rb_eSystemCallError: Value;
    pub static rb_eSystemExit: Value;
    pub static rb_eThreadError: Value;
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
    pub fn rb_frozen_error_raise(object: Value, fmt: *const c_char, ...) -> !;
    // void
    // rb_bug(const char *fmt, ...)
    //
    // Aborts the process with a bug report. Library code must never call it.
    pub fn rb_bug(fmt: *const c_char, ...) -> !;
    // VALUE
    // rb_exc_new(VALUE etype, const char *ptr, long len)
    pub fn rb_exc_new(exception_class: Value, message: *const c_char, len: c_long) -> Value;
    // VALUE
    // rb_exc_new_cstr(VALUE etype, const char *s)
    pub fn rb_exc_new_cstr(exception_class: Value, message: *const c_char) -> Value;
    // void
    // rb_exc_fatal(VALUE mesg)
    //
    // Raises a `fatal` error, which cannot be rescued.
    pub fn rb_exc_fatal(exception: Value) -> !;
    // void
    // rb_interrupt(void)
    pub fn rb_interrupt() -> !;
    // void
    // rb_mod_syserr_fail(VALUE mod, int e, const char *mesg)
    pub fn rb_mod_syserr_fail(module: Value, errno: c_int, message: *const c_char) -> !;
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
    // int
    // rb_errno(void)
    //
    // The calling thread's `errno`.
    #[cfg(ruby_gte_3_3)]
    pub fn rb_errno() -> c_int;
    // void
    // rb_errno_set(int err)
    #[cfg(ruby_gte_3_3)]
    pub fn rb_errno_set(err: c_int);
    // int *
    // rb_errno_ptr(void)
    //
    // The location of the calling thread's `errno`.
    #[cfg(ruby_gte_3_3)]
    pub fn rb_errno_ptr() -> *mut c_int;
    // void
    // rb_warn(const char *fmt, ...)
    pub fn rb_warn(fmt: *const c_char, ...);
    // void
    // rb_warning(const char *fmt, ...)
    pub fn rb_warning(fmt: *const c_char, ...);
}

// Win32 error codes (`GetLastError`) are not `errno` values; Ruby maps them
// for its own system calls with this.
#[cfg(windows)]
#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // int
    // rb_w32_map_errno(DWORD winerr)
    pub fn rb_w32_map_errno(winerr: crate::rubysys::libc::c_ulong) -> c_int;
}
