use rubysys::types::{CallbackPtr, c_void, Value, c_int};

#[cfg(unix)]
use rubysys::types::RawFd;

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
extern "C" {
    // void *
    // rb_thread_call_without_gvl(void *(*func)(void *data), void *data1,
    //                            rb_unblock_function_t *ubf, void *data2)
    pub fn rb_thread_call_without_gvl(func: CallbackPtr,
                                      args: *const c_void,
                                      unblock_func: CallbackPtr,
                                      unblock_args: *const c_void)
                                      -> *mut c_void;

    // void *
    // rb_thread_call_without_gvl2(void *(*func)(void *), void *data1,
    //                             rb_unblock_function_t *ubf, void *data2)
    pub fn rb_thread_call_without_gvl2(func: CallbackPtr,
                                       args: *const c_void,
                                       unblock_func: CallbackPtr,
                                       unblock_args: *const c_void)
                                       -> *mut c_void;

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
    pub fn rb_thread_create(function: extern "C" fn(*mut c_void) -> Value,
                            data: *mut c_void)
                            -> Value;

    // void
    // rb_thread_wait_fd(int fd)
    #[cfg(unix)]
    pub fn rb_thread_wait_fd(fd: RawFd);

    // This function can be called in blocking region.
    //
    // int
    // rb_thread_interrupted(VALUE thval)
    pub fn rb_thread_interrupted(thread: Value) -> c_int;
}
