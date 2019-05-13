use std::ffi::{CStr, CString};
use std::ptr;
use std::slice;

use binding::global::{RubySpecialConsts, rb_cObject};
use binding::class::const_get;
use types::{c_char, c_int, c_void, Argc, InternalValue, Value};

use {AnyObject, Object, Boolean};
use crate::rubysys::rproc::{
    rb_obj_is_proc,
    rb_obj_is_method,
};

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

pub fn arguments_to_values(arguments: &[AnyObject]) -> Vec<Value> {
    arguments.as_ref().iter().map(Object::value).collect()
}

pub fn process_arguments(arguments: &[Value]) -> (Argc, *const Value) {
    (arguments.len() as Argc, arguments.as_ptr())
}

pub fn option_to_slice<'a, T>(option: &'a Option<T>) -> &'a [T] {
    match option {
        &Some(ref v) => unsafe { slice::from_raw_parts(v, 1) },
        &None => &[],
    }
}

// Converts a pointer to array of `AnyObject`s to `Vec<AnyObject>`.
//
// This function is a helper for callbacks, do not use it directly.
//
// It will be moved to other struct, because it is not related to VM itself.
//
// # Examples
//
// ```no_run
// use rutie::types::Argc;
// use rutie::{AnyObject, Boolean, Class, Object, RString, util};
//
// #[no_mangle]
// pub extern fn string_eq(argc: Argc, argv: *const AnyObject, itself: RString) -> Boolean {
//     let argv = util::parse_arguments(argc, argv);
//     let other_string = argv[0].try_convert_to::<RString>().unwrap();
//
//     Boolean::new(itself.to_str() == other_string.to_str())
// }
//
// fn main() {
//     Class::from_existing("String").define_method("==", string_eq);
// }
// ```
pub fn parse_arguments(argc: Argc, arguments: *const AnyObject) -> Vec<AnyObject> {
    unsafe { slice::from_raw_parts(arguments, argc as usize).to_vec() }
}

pub fn closure_to_ptr<F, R>(mut func: F) -> *const c_void
where
    F: FnMut() -> R,
{
    let wrap_return = move || {
        let r = func();
        Box::into_raw(Box::new(r)) as *const c_void
    };

    let fnbox = Box::new(wrap_return) as Box<FnMut() -> *const c_void>;

    Box::into_raw(Box::new(fnbox)) as *const c_void
}

pub unsafe fn ptr_to_data<R>(ptr: *mut c_void) -> R {
    *Box::from_raw(ptr as *mut R)
}

pub fn is_proc(obj: Value) -> bool {
    Boolean::from(unsafe { rb_obj_is_proc(obj) }).to_bool()
}

pub fn is_method(obj: Value) -> bool {
    Boolean::from(unsafe { rb_obj_is_method(obj) }).to_bool()
}

// Recurses to the deepest ruby object.
//
// Given `"A::B::C"` it will return the object instance of `C`.
pub fn inmost_rb_object(klass: &str) -> Value {
    let object = unsafe { rb_cObject };

    klass.split("::").fold(object, |acc, x| const_get(acc, x))
}
