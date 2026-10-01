use std::{ffi::CStr, ptr};

use crate::{
    binding::global::RubySpecialConsts,
    rubysys::{
        exception,
        process::{self, rb_pid_t},
    },
    types::{c_char, c_int, InternalValue, Value},
    util,
};

fn nil() -> Value {
    Value::from(RubySpecialConsts::Nil as InternalValue)
}

// The pid of the child; raises for invalid arguments and the
// `SystemCallError` when the child could not be started.
pub fn spawn(arguments: &[Value]) -> rb_pid_t {
    let (argc, argv) = util::process_arguments(arguments);
    let mut message = [0 as c_char; 256];

    let pid = unsafe { process::rb_spawn_err(argc, argv, message.as_mut_ptr(), message.len()) };

    if pid == -1 {
        // `errno` is still the one of the failure.
        let message = if message[0] == 0 {
            ptr::null()
        } else {
            message.as_ptr()
        };

        unsafe { exception::rb_sys_fail(message) }
    }

    pid
}

// `Some((pid, status))`, or `None` for `WNOHANG` and no child that exited;
// raises the `SystemCallError` on failure.
pub fn waitpid(pid: rb_pid_t, flags: c_int) -> Option<(rb_pid_t, c_int)> {
    let mut status = 0;

    match unsafe { process::rb_waitpid(pid, &mut status, flags) } {
        -1 => unsafe { exception::rb_sys_fail(ptr::null()) },
        0 => None,
        pid => Some((pid, status)),
    }
}

pub fn detach(pid: rb_pid_t) -> Value {
    unsafe { process::rb_detach_process(pid) }
}

pub fn last_status() -> Value {
    unsafe { process::rb_last_status_get() }
}

pub fn set_last_status(status: c_int, pid: rb_pid_t) {
    unsafe { process::rb_last_status_set(status, pid) }
}

pub fn times() -> Value {
    unsafe { process::rb_proc_times(nil()) }
}

pub fn kill(arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { process::rb_f_kill(argc, argv) }
}

pub fn signal_name(signo: c_int) -> Option<&'static str> {
    let name = unsafe { process::ruby_signal_name(signo) };

    if name.is_null() {
        None
    } else {
        // Static ASCII names.
        unsafe { CStr::from_ptr(name) }.to_str().ok()
    }
}

pub fn default_signal(signo: c_int) {
    unsafe { process::ruby_default_signal(signo) }
}

// Only returns by raising.
pub fn exec(arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { process::rb_f_exec(argc, argv) }
}

// Only returns by raising the `SystemCallError` of the failure.
pub fn exec_command(command: &CStr) -> ! {
    unsafe {
        process::rb_proc_exec(command.as_ptr());

        exception::rb_sys_fail(command.as_ptr())
    }
}

pub fn setenv(name: &CStr, value: Option<&CStr>) {
    unsafe { process::ruby_setenv(name.as_ptr(), value.map_or(ptr::null(), CStr::as_ptr)) }
}

pub fn unsetenv(name: &CStr) {
    unsafe { process::ruby_unsetenv(name.as_ptr()) }
}
