use std::{
    any::Any,
    ffi::CStr,
    panic::{self, AssertUnwindSafe},
    ptr, slice,
};

use crate::{
    binding::{class, exception, global::RubySpecialConsts, symbol::internal_id},
    rubysys::{exception::rb_eRuntimeError, thread, vm},
    types::{c_char, c_int, c_void, CallbackMutPtr, CallbackPtr, InternalValue, Value, VmPointer},
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

// `ruby` calls `ruby_init_stack` before booting the VM (`RUBY_INIT_STACK`):
// it records the machine stack the GC scans. Without it a VM booted off the
// process's main thread takes a local inside `ruby_init`'s own frames as the
// top of its stack, and from Ruby 3.4 the GC scans no stack at all, freeing objects held
// only in Rust locals. Any address on the thread's stack works: Ruby then
// reads the thread's stack bounds. Skipped once the VM exists, since the call
// also records the calling thread as Ruby's main thread.
fn init_stack() {
    if is_initialized() {
        return;
    }

    let mut marker = Value::from(0);
    unsafe { vm::ruby_init_stack(&mut marker) };
}

// `ruby.exe` calls `ruby_sysinit` before booting the VM; on Windows Ruby's
// IO, environment and sockets do not work without it, and Ruby 3's boot
// (`finish_boot`) uses them.
#[cfg(windows)]
fn sysinit() {
    static SYSINIT: std::sync::Once = std::sync::Once::new();

    SYSINIT.call_once(|| {
        let mut argc: c_int = 0;
        let mut argv: *mut *mut c_char = ptr::null_mut();

        unsafe { vm::ruby_sysinit(&mut argc, &mut argv) };
    });
}

#[cfg(not(windows))]
fn sysinit() {}

pub fn init() {
    init_stack();
    sysinit();

    unsafe {
        vm::ruby_init();
    }

    if let Err(state) = finish_boot() {
        panic!("Ruby failed to finish booting (state {})", state);
    }
}

// Ruby 3 defines part of its core library (`Marshal.load`, `Time.at`, much of
// `Integer`, `Kernel#tap`, ...) in Ruby files compiled into the interpreter,
// and loads them (`rb_call_builtin_inits`) while processing the command line,
// not in `ruby_init`. So after `ruby_init`/`ruby_setup` the command line is
// processed once, with an empty `-e` script. RubyGems and `RUBYOPT` stay off:
// an embedded VM starts with the core library only, as it always did.
// When a `ruby` process already did this (an extension), nothing happens.
//
// Ruby also starts its main thread with C methods treated as Ractor-safe
// (extensions loaded by `require` start unsafe). An embedded VM is switched
// to unsafe as well, so methods defined through Rutie are not called from
// other Ractors unless `ext_ractor_safe(true)` opts in.
fn finish_boot() -> Result<(), c_int> {
    if has_run_options() {
        return Ok(());
    }

    match run_options(&["ruby", "--disable=gems,rubyopt", "-e", ""]) {
        Ok(_) => {}
        Err(Ok(_exception)) => return Err(1),
        Err(Err(status)) => return Err(status),
    }

    // `-e` made `$0` "-e"; name the program instead.
    let program = std::env::args()
        .next()
        .unwrap_or_else(|| "ruby".to_string());
    set_script_name(&program);

    ext_ractor_safe(false);

    Ok(())
}

pub fn ext_ractor_safe(flag: bool) {
    unsafe { vm::rb_ext_ractor_safe(flag) };
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

// `rb_raise` takes a printf format, so `%` in `message` is interpreted as
// in C (use `%%` for a literal `%`); see `raise_message` for plain text.
pub fn raise(exception: Value, message: &str) {
    let message = util::str_to_cstring(message);

    unsafe {
        vm::rb_raise(exception, message.as_ptr());
    }
}

// The message is never used as a printf format, and the exception is
// built before raising so no Rust allocation is alive during the longjmp.
pub fn raise_message(exception: Value, message: &str) -> ! {
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

rutie_callback! {
    fn callbox(boxptr: *mut c_void) -> *const c_void {
        let mut fnbox: Box<Box<dyn FnMut() -> *const c_void>> =
            unsafe { Box::from_raw(boxptr as *mut Box<dyn FnMut() -> *const c_void>) };

        fnbox()
    }
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
rutie_callback! {
    fn call_once_callback<F>(data: CallbackMutPtr) -> Value
    where
        F: FnOnce() -> Value,
    {
        match unsafe { (*(data as *mut Option<F>)).take() } {
            Some(func) => call_catching_panic(func),
            None => nil(),
        }
    }
}

// `data` is a boxed `FnOnce` whose pointer is tagged as a Fixnum (see `at_exit`).
rutie_callback! {
    fn end_proc_callback(data: Value) {
        let boxed = unsafe {
            Box::from_raw((data.value & !(1 as InternalValue)) as *mut Box<dyn FnOnce(VmPointer)>)
        };
        let func: Box<dyn FnOnce(VmPointer)> = *boxed;

        call_catching_panic(move || func(ptr::null()))
    }
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
rutie_callback! {
    fn call_protected_callback<F>(closure: *mut c_void) -> Value
    where
        F: FnMut(VmPointer),
    {
        let f = closure as *mut F;
        unsafe { (*f)(ptr::null()) };

        nil()
    }
}

// Runs `func` immediately under `rb_protect` (what `at_exit` did before 0.10).
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

pub fn setup() -> c_int {
    init_stack();
    sysinit();

    match unsafe { vm::ruby_setup() } {
        0 => finish_boot().err().unwrap_or(0),
        state => state,
    }
}

// `rb_cObject` is set while the VM boots and never cleared, so this is true
// from `ruby_setup` on, and in extensions loaded by a running Ruby.
pub fn is_initialized() -> bool {
    unsafe { crate::rubysys::rb_cObject.value != 0 }
}

// `ruby_native_thread_p` reads a thread-local key that only exists once the
// VM has started, so it is not called before that.
pub fn is_ruby_thread() -> bool {
    is_initialized() && util::c_int_to_bool(unsafe { vm::ruby_native_thread_p() })
}

pub fn set_script_name(name: &str) {
    let name = util::str_to_cstring(name);

    unsafe { vm::ruby_script(name.as_ptr()) }
}

// Ruby copies each argument into a frozen string, so the C strings only need
// to live for the call.
pub fn set_argv(arguments: &[&str]) {
    let arguments: Vec<_> = arguments.iter().map(|a| util::str_to_cstring(a)).collect();
    let mut pointers: Vec<_> = arguments
        .iter()
        .map(|a| a.as_ptr() as *mut c_char)
        .collect();

    unsafe { vm::ruby_set_argv(pointers.len() as c_int, pointers.as_mut_ptr()) }
}

// Takes the pending exception, if any, out of `$!`.
fn take_errinfo() -> Option<Value> {
    let error = errinfo();

    if error.is_nil() {
        None
    } else {
        set_errinfo(nil());
        Some(error)
    }
}

// Whether `ruby_options` has run in this process (it sets `rb_argv0`): the
// `ruby` command runs it before loading any extension.
pub fn has_run_options() -> bool {
    unsafe { vm::rb_argv0.value != 0 }
}

// Runs `options` like the `ruby` command would. Callers must check
// `has_run_options` first, since `ruby_options` works once per process. `ruby_options` keeps the
// argument vector for the life of the process (`$0 =` writes into it, as
// with a real `argv`), so it is leaked on purpose.
//
// Ok is the exit status of a script that compiled and ran without raising;
// Err is the exception, or the exit status when Ruby left none (it has
// printed its error already).
pub fn run_options(options: &[&str]) -> Result<c_int, Result<Value, c_int>> {
    let mut pointers: Vec<*mut c_char> = options
        .iter()
        .map(|option| util::str_to_cstring(option).into_raw())
        .collect();
    let argc = pointers.len() as c_int;
    pointers.push(ptr::null_mut());
    let argv = Box::leak(pointers.into_boxed_slice()).as_mut_ptr();

    let node = unsafe { vm::ruby_options(argc, argv) };
    let mut status = 0;

    if !util::c_int_to_bool(unsafe { vm::ruby_executable_node(node, &mut status) }) {
        return match take_errinfo() {
            Some(error) => Err(Ok(error)),
            None if status == 0 => Ok(0),
            None => Err(Err(status)),
        };
    }

    let state = unsafe { vm::ruby_exec_node(node) };

    match take_errinfo() {
        Some(error) if state != 0 => Err(Ok(error)),
        _ if state != 0 => Err(Err(state)),
        _ => Ok(0),
    }
}

pub fn is_stack_near_limit() -> bool {
    util::c_int_to_bool(unsafe { vm::ruby_stack_check() })
}

pub fn stack_length() -> usize {
    unsafe { vm::ruby_stack_length(ptr::null_mut()) as usize }
}

pub fn at_vm_exit(func: extern "C" fn(VmPointer)) {
    unsafe { vm::ruby_vm_at_exit(func as VmPointer) }
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

rutie_callback! {
    fn ensure_callback<E>(data: CallbackMutPtr) -> Value
    where
        E: FnOnce(),
    {
        if let Some(func) = unsafe { (*(data as *mut Option<E>)).take() } {
            call_catching_panic(func)
        }

        nil()
    }
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
rutie_callback! {
    fn rescue_callback<R>(data: CallbackMutPtr, exception: Value) -> Value
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

// Runs `func`, which may create, resume or yield a fiber, and returns an
// exception it raises as `Err`.
//
// Ruby switches fibers only while the thread has an active tag, and resumes a
// fiber only under the `rb_protect` it was created under ("fiber called across
// stack rewinding barrier"), so this uses `rb_rescue2`: a tag, and no barrier.
// (Ruby 3 has native fibers on every platform Rutie supports; Ruby 2.5 and 2.6
// on arm64 macOS copied the machine stack and needed `rb_protect` here.)
pub fn fiber_call<F>(func: F) -> Result<Value, Value>
where
    F: FnOnce() -> Value,
{
    let mut error = None;

    let result = rescue(
        func,
        |exception| {
            error = Some(exception);

            exception
        },
        &[unsafe { crate::rubysys::exception::rb_eException }],
    );

    match error {
        Some(exception) => Err(exception),
        None => Ok(result),
    }
}

rutie_callback! {
    fn catch_callback<F>(
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

pub fn is_keyword_given() -> bool {
    util::c_int_to_bool(unsafe { vm::rb_keyword_given_p() })
}

// `data` points to the block closure; all yielded values are in `argv`.
rutie_callback! {
    fn block_callback<F>(
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

pub fn clear_constant_cache_for(name: &str) {
    unsafe { vm::rb_clear_constant_cache_for_id(internal_id(name)) }
}

pub fn ext_resolve_symbol(feature: &CStr, symbol: &CStr) -> *mut c_void {
    unsafe { vm::rb_ext_resolve_symbol(feature.as_ptr(), symbol.as_ptr()) }
}

pub fn free_at_exit() -> bool {
    unsafe { vm::ruby_free_at_exit_p() }
}
