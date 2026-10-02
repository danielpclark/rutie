use std::ffi::CString;

use crate::{
    binding::{process, vm},
    types::Value,
    util, AnyException, AnyObject, Exception, Fixnum, Integer, NilClass, Object, Struct, Thread,
};

fn protect<F>(func: F) -> Result<Value, AnyException>
where
    F: FnOnce() -> Value,
{
    vm::protect_value(func).map_err(AnyException::from)
}

fn to_cstring(string: &str, what: &str) -> Result<CString, AnyException> {
    CString::new(string).map_err(|_| {
        AnyException::new(
            "ArgumentError",
            Some(&format!("{} contains a NUL byte", what)),
        )
    })
}

/// Child processes, signals and the environment of the current process
/// (Ruby's `Process`, `Kernel#exec`, `$?` and `ENV`).
///
/// Process ids are `i32` (`pid_t`).
pub struct Process;

impl Process {
    /// Starts a child process like Ruby's `Process.spawn` and returns its
    /// pid without waiting for it (`rb_spawn_err`). `arguments` are those of
    /// `Process.spawn`: an optional environment Hash, the command (one
    /// String run by the shell when it needs one, or the program and its
    /// arguments) and an optional options Hash.
    ///
    /// Returns the error for invalid arguments, or the `SystemCallError`
    /// when the child could not be started (`Errno::ENOENT` for a missing
    /// program).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Fixnum, Object, Process, RString, VM};
    /// # VM::init();
    ///
    /// let ruby = VM::eval("require 'rbconfig'; RbConfig.ruby").unwrap();
    /// let script = RString::new_utf8("exit 3");
    ///
    /// let pid = Process::spawn(&[ruby, RString::new_utf8("-e").into(), script.into()]).unwrap();
    ///
    /// assert_eq!(Process::waitpid(pid, 0).unwrap().unwrap().0, pid);
    ///
    /// let status = Process::last_status().unwrap();
    /// assert_eq!(unsafe { status.send("exitstatus", &[]) }.try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
    ///
    /// let error = Process::spawn(&[RString::new_utf8("/no/such/rutie/program").into()]).unwrap_err();
    /// assert!(Class::from_existing("Errno").get_nested_class("ENOENT").case_equals(&error));
    /// ```
    pub fn spawn(arguments: &[AnyObject]) -> Result<i32, AnyException> {
        let arguments = util::arguments_to_values(arguments);
        let mut pid = 0;

        protect(|| {
            pid = process::spawn(&arguments);

            NilClass::new().value()
        })
        .map(|_| pid as i32)
    }

    /// Waits for the child `pid` (-1 for any child) with `waitpid(2)` flags
    /// (`Process::WNOHANG` is 1), letting other Ruby threads run meanwhile
    /// (`rb_waitpid`). Returns the pid and the raw wait status, or `None`
    /// when `WNOHANG` found no child that had finished, and sets `$?` (see
    /// [`Process::last_status`](#method.last_status)).
    ///
    /// Returns the `SystemCallError` on failure (`Errno::ECHILD` when there
    /// is no such child).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Fixnum, Object, Process, RString, VM};
    /// # VM::init();
    ///
    /// let ruby = VM::eval("require 'rbconfig'; RbConfig.ruby").unwrap();
    /// let pid = Process::spawn(&[ruby, RString::new_utf8("-e").into(), RString::new_utf8("sleep 0.2").into()]).unwrap();
    ///
    /// // WNOHANG (1 on Unix, -1 on Windows): the child is still running.
    /// let wnohang = VM::eval("Process::WNOHANG").unwrap().try_convert_to::<Fixnum>().unwrap().to_i32();
    /// assert_eq!(Process::waitpid(pid, wnohang).unwrap(), None);
    ///
    /// let (waited, status) = Process::waitpid(pid, 0).unwrap().unwrap();
    /// assert_eq!(waited, pid);
    /// assert_eq!(status, 0);
    ///
    /// let error = Process::waitpid(pid, 0).unwrap_err();
    /// assert!(Class::from_existing("Errno").get_nested_class("ECHILD").case_equals(&error));
    /// ```
    pub fn waitpid(pid: i32, flags: i32) -> Result<Option<(i32, i32)>, AnyException> {
        let mut result = None;

        protect(|| {
            result = process::waitpid(pid as _, flags);

            NilClass::new().value()
        })
        .map(|_| result.map(|(pid, status)| (pid as i32, status)))
    }

    /// Reaps the child `pid` in a new thread, so that it does not remain a
    /// zombie, and returns the thread; its value is the child's
    /// `Process::Status` (Ruby's `Process.detach`, `rb_detach_process`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Process, RString, VM};
    /// # VM::init();
    ///
    /// let ruby = VM::eval("require 'rbconfig'; RbConfig.ruby").unwrap();
    /// let pid = Process::spawn(&[ruby, RString::new_utf8("-e").into(), RString::new_utf8("exit 5").into()]).unwrap();
    ///
    /// let status = Process::detach(pid).join_value().unwrap();
    ///
    /// assert_eq!(unsafe { status.send("exitstatus", &[]) }.try_convert_to::<Fixnum>(), Ok(Fixnum::new(5)));
    /// ```
    pub fn detach(pid: i32) -> Thread {
        Thread::from(process::detach(pid as _))
    }

    /// Returns `$?`, the `Process::Status` of the last child the current
    /// thread waited for, or `None` (`rb_last_status_get`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Boolean, Object, Process, VM};
    /// # VM::init();
    ///
    /// VM::eval("require 'rbconfig'; system(RbConfig.ruby, '-e', 'exit 0')").unwrap();
    ///
    /// let status = Process::last_status().unwrap();
    /// assert_eq!(unsafe { status.send("success?", &[]) }.try_convert_to::<Boolean>(), Ok(Boolean::new(true)));
    /// ```
    pub fn last_status() -> Option<AnyObject> {
        let status = process::last_status();

        if status.is_nil() {
            None
        } else {
            Some(AnyObject::from(status))
        }
    }

    /// Sets `$?` of the current thread to a `Process::Status` for `pid` with
    /// the raw `waitpid(2)` `status` (`rb_last_status_set`), as when a
    /// child reaped by C code exited.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Process, VM};
    /// # VM::init();
    ///
    /// // An exit status of 2 is the wait status 0x200 (Ruby shifts the exit
    /// // code the same way on Windows).
    /// Process::set_last_status(2 << 8, 12345);
    ///
    /// let last = Process::last_status().unwrap();
    /// assert_eq!(unsafe { last.send("pid", &[]) }.try_convert_to::<Fixnum>(), Ok(Fixnum::new(12345)));
    /// assert_eq!(unsafe { last.send("exitstatus", &[]) }.try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// ```
    pub fn set_last_status(status: i32, pid: i32) {
        process::set_last_status(status, pid as _);
    }

    /// Returns the user and system CPU times of the process and its
    /// children as a `Process::Tms` (Ruby's `Process.times`,
    /// `rb_proc_times`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Float, Object, Process, VM};
    /// # VM::init();
    ///
    /// let times = Process::times();
    ///
    /// assert_eq!(times.members().length(), 4);
    /// assert!(times.get("utime").unwrap().try_convert_to::<Float>().unwrap().to_f64() >= 0.0);
    /// ```
    pub fn times() -> Struct {
        Struct::from(process::times())
    }

    /// Sends `signal` (a signal number, or a name such as `"TERM"` or
    /// `"SIGTERM"`, as a String or Symbol) to each of `pids` and returns how
    /// many were signalled (Ruby's `Process.kill`, `rb_f_kill`). Signal 0
    /// only checks that the processes exist.
    ///
    /// Returns the `ArgumentError` for an unknown signal, or the
    /// `SystemCallError` when a process cannot be signalled (`Errno::ESRCH`
    /// when there is no such process).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Fixnum, Object, Process, RString, VM};
    /// # VM::init();
    ///
    /// let me = std::process::id() as i32;
    ///
    /// assert_eq!(Process::kill(&Fixnum::new(0), &[me]).unwrap(), 1);
    ///
    /// let error = Process::kill(&RString::new_utf8("NO_SUCH_SIGNAL"), &[me]).unwrap_err();
    /// assert!(Class::argument_error().case_equals(&error));
    /// ```
    pub fn kill<T: Object>(signal: &T, pids: &[i32]) -> Result<i64, AnyException> {
        let mut arguments = vec![signal.value()];
        arguments.extend(pids.iter().map(|&pid| Fixnum::new(i64::from(pid)).value()));

        protect(|| process::kill(&arguments)).map(|count| Integer::from(count).to_i64())
    }

    /// Returns the name of signal `signo` without its `SIG` prefix, or
    /// `None` for an unknown signal (`ruby_signal_name`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Process, VM};
    /// # VM::init();
    ///
    /// // SIGINT is 2 and SIGTERM 15 on Windows too.
    /// assert_eq!(Process::signal_name(2), Some("INT"));
    /// assert_eq!(Process::signal_name(15), Some("TERM"));
    /// assert_eq!(Process::signal_name(12345), None);
    /// ```
    pub fn signal_name(signo: i32) -> Option<&'static str> {
        process::signal_name(signo)
    }

    /// Restores the default action of signal `signo` (dropping Ruby's
    /// handler and any `trap`) and sends the signal to the current process
    /// (`ruby_default_signal`).
    ///
    /// # Safety
    ///
    /// For most signals the default action ends the process at once,
    /// without running `at_exit` handlers or Rust destructors.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Process, VM};
    /// # VM::init();
    ///
    /// // The default action of SIGWINCH (Unix only) is to ignore it.
    /// if !cfg!(windows) {
    ///     let winch = VM::eval("Signal.list['WINCH']").unwrap().try_convert_to::<Fixnum>().unwrap();
    ///
    ///     unsafe { Process::default_signal(winch.to_i32()) };
    /// }
    ///
    /// // Still running.
    /// assert!(VM::eval("1 + 1").is_ok());
    /// ```
    pub unsafe fn default_signal(signo: i32) {
        process::default_signal(signo);
    }

    /// Replaces the current process with a command like Ruby's
    /// `Kernel#exec`, with the same `arguments` as
    /// [`Process::spawn`](#method.spawn) (`rb_f_exec`). It only returns
    /// when that fails, with the error.
    ///
    /// # Safety
    ///
    /// On success the process image is replaced: nothing after the call
    /// runs, including `at_exit` handlers and Rust destructors, and other
    /// threads vanish with it.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, Process, RString, VM};
    /// # VM::init();
    ///
    /// let error = unsafe { Process::exec(&[RString::new_utf8("/no/such/rutie/program").into()]) };
    ///
    /// assert!(Class::from_existing("Errno").get_nested_class("ENOENT").case_equals(&error));
    ///
    /// // Ruby goes on after a failed exec.
    /// assert!(VM::eval("1 + 1").is_ok());
    /// ```
    pub unsafe fn exec(arguments: &[AnyObject]) -> AnyException {
        let arguments = util::arguments_to_values(arguments);

        match protect(|| process::exec(&arguments)) {
            Err(error) => error,
            Ok(_) => unreachable!("rb_f_exec returned"),
        }
    }

    /// Replaces the current process with the shell running `command`
    /// (`/bin/sh -c`, or `cmd.exe` on Windows) (`rb_proc_exec`). It only
    /// returns when that fails, with the `SystemCallError` (an empty command
    /// is `Errno::ENOENT`).
    ///
    /// # Safety
    ///
    /// The same as for [`Process::exec`](#method.exec).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, Process, VM};
    /// # VM::init();
    ///
    /// let error = unsafe { Process::exec_command("  ") };
    ///
    /// assert!(Class::from_existing("Errno").get_nested_class("ENOENT").case_equals(&error));
    /// ```
    pub unsafe fn exec_command(command: &str) -> AnyException {
        let command = match to_cstring(command, "command") {
            Ok(command) => command,
            Err(error) => return error,
        };

        match protect(|| process::exec_command(&command)) {
            Err(error) => error,
            Ok(_) => unreachable!("rb_proc_exec returned"),
        }
    }

    /// Sets the environment variable `name` to `value`, or removes it for
    /// `None`, for the process and the `ENV` of Ruby code (`ruby_setenv`).
    ///
    /// Returns the error for a name or value with a NUL byte, or the
    /// `SystemCallError` when the OS refuses (such as for a name with `=`).
    ///
    /// # Safety
    ///
    /// No other thread may read or write the environment at the same time
    /// other than through Ruby (for example with `std::env::var`); see
    /// `std::env::set_var`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Process, RString, VM};
    /// # VM::init();
    ///
    /// unsafe { Process::setenv("RUTIE_SETENV_EXAMPLE", Some("yes")) }.unwrap();
    ///
    /// let value = VM::eval("ENV['RUTIE_SETENV_EXAMPLE']").unwrap();
    /// assert_eq!(value.try_convert_to::<RString>().unwrap().to_str(), "yes");
    /// assert_eq!(std::env::var("RUTIE_SETENV_EXAMPLE").unwrap(), "yes");
    ///
    /// unsafe { Process::setenv("RUTIE_SETENV_EXAMPLE", None) }.unwrap();
    /// assert!(std::env::var("RUTIE_SETENV_EXAMPLE").is_err());
    ///
    /// assert!(unsafe { Process::setenv("A=B", Some("x")) }.is_err());
    /// ```
    pub unsafe fn setenv(name: &str, value: Option<&str>) -> Result<(), AnyException> {
        let name = to_cstring(name, "name")?;
        let value = match value {
            Some(value) => Some(to_cstring(value, "value")?),
            None => None,
        };

        protect(|| {
            process::setenv(&name, value.as_deref());

            NilClass::new().value()
        })
        .map(|_| ())
    }

    /// Removes the environment variable `name` (`ruby_unsetenv`); see
    /// [`Process::setenv`](#method.setenv).
    ///
    /// # Safety
    ///
    /// The same as for [`Process::setenv`](#method.setenv).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Process, VM};
    /// # VM::init();
    ///
    /// VM::eval("ENV['RUTIE_UNSETENV_EXAMPLE'] = '1'").unwrap();
    ///
    /// unsafe { Process::unsetenv("RUTIE_UNSETENV_EXAMPLE") }.unwrap();
    ///
    /// assert!(VM::eval("ENV['RUTIE_UNSETENV_EXAMPLE']").unwrap().is_nil());
    /// ```
    pub unsafe fn unsetenv(name: &str) -> Result<(), AnyException> {
        let name = to_cstring(name, "name")?;

        protect(|| {
            process::unsetenv(&name);

            NilClass::new().value()
        })
        .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use crate::{AnyObject, Class, Fixnum, Object, Process, RString, Symbol, VM};

    fn ruby_command(script: &str) -> Vec<AnyObject> {
        vec![
            VM::eval("require 'rbconfig'; RbConfig.ruby").unwrap(),
            RString::new_utf8("-e").into(),
            RString::new_utf8(script).into(),
        ]
    }

    fn errno_class(name: &str) -> Class {
        Class::from_existing("Errno").get_nested_class(name)
    }

    #[test]
    fn test_spawn_wait_and_status() {
        crate::on_ruby_thread(|| {
            let pid = Process::spawn(&ruby_command("exit 4")).unwrap();
            let (waited, status) = Process::waitpid(pid, 0).unwrap().unwrap();
            assert_eq!(waited, pid);
            if cfg!(unix) {
                assert_eq!(status >> 8, 4);
            }
            let last = Process::last_status().unwrap();
            assert_eq!(
                unsafe { last.send("exitstatus", &[]) }.try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(4))
            );
            assert!(errno_class("ECHILD").case_equals(&Process::waitpid(pid, 0).unwrap_err()));

            // An options Hash, and the environment.
            let mut arguments = vec![VM::eval("{ 'RUTIE_SPAWN_TEST' => '9' }").unwrap()];
            arguments.extend(ruby_command("exit ENV['RUTIE_SPAWN_TEST'].to_i"));
            let pid = Process::spawn(&arguments).unwrap();
            let status = Process::detach(pid).join_value().unwrap();
            assert_eq!(
                unsafe { status.send("exitstatus", &[]) }.try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(9))
            );

            assert!(errno_class("ENOENT").case_equals(
                &Process::spawn(&[RString::new_utf8("/no/such/rutie").into()]).unwrap_err()
            ));
            assert!(Class::type_error()
                .case_equals(&Process::spawn(&[Fixnum::new(1).into()]).unwrap_err()));

            Process::set_last_status(0, 4242);
            let last = Process::last_status().unwrap();
            assert_eq!(
                unsafe { last.send("pid", &[]) }.try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(4242))
            );

            let times = Process::times();
            assert_eq!(times.class_name(), "Process::Tms");
            assert!(times.get("stime").is_some());
        });
    }

    #[test]
    fn test_signals_and_exec_failures() {
        crate::on_ruby_thread(|| {
            let me = std::process::id() as i32;

            assert_eq!(Process::kill(&Fixnum::new(0), &[me, me]).unwrap(), 2);
            // At least one process.
            assert!(Class::argument_error()
                .case_equals(&Process::kill(&Symbol::new("SIGINT"), &[]).unwrap_err()));
            assert!(Class::argument_error()
                .case_equals(&Process::kill(&RString::new_utf8("RUTIE"), &[me]).unwrap_err()));

            assert_eq!(Process::signal_name(9), Some("KILL"));
            assert_eq!(Process::signal_name(-1), None);

            let error = unsafe { Process::exec(&[RString::new_utf8("/no/such/rutie").into()]) };
            assert!(errno_class("ENOENT").case_equals(&error));
            let error = unsafe { Process::exec(&[]) };
            assert!(Class::argument_error().case_equals(&error));
            let error = unsafe { Process::exec_command("") };
            assert!(errno_class("ENOENT").case_equals(&error));
            let error = unsafe { Process::exec_command("nul\0") };
            assert!(Class::argument_error().case_equals(&error));

            // Ruby is still fine after failed execs.
            assert_eq!(
                VM::eval("40 + 2").unwrap().try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(42))
            );
        });
    }

    #[test]
    fn test_environment() {
        crate::on_ruby_thread(|| unsafe {
            Process::setenv("RUTIE_ENV_UNIT_TEST", Some("a b")).unwrap();
            assert_eq!(std::env::var("RUTIE_ENV_UNIT_TEST").unwrap(), "a b");
            assert_eq!(
                VM::eval("ENV.fetch('RUTIE_ENV_UNIT_TEST')")
                    .unwrap()
                    .try_convert_to::<RString>()
                    .unwrap()
                    .to_str(),
                "a b"
            );

            Process::unsetenv("RUTIE_ENV_UNIT_TEST").unwrap();
            assert!(std::env::var("RUTIE_ENV_UNIT_TEST").is_err());
            // Removing it again is fine.
            Process::unsetenv("RUTIE_ENV_UNIT_TEST").unwrap();

            assert!(Process::setenv("RUTIE=BAD", Some("x")).is_err());
            assert!(Process::setenv("RUTIE_NUL", Some("x\0")).is_err());
            assert!(Process::unsetenv("RUTIE\0NUL").is_err());
        });
    }
}
