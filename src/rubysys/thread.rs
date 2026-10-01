use crate::rubysys::{
    libc::timeval,
    types::{c_int, c_void, Argc, BlockCallFunction, CallbackMutPtr, CallbackPtr, Id, Value},
};

#[cfg(any(unix, windows))]
use crate::rubysys::types::RawFd;

// rb_thread_call_without_gvl - permit concurrent/parallel execution.
// rb_thread_call_without_gvl2 - permit concurrent/parallel execution
//                               without interrupt process.
//
// rb_thread_call_without_gvl() does:
//   (1) Check interrupts.
//   (2) release GVL.
//       Other Ruby threads may run in parallel.
//   (3) call func with data1
//   (4) acquire GVL.
//       Other Ruby threads can not run in parallel any more.
//   (5) Check interrupts.
//
// rb_thread_call_without_gvl2() does:
//   (1) Check interrupt and return if interrupted.
//   (2) release GVL.
//   (3) call func with data1 and a pointer to the flags.
//   (4) acquire GVL.
//
// If another thread interrupts this thread (Thread#kill, signal delivery,
// VM-shutdown request, and so on), `ubf()' is called (`ubf()' means
// "un-blocking function").  `ubf()' should interrupt `func()' execution by
// toggling a cancellation flag, canceling the invocation of a call inside
// `func()' or similar.  Note that `ubf()' may not be called with the GVL.
//
// There are built-in ubfs and you can specify these ubfs:
//
// * RUBY_UBF_IO: ubf for IO operation
// * RUBY_UBF_PROCESS: ubf for process operation
//
// However, we can not guarantee our built-in ubfs interrupt your `func()'
// correctly. Be careful to use rb_thread_call_without_gvl(). If you don't
// provide proper ubf(), your program will not stop for Control+C or other
// shutdown events.
//
// "Check interrupts" on above list means checking asynchronous
// interrupt events (such as Thread#kill, signal delivery, VM-shutdown
// request, and so on) and calling corresponding procedures
// (such as `trap' for signals, raise an exception for Thread#raise).
// If `func()' finished and received interrupts, you may skip interrupt
// checking.  For example, assume the following func() it reads data from file.
//
//   read_func(...) {
//                   // (a) before read
//     read(buffer); // (b) reading
//                   // (c) after read
//   }
//
// If an interrupt occurs at (a) or (b), then `ubf()' cancels this
// `read_func()' and interrupts are checked. However, if an interrupt occurs
// at (c), after *read* operation is completed, checking interrupts is harmful
// because it causes irrevocable side-effect, the read data will vanish.  To
// avoid such problem, the `read_func()' should be used with
// `rb_thread_call_without_gvl2()'.
//
// If `rb_thread_call_without_gvl2()' detects interrupt, it returns
// immediately. This function does not show when the execution was interrupted.
// For example, there are 4 possible timing (a), (b), (c) and before calling
// read_func(). You need to record progress of a read_func() and check
// the progress after `rb_thread_call_without_gvl2()'. You may need to call
// `rb_thread_check_ints()' correctly or your program can not process proper
// process such as `trap' and so on.
//
// NOTE: You can not execute most of Ruby C API and touch Ruby
//       objects in `func()' and `ubf()', including raising an
//       exception, because current thread doesn't acquire GVL
//       (it causes synchronization problems).  If you need to
//       call ruby functions either use rb_thread_call_with_gvl()
//       or read source code of C APIs and confirm safety by
//       yourself.
//
// NOTE: In short, this API is difficult to use safely.  I recommend you
//       use other ways if you have.  We lack experiences to use this API.
//       Please report your problem related on it.
//
// NOTE: Releasing GVL and re-acquiring GVL may be expensive operations
//       for a short running `func()'. Be sure to benchmark and use this
//       mechanism when `func()' consumes enough time.
//
// Safe C API:
// * rb_thread_interrupted() - check interrupt flag
// * ruby_xmalloc(), ruby_xrealloc(), ruby_xfree() -
//   they will work without GVL, and may acquire GVL when GC is needed.
//
#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // void *
    // rb_thread_call_without_gvl(void *(*func)(void *data), void *data1,
    //                            rb_unblock_function_t *ubf, void *data2)
    pub fn rb_thread_call_without_gvl(
        func: CallbackPtr,
        args: *const c_void,
        unblock_func: CallbackPtr,
        unblock_args: *const c_void,
    ) -> *mut c_void;

    // void *
    // rb_thread_call_without_gvl2(void *(*func)(void *), void *data1,
    //                             rb_unblock_function_t *ubf, void *data2)
    pub fn rb_thread_call_without_gvl2(
        func: CallbackPtr,
        args: *const c_void,
        unblock_func: CallbackPtr,
        unblock_args: *const c_void,
    ) -> *mut c_void;

    // rb_thread_call_with_gvl - re-enter the Ruby world after GVL release.
    //
    // After releasing GVL using
    // rb_thread_call_without_gvl() you can not access Ruby values or invoke
    // methods. If you need to access Ruby you must use this function
    // rb_thread_call_with_gvl().
    //
    // This function rb_thread_call_with_gvl() does:
    // (1) acquire GVL.
    // (2) call passed function `func'.
    // (3) release GVL.
    // (4) return a value which is returned at (2).
    //
    // NOTE: You should not return Ruby object at (2) because such Object
    //       will not be marked.
    //
    // NOTE: If an exception is raised in `func', this function DOES NOT
    //       protect (catch) the exception.  If you have any resources
    //       which should free before throwing exception, you need use
    //       rb_protect() in `func' and return a value which represents
    //       exception was raised.
    //
    // NOTE: This function should not be called by a thread which was not
    //       created as Ruby thread (created by Thread.new or so).  In other
    //       words, this function *DOES NOT* associate or convert a NON-Ruby
    //       thread to a Ruby thread.
    //
    // void *
    // rb_thread_call_with_gvl(void *(*func)(void *), void *data1)
    pub fn rb_thread_call_with_gvl(func: CallbackPtr, args: *const c_void) -> *mut c_void;

    // VALUE
    // rb_thread_create(VALUE (*fn)(ANYARGS), void *arg)
    pub fn rb_thread_create(
        function: rutie_callback!(type fn(*mut c_void) -> Value),
        data: *mut c_void,
    ) -> Value;

    // void
    // rb_thread_wait_fd(int fd)
    #[cfg(any(unix, windows))]
    pub fn rb_thread_wait_fd(fd: RawFd);

    // This function can be called in blocking region.
    //
    // int
    // rb_thread_interrupted(VALUE thval)
    pub fn rb_thread_interrupted(thread: Value) -> c_int;
    // VALUE
    // rb_fiber_alive_p(VALUE fib)
    pub fn rb_fiber_alive_p(fiber: Value) -> Value;
    // VALUE
    // rb_fiber_current(void)
    pub fn rb_fiber_current() -> Value;
    // VALUE
    // rb_fiber_new(VALUE (*func)(ANYARGS), VALUE obj)
    //
    // `func` is called like a block function with `obj` as its second argument.
    pub fn rb_fiber_new(func: BlockCallFunction, obj: Value) -> Value;
    // VALUE
    // rb_fiber_new_storage(rb_block_call_func_t func, VALUE callback_obj, VALUE storage)
    //
    // `storage` is `Qnil` for an empty storage, or a `Hash` (with `Symbol`
    // keys, not frozen) that is copied. Ruby 3.2+.
    #[cfg(ruby_gte_3_2)]
    pub fn rb_fiber_new_storage(func: BlockCallFunction, obj: Value, storage: Value) -> Value;
    // VALUE
    // rb_fiber_resume(VALUE fib, int argc, const VALUE *argv)
    pub fn rb_fiber_resume(fiber: Value, argc: Argc, argv: *const Value) -> Value;
    // VALUE
    // rb_fiber_resume_kw(VALUE fiber, int argc, const VALUE *argv, int kw_splat)
    //
    // With `kw_splat` non-zero, the last of `argv` is a `Hash` passed as
    // keyword arguments.
    pub fn rb_fiber_resume_kw(
        fiber: Value,
        argc: Argc,
        argv: *const Value,
        kw_splat: c_int,
    ) -> Value;
    // VALUE
    // rb_fiber_yield(int argc, const VALUE *argv)
    pub fn rb_fiber_yield(argc: Argc, argv: *const Value) -> Value;
    // VALUE
    // rb_fiber_yield_kw(int argc, const VALUE *argv, int kw_splat)
    pub fn rb_fiber_yield_kw(argc: Argc, argv: *const Value, kw_splat: c_int) -> Value;
    // VALUE
    // rb_fiber_transfer(VALUE fiber, int argc, const VALUE *argv)
    pub fn rb_fiber_transfer(fiber: Value, argc: Argc, argv: *const Value) -> Value;
    // VALUE
    // rb_fiber_transfer_kw(VALUE fiber, int argc, const VALUE *argv, int kw_splat)
    pub fn rb_fiber_transfer_kw(
        fiber: Value,
        argc: Argc,
        argv: *const Value,
        kw_splat: c_int,
    ) -> Value;
    // VALUE
    // rb_fiber_raise(VALUE fiber, int argc, const VALUE *argv)
    //
    // `argv` is what `Kernel#raise` takes: an exception, or a class or
    // message, then an optional message and backtrace.
    pub fn rb_fiber_raise(fiber: Value, argc: Argc, argv: *const Value) -> Value;
    // VALUE
    // rb_obj_is_fiber(VALUE obj)
    pub fn rb_obj_is_fiber(object: Value) -> Value;
    // VALUE
    // rb_mutex_lock(VALUE mutex)
    pub fn rb_mutex_lock(mutex: Value) -> Value;
    // VALUE
    // rb_mutex_locked_p(VALUE mutex)
    pub fn rb_mutex_locked_p(mutex: Value) -> Value;
    // VALUE
    // rb_mutex_new(void)
    pub fn rb_mutex_new() -> Value;
    // VALUE
    // rb_mutex_sleep(VALUE self, VALUE timeout)
    pub fn rb_mutex_sleep(mutex: Value, timeout: Value) -> Value;
    // VALUE
    // rb_mutex_synchronize(VALUE mutex, VALUE (*func)(VALUE arg), VALUE arg)
    pub fn rb_mutex_synchronize(
        mutex: Value,
        func: rutie_callback!(type fn(CallbackMutPtr) -> Value),
        arg: CallbackMutPtr,
    ) -> Value;
    // VALUE
    // rb_mutex_trylock(VALUE mutex)
    pub fn rb_mutex_trylock(mutex: Value) -> Value;
    // VALUE
    // rb_mutex_unlock(VALUE mutex)
    pub fn rb_mutex_unlock(mutex: Value) -> Value;
    // int
    // rb_thread_alone(void)
    pub fn rb_thread_alone() -> c_int;
    // void
    // rb_thread_atfork(void)
    pub fn rb_thread_atfork();
    // void
    // rb_thread_check_ints(void)
    pub fn rb_thread_check_ints();
    // VALUE
    // rb_thread_current(void)
    pub fn rb_thread_current() -> Value;
    // int
    // rb_thread_fd_writable(int fd)
    pub fn rb_thread_fd_writable(fd: c_int) -> c_int;
    // VALUE
    // rb_thread_kill(VALUE thread)
    pub fn rb_thread_kill(thread: Value) -> Value;
    // VALUE
    // rb_thread_local_aref(VALUE thread, ID id)
    pub fn rb_thread_local_aref(thread: Value, name: Id) -> Value;
    // VALUE
    // rb_thread_local_aset(VALUE thread, ID id, VALUE val)
    pub fn rb_thread_local_aset(thread: Value, name: Id, value: Value) -> Value;
    // VALUE
    // rb_thread_main(void)
    pub fn rb_thread_main() -> Value;
    // VALUE
    // rb_thread_run(VALUE thread)
    pub fn rb_thread_run(thread: Value) -> Value;
    // void
    // rb_thread_schedule(void)
    pub fn rb_thread_schedule();
    // void
    // rb_thread_sleep(int sec)
    pub fn rb_thread_sleep(seconds: c_int);
    // void
    // rb_thread_sleep_forever(void)
    pub fn rb_thread_sleep_forever();
    // void
    // rb_thread_wait_for(struct timeval time)
    pub fn rb_thread_wait_for(time: timeval);
    // VALUE
    // rb_thread_wakeup(VALUE thread)
    pub fn rb_thread_wakeup(thread: Value) -> Value;
}

// `RUBY_UBF_IO` / `RUBY_UBF_PROCESS`: `(rb_unblock_function_t *)-1`, which
// makes Ruby interrupt a blocking system call when the thread must stop.
pub const RUBY_UBF_IO: usize = usize::MAX;
pub const RUBY_UBF_PROCESS: usize = usize::MAX;

// Thread events for `rb_internal_thread_add_event_hook` (Ruby 3.2+), an
// `rb_event_flag_t` mask.
//
// Thread started.
#[cfg(ruby_gte_3_2)]
pub const RUBY_INTERNAL_THREAD_EVENT_STARTED: u32 = 1 << 0;
// Acquiring the GVL; the hook runs without it.
#[cfg(ruby_gte_3_2)]
pub const RUBY_INTERNAL_THREAD_EVENT_READY: u32 = 1 << 1;
// Acquired the GVL; the hook runs with it.
#[cfg(ruby_gte_3_2)]
pub const RUBY_INTERNAL_THREAD_EVENT_RESUMED: u32 = 1 << 2;
// Released the GVL; the hook runs without it.
#[cfg(ruby_gte_3_2)]
pub const RUBY_INTERNAL_THREAD_EVENT_SUSPENDED: u32 = 1 << 3;
// Thread terminated; the hook runs without the GVL.
#[cfg(ruby_gte_3_2)]
pub const RUBY_INTERNAL_THREAD_EVENT_EXITED: u32 = 1 << 4;
// All thread events.
#[cfg(ruby_gte_3_2)]
pub const RUBY_INTERNAL_THREAD_EVENT_MASK: u32 = 0xff;

// typedef void rb_internal_thread_event_data_t; // for future extension.
//
// Ruby 3.2 passes no event data (a null pointer).
#[cfg(all(ruby_gte_3_2, not(ruby_gte_3_3)))]
pub type InternalThreadEventData = c_void;

// typedef struct rb_internal_thread_event_data {
//    VALUE thread;
// } rb_internal_thread_event_data_t;
//
// `thread` is the Ruby thread the event is about; the hook may run on
// another native thread.
#[cfg(ruby_gte_3_3)]
#[repr(C)]
pub struct InternalThreadEventData {
    pub thread: Value,
}

// typedef void (*rb_internal_thread_event_callback)(rb_event_flag_t event,
//               const rb_internal_thread_event_data_t *event_data,
//               void *user_data);
#[cfg(ruby_gte_3_2)]
pub type InternalThreadEventCallback = rutie_callback!(type fn(
    event: u32,
    event_data: *const InternalThreadEventData,
    user_data: *mut c_void,
));

// typedef struct rb_internal_thread_event_hook rb_internal_thread_event_hook_t;
#[cfg(ruby_gte_3_2)]
#[repr(C)]
pub struct InternalThreadEventHook {
    _private: [u8; 0],
}

// typedef int rb_internal_thread_specific_key_t;
#[cfg(ruby_gte_3_3)]
pub type InternalThreadSpecificKey = c_int;

// #define RB_INTERNAL_THREAD_SPECIFIC_KEY_MAX 8
#[cfg(ruby_gte_3_3)]
pub const RB_INTERNAL_THREAD_SPECIFIC_KEY_MAX: c_int = 8;

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // Registers `func` to run with `data` on the thread events in `events`.
    // It runs without the GVL (except for `RESUMED`), on any native thread.
    // Returns NULL on Windows, where Ruby does not implement it.
    //
    // rb_internal_thread_event_hook_t *
    // rb_internal_thread_add_event_hook(rb_internal_thread_event_callback func,
    //                                   rb_event_flag_t events, void *data)
    #[cfg(ruby_gte_3_2)]
    pub fn rb_internal_thread_add_event_hook(
        func: InternalThreadEventCallback,
        events: u32,
        data: *mut c_void,
    ) -> *mut InternalThreadEventHook;
    // Unregisters (and frees) `hook`; it must not be called from a hook, and
    // crashes when no hook at all is registered.
    //
    // bool
    // rb_internal_thread_remove_event_hook(rb_internal_thread_event_hook_t * hook)
    #[cfg(ruby_gte_3_2)]
    pub fn rb_internal_thread_remove_event_hook(hook: *mut InternalThreadEventHook) -> bool;
    // Raises a `ThreadError` once `RB_INTERNAL_THREAD_SPECIFIC_KEY_MAX` keys
    // exist. (Ruby 3.3 and 3.4 check this one key too late and return
    // `RB_INTERNAL_THREAD_SPECIFIC_KEY_MAX` itself, which is out of range.)
    //
    // rb_internal_thread_specific_key_t
    // rb_internal_thread_specific_key_create(void)
    #[cfg(ruby_gte_3_3)]
    pub fn rb_internal_thread_specific_key_create() -> InternalThreadSpecificKey;
    // Async signal safe and thread safe.
    //
    // void *
    // rb_internal_thread_specific_get(VALUE thread_val, rb_internal_thread_specific_key_t key)
    #[cfg(ruby_gte_3_3)]
    pub fn rb_internal_thread_specific_get(
        thread: Value,
        key: InternalThreadSpecificKey,
    ) -> *mut c_void;
    // Async signal safe and thread safe.
    //
    // void
    // rb_internal_thread_specific_set(VALUE thread_val, rb_internal_thread_specific_key_t key,
    //                                 void *data)
    #[cfg(ruby_gte_3_3)]
    pub fn rb_internal_thread_specific_set(
        thread: Value,
        key: InternalThreadSpecificKey,
        data: *mut c_void,
    );
}
