use std::ptr;

use crate::{
    rubysys::thread,
    types::{c_void, CallbackMutPtr, CallbackPtr, Value},
    util, Object,
};

#[cfg(any(unix, windows))]
use crate::types::RawFd;

pub fn create<F, R>(func: F) -> Value
where
    F: FnMut() -> R,
    R: Object,
{
    let fnbox = Box::new(func) as Box<dyn FnMut() -> R>;

    let closure_ptr = Box::into_raw(Box::new(fnbox)) as CallbackMutPtr;

    unsafe { thread::rb_thread_create(thread_create_callbox::<R>, closure_ptr) }
}

#[cfg(any(unix, windows))]
pub fn wait_fd(fd: RawFd) {
    unsafe { thread::rb_thread_wait_fd(fd) };
}

pub fn call_without_gvl<F, R, G>(func: F, unblock_func: Option<G>) -> R
where
    F: FnMut() -> R,
    G: FnMut(),
{
    unsafe {
        let ptr = if let Some(ubf) = unblock_func {
            thread::rb_thread_call_without_gvl(
                thread_call_callbox as CallbackPtr,
                util::closure_to_ptr(func),
                thread_call_callbox as CallbackPtr,
                util::closure_to_ptr(ubf),
            )
        } else {
            thread::rb_thread_call_without_gvl(
                thread_call_callbox as CallbackPtr,
                util::closure_to_ptr(func),
                ptr::null() as CallbackPtr,
                ptr::null() as CallbackPtr,
            )
        };

        util::ptr_to_data(ptr)
    }
}

pub fn call_without_gvl2<F, R, G>(func: F, unblock_func: Option<G>) -> R
where
    F: FnMut() -> R,
    G: FnMut(),
{
    unsafe {
        let ptr = if let Some(ubf) = unblock_func {
            thread::rb_thread_call_without_gvl2(
                thread_call_callbox as CallbackPtr,
                util::closure_to_ptr(func),
                thread_call_callbox as CallbackPtr,
                util::closure_to_ptr(ubf),
            )
        } else {
            thread::rb_thread_call_without_gvl2(
                thread_call_callbox as CallbackPtr,
                util::closure_to_ptr(func),
                ptr::null() as CallbackPtr,
                ptr::null() as CallbackPtr,
            )
        };

        util::ptr_to_data(ptr)
    }
}

pub fn call_with_gvl<F, R>(func: F) -> R
where
    F: FnMut() -> R,
{
    unsafe {
        let ptr = thread::rb_thread_call_with_gvl(
            thread_call_callbox as CallbackPtr,
            util::closure_to_ptr(func),
        );

        util::ptr_to_data(ptr)
    }
}

rutie_callback! {
    fn thread_create_callbox<R>(boxptr: CallbackMutPtr) -> Value
    where
        R: Object,
    {
        let mut fnbox: Box<Box<dyn FnMut() -> R>> =
            unsafe { Box::from_raw(boxptr as *mut Box<dyn FnMut() -> R>) };

        fnbox().value()
    }
}

rutie_callback! {
    fn thread_call_callbox(boxptr: CallbackMutPtr) -> CallbackPtr {
        let mut fnbox: Box<Box<dyn FnMut() -> CallbackPtr>> =
            unsafe { Box::from_raw(boxptr as *mut Box<dyn FnMut() -> CallbackPtr>) };

        fnbox()
    }
}

pub fn current() -> Value {
    unsafe { thread::rb_thread_current() }
}

pub fn main() -> Value {
    unsafe { thread::rb_thread_main() }
}

pub fn is_alone() -> bool {
    util::c_int_to_bool(unsafe { thread::rb_thread_alone() })
}

pub fn schedule() {
    unsafe { thread::rb_thread_schedule() }
}

// Windows' `struct timeval` is Winsock's: both fields are a 32-bit `long`.
#[cfg(windows)]
type TimevalSeconds = libc::c_long;
#[cfg(windows)]
type TimevalMicros = libc::c_long;

#[cfg(not(windows))]
type TimevalSeconds = libc::time_t;
#[cfg(not(windows))]
type TimevalMicros = libc::suseconds_t;

pub fn sleep_for(duration: std::time::Duration) {
    use std::convert::TryFrom;

    let time = libc::timeval {
        // Saturate rather than wrap to a negative (or short) sleep.
        tv_sec: TimevalSeconds::try_from(duration.as_secs()).unwrap_or(TimevalSeconds::MAX),
        tv_usec: duration.subsec_micros() as TimevalMicros,
    };

    unsafe { thread::rb_thread_wait_for(time) }
}

pub fn check_interrupts() {
    unsafe { thread::rb_thread_check_ints() }
}

pub fn kill(thread: Value) -> Value {
    unsafe { thread::rb_thread_kill(thread) }
}

pub fn wakeup(thread: Value) -> Value {
    unsafe { thread::rb_thread_wakeup(thread) }
}

pub fn run(thread: Value) -> Value {
    unsafe { thread::rb_thread_run(thread) }
}

pub fn local_get(thread: Value, name: &str) -> Value {
    unsafe { thread::rb_thread_local_aref(thread, crate::binding::symbol::internal_id(name)) }
}

pub fn local_set(thread: Value, name: &str, value: Value) -> Value {
    unsafe {
        thread::rb_thread_local_aset(thread, crate::binding::symbol::internal_id(name), value)
    }
}

#[cfg(any(unix, windows))]
pub fn wait_fd_writable(fd: RawFd) {
    unsafe { thread::rb_thread_fd_writable(fd) };
}

// Like `call_without_gvl`, with `RUBY_UBF_IO` as the unblocking function:
// Ruby interrupts a blocking system call in `func` when the thread must stop.
pub fn call_without_gvl_io<F, R>(func: F) -> R
where
    F: FnMut() -> R,
{
    unsafe {
        let ptr = thread::rb_thread_call_without_gvl(
            thread_call_callbox as CallbackPtr,
            util::closure_to_ptr(func),
            thread::RUBY_UBF_IO as CallbackPtr,
            ptr::null() as CallbackPtr,
        );

        util::ptr_to_data(ptr)
    }
}

pub fn mutex_new() -> Value {
    unsafe { thread::rb_mutex_new() }
}

pub fn mutex_lock(mutex: Value) -> Value {
    unsafe { thread::rb_mutex_lock(mutex) }
}

pub fn mutex_unlock(mutex: Value) -> Value {
    unsafe { thread::rb_mutex_unlock(mutex) }
}

pub fn mutex_try_lock(mutex: Value) -> bool {
    unsafe { thread::rb_mutex_trylock(mutex) }.is_true()
}

pub fn mutex_is_locked(mutex: Value) -> bool {
    unsafe { thread::rb_mutex_locked_p(mutex) }.is_true()
}

pub fn mutex_sleep(mutex: Value, timeout: Value) -> Value {
    unsafe { thread::rb_mutex_sleep(mutex, timeout) }
}

rutie_callback! {
    fn synchronize_callback<F>(data: CallbackMutPtr) -> Value
    where
        F: FnOnce() -> Value,
    {
        match unsafe { (*(data as *mut Option<F>)).take() } {
            Some(func) => crate::binding::vm::call_catching_panic(func),
            None => Value::from(
                crate::binding::global::RubySpecialConsts::Nil as crate::types::InternalValue,
            ),
        }
    }
}

// Runs `func` holding `mutex`; the mutex is unlocked (through `rb_ensure`)
// even when `func` raises.
pub fn mutex_synchronize<F>(mutex: Value, func: F) -> Value
where
    F: FnOnce() -> Value,
{
    let mut func = Some(func);

    unsafe {
        thread::rb_mutex_synchronize(
            mutex,
            synchronize_callback::<F>,
            &mut func as *mut Option<F> as CallbackMutPtr,
        )
    }
}

pub fn fiber_new<F>(func: F) -> Value
where
    F: FnMut(&[Value]) -> Value + 'static,
{
    let data = crate::binding::rproc::closure_data(func);

    unsafe { thread::rb_fiber_new(crate::binding::rproc::proc_callback, data) }
}

pub fn fiber_resume(fiber: Value, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { thread::rb_fiber_resume(fiber, argc, argv) }
}

pub fn fiber_yield(arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { thread::rb_fiber_yield(argc, argv) }
}

pub fn fiber_current() -> Value {
    unsafe { thread::rb_fiber_current() }
}

pub fn fiber_is_alive(fiber: Value) -> bool {
    unsafe { thread::rb_fiber_alive_p(fiber) }.is_true()
}
