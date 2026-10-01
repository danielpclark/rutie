use std::hint::black_box;

use crate::{
    binding::{string, symbol, vm},
    rubysys::{exception, string::rb_string_value_cstr},
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

pub fn errno() -> c_int {
    unsafe { exception::rb_errno() }
}

pub fn set_errno(errno: c_int) {
    unsafe { exception::rb_errno_set(errno) }
}

// A NUL-terminated copy of a message owned by Ruby (cut at the message's
// first NUL), for C functions that take a `const char *` and may raise: a
// Rust `CString` would leak when they jump out of the caller's frame.
//
// Keep it in a local of the frame making the call. `as_ptr` lets its
// address escape (`black_box`, and `rb_string_value_cstr` takes a pointer to
// it), so the string stays on that frame's stack, where the GC finds it, as
// `RB_GC_GUARD` does in C.
struct RubyCStr {
    string: Value,
}

impl RubyCStr {
    fn new(text: &str) -> Self {
        let end = text.find('\0').unwrap_or(text.len());

        RubyCStr {
            string: string::new_utf8(&text[..end]),
        }
    }

    fn as_ptr(&self) -> *const c_char {
        black_box(self);

        unsafe { rb_string_value_cstr(&self.string) }
    }
}

fn percent_s() -> *const c_char {
    PERCENT_S.as_ptr() as *const c_char
}

pub fn category_warn(category: c_int, message: &str) {
    let message = RubyCStr::new(message);

    unsafe { exception::rb_category_warn(category, percent_s(), message.as_ptr()) }
}

pub fn category_warning(category: c_int, message: &str) {
    let message = RubyCStr::new(message);

    unsafe { exception::rb_category_warning(category, percent_s(), message.as_ptr()) }
}

pub fn compile_warn(file: &str, line: c_int, message: &str) {
    let (file, message) = (RubyCStr::new(file), RubyCStr::new(message));

    unsafe { exception::rb_compile_warn(file.as_ptr(), line, percent_s(), message.as_ptr()) }
}

pub fn compile_warning(file: &str, line: c_int, message: &str) {
    let (file, message) = (RubyCStr::new(file), RubyCStr::new(message));

    unsafe { exception::rb_compile_warning(file.as_ptr(), line, percent_s(), message.as_ptr()) }
}

pub fn category_compile_warn(category: c_int, file: &str, line: c_int, message: &str) {
    let (file, message) = (RubyCStr::new(file), RubyCStr::new(message));

    unsafe {
        exception::rb_category_compile_warn(
            category,
            file.as_ptr(),
            line,
            percent_s(),
            message.as_ptr(),
        )
    }
}

pub fn sys_warning(message: &str) {
    let message = RubyCStr::new(message);

    unsafe { exception::rb_sys_warning(percent_s(), message.as_ptr()) }
}

pub fn fatal(message: &str) -> ! {
    let message = RubyCStr::new(message);

    unsafe { exception::rb_fatal(percent_s(), message.as_ptr()) }
}

pub fn syserr_fail_str(errno: c_int, message: &str) -> ! {
    let message = string::new_utf8(message);

    unsafe { exception::rb_syserr_fail_str(errno, message) }
}

pub fn mod_syserr_fail_str(module: Value, errno: c_int, message: &str) -> ! {
    let message = string::new_utf8(message);

    unsafe { exception::rb_mod_syserr_fail_str(module, errno, message) }
}

pub fn readwrite_syserr_fail(waiting: c_int, errno: c_int, message: &str) -> ! {
    let message = RubyCStr::new(message);

    unsafe { exception::rb_readwrite_syserr_fail(waiting, errno, message.as_ptr()) }
}

pub fn loaderror(message: &str) -> ! {
    let message = RubyCStr::new(message);

    unsafe { exception::rb_loaderror(percent_s(), message.as_ptr()) }
}

pub fn loaderror_with_path(path: &str, message: &str) -> ! {
    let path = string::new_utf8(path);
    let message = RubyCStr::new(message);

    unsafe { exception::rb_loaderror_with_path(path, percent_s(), message.as_ptr()) }
}

pub fn name_error(name: &str, message: &str) -> ! {
    let name = symbol::id_to_sym(symbol::internal_id(name));
    let message = RubyCStr::new(message);

    unsafe { exception::rb_name_error_str(name, percent_s(), message.as_ptr()) }
}

pub fn error_frozen(what: &str) -> ! {
    let what = RubyCStr::new(what);

    unsafe { exception::rb_error_frozen(what.as_ptr()) }
}

pub fn invalid_str(value: &str, type_name: &str) -> ! {
    let (value, type_name) = (RubyCStr::new(value), RubyCStr::new(type_name));

    unsafe { exception::rb_invalid_str(value.as_ptr(), type_name.as_ptr()) }
}

pub fn unexpected_type(object: Value, value_type: ValueType) -> ! {
    unsafe { exception::rb_unexpected_type(object, value_type as c_int) }
}
