// Processes, signals and the environment: `ruby/internal/intern/process.h`,
// `ruby/internal/intern/signal.h` and `ruby/util.h`.

use crate::rubysys::types::{c_char, c_int, size_t, Argc, Value};

// `rb_pid_t`: `pid_t` (see `scheduler::RbPid`).
#[allow(non_camel_case_types)]
pub type rb_pid_t = crate::rubysys::scheduler::RbPid;

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_detach_process(rb_pid_t pid)
    //
    // `Process.detach(pid)`: a thread that reaps `pid`; its value is the
    // `Process::Status`.
    pub fn rb_detach_process(pid: rb_pid_t) -> Value;
    // VALUE
    // rb_f_exec(int argc, const VALUE *argv)
    //
    // `Kernel#exec`: replaces the process, or raises; it never returns.
    pub fn rb_f_exec(argc: Argc, argv: *const Value) -> Value;
    // VALUE
    // rb_last_status_get(void)
    //
    // `$?`: the `Process::Status` of the last child waited for by this
    // thread, or `Qnil`.
    pub fn rb_last_status_get() -> Value;
    // void
    // rb_last_status_set(int status, rb_pid_t pid)
    //
    // Sets `$?` from a `waitpid(2)` status.
    pub fn rb_last_status_set(status: c_int, pid: rb_pid_t);
    // int
    // rb_proc_exec(const char *cmd)
    //
    // Replaces the process with `cmd` (run by the shell when it needs one).
    // Returns -1 with `errno` set only when that fails.
    pub fn rb_proc_exec(cmd: *const c_char) -> c_int;
    // VALUE
    // rb_proc_times(VALUE _)
    //
    // `Process.times`: a `Process::Tms`.
    pub fn rb_proc_times(_unused: Value) -> Value;
    // rb_pid_t
    // rb_spawn(int argc, const VALUE *argv)
    //
    // `Process.spawn(*argv)`; raises for invalid arguments and returns -1
    // when the child could not be started.
    pub fn rb_spawn(argc: Argc, argv: *const Value) -> rb_pid_t;
    // rb_pid_t
    // rb_spawn_err(int argc, const VALUE *argv, char *errbuf, size_t buflen)
    //
    // Like `rb_spawn`; on -1 (with `errno` set), may write the step that
    // failed (such as `"chdir"`) to `errbuf` (`buflen` bytes, including the
    // NUL); it stays empty when the program could not be run.
    pub fn rb_spawn_err(
        argc: Argc,
        argv: *const Value,
        errbuf: *mut c_char,
        buflen: size_t,
    ) -> rb_pid_t;
    // void
    // rb_syswait(rb_pid_t pid)
    //
    // `rb_waitpid(pid, NULL, 0)`, also setting `$?`.
    pub fn rb_syswait(pid: rb_pid_t);
    // rb_pid_t
    // rb_waitpid(rb_pid_t pid, int *status, int flags)
    //
    // `waitpid(2)` without the GVL; `status` may be NULL. Returns the pid
    // reaped, 0 (with `WNOHANG`) or -1 with `errno` set, and sets `$?`.
    pub fn rb_waitpid(pid: rb_pid_t, status: *mut c_int, flags: c_int) -> rb_pid_t;

    // VALUE
    // rb_f_kill(int argc, const VALUE *argv)
    //
    // `Process.kill(signal, *pids)`; the signal is a number or a name.
    pub fn rb_f_kill(argc: Argc, argv: *const Value) -> Value;
    // void
    // ruby_default_signal(int sig)
    //
    // Restores the default action of `sig` and sends it to this process.
    pub fn ruby_default_signal(sig: c_int);
    // const char *
    // ruby_signal_name(int signo)
    //
    // The signal's name without `SIG` (`"KILL"`), or NULL.
    pub fn ruby_signal_name(signo: c_int) -> *const c_char;

    // void
    // ruby_setenv(const char *key, const char *val)
    //
    // Sets the variable, or unsets it when `val` is NULL; raises a
    // `SystemCallError` on failure (such as for a name with `=`).
    pub fn ruby_setenv(key: *const c_char, val: *const c_char);
    // void
    // ruby_unsetenv(const char *key)
    pub fn ruby_unsetenv(key: *const c_char);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::{string, vm};
    use std::ffi::CStr;

    #[test]
    fn test_processes_and_signals() {
        crate::on_ruby_thread(|| unsafe {
            let arguments = [
                vm::eval_string("require 'rbconfig'; RbConfig.ruby"),
                string::new_utf8("-e"),
                string::new_utf8("exit 6"),
            ];
            let pid = rb_spawn(3, arguments.as_ptr());
            assert!(pid > 0);
            rb_syswait(pid);
            let status = rb_last_status_get();
            assert_eq!(
                crate::binding::fixnum::num_to_i32(vm::call_method(status, "exitstatus", &[])),
                6
            );

            let pid = rb_spawn(3, arguments.as_ptr());
            let mut raw_status = 0;
            assert_eq!(rb_waitpid(pid, &mut raw_status, 0), pid);
            assert_eq!(rb_waitpid(pid, std::ptr::null_mut(), 0), -1);

            let missing = [string::new_utf8("/no/such/rutie")];
            let mut message = [0 as c_char; 64];
            assert_eq!(
                rb_spawn_err(1, missing.as_ptr(), message.as_mut_ptr(), message.len()),
                -1
            );
            let chdir = [
                vm::eval_string("require 'rbconfig'; RbConfig.ruby"),
                vm::eval_string("{ chdir: '/no/such/rutie/dir' }"),
            ];
            assert_eq!(
                rb_spawn_err(2, chdir.as_ptr(), message.as_mut_ptr(), message.len()),
                -1
            );
            if cfg!(unix) {
                assert_eq!(CStr::from_ptr(message.as_ptr()).to_str().unwrap(), "chdir");
            }

            let pid = rb_spawn(3, arguments.as_ptr());
            let thread = rb_detach_process(pid);
            vm::call_method(thread, "join", &[]);

            rb_last_status_set(0, 77);
            assert_eq!(
                crate::binding::fixnum::num_to_i32(vm::call_method(
                    rb_last_status_get(),
                    "pid",
                    &[]
                )),
                77
            );

            let times = rb_proc_times(Value::from(0));
            let class = vm::call_method(times, "class", &[]);
            assert_eq!(
                string::value_to_string(vm::call_method(class, "name", &[])),
                "Process::Tms"
            );

            let kill = [
                crate::binding::fixnum::i32_to_num(0),
                crate::binding::fixnum::i32_to_num(std::process::id() as i32),
            ];
            assert_eq!(
                crate::binding::fixnum::num_to_i32(rb_f_kill(2, kill.as_ptr())),
                1
            );
            assert_eq!(
                CStr::from_ptr(ruby_signal_name(15)).to_str().unwrap(),
                "TERM"
            );
            assert!(ruby_signal_name(4096).is_null());

            let error = vm::protect_value(|| rb_f_exec(1, missing.as_ptr()));
            assert!(error.is_err());
            assert_eq!(rb_proc_exec(b"\0".as_ptr() as *const c_char), -1);

            ruby_setenv(
                b"RUTIE_SYS_ENV\0".as_ptr() as *const c_char,
                b"1\0".as_ptr() as *const c_char,
            );
            assert_eq!(std::env::var("RUTIE_SYS_ENV").unwrap(), "1");
            ruby_setenv(
                b"RUTIE_SYS_ENV\0".as_ptr() as *const c_char,
                std::ptr::null(),
            );
            assert!(std::env::var("RUTIE_SYS_ENV").is_err());
            ruby_setenv(
                b"RUTIE_SYS_ENV\0".as_ptr() as *const c_char,
                b"2\0".as_ptr() as *const c_char,
            );
            ruby_unsetenv(b"RUTIE_SYS_ENV\0".as_ptr() as *const c_char);
            assert!(std::env::var("RUTIE_SYS_ENV").is_err());
        });
    }
}
