use crate::{
    binding::{
        class::const_get,
        global::{rb_cObject, RubySpecialConsts},
        vm,
    },
    rubysys::rproc::{rb_obj_is_method, rb_obj_is_proc},
    types::{c_char, c_int, c_void, Argc, InternalValue, Value},
    AnyObject, Boolean, Object,
};

use std::{
    ffi::{CStr, CString},
    ptr, slice,
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
// ```
// use rutie::types::Argc;
// use rutie::{AnyObject, Boolean, Class, Object, RString, util};
//
// #[no_mangle]
// pub extern fn string_eq(argc: Argc, argv: *const AnyObject, rtself: RString) -> Boolean {
//     let argv = util::parse_arguments(argc, argv);
//     let other_string = argv[0].try_convert_to::<RString>().unwrap();
//
//     Boolean::new(rtself.to_str() == other_string.to_str())
// }
//
// fn main() {
//     Class::from_existing("String").define_method("==", string_eq);
// }
// ```
pub fn parse_arguments(argc: Argc, arguments: *const AnyObject) -> Vec<AnyObject> {
    unsafe { slice::from_raw_parts(arguments, argc as usize).to_vec() }
}

pub fn closure_to_ptr<F, R>(mut func: F) -> *mut c_void
where
    F: FnMut() -> R,
{
    let wrap_return = move || {
        let r = func();
        Box::into_raw(Box::new(r)) as *const c_void
    };

    let fnbox = Box::new(wrap_return) as Box<dyn FnMut() -> *const c_void>;

    Box::into_raw(Box::new(fnbox)) as *mut c_void
}

pub unsafe fn ptr_to_data<R>(ptr: *mut c_void) -> R {
    *Box::from_raw(ptr as *mut R)
}

pub fn is_proc(obj: Value) -> bool {
    Boolean::from(unsafe { rb_obj_is_proc(obj.into()) }).to_bool()
}

pub fn is_method(obj: Value) -> bool {
    Boolean::from(unsafe { rb_obj_is_method(obj.into()) }).to_bool()
}

// Recurses to the deepest ruby object.
//
// Given `"A::B::C"` it will return the object instance of `C`.
pub fn inmost_rb_object(klass: &str) -> Value {
    let object = unsafe { rb_cObject };

    klass
        .split("::")
        .fold(object.into(), |acc, x| const_get(acc, x))
}

pub mod callback_call {
    use crate::types::{c_void, st_retval, CallbackMutPtr};

    pub fn no_parameters<F: FnMut() -> R, R>(ptr: CallbackMutPtr) -> R {
        let f = ptr as *mut F;
        unsafe { (*f)() }
    }

    pub fn one_parameter<F: FnMut(A) -> R, A, R>(a: A, ptr: CallbackMutPtr) -> R {
        let f = ptr as *mut F;
        unsafe { (*f)(a) }
    }

    pub fn hash_foreach_callback<F: FnMut(A, B), A, B>(
        a: A,
        b: B,
        ptr: CallbackMutPtr,
    ) -> st_retval {
        let f = ptr as *mut F;
        unsafe {
            (*f)(a, b);
        }
        st_retval::Continue
    }
}
