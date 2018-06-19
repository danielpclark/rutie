use std::ffi::{CStr, CString};
use std::ptr;

use binding::global::RubySpecialConsts;
use types::{c_char, c_int, c_void, Argc, InternalValue, Value};

use {AnyObject, Object};

pub unsafe fn cstr_to_string(str: *const c_char) -> String {
    CStr::from_ptr(str).to_string_lossy().into_owned()
}

pub unsafe fn cstr_to_str<'a>(str: *const c_char) -> &'a str {
    CStr::from_ptr(str).to_str().unwrap()
}

pub fn str_to_cstring(str: &str) -> CString {
    CString::new(str).unwrap()
}

pub fn bool_to_value(state: bool) -> Value {
    let internal_value = if state {
        RubySpecialConsts::True
    } else {
        RubySpecialConsts::False
    };

    Value::from(internal_value as InternalValue)
}

#[inline]
pub fn c_int_to_bool(int: c_int) -> bool {
    int != 0
}

#[inline]
pub fn bool_to_c_int(state: bool) -> c_int {
    state as c_int
}

pub fn arguments_to_values(arguments: Option<&[AnyObject]>) -> Option<Vec<Value>> {
    arguments.map(|arguments| arguments.iter().map(Object::value).collect())
}

pub fn process_arguments(arguments: &Option<Vec<Value>>) -> (Argc, *const Value) {
    match *arguments {
        Some(ref arguments) => (arguments.len() as Argc, arguments.as_ptr()),
        None => (0, ptr::null()),
    }
}

pub fn closure_to_ptr<F, R>(func: F) -> *const c_void
where
    F: FnOnce() -> R,
{
    let wrap_return = || {
        let r = func();
        Box::into_raw(Box::new(r)) as *const c_void
    };

    let fnbox = Box::new(wrap_return) as Box<FnOnce() -> *const c_void>;

    Box::into_raw(Box::new(fnbox)) as *const c_void
}

pub unsafe fn ptr_to_data<R>(ptr: *mut c_void) -> R {
    *Box::from_raw(ptr as *mut R)
}
