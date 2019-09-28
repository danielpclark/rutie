use std::ptr;

use ::AnyObject;
use rubysys::{thread, vm};

use types::{c_int, c_void, CallbackPtr, Value, VmPointer};
use binding::symbol::internal_id;
use util;

pub fn block_proc() -> Value {
    unsafe { vm::rb_block_proc() }
}

pub fn is_block_given() -> bool {
    let result = unsafe { vm::rb_block_given_p() };

    util::c_int_to_bool(result)
}

pub fn yield_object(value: Value) -> Value {
    unsafe { vm::rb_yield(value) }
}

pub fn yield_splat(values: Value) -> Value {
    unsafe { vm::rb_yield_splat(values) }
}

pub fn init() {
    unsafe {
        vm::ruby_init();
    }
}

fn force_loading() {
    let utf8 = util::str_to_cstring("UTF-8");
    let ascii_8bit = util::str_to_cstring("ASCII-8BIT");
    let binary = util::str_to_cstring("BINARY");
    let dummy = util::str_to_cstring("DUMMY");
    let single_byte = util::str_to_cstring("single_byte");
    unsafe {
        vm::rb_encdb_declare(ascii_8bit.as_ptr());
        vm::rb_encdb_alias(binary.as_ptr(), ascii_8bit.as_ptr());
        vm::rb_encdb_replicate(ascii_8bit.as_ptr(), ascii_8bit.as_ptr());
        vm::rb_enc_set_base(ascii_8bit.as_ptr(), ascii_8bit.as_ptr());
        let i = vm::rb_encdb_dummy(dummy.as_ptr());
        vm::rb_enc_set_dummy(i);
        vm::rb_encdb_set_unicode(1); // see encindex.h RUBY_ENCINDEX_UTF_8
        vm::rb_declare_transcoder(utf8.as_ptr(), ascii_8bit.as_ptr(), single_byte.as_ptr());
    }
}

pub fn init_loadpath() {
    unsafe {
        force_loading();
        vm::ruby_init_loadpath();
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
    F: FnMut() -> R,
    G: FnMut(),
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
    F: FnMut() -> R,
    G: FnMut(),
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
    F: FnMut() -> R,
{
    unsafe {
        let ptr =
            thread::rb_thread_call_with_gvl(callbox as CallbackPtr, util::closure_to_ptr(func));

        util::ptr_to_data(ptr)
    }
}

extern "C" fn callbox(boxptr: *mut c_void) -> *const c_void {
    let mut fnbox: Box<Box<dyn FnMut() -> *const c_void>> =
        unsafe { Box::from_raw(boxptr as *mut Box<dyn FnMut() -> *const c_void>) };

    fnbox()
}

use util::callback_call::no_parameters as callback_protect;

pub fn protect<F>(func: F) -> Result<AnyObject, c_int>
where
    F: FnMut() -> AnyObject,
{
    let mut state = 0;
    let value = unsafe {
        let closure = &func as *const F as *const c_void;
        vm::rb_protect(callback_protect::<F, AnyObject> as CallbackPtr, closure, &mut state as *mut c_int)
    };
    if state == 0 {
        Ok(value.into())
    } else {
        Err(state)
    }
}

pub fn exit(status: i32) {
    unsafe { vm::rb_exit(status as c_int) }
}

pub fn abort(arguments: &[Value]) {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { vm::rb_f_abort(argc, argv) };
}

use util::callback_call::one_parameter as at_exit_callback;

pub fn at_exit<F>(func: F)
where F: FnMut(VmPointer) -> () {
    let mut state = 0;
    unsafe {
        let closure = &func as *const F as *const c_void;
        vm::rb_protect(at_exit_callback::<F, VmPointer, ()> as CallbackPtr, closure, &mut state as *mut c_int)
    };
}
