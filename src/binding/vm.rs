use std::ptr;

use rubysys::{thread, vm};

use types::{c_int, c_void, CallbackPtr, Value};
use binding::symbol::internal_id;
use util;

pub fn block_proc() -> Value {
    unsafe { vm::rb_block_proc() }
}

pub fn is_block_given() -> bool {
    let result = unsafe { vm::rb_block_given_p() };

    util::c_int_to_bool(result)
}

pub fn init() {
    unsafe {
        vm::ruby_init();
    }
}

pub fn require(name: &str) {
    let name = util::str_to_cstring(name);

    unsafe {
        vm::rb_require(name.as_ptr());
    }
}

pub fn call_method(receiver: Value, method: &str, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);
    let method_id = internal_id(method);

    // TODO: Update the signature of `rb_funcallv` in ruby-sys to receive an `Option`
    unsafe { vm::rb_funcallv(receiver, method_id, argc, argv) }
}

pub fn call_public_method(receiver: Value, method: &str, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);
    let method_id = internal_id(method);

    // TODO: Update the signature of `rb_funcallv_public` in ruby-sys to receive an `Option`
    unsafe { vm::rb_funcallv_public(receiver, method_id, argc, argv) }
}

// "evaluation can raise an exception."
pub fn eval_string(string: &str) -> Value {
    let s = util::str_to_cstring(string);

    unsafe {
        vm::rb_eval_string(s.as_ptr())
    }
}

pub fn eval_string_protect(string: &str) -> Result<Value, c_int> {
    let s = util::str_to_cstring(string);
    let mut state = 0;
    let value = unsafe {
        vm::rb_eval_string_protect(
            s.as_ptr(),
            &mut state as *mut c_int
        )
    };
    if state == 0 {
        Ok(value)
    } else {
        Err(state)
    }
}

pub fn raise(exception: Value, message: &str) {
    let message = util::str_to_cstring(message);

    unsafe {
        vm::rb_raise(exception, message.as_ptr());
    }
}

pub fn raise_ex(exception: Value) {
    unsafe { vm::rb_exc_raise(exception); }
}

pub fn errinfo() -> Value {
    unsafe { vm::rb_errinfo() }
}

pub fn set_errinfo(err: Value) {
    unsafe { vm::rb_set_errinfo(err) }
}

pub fn thread_call_without_gvl<F, R, G>(func: F, unblock_func: Option<G>) -> R
where
    F: FnOnce() -> R,
    G: FnOnce(),
{
    unsafe {
        let ptr = if let Some(ubf) = unblock_func {
            thread::rb_thread_call_without_gvl(
                callbox as CallbackPtr,
                util::closure_to_ptr(func),
                callbox as CallbackPtr,
                util::closure_to_ptr(ubf),
            )
        } else {
            thread::rb_thread_call_without_gvl(
                callbox as CallbackPtr,
                util::closure_to_ptr(func),
                ptr::null() as CallbackPtr,
                ptr::null() as *const c_void,
            )
        };

        util::ptr_to_data(ptr)
    }
}

pub fn thread_call_without_gvl2<F, R, G>(func: F, unblock_func: Option<G>) -> R
where
    F: FnOnce() -> R,
    G: FnOnce(),
{
    unsafe {
        let ptr = if let Some(ubf) = unblock_func {
            thread::rb_thread_call_without_gvl2(
                callbox as CallbackPtr,
                util::closure_to_ptr(func),
                callbox as CallbackPtr,
                util::closure_to_ptr(ubf),
            )
        } else {
            thread::rb_thread_call_without_gvl2(
                callbox as CallbackPtr,
                util::closure_to_ptr(func),
                ptr::null() as CallbackPtr,
                ptr::null() as *const c_void,
            )
        };

        util::ptr_to_data(ptr)
    }
}

pub fn thread_call_with_gvl<F, R>(func: F) -> R
where
    F: FnOnce() -> R,
{
    unsafe {
        let ptr =
            thread::rb_thread_call_with_gvl(callbox as CallbackPtr, util::closure_to_ptr(func));

        util::ptr_to_data(ptr)
    }
}

extern "C" fn callbox(boxptr: *mut c_void) -> *const c_void {
    let mut fnbox: Box<Box<FnMut() -> *const c_void>> =
        unsafe { Box::from_raw(boxptr as *mut Box<FnMut() -> *const c_void>) };

    fnbox()
}

pub fn protect<F>(func: F) -> Result<Value, c_int>
where
    F: FnMut() -> Value,
{
    let mut state = 0;
    let value = unsafe {
        let closure = &func as *const F as *const c_void;
        vm::rb_protect(callback_protect::<F> as CallbackPtr, closure, &mut state as *mut c_int)
    };
    if state == 0 {
        Ok(value)
    } else {
        Err(state)
    }
}

fn callback_protect<F: FnMut() -> Value>(ptr: *const c_void) -> Value {
    let f = ptr as *mut F;
    unsafe { (*f)() }
}
