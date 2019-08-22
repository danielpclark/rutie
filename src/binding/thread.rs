use std::ptr;

use rubysys::thread;

use types::{c_void, CallbackPtr, CallbackMutPtr, Value};
use util;

#[cfg(unix)]
use types::RawFd;

use Object;

pub fn create<F, R>(func: F) -> Value
where
    F: FnMut() -> R,
    R: Object,
{
    let fnbox = Box::new(func) as Box<dyn FnMut() -> R>;

    let closure_ptr = Box::into_raw(Box::new(fnbox)) as CallbackMutPtr;

    unsafe { thread::rb_thread_create(thread_create_callbox::<R>, closure_ptr) }
}

#[cfg(unix)]
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

extern "C" fn thread_create_callbox<R>(boxptr: CallbackMutPtr) -> Value
where
    R: Object,
{
    let mut fnbox: Box<Box<dyn FnMut() -> R>> =
        unsafe { Box::from_raw(boxptr as *mut Box<dyn FnMut() -> R>) };

    fnbox().value()
}

extern "C" fn thread_call_callbox(boxptr: CallbackMutPtr) -> CallbackPtr {
    let mut fnbox: Box<Box<dyn FnMut() -> CallbackPtr>> =
        unsafe { Box::from_raw(boxptr as *mut Box<dyn FnMut() -> CallbackPtr>) };

    fnbox()
}
