use std::{
    any::Any,
    panic::{self, AssertUnwindSafe},
    ptr, slice,
};

use crate::{
    binding::{class, exception, global::RubySpecialConsts, symbol::internal_id},
    rubysys::{exception::rb_eRuntimeError, thread, vm},
    types::{c_int, c_void, CallbackMutPtr, CallbackPtr, InternalValue, Value, VmPointer},
    util, AnyObject,
};

fn nil() -> Value {
    Value::from(RubySpecialConsts::Nil as InternalValue)
}

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

pub fn init_loadpath() {
    unsafe {
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

pub fn call_super(arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { vm::rb_call_super(argc, argv) }
}

// "evaluation can raise an exception."
pub fn eval_string(string: &str) -> Value {
    let s = util::str_to_cstring(string);

    unsafe { vm::rb_eval_string(s.as_ptr()) }
}

pub fn eval_string_protect(string: &str) -> Result<Value, c_int> {
    let s = util::str_to_cstring(string);
    let mut state = 0;
    let value = unsafe { vm::rb_eval_string_protect(s.as_ptr(), &mut state as *mut c_int) };
    if state == 0 {
        Ok(value)
    } else {
        Err(state)
    }
}

// The message is never used as a printf format, and the exception is
// built before raising so no Rust allocation is alive during the longjmp.
pub fn raise(exception: Value, message: &str) -> ! {
    let exception = exception::new(exception, message);

    raise_ex(exception)
}

pub fn raise_ex(exception: Value) -> ! {
    unsafe { vm::rb_exc_raise(exception) }
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

use crate::util::callback_call::no_parameters as callback_protect;

pub fn protect<F>(func: F) -> Result<AnyObject, c_int>
where
    F: FnMut() -> AnyObject,
{
    let mut state = 0;
    let value = unsafe {
        let closure = &func as *const F as *const c_void;
        vm::rb_protect(
            callback_protect::<F, AnyObject> as CallbackPtr,
            closure,
            &mut state as *mut c_int,
        )
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

use crate::rubysys::types::{Argc, BlockCallFunction};

// Builds a Ruby `RuntimeError` describing a caught Rust panic.
pub fn panic_to_exception(payload: Box<dyn Any + Send>) -> Value {
    let message = if let Some(message) = payload.downcast_ref::<&str>() {
        format!("Rust panic: {}", message)
    } else if let Some(message) = payload.downcast_ref::<String>() {
        format!("Rust panic: {}", message)
    } else {
        String::from("Rust panic")
    };

    exception::new(unsafe { rb_eRuntimeError }, &message)
}

// Every `extern "C"` function that Ruby calls into goes through this: a panic
// must never unwind into Ruby's C frames, so it is caught and re-raised as a
// Ruby `RuntimeError` once the panic payload has been dropped.
pub(crate) fn call_catching_panic<F, R>(func: F) -> R
where
    F: FnOnce() -> R,
{
    let exception = match panic::catch_unwind(AssertUnwindSafe(func)) {
        Ok(result) => return result,
        Err(payload) => panic_to_exception(payload),
    };

    raise_ex(exception)
}

// `data` points to an `Option<F>` on the caller's stack, taken exactly once.
extern "C" fn call_once_callback<F>(data: CallbackMutPtr) -> Value
where
    F: FnOnce() -> Value,
{
    match unsafe { (*(data as *mut Option<F>)).take() } {
        Some(func) => call_catching_panic(func),
        None => nil(),
    }
}

// `data` is a boxed `FnOnce` whose pointer is tagged as a Fixnum (see `at_exit`).
extern "C" fn end_proc_callback(data: Value) {
    let boxed = unsafe {
        Box::from_raw((data.value & !(1 as InternalValue)) as *mut Box<dyn FnOnce(VmPointer)>)
    };
    let func: Box<dyn FnOnce(VmPointer)> = *boxed;

    call_catching_panic(move || func(ptr::null()))
}

// Registers `func` to run when the VM runs its end procs (Ruby's `at_exit`).
//
// `rb_set_end_proc` hands `data` to `rb_gc_mark`, which must never see a raw
// pointer. The closure's heap address is at least 2-byte aligned, so setting
// its low bit turns it into a Fixnum, which the GC skips.
pub fn at_exit<F>(func: F)
where
    F: FnOnce(VmPointer) + 'static,
{
    let boxed: Box<Box<dyn FnOnce(VmPointer)>> = Box::new(Box::new(func));
    let address = Box::into_raw(boxed) as InternalValue;

    debug_assert_eq!(address & 1, 0, "end proc closure is not 2-byte aligned");

    unsafe { vm::rb_set_end_proc(end_proc_callback, Value::from(address | 1)) }
}

// `rb_protect` calls this with the closure pointer as its only argument.
extern "C" fn call_protected_callback<F>(closure: *mut c_void) -> Value
where
    F: FnMut(VmPointer),
{
    let f = closure as *mut F;
    unsafe { (*f)(ptr::null()) };

    nil()
}

// Runs `func` immediately under `rb_protect` (what `at_exit` did before 0.11).
pub fn call_protected<F>(mut func: F)
where
    F: FnMut(VmPointer),
{
    let mut state = 0;
    unsafe {
        let closure = &mut func as *mut F as *const c_void;
        vm::rb_protect(
            call_protected_callback::<F> as CallbackPtr,
            closure,
            &mut state as *mut c_int,
        )
    };
}

pub fn cleanup(status: c_int) -> c_int {
    unsafe { vm::ruby_cleanup(status) }
}

// Runs `func` under `rb_protect`. On an exception the error info is cleared
// and the exception is returned.
pub fn protect_value<F>(func: F) -> Result<Value, Value>
where
    F: FnOnce() -> Value,
{
    let mut func = Some(func);
    let mut state = 0;
    let value = unsafe {
        vm::rb_protect(
            call_once_callback::<F> as CallbackPtr,
            &mut func as *mut Option<F> as *const c_void,
            &mut state as *mut c_int,
        )
    };

    if state == 0 {
        Ok(value)
    } else {
        let exception = errinfo();
        set_errinfo(nil());

        Err(exception)
    }
}

extern "C" fn ensure_callback<E>(data: CallbackMutPtr) -> Value
where
    E: FnOnce(),
{
    if let Some(func) = unsafe { (*(data as *mut Option<E>)).take() } {
        call_catching_panic(func)
    }

    nil()
}

pub fn ensure<B, E>(body: B, ensure: E) -> Value
where
    B: FnOnce() -> Value,
    E: FnOnce(),
{
    let mut body = Some(body);
    let mut ensure = Some(ensure);

    unsafe {
        vm::rb_ensure(
            call_once_callback::<B>,
            &mut body as *mut Option<B> as CallbackMutPtr,
            ensure_callback::<E>,
            &mut ensure as *mut Option<E> as CallbackMutPtr,
        )
    }
}

struct Rescue<'a, R> {
    handler: Option<R>,
    classes: &'a [Value],
}

// Exceptions that are not one of the requested classes are raised again.
extern "C" fn rescue_callback<R>(data: CallbackMutPtr, exception: Value) -> Value
where
    R: FnOnce(Value) -> Value,
{
    let rescue = unsafe { &mut *(data as *mut Rescue<R>) };

    if !rescue
        .classes
        .iter()
        .any(|&klass| class::is_kind_of(exception, klass))
    {
        raise_ex(exception)
    }

    match rescue.handler.take() {
        Some(handler) => call_catching_panic(move || handler(exception)),
        None => nil(),
    }
}

// Calls `body`; when it raises an exception that is kind of one of
// `classes`, `handler` is called with it and its result is returned instead.
pub fn rescue<B, R>(body: B, handler: R, classes: &[Value]) -> Value
where
    B: FnOnce() -> Value,
    R: FnOnce(Value) -> Value,
{
    let mut body = Some(body);
    let mut rescue = Rescue {
        handler: Some(handler),
        classes,
    };

    unsafe {
        vm::rb_rescue2(
            call_once_callback::<B>,
            &mut body as *mut Option<B> as CallbackMutPtr,
            rescue_callback::<R>,
            &mut rescue as *mut Rescue<R> as CallbackMutPtr,
            crate::rubysys::exception::rb_eException,
            Value::from(0),
        )
    }
}

extern "C" fn catch_callback<F>(
    tag: Value,
    data: Value,
    _argc: c_int,
    _argv: *const Value,
    _block_arg: Value,
) -> Value
where
    F: FnOnce(Value) -> Value,
{
    match unsafe { (*(data.value as *mut Option<F>)).take() } {
        Some(func) => call_catching_panic(move || func(tag)),
        None => nil(),
    }
}

pub fn catch<F>(tag: Value, body: F) -> Value
where
    F: FnOnce(Value) -> Value,
{
    let mut body = Some(body);
    let data = Value::from(&mut body as *mut Option<F> as InternalValue);

    unsafe { vm::rb_catch_obj(tag, catch_callback::<F> as BlockCallFunction, data) }
}

pub fn throw(tag: Value, value: Value) -> ! {
    unsafe { vm::rb_throw_obj(tag, value) }
}

pub fn iter_break() -> ! {
    unsafe { vm::rb_iter_break() }
}

pub fn iter_break_value(value: Value) -> ! {
    unsafe { vm::rb_iter_break_value(value) }
}

pub fn jump_tag(state: c_int) -> ! {
    unsafe { vm::rb_jump_tag(state) }
}

#[cfg(ruby_gte_2_7)]
pub fn is_keyword_given() -> bool {
    util::c_int_to_bool(unsafe { vm::rb_keyword_given_p() })
}

// `data` points to the block closure; all yielded values are in `argv`.
extern "C" fn block_callback<F>(
    _yielded: Value,
    data: Value,
    argc: c_int,
    argv: *const Value,
    _block_arg: Value,
) -> Value
where
    F: FnMut(&[Value]) -> Value,
{
    let block = unsafe { &mut *(data.value as *mut F) };
    let arguments: &[Value] = if argc > 0 && !argv.is_null() {
        unsafe { slice::from_raw_parts(argv, argc as usize) }
    } else {
        &[]
    };

    call_catching_panic(move || block(arguments))
}

pub fn call_method_with_block<F>(
    receiver: Value,
    method: &str,
    arguments: &[Value],
    mut block: F,
) -> Value
where
    F: FnMut(&[Value]) -> Value,
{
    let (argc, argv) = util::process_arguments(arguments);
    let data = Value::from(&mut block as *mut F as InternalValue);

    unsafe {
        vm::rb_block_call(
            receiver,
            internal_id(method),
            argc,
            argv,
            block_callback::<F> as BlockCallFunction,
            data,
        )
    }
}

pub fn call_init(object: Value, arguments: &[Value]) {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { vm::rb_obj_call_init(object, argc, argv) }
}

pub fn call_method_with_proc(
    receiver: Value,
    method: &str,
    arguments: &[Value],
    block: Value,
) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { vm::rb_funcall_with_block(receiver, internal_id(method), argc, argv, block) }
}

// The last argument must be a `Hash`; it is passed as keywords.
#[cfg(ruby_gte_2_7)]
pub fn call_method_with_keywords(receiver: Value, method: &str, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { vm::rb_funcallv_kw(receiver, internal_id(method), argc, argv, 1) }
}

pub fn yield_values(values: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(values);

    unsafe { vm::rb_yield_values2(argc, argv) }
}

pub fn need_block() {
    unsafe { vm::rb_need_block() }
}
