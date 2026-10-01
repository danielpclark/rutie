use crate::{
    binding::{string, vm},
    rubysys::exception,
    types::{c_char, c_int, Value, ValueType},
    util,
};

// `rb_warn` and friends take a printf format, so messages always go through
// `%s` rather than being used as the format themselves.
const PERCENT_S: &[u8] = b"%s\0";

// Ruby copies C string messages up to the first NUL, and `CString` refuses
// interior NULs, so the message is cut at the first NUL instead of panicking.
fn message_to_cstring(message: &str) -> std::ffi::CString {
    let end = message.find('\0').unwrap_or(message.len());

    util::str_to_cstring(&message[..end])
}

pub fn new(exception_class: Value, message: &str) -> Value {
    let message = string::new_utf8(message);

    unsafe { exception::rb_exc_new_str(exception_class, message) }
}

pub fn check_frozen(object: Value) {
    unsafe { exception::rb_check_frozen(object) }
}

pub fn check_type(object: Value, value_type: ValueType) {
    unsafe { exception::rb_check_type(object, value_type as c_int) }
}

pub fn error_arity(argc: c_int, min: c_int, max: c_int) -> ! {
    unsafe { exception::rb_error_arity(argc, min, max) }
}

pub fn error_frozen_object(object: Value) -> ! {
    unsafe { exception::rb_error_frozen_object(object) }
}

pub fn num_zerodiv() -> ! {
    unsafe { exception::rb_num_zerodiv() }
}

pub fn not_implemented() -> ! {
    unsafe { exception::rb_notimplement() }
}

// The `errno` for an OS error code from Rust (`std::io::Error::raw_os_error`).
// On Windows that code is a Win32 error (`GetLastError`), which is mapped the
// way Ruby maps its own (`ERROR_FILE_NOT_FOUND` is `ENOENT`, ...).
#[cfg(windows)]
pub fn os_error_to_errno(code: c_int) -> c_int {
    unsafe { exception::rb_w32_map_errno(code as u32 as libc::c_ulong) }
}

#[cfg(not(windows))]
pub fn os_error_to_errno(code: c_int) -> c_int {
    code
}

pub fn syserr_new(errno: c_int, message: &str) -> Value {
    let message = message_to_cstring(message);

    unsafe { exception::rb_syserr_new(errno, message.as_ptr()) }
}

// Uses `rb_syserr_new` + `rb_exc_raise` instead of `rb_syserr_fail` so that
// no Rust allocation is alive when Ruby jumps out of this frame.
pub fn syserr_fail(errno: c_int, message: &str) -> ! {
    let exception = syserr_new(errno, message);

    vm::raise_ex(exception)
}

pub fn warn(message: &str) {
    let message = message_to_cstring(message);

    unsafe { exception::rb_warn(PERCENT_S.as_ptr() as *const c_char, message.as_ptr()) }
}

pub fn warning(message: &str) {
    let message = message_to_cstring(message);

    unsafe { exception::rb_warning(PERCENT_S.as_ptr() as *const c_char, message.as_ptr()) }
}

pub fn interrupt() -> ! {
    unsafe { exception::rb_interrupt() }
}
