use rubysys::types::{CallbackPtr, c_void, Value};

#[cfg(unix)]
use rubysys::types::RawFd;

extern "C" {
    pub fn rb_thread_call_without_gvl(func: CallbackPtr,
                                      args: *const c_void,
                                      unblock_func: CallbackPtr,
                                      unblock_args: *const c_void)
                                      -> *mut c_void;

    pub fn rb_thread_call_without_gvl2(func: CallbackPtr,
                                       args: *const c_void,
                                       unblock_func: CallbackPtr,
                                       unblock_args: *const c_void)
                                       -> *mut c_void;

    pub fn rb_thread_call_with_gvl(func: CallbackPtr, args: *const c_void) -> *mut c_void;

    pub fn rb_thread_create(function: extern "C" fn(*mut c_void) -> Value,
                            data: *mut c_void)
                            -> Value;

    #[cfg(unix)]
    pub fn rb_thread_wait_fd(fd: RawFd);
}
