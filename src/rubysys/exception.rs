use crate::rubysys::types::{c_char, c_int, c_long, Id, Value};

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
    // `IO::TimeoutError` (`ruby/io.h`).
    pub static rb_eIOTimeoutError: Value;

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
    pub fn rb_errno() -> c_int;
    // void
    // rb_errno_set(int err)
    pub fn rb_errno_set(err: c_int);
    // int *
    // rb_errno_ptr(void)
    //
    // The location of the calling thread's `errno`.
    pub fn rb_errno_ptr() -> *mut c_int;
    // void
    // rb_warn(const char *fmt, ...)
    pub fn rb_warn(fmt: *const c_char, ...);
    // void
    // rb_warning(const char *fmt, ...)
    pub fn rb_warning(fmt: *const c_char, ...);
}

// `rb_warning_category_t` values.
pub const RB_WARN_CATEGORY_NONE: c_int = 0;
pub const RB_WARN_CATEGORY_DEPRECATED: c_int = 1;
pub const RB_WARN_CATEGORY_EXPERIMENTAL: c_int = 2;
pub const RB_WARN_CATEGORY_PERFORMANCE: c_int = 3;
pub const RB_WARN_CATEGORY_STRICT_UNUSED_BLOCK: c_int = 4;
// The categories enabled by default, and all of them, as bit masks.
pub const RB_WARN_CATEGORY_DEFAULT_BITS: c_int =
    (1 << RB_WARN_CATEGORY_DEPRECATED) | (1 << RB_WARN_CATEGORY_EXPERIMENTAL);
pub const RB_WARN_CATEGORY_ALL_BITS: c_int = (1 << RB_WARN_CATEGORY_DEPRECATED)
    | (1 << RB_WARN_CATEGORY_EXPERIMENTAL)
    | (1 << RB_WARN_CATEGORY_PERFORMANCE)
    | (1 << RB_WARN_CATEGORY_STRICT_UNUSED_BLOCK);

// `enum rb_io_wait_readwrite` values.
pub const RB_IO_WAIT_READABLE: c_int = 0;
pub const RB_IO_WAIT_WRITABLE: c_int = 1;

// The printf-style functions below take a format: never pass untrusted text
// as `fmt` (use `"%s"` and pass the text as an argument).
#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // void
    // rb_bug_errno(const char *msg, int err)
    //
    // Aborts the process with a bug report. Library code must never call it.
    pub fn rb_bug_errno(message: *const c_char, errno: c_int) -> !;
    // void
    // rb_category_compile_warn(rb_warning_category_t cat, const char *file, int line,
    //                          const char *fmt, ...)
    pub fn rb_category_compile_warn(
        category: c_int,
        file: *const c_char,
        line: c_int,
        fmt: *const c_char,
        ...
    );
    // void
    // rb_category_warn(rb_warning_category_t cat, const char *fmt, ...)
    //
    // Prints unless `$VERBOSE` is `nil` and the category is enabled
    // (`Warning[category]`).
    pub fn rb_category_warn(category: c_int, fmt: *const c_char, ...);
    // void
    // rb_category_warning(rb_warning_category_t cat, const char *fmt, ...)
    //
    // Prints only when `$VERBOSE` is `true` and the category is enabled.
    pub fn rb_category_warning(category: c_int, fmt: *const c_char, ...);
    // void
    // rb_compile_warn(const char *file, int line, const char *fmt, ...)
    //
    // Prints `file:line: warning: ...` unless `$VERBOSE` is `nil`.
    pub fn rb_compile_warn(file: *const c_char, line: c_int, fmt: *const c_char, ...);
    // void
    // rb_compile_warning(const char *file, int line, const char *fmt, ...)
    //
    // `rb_compile_warn` that prints only when `$VERBOSE` is `true`.
    pub fn rb_compile_warning(file: *const c_char, line: c_int, fmt: *const c_char, ...);
    // void
    // rb_fatal(const char *fmt, ...)
    //
    // Raises `fatal`, which `rescue` cannot catch.
    pub fn rb_fatal(fmt: *const c_char, ...) -> !;
    // void
    // rb_mod_sys_fail(VALUE mod, const char *msg)
    //
    // Uses `errno`, and calls `rb_bug` (aborting the process) when it is `0`.
    pub fn rb_mod_sys_fail(module: Value, message: *const c_char) -> !;
    // void
    // rb_mod_sys_fail_str(VALUE mod, VALUE msg)
    //
    // Uses `errno`, and calls `rb_bug` (aborting the process) when it is `0`.
    pub fn rb_mod_sys_fail_str(module: Value, message: Value) -> !;
    // void
    // rb_mod_syserr_fail_str(VALUE mod, int err, VALUE msg)
    //
    // Raises the `SystemCallError` for `err`, extended with `mod`.
    pub fn rb_mod_syserr_fail_str(module: Value, errno: c_int, message: Value) -> !;
    // void
    // rb_readwrite_sys_fail(enum rb_io_wait_readwrite waiting, const char *msg)
    //
    // Uses `errno`, and calls `rb_bug` (aborting the process) when it is `0`.
    pub fn rb_readwrite_sys_fail(waiting: c_int, message: *const c_char) -> !;
    // void
    // rb_readwrite_syserr_fail(enum rb_io_wait_readwrite waiting, int err, const char *msg)
    //
    // Raises the `SystemCallError` for `err` extended with `IO::WaitReadable`
    // or `IO::WaitWritable` (`IO::EAGAINWaitReadable` for `EAGAIN`, ...).
    pub fn rb_readwrite_syserr_fail(waiting: c_int, errno: c_int, message: *const c_char) -> !;
    // VALUE *
    // rb_ruby_debug_ptr(void)
    //
    // The storage of `$DEBUG` for the current Ractor.
    pub fn rb_ruby_debug_ptr() -> *mut Value;
    // VALUE *
    // rb_ruby_verbose_ptr(void)
    //
    // The storage of `$VERBOSE` for the current Ractor.
    pub fn rb_ruby_verbose_ptr() -> *mut Value;
    // void
    // rb_sys_fail_str(VALUE msg)
    //
    // Uses `errno`, and calls `rb_bug` (aborting the process) when it is `0`.
    pub fn rb_sys_fail_str(message: Value) -> !;
    // void
    // rb_sys_warning(const char *fmt, ...)
    //
    // Warns (when `$VERBOSE` is `true`) with the message for `errno`.
    pub fn rb_sys_warning(fmt: *const c_char, ...);
    // void
    // rb_syserr_fail_str(int err, VALUE msg)
    pub fn rb_syserr_fail_str(errno: c_int, message: Value) -> !;
    // VALUE
    // rb_syserr_new_str(int n, VALUE arg)
    pub fn rb_syserr_new_str(errno: c_int, message: Value) -> Value;
    // void
    // rb_unexpected_type(VALUE self, int t)
    //
    // Raises `TypeError` ("wrong argument type X (expected Y)").
    pub fn rb_unexpected_type(object: Value, value_type: c_int) -> !;
    // void
    // rb_check_copyable(VALUE obj, VALUE orig)
    //
    // Raises `FrozenError` if `obj` is frozen (for `initialize_copy`).
    pub fn rb_check_copyable(object: Value, original: Value);
    // void
    // rb_error_frozen(const char *what)
    //
    // Raises `FrozenError` ("can't modify frozen <what>").
    pub fn rb_error_frozen(what: *const c_char) -> !;
    // void
    // rb_invalid_str(const char *str, const char *type)
    //
    // Raises `ArgumentError` ("invalid value for <type>: <str>").
    pub fn rb_invalid_str(string: *const c_char, type_name: *const c_char) -> !;
    // void
    // rb_loaderror(const char *fmt, ...)
    pub fn rb_loaderror(fmt: *const c_char, ...) -> !;
    // void
    // rb_loaderror_with_path(VALUE path, const char *fmt, ...)
    //
    // Raises `LoadError` whose `path` is `path`.
    pub fn rb_loaderror_with_path(path: Value, fmt: *const c_char, ...) -> !;
    // void
    // rb_name_error(ID name, const char *fmt, ...)
    //
    // Raises `NameError` whose `name` is `name`.
    pub fn rb_name_error(name: Id, fmt: *const c_char, ...) -> !;
    // void
    // rb_name_error_str(VALUE name, const char *fmt, ...)
    pub fn rb_name_error_str(name: Value, fmt: *const c_char, ...) -> !;
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
