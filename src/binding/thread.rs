use std::ptr;

use crate::{
    rubysys::{scheduler, thread},
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

pub fn has_gvl() -> bool {
    unsafe { thread::ruby_thread_has_gvl_p() != 0 }
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

fn timeval_from(duration: std::time::Duration) -> libc::timeval {
    use std::convert::TryFrom;

    libc::timeval {
        // Saturate rather than wrap to a negative (or short) time.
        tv_sec: TimevalSeconds::try_from(duration.as_secs()).unwrap_or(TimevalSeconds::MAX),
        tv_usec: duration.subsec_micros() as TimevalMicros,
    }
}

pub fn sleep_for(duration: std::time::Duration) {
    unsafe { thread::rb_thread_wait_for(timeval_from(duration)) }
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

// Ruby 3.3 and later on arm64 enter a new fiber by `ret`urning into its entry
// function (`fiber_entry`) with that same address in the link register, so
// the entry function's frame record holds a return address where Ruby 3.2
// and x86_64 have 0. A stack walk from inside the fiber, such as the Rust
// backtrace of a panic with `RUST_BACKTRACE` set, then carries on below the
// fiber stack with a null frame pointer and crashes (and Ruby's crash report
// hangs doing the same walk). `fiber_entry` never returns (`rb_fiber_start`
// ends in `rb_fiber_terminate`), so its saved return address is cleared,
// which ends the walk there as on Ruby 3.2.
//
// Frame records are only walked on macOS, whose arm64 ABI requires them.
#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
#[inline(never)]
fn end_fiber_frame_chain() {
    // A frame record is the caller's frame pointer followed by the return
    // address; x29 points at this function's own record.
    let mut record: *mut usize;
    unsafe { std::arch::asm!("mov {}, x29", out(reg) record) };

    for _ in 0..4096 {
        if record.is_null() {
            return;
        }

        let caller = unsafe { *record } as *mut usize;

        if caller.is_null() {
            // The bottom of the fiber stack.
            unsafe { *record.add(1) = 0 };
            return;
        }

        // Records are further up the stack the further out they are; stop
        // on anything else rather than follow a broken chain.
        if caller <= record {
            return;
        }

        record = caller;
    }
}

#[cfg(not(all(target_arch = "aarch64", target_os = "macos")))]
fn end_fiber_frame_chain() {}

// The body of a fiber created from Rust; see `end_fiber_frame_chain`.
fn fiber_body<F>(mut func: F) -> impl FnMut(&[Value]) -> Value + 'static
where
    F: FnMut(&[Value]) -> Value + 'static,
{
    move |arguments| {
        end_fiber_frame_chain();

        func(arguments)
    }
}

pub fn fiber_new<F>(func: F) -> Value
where
    F: FnMut(&[Value]) -> Value + 'static,
{
    let data = crate::binding::rproc::closure_data(fiber_body(func));

    unsafe { thread::rb_fiber_new(crate::binding::rproc::proc_callback, data) }
}

pub fn fiber_new_storage<F>(func: F, storage: Value) -> Value
where
    F: FnMut(&[Value]) -> Value + 'static,
{
    let data = crate::binding::rproc::closure_data(fiber_body(func));

    unsafe { thread::rb_fiber_new_storage(crate::binding::rproc::proc_callback, data, storage) }
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

// The `_kw` variants take keywords as a `Hash` at the end of `arguments`.
pub fn fiber_resume_kw(fiber: Value, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { thread::rb_fiber_resume_kw(fiber, argc, argv, 1) }
}

pub fn fiber_yield_kw(arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { thread::rb_fiber_yield_kw(argc, argv, 1) }
}

pub fn fiber_transfer(fiber: Value, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { thread::rb_fiber_transfer(fiber, argc, argv) }
}

pub fn fiber_transfer_kw(fiber: Value, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { thread::rb_fiber_transfer_kw(fiber, argc, argv, 1) }
}

pub fn fiber_raise(fiber: Value, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { thread::rb_fiber_raise(fiber, argc, argv) }
}

pub fn is_fiber(object: Value) -> bool {
    unsafe { thread::rb_obj_is_fiber(object) }.is_true()
}

pub fn fiber_scheduler_get() -> Value {
    unsafe { scheduler::rb_fiber_scheduler_get() }
}

pub fn fiber_scheduler_set(scheduler: Value) -> Value {
    unsafe { scheduler::rb_fiber_scheduler_set(scheduler) }
}

pub fn fiber_scheduler_current() -> Value {
    unsafe { scheduler::rb_fiber_scheduler_current() }
}

pub fn fiber_scheduler_current_for_thread(thread: Value) -> Value {
    unsafe { scheduler::rb_fiber_scheduler_current_for_thread(thread) }
}

pub fn fiber_scheduler_yield(scheduler: Value) -> Value {
    unsafe { scheduler::rb_fiber_scheduler_yield(scheduler) }
}

// `Qundef` when `scheduler` has no `fiber_interrupt`.
pub fn fiber_scheduler_fiber_interrupt(scheduler: Value, fiber: Value, exception: Value) -> Value {
    unsafe { scheduler::rb_fiber_scheduler_fiber_interrupt(scheduler, fiber, exception) }
}

pub fn fiber_scheduler_make_timeout(timeout: Option<std::time::Duration>) -> Value {
    match timeout {
        Some(duration) => {
            let mut time = timeval_from(duration);

            unsafe { scheduler::rb_fiber_scheduler_make_timeout(&mut time) }
        }
        None => unsafe { scheduler::rb_fiber_scheduler_make_timeout(ptr::null_mut()) },
    }
}

// `func` is called with the event and its data, on any native thread and
// mostly without the GVL; a panic in it is caught and dropped (it cannot
// unwind into Ruby, nor be raised without the GVL).
rutie_callback! {
    fn internal_thread_event_callback<F>(
        event: u32,
        event_data: *const thread::InternalThreadEventData,
        user_data: *mut c_void,
    )
    where
        F: Fn(u32, *const thread::InternalThreadEventData) + Send + Sync + 'static,
    {
        let func = unsafe { &*(user_data as *const F) };

        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| func(event, event_data)));
    }
}

// Registers `func` for the thread events in `events`. Returns the hook and
// the boxed `func`, which must be passed to
// `internal_thread_remove_event_hook` exactly once, or `None` where Ruby does
// not implement hooks (Windows).
pub fn internal_thread_add_event_hook<F>(
    events: u32,
    func: F,
) -> Option<(*mut thread::InternalThreadEventHook, *mut F)>
where
    F: Fn(u32, *const thread::InternalThreadEventData) + Send + Sync + 'static,
{
    let data = Box::into_raw(Box::new(func));
    let hook = unsafe {
        thread::rb_internal_thread_add_event_hook(
            internal_thread_event_callback::<F>,
            events,
            data as *mut c_void,
        )
    };

    if hook.is_null() {
        drop(unsafe { Box::from_raw(data) });

        None
    } else {
        Some((hook, data))
    }
}

// Ruby takes its hook list's write lock to unregister, so no call of `func`
// is running once this returns, and `func` is dropped.
pub unsafe fn internal_thread_remove_event_hook<F>(
    hook: *mut thread::InternalThreadEventHook,
    func: *mut F,
) -> bool {
    let removed = thread::rb_internal_thread_remove_event_hook(hook);

    // A hook Ruby did not find may still be called; keep `func` then.
    if removed {
        drop(Box::from_raw(func));
    }

    removed
}

pub fn internal_thread_specific_key_create() -> thread::InternalThreadSpecificKey {
    unsafe { thread::rb_internal_thread_specific_key_create() }
}

// `key` must come from `internal_thread_specific_key_create` (and be in
// range), and `thread` must be a `Thread`. Ruby only stores `data`.
pub fn internal_thread_specific_get(
    thread: Value,
    key: thread::InternalThreadSpecificKey,
) -> *mut c_void {
    unsafe { thread::rb_internal_thread_specific_get(thread, key) }
}

pub fn internal_thread_specific_set(
    thread: Value,
    key: thread::InternalThreadSpecificKey,
    data: *mut c_void,
) {
    unsafe { thread::rb_internal_thread_specific_set(thread, key, data) }
}

pub fn lock_native_thread() -> bool {
    unsafe { thread::rb_thread_lock_native_thread() }
}

pub fn stop() -> Value {
    unsafe { thread::rb_thread_stop() }
}

// `false` for a dead thread.
pub fn wakeup_alive(thread: Value) -> bool {
    !unsafe { thread::rb_thread_wakeup_alive(thread) }.is_nil()
}
