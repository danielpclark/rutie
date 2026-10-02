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
// ```no_run
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
/// Copies the `argc` arguments Ruby passed to a variable-arity method
/// (`rb_define_method` with arity -1) out of `arguments`.
///
/// `arguments` may be NULL when `argc` is 0: Ruby's own C code calls
/// methods with no arguments through `rb_funcall`, which passes no array
/// (`format('%s', obj)`, `puts obj` and `Array#join` call `to_s` that way).
/// Both are taken as Ruby handed them to the method; the pointer itself is
/// not validated beyond that.
pub fn parse_arguments(argc: Argc, arguments: *const AnyObject) -> Vec<AnyObject> {
    if argc <= 0 || arguments.is_null() {
        return Vec::new();
    }

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

    let fnbox = Box::new(wrap_return) as Box<dyn FnMut() -> *const c_void>;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{types::CallbackMutPtr, Class, Fixnum, NilClass, Proc, VM};

    #[test]
    fn test_conversions() {
        crate::on_ruby_thread(|| {
            let owned = CString::new("héllo").unwrap();
            assert_eq!(unsafe { cstr_to_string(owned.as_ptr()) }, "héllo");
            assert_eq!(unsafe { cstr_to_str(owned.as_ptr()) }, "héllo");
            assert_eq!(str_to_cstring("abc").as_bytes(), b"abc");

            assert!(bool_to_value(true).is_true());
            assert!(bool_to_value(false).is_false());
            assert!(c_int_to_bool(2));
            assert!(!c_int_to_bool(0));
            assert_eq!(bool_to_c_int(true), 1);
            assert_eq!(bool_to_c_int(false), 0);

            let arguments = [
                Fixnum::new(1).to_any_object(),
                NilClass::new().to_any_object(),
            ];
            let values = arguments_to_values(&arguments);
            assert_eq!(values, vec![arguments[0].value(), arguments[1].value()]);

            let (argc, argv) = process_arguments(&values);
            assert_eq!(argc, 2);
            assert_eq!(argv, values.as_ptr());
            assert_eq!(
                parse_arguments(argc, arguments.as_ptr()),
                arguments.to_vec()
            );

            assert_eq!(option_to_slice(&Some(5)), &[5]);
            assert_eq!(option_to_slice::<i32>(&None), &[] as &[i32]);
        });
    }

    #[test]
    fn test_closure_pointers_and_callbacks() {
        crate::on_ruby_thread(|| {
            let mut calls = 0;
            let ptr = closure_to_ptr(|| {
                calls += 1;
                41 + calls
            });
            // The pointer is to a boxed closure returning a boxed result.
            let result = unsafe {
                let closure = &mut *(ptr as *mut Box<dyn FnMut() -> *const c_void>);
                let value = ptr_to_data::<i32>(closure() as *mut c_void);
                drop(Box::from_raw(ptr as *mut Box<dyn FnMut() -> *const c_void>));
                value
            };
            assert_eq!(result, 42);

            fn call0<F: FnMut() -> R, R>(f: &mut F) -> R {
                callback_call::no_parameters::<F, R>(f as *mut F as CallbackMutPtr)
            }
            fn call1<F: FnMut(A) -> R, A, R>(f: &mut F, a: A) -> R {
                callback_call::one_parameter::<F, A, R>(a, f as *mut F as CallbackMutPtr)
            }
            fn call2<F: FnMut(A, B), A, B>(f: &mut F, a: A, b: B) {
                let _ = callback_call::hash_foreach_callback::<F, A, B>(
                    a,
                    b,
                    f as *mut F as CallbackMutPtr,
                );
            }

            let mut counter = 0;
            let mut increment = || {
                counter += 1;
                counter
            };
            assert_eq!(call0(&mut increment), 1);

            let mut add = |n: i32| n + 10;
            assert_eq!(call1(&mut add, 5), 15);

            let mut pairs = Vec::new();
            let mut collect = |a: i32, b: i32| pairs.push((a, b));
            call2(&mut collect, 1, 2);
            assert_eq!(pairs, vec![(1, 2)]);
        });
    }

    #[test]
    fn test_ruby_object_helpers() {
        crate::on_ruby_thread(|| {
            let lambda = Proc::new(|_| NilClass::new().into());
            assert!(is_proc(lambda.value()));
            assert!(!is_proc(Fixnum::new(1).value()));

            let method = VM::eval("1.method(:succ)").unwrap();
            assert!(is_method(method.value()));
            assert!(!is_method(lambda.value()));

            VM::eval("module RutieUtilOuter; class Inner; end; end").unwrap();
            let inner = inmost_rb_object("RutieUtilOuter::Inner");
            assert_eq!(
                Class::from(inner).name().unwrap().to_str(),
                "RutieUtilOuter::Inner"
            );
            assert_eq!(inmost_rb_object("String"), Class::string().value());
        });
    }

    // Ruby's C code calls arity -1 methods with no arguments through
    // rb_funcall, which passes a NULL argv (#190).
    #[test]
    fn test_parse_arguments_with_null_argv() {
        use crate::{methods, Class, RString, VM};

        methods!(
            AnyObject,
            _itself,
            fn rutie_null_argv_to_s() -> RString {
                RString::new_utf8("null argv")
            }
        );

        crate::on_ruby_thread(|| {
            assert!(parse_arguments(0, std::ptr::null()).is_empty());

            Class::new("RutieNullArgvIssue190", None).define(|klass| {
                klass.def("to_s", rutie_null_argv_to_s);
            });

            // Kernel#format's %s -> rb_obj_as_string -> rb_funcall(obj, :to_s)
            let formatted = VM::eval("format('%s', RutieNullArgvIssue190.new)").unwrap();

            assert_eq!(
                formatted.try_convert_to::<RString>().unwrap().to_str(),
                "null argv"
            );
        });
    }
}
