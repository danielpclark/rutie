use std::{convert::From, time::Duration};

use crate::{
    binding::{io, thread, vm},
    types::Value,
};

#[cfg(unix)]
use crate::types::RawFd;

use crate::{AnyException, AnyObject, Class, Float, NilClass, Object, VerifiedObject, IO};

/// `Thread`
#[derive(Debug)]
#[repr(C)]
pub struct Thread {
    value: Value,
}

impl Thread {
    /// Creates a new green thread.
    ///
    /// The returning value of the closure will be available as `#value` of the thread
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Thread, VM};
    /// # VM::init();
    ///
    /// let thread = Thread::new(|| {
    ///     let computation_result = 1 + 2;
    ///
    ///     Fixnum::new(computation_result)
    /// });
    ///
    /// let value = thread.join_value().unwrap();
    /// assert_eq!(value.try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// Thread.new do
    ///   computation_result = 1 + 2
    ///
    ///   computation_result
    /// end
    /// ```
    pub fn new<F, R>(func: F) -> Self
    where
        F: FnMut() -> R,
        R: Object,
    {
        Self::from(thread::create(func))
    }

    /// Tells scheduler to switch to other threads while current thread is waiting for a
    /// readable event on the given file descriptor.
    ///
    /// Ruby deprecates `rb_thread_wait_fd` from 3.1; prefer
    /// [`Thread::wait_readable`](#method.wait_readable).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Read, Write};
    /// use std::os::unix::io::AsRawFd;
    /// use std::os::unix::net::UnixStream;
    ///
    /// use rutie::{Thread, VM};
    /// # VM::init();
    ///
    /// let (mut reader, mut writer) = UnixStream::pair().unwrap();
    /// writer.write_all(b"ready").unwrap();
    ///
    /// // Returns once the socket has data (other Ruby threads run meanwhile).
    /// Thread::wait_fd(reader.as_raw_fd());
    ///
    /// let mut buffer = [0; 5];
    /// reader.read_exact(&mut buffer).unwrap();
    /// assert_eq!(&buffer, b"ready");
    /// ```
    #[cfg(unix)]
    pub fn wait_fd(fd: RawFd) {
        thread::wait_fd(fd);
    }

    /// Release GVL for current thread.
    ///
    /// **Warning!** Due to MRI limitations, interaction with Ruby objects is not allowed while
    /// GVL is released, it may cause unexpected behaviour.
    /// [Read more at Ruby documentation](https://github.com/ruby/ruby/blob/2fc5210f31ad23463d7b0a0e36bcfbeee7b41b3e/thread.c#L1314-L1398)
    ///
    /// You should extract all the information from Ruby world before invoking
    /// `thread_call_without_gvl`.
    ///
    /// GVL will be re-acquired when the closure is finished.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Fixnum, Object, Thread, VM};
    ///
    /// class!(Calculator);
    ///
    /// methods!(
    ///     Calculator,
    ///     rtself,
    ///
    ///     fn heavy_computation() -> Fixnum {
    ///         let computation = || { 2 * 2 };
    ///         let unblocking_function = || {};
    ///
    ///         // release GVL for current thread until `computation` is completed
    ///         let result = Thread::call_without_gvl(
    ///             computation,
    ///             Some(unblocking_function)
    ///         );
    ///
    ///         // GVL is re-acquired, we can interact with Ruby-world
    ///         Fixnum::new(result)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::new("Calculator", None).define(|klass| {
    ///         klass.def("heavy_computation", heavy_computation);
    ///     });
    ///
    ///     let result = VM::eval("Calculator.new.heavy_computation").unwrap();
    ///     assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(4)));
    /// }
    /// ```
    pub fn call_without_gvl<F, R, G>(func: F, unblock_func: Option<G>) -> R
    where
        F: FnMut() -> R,
        G: FnMut(),
    {
        thread::call_without_gvl(func, unblock_func)
    }

    /// Like [`call_without_gvl`](#method.call_without_gvl)
    /// (`rb_thread_call_without_gvl2`), but it does not process pending
    /// interrupts before returning, so the caller can clean up first and
    /// then call [`Thread::check_interrupts`](#method.check_interrupts).
    /// `func` must not touch Ruby objects.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Thread, VM};
    /// # VM::init();
    ///
    /// let total = Thread::call_without_gvl2(|| (1..=4u32).product::<u32>(), None::<fn()>);
    ///
    /// assert_eq!(total, 24);
    /// Thread::check_interrupts();
    /// ```
    pub fn call_without_gvl2<F, R, G>(func: F, unblock_func: Option<G>) -> R
    where
        F: FnMut() -> R,
        G: FnMut(),
    {
        thread::call_without_gvl2(func, unblock_func)
    }

    /// Re-acquires the GVL to run `func` from inside a
    /// [`call_without_gvl`](#method.call_without_gvl) closure
    /// (`rb_thread_call_with_gvl`), so it may use Ruby objects again.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, Thread, VM};
    /// # VM::init();
    ///
    /// let text = Thread::call_without_gvl(
    ///     || {
    ///         // No Ruby here: this may run in parallel with Ruby code...
    ///         let label = format!("{}-{}", "rust", 42);
    ///
    ///         // ...until the GVL is taken back.
    ///         Thread::call_with_gvl(|| RString::new_utf8(&label).to_string())
    ///     },
    ///     None::<fn()>,
    /// );
    ///
    /// assert_eq!(text, "rust-42");
    /// ```
    pub fn call_with_gvl<F, R>(func: F) -> R
    where
        F: FnMut() -> R,
    {
        thread::call_with_gvl(func)
    }

    /// Like [`call_without_gvl`](#method.call_without_gvl), with Ruby's
    /// `RUBY_UBF_IO` unblocking function: when the thread must stop (it is
    /// killed, or an exception is raised in it), Ruby interrupts a blocking
    /// system call in `func` with a signal.
    ///
    /// `func` runs without the GVL, possibly in parallel with Ruby code, so
    /// it must not touch Ruby objects or call Ruby APIs (use
    /// [`call_with_gvl`](#method.call_with_gvl) for that), and anything it
    /// shares with other threads must be safe to share (`Send`/`Sync`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Thread, VM};
    /// # VM::init();
    ///
    /// let sum = Thread::call_without_gvl_io(|| (1..=100u64).sum::<u64>());
    ///
    /// assert_eq!(sum, 5050);
    /// ```
    pub fn call_without_gvl_io<F, R>(func: F) -> R
    where
        F: FnMut() -> R,
    {
        thread::call_without_gvl_io(func)
    }

    /// Returns the Ruby thread running now (Ruby's `Thread.current`,
    /// `rb_thread_current`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Thread, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Thread::current(), Thread::main());
    /// ```
    pub fn current() -> Self {
        Thread::from(thread::current())
    }

    /// Returns the main Ruby thread (Ruby's `Thread.main`, `rb_thread_main`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Thread, VM};
    /// # VM::init();
    ///
    /// assert!(Thread::main().is_alive());
    /// ```
    pub fn main() -> Self {
        Thread::from(thread::main())
    }

    /// Returns `true` if no other Ruby thread is running
    /// (`rb_thread_alone`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Thread, VM};
    /// # VM::init();
    ///
    /// assert!(Thread::is_alone());
    /// ```
    pub fn is_alone() -> bool {
        thread::is_alone()
    }

    /// Lets other Ruby threads run (Ruby's `Thread.pass`,
    /// `rb_thread_schedule`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Thread, VM};
    /// # VM::init();
    ///
    /// // Give a started thread a chance to run.
    /// let thread = Thread::new(|| rutie::Fixnum::new(1));
    /// while thread.is_alive() {
    ///     Thread::pass();
    /// }
    ///
    /// assert!(!thread.is_alive());
    /// ```
    pub fn pass() {
        thread::schedule()
    }

    /// Sleeps the current Ruby thread for `duration`, letting other Ruby
    /// threads run (`rb_thread_wait_for`). The sleep may end early when the
    /// thread is woken up.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Thread, VM};
    /// use std::time::{Duration, Instant};
    /// # VM::init();
    ///
    /// let start = Instant::now();
    /// Thread::sleep(Duration::from_millis(20));
    ///
    /// assert!(start.elapsed() >= Duration::from_millis(15));
    /// ```
    pub fn sleep(duration: Duration) {
        thread::sleep_for(duration)
    }

    /// Handles pending interrupts for the current thread, such as a
    /// `Thread#raise` or `Thread#kill` from another thread
    /// (`rb_thread_check_ints`). Long-running Rust loops holding the GVL
    /// should call it now and then.
    ///
    /// Raises the pending exception, if any; see
    /// [`VM::protect`](struct.VM.html#method.protect).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Thread, VM};
    /// # VM::init();
    ///
    /// let mut done = 0;
    /// for _ in 0..3 {
    ///     // ... work ...
    ///     done += 1;
    ///     Thread::check_interrupts(); // raises here if the thread was killed
    /// }
    ///
    /// assert_eq!(done, 3);
    /// ```
    pub fn check_interrupts() {
        thread::check_interrupts()
    }

    /// Tells the scheduler to switch to other threads until the file
    /// descriptor is writable (`rb_thread_fd_writable`).
    ///
    /// Ruby deprecates `rb_thread_fd_writable` from 3.1; prefer
    /// [`Thread::wait_writable`](#method.wait_writable).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Read, Write};
    /// use std::os::unix::io::AsRawFd;
    /// use std::os::unix::net::UnixStream;
    ///
    /// use rutie::{Thread, VM};
    /// # VM::init();
    ///
    /// let (mut writer, mut reader) = UnixStream::pair().unwrap();
    ///
    /// // A fresh socket has buffer space, so this returns right away.
    /// Thread::wait_fd_writable(writer.as_raw_fd());
    /// writer.write_all(b"ok").unwrap();
    ///
    /// let mut buffer = [0; 2];
    /// reader.read_exact(&mut buffer).unwrap();
    /// assert_eq!(&buffer, b"ok");
    /// ```
    #[cfg(unix)]
    pub fn wait_fd_writable(fd: RawFd) {
        thread::wait_fd_writable(fd);
    }

    /// Waits, letting other Ruby threads run, until `io` is readable or
    /// `timeout` passes (`rb_io_wait` with `RUBY_IO_READABLE`).
    ///
    /// Returns `Ok(true)` when `io` is readable and `Ok(false)` on timeout.
    /// `None` waits without a limit (on Ruby 3.2+, up to the IO's
    /// `#timeout`, if one is set). Unlike
    /// [`Thread::wait_fd`](#method.wait_fd), which Ruby deprecates from 3.1,
    /// it works with a Fiber scheduler and reports errors, such as a closed
    /// `io`, as `Err`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use rutie::{Object, Thread, IO, VM};
    /// # VM::init();
    ///
    /// let pipe = VM::eval("IO.pipe").unwrap().try_convert_to::<rutie::Array>().unwrap();
    /// let reader = pipe.at(0).try_convert_to::<IO>().unwrap();
    /// let writer = pipe.at(1).try_convert_to::<IO>().unwrap();
    ///
    /// // Nothing written yet.
    /// assert_eq!(Thread::wait_readable(&reader, Some(Duration::from_millis(10))), Ok(false));
    ///
    /// writer.write(&rutie::RString::new_utf8("ready")).unwrap();
    /// assert_eq!(Thread::wait_readable(&reader, None), Ok(true));
    ///
    /// reader.close().unwrap();
    /// assert!(Thread::wait_readable(&reader, None).is_err());
    /// # writer.close().unwrap();
    /// ```
    pub fn wait_readable(io: &IO, timeout: Option<Duration>) -> Result<bool, AnyException> {
        Self::wait_io(io, io::RUBY_IO_READABLE, timeout)
    }

    /// Waits, letting other Ruby threads run, until `io` is writable or
    /// `timeout` passes (`rb_io_wait` with `RUBY_IO_WRITABLE`).
    ///
    /// Returns `Ok(true)` when `io` is writable and `Ok(false)` on timeout;
    /// see [`Thread::wait_readable`](#method.wait_readable). This replaces
    /// [`Thread::wait_fd_writable`](#method.wait_fd_writable), which Ruby
    /// deprecates from 3.1.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use rutie::{Object, Thread, IO, VM};
    /// # VM::init();
    ///
    /// let pipe = VM::eval("IO.pipe").unwrap().try_convert_to::<rutie::Array>().unwrap();
    /// let reader = pipe.at(0).try_convert_to::<IO>().unwrap();
    /// let writer = pipe.at(1).try_convert_to::<IO>().unwrap();
    ///
    /// // An empty pipe has buffer space.
    /// assert_eq!(Thread::wait_writable(&writer, Some(Duration::from_secs(1))), Ok(true));
    ///
    /// writer.close().unwrap();
    /// assert!(Thread::wait_writable(&writer, None).is_err());
    /// # reader.close().unwrap();
    /// ```
    pub fn wait_writable(io: &IO, timeout: Option<Duration>) -> Result<bool, AnyException> {
        Self::wait_io(io, io::RUBY_IO_WRITABLE, timeout)
    }

    fn wait_io(io: &IO, events: i32, timeout: Option<Duration>) -> Result<bool, AnyException> {
        let io_value = io.value();
        let timeout = match timeout {
            Some(timeout) => Float::new(timeout.as_secs_f64()).value(),
            None => NilClass::new().value(),
        };

        vm::protect_value(|| io::wait(io_value, events, timeout))
            .map(|ready| !ready.is_false())
            .map_err(AnyException::from)
    }

    /// Waits for the thread to finish and returns it, or returns the
    /// exception that ended it (Ruby's `join`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Exception, Fixnum, Object, Thread, VM};
    /// # VM::init();
    ///
    /// let worker = Thread::new(|| Fixnum::new(1));
    ///
    /// assert!(worker.join().is_ok());
    /// assert!(!worker.is_alive());
    ///
    /// let failing = VM::eval("Thread.new { Thread.current.report_on_exception = false; raise 'thread failed' }")
    ///     .unwrap().try_convert_to::<Thread>().unwrap();
    ///
    /// assert_eq!(failing.join().unwrap_err().message(), "thread failed");
    /// ```
    pub fn join(&self) -> Result<Thread, AnyException> {
        self.protect_send("join", &[])
            .map(|thread| Thread::from(thread.value()))
    }

    /// Waits for the thread to finish and returns its value, or the
    /// exception that ended it (Ruby's `Thread#value`).
    ///
    /// Named `join_value` so it does not shadow `Object::value`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Thread, VM};
    /// # VM::init();
    ///
    /// let worker = Thread::new(|| Fixnum::new(6 * 7));
    ///
    /// assert_eq!(worker.join_value().unwrap().try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));
    /// ```
    pub fn join_value(&self) -> Result<AnyObject, AnyException> {
        self.protect_send("value", &[])
    }

    /// Returns `true` while the thread is running or sleeping (Ruby's
    /// `alive?`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Thread, VM};
    /// # VM::init();
    ///
    /// assert!(Thread::current().is_alive());
    /// ```
    pub fn is_alive(&self) -> bool {
        unsafe { self.send("alive?", &[]) }.value().is_true()
    }

    /// Terminates the thread (Ruby's `kill`, `rb_thread_kill`). Killing the
    /// current thread does not return.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Thread, VM};
    /// # VM::init();
    ///
    /// let sleeper = VM::eval("Thread.new { sleep }").unwrap().try_convert_to::<Thread>().unwrap();
    ///
    /// sleeper.kill();
    /// sleeper.join().unwrap();
    ///
    /// assert!(!sleeper.is_alive());
    /// ```
    pub fn kill(&self) {
        thread::kill(self.value());
    }

    /// Marks a sleeping thread as eligible to run (Ruby's `wakeup`,
    /// `rb_thread_wakeup`), or returns the `ThreadError` for a dead thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Thread, VM};
    /// # VM::init();
    ///
    /// let sleeper = VM::eval("Thread.new { sleep; :woke }").unwrap().try_convert_to::<Thread>().unwrap();
    ///
    /// while unsafe { sleeper.send("status", &[]) }.try_convert_to::<rutie::RString>()
    ///     .map(|status| status.to_str() != "sleep").unwrap_or(true) {
    ///     Thread::pass();
    /// }
    ///
    /// sleeper.wakeup().unwrap();
    ///
    /// assert_eq!(sleeper.join_value().unwrap().try_convert_to::<rutie::Symbol>(), Ok(rutie::Symbol::new("woke")));
    /// assert!(sleeper.wakeup().is_err());
    /// ```
    pub fn wakeup(&self) -> Result<(), AnyException> {
        let thread_value = self.value();

        vm::protect_value(|| thread::wakeup(thread_value))
            .map(|_| ())
            .map_err(AnyException::from)
    }

    /// Returns the fiber-local variable `name` of the thread, or `nil`
    /// (Ruby's `thread[name]`, `rb_thread_local_aref`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Thread, VM};
    /// # VM::init();
    ///
    /// let current = Thread::current();
    ///
    /// assert!(current.local_get("rutie_count").is_nil());
    ///
    /// current.local_set("rutie_count", Fixnum::new(3));
    ///
    /// assert_eq!(current.local_get("rutie_count").try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
    /// ```
    pub fn local_get(&self, name: &str) -> AnyObject {
        AnyObject::from(thread::local_get(self.value(), name))
    }

    /// Sets the fiber-local variable `name` of the thread (Ruby's
    /// `thread[name] = value`, `rb_thread_local_aset`).
    ///
    /// Ruby raises `FrozenError` for a frozen thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, Thread, VM};
    /// # VM::init();
    ///
    /// Thread::current().local_set("rutie_name", RString::new_utf8("main"));
    ///
    /// let name = VM::eval("Thread.current[:rutie_name]").unwrap();
    ///
    /// assert_eq!(name.try_convert_to::<RString>().unwrap().to_str(), "main");
    /// ```
    pub fn local_set<T: Object>(&self, name: &str, value: T) -> AnyObject {
        AnyObject::from(thread::local_set(self.value(), name, value.value()))
    }
}

impl From<Value> for Thread {
    fn from(value: Value) -> Self {
        Thread { value }
    }
}

impl Into<Value> for Thread {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Thread {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for Thread {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Thread {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.class() == Class::thread()
    }

    fn error_message() -> &'static str {
        "Error converting to Thread"
    }
}

impl PartialEq for Thread {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Array, Exception, Fixnum, Object, RString, Thread, IO, VM};
    use std::time::{Duration, Instant};

    #[test]
    fn test_thread_management() {
        crate::on_ruby_thread(|| {
            assert!(Thread::current() == Thread::main());
            assert!(Thread::current().is_alive());

            let worker = Thread::new(|| Fixnum::new(21 * 2));
            assert_eq!(
                worker.join_value().unwrap().try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(42))
            );
            assert!(!worker.is_alive());
            assert!(worker.join().is_ok());

            let failing = VM::eval(
                "Thread.new { Thread.current.report_on_exception = false; raise ArgumentError, 'bad' }",
            )
            .unwrap()
            .try_convert_to::<Thread>()
            .unwrap();
            assert_eq!(failing.join_value().unwrap_err().message(), "bad");

            let sleeper = VM::eval("Thread.new { sleep }")
                .unwrap()
                .try_convert_to::<Thread>()
                .unwrap();
            sleeper.kill();
            sleeper.join().unwrap();
            assert!(!sleeper.is_alive());
            assert!(sleeper.wakeup().is_err());

            let current = Thread::current();
            current.local_set("rutie_thread_test", RString::new_utf8("local"));
            assert_eq!(
                current
                    .local_get("rutie_thread_test")
                    .try_convert_to::<RString>()
                    .unwrap()
                    .to_str(),
                "local"
            );

            let start = Instant::now();
            Thread::sleep(Duration::from_millis(5));
            assert!(start.elapsed() >= Duration::from_millis(4));

            Thread::pass();
            Thread::check_interrupts();
            assert!(Thread::is_alone());
            assert_eq!(Thread::call_without_gvl_io(|| 2 + 2), 4);
        });
    }

    #[test]
    fn test_gvl_release_and_fd_waits() {
        crate::on_ruby_thread(|| {
            // Rust-only work without the GVL.
            let sum = Thread::call_without_gvl(|| (1..=10u64).sum::<u64>(), Some(|| {}));
            assert_eq!(sum, 55);

            let product = Thread::call_without_gvl2(|| 6 * 7, None::<fn()>);
            assert_eq!(product, 42);

            // Back inside Ruby from a GVL-free section.
            let text = Thread::call_without_gvl(
                || Thread::call_with_gvl(|| RString::new_utf8("with gvl").to_string()),
                None::<fn()>,
            );
            assert_eq!(text, "with gvl");

            #[cfg(unix)]
            {
                use std::io::Write;
                use std::os::unix::io::AsRawFd;
                use std::os::unix::net::UnixStream;

                let (mut writer, reader) = UnixStream::pair().unwrap();
                Thread::wait_fd_writable(writer.as_raw_fd());
                writer.write_all(b"x").unwrap();
                // Returns once the reader has data.
                Thread::wait_fd(reader.as_raw_fd());
            }
        });
    }

    #[test]
    fn test_wait_readable_and_writable() {
        crate::on_ruby_thread(|| {
            let pipe = Array::from(VM::eval("IO.pipe").unwrap().value());
            let reader = pipe.at(0).try_convert_to::<IO>().unwrap();
            let writer = pipe.at(1).try_convert_to::<IO>().unwrap();
            let short = Some(Duration::from_millis(5));

            assert_eq!(Thread::wait_readable(&reader, short), Ok(false));
            assert_eq!(Thread::wait_writable(&writer, short), Ok(true));

            writer.write(&RString::new_utf8("x")).unwrap();
            assert_eq!(Thread::wait_readable(&reader, short), Ok(true));
            assert_eq!(Thread::wait_readable(&reader, None), Ok(true));

            // Waiting on a closed stream is an `IOError`, not a crash.
            reader.close().unwrap();
            writer.close().unwrap();
            let error = Thread::wait_readable(&reader, None).unwrap_err();
            assert_eq!(error.class().name().unwrap().to_string(), "IOError");
            assert!(Thread::wait_writable(&writer, short).is_err());
        });
    }
}
