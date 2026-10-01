use std::{convert::From, fmt, marker::PhantomData, time::Duration};

use crate::{
    binding::{debug, io, thread, vm},
    rubysys::thread as rubysys_thread,
    types::{c_void, Value},
};

#[cfg(any(unix, windows))]
use crate::types::RawFd;

use crate::{
    AnyException, AnyObject, Class, Exception, Float, NilClass, Object, ProfileFrame,
    VerifiedObject, IO,
};

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
    /// On Unix this is any file descriptor. On Windows it is a descriptor of
    /// Ruby's C runtime, as Ruby's `IO#fileno` returns it (not a `HANDLE` or
    /// `SOCKET`).
    ///
    /// **Deprecated:** Ruby deprecates `rb_thread_wait_fd` from 3.1; use
    /// [`Thread::wait_readable`](#method.wait_readable).
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(deprecated)]
    /// use rutie::{Fixnum, Object, Thread, VM};
    /// # VM::init();
    ///
    /// let reader = VM::eval("reader, $writer = IO.pipe; $writer.write('ready'); reader").unwrap();
    /// let fd = unsafe { reader.send("fileno", &[]) }.try_convert_to::<Fixnum>().unwrap();
    ///
    /// // Returns once the pipe has data (other Ruby threads run meanwhile).
    /// Thread::wait_fd(fd.to_i32());
    ///
    /// let data = unsafe { reader.send("readpartial", &[Fixnum::new(5).into()]) };
    /// assert_eq!(data.try_convert_to::<rutie::RString>().unwrap().to_str(), "ready");
    /// ```
    ///
    /// On Unix a descriptor from Rust works too:
    ///
    /// ```
    /// # #![allow(deprecated)]
    /// # #[cfg(unix)] {
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
    /// # }
    /// ```
    #[cfg(any(unix, windows))]
    #[deprecated(since = "0.12.0", note = "use Thread::wait_readable")]
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
    /// From Ruby 4.0 this also works when the thread already holds the GVL
    /// (then `func` simply runs); [`Thread::has_gvl`](#method.has_gvl) says
    /// which is the case.
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

    /// Gives the current Ruby thread a dedicated native thread when Ruby
    /// runs Ruby threads on fewer native threads (the M:N thread scheduler,
    /// `RUBY_MN_THREADS=1`), for code that relies on thread-local storage
    /// (`rb_thread_lock_native_thread`). Returns `false` if the thread
    /// already had one, which is always the case without M:N threads, and
    /// `true` if it was given one now.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Thread, VM};
    /// # VM::init();
    ///
    /// Thread::lock_native_thread();
    ///
    /// // From now on the thread keeps its own native thread.
    /// assert!(!Thread::lock_native_thread());
    /// ```
    pub fn lock_native_thread() -> bool {
        thread::lock_native_thread()
    }

    /// Returns `true` if the current thread holds the GVL
    /// (`ruby_thread_has_gvl_p`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Thread, VM};
    /// # VM::init();
    ///
    /// assert!(Thread::has_gvl());
    ///
    /// let inside = Thread::call_without_gvl(
    ///     || {
    ///         let without = Thread::has_gvl();
    ///         let with = Thread::call_with_gvl(Thread::has_gvl);
    ///
    ///         (without, with)
    ///     },
    ///     None::<fn()>,
    /// );
    ///
    /// assert_eq!(inside, (false, true));
    ///
    /// // Ruby 4.0 runs `call_with_gvl` on a thread that already holds it.
    /// assert!(Thread::call_with_gvl(Thread::has_gvl));
    /// ```
    pub fn has_gvl() -> bool {
        thread::has_gvl()
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
    /// The descriptor is as for [`Thread::wait_fd`](#method.wait_fd).
    ///
    /// **Deprecated:** Ruby deprecates `rb_thread_fd_writable` from 3.1; use
    /// [`Thread::wait_writable`](#method.wait_writable).
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(deprecated)]
    /// use rutie::{Fixnum, Object, Thread, VM};
    /// # VM::init();
    ///
    /// let writer = VM::eval("$reader, writer = IO.pipe; writer").unwrap();
    /// let fd = unsafe { writer.send("fileno", &[]) }.try_convert_to::<Fixnum>().unwrap();
    ///
    /// // An empty pipe has buffer space, so this returns right away.
    /// Thread::wait_fd_writable(fd.to_i32());
    /// ```
    ///
    /// On Unix a descriptor from Rust works too:
    ///
    /// ```
    /// # #![allow(deprecated)]
    /// # #[cfg(unix)] {
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
    /// # }
    /// ```
    #[cfg(any(unix, windows))]
    #[deprecated(since = "0.12.0", note = "use Thread::wait_writable")]
    pub fn wait_fd_writable(fd: RawFd) {
        thread::wait_fd_writable(fd);
    }

    /// Waits, letting other Ruby threads run, until `io` is readable or
    /// `timeout` passes (`rb_io_wait` with `RUBY_IO_READABLE`).
    ///
    /// Returns `Ok(true)` when `io` is readable and `Ok(false)` on timeout.
    /// `None` waits without a limit (up to the IO's `#timeout`, if one is
    /// set). Unlike
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

    /// Like [`VM::profile_frames`](struct.VM.html#method.profile_frames),
    /// for this thread's stack (`rb_profile_thread_frames`, Ruby 3.3+).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Thread, VM};
    /// # VM::init();
    ///
    /// let worker = VM::eval(
    ///     "q = Queue.new
    ///      t = Thread.new { def wait_here(q) = q.pop; wait_here(q) }
    ///      Thread.pass until t.status == 'sleep'
    ///      $rutie_queue = q
    ///      t",
    /// )
    /// .unwrap()
    /// .try_convert_to::<Thread>()
    /// .unwrap();
    ///
    /// let frames = worker.profile_frames(0, 10);
    /// let names: Vec<String> = frames
    ///     .iter()
    ///     .filter_map(|frame| frame.method_name())
    ///     .map(|name| name.to_string())
    ///     .collect();
    /// assert_eq!(names, ["pop", "wait_here"]);
    ///
    /// VM::eval("$rutie_queue << 1").unwrap();
    /// worker.join().unwrap();
    /// ```
    pub fn profile_frames(&self, start: usize, limit: usize) -> Vec<ProfileFrame> {
        ProfileFrame::from_frames(debug::profile_thread_frames(self.value(), start, limit))
    }

    /// Registers `func` to be called on the thread events in `events`, a
    /// mask of the [`InternalThreadEvent`](struct.InternalThreadEvent.html)
    /// flags, for every Ruby thread (`rb_internal_thread_add_event_hook`,
    /// Ruby 3.2+). Meant for tools such as GVL profilers.
    ///
    /// The hook stays registered until the returned handle is dropped.
    /// Returns `None` where Ruby does not implement thread event hooks
    /// (Windows).
    ///
    /// `func` runs on whichever native thread the event happens on, without
    /// the GVL for every event but `RESUMED`, in parallel with Ruby code and
    /// with other calls of itself (hence `Send + Sync`). It must not use
    /// Ruby objects or call the Ruby API (only
    /// [`Thread::internal_specific`](#method.internal_specific) and
    /// [`Thread::set_internal_specific`](#method.set_internal_specific) are
    /// safe to use there), must not block for long, and must not add or
    /// remove hooks. A panic in `func` is caught and ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    /// use std::sync::Arc;
    ///
    /// use rutie::{InternalThreadEvent, Thread, VM};
    /// # VM::init();
    ///
    /// let started = Arc::new(AtomicUsize::new(0));
    /// let resumed = Arc::new(AtomicUsize::new(0));
    ///
    /// let hook = {
    ///     let (started, resumed) = (started.clone(), resumed.clone());
    ///
    ///     Thread::add_internal_event_hook(
    ///         InternalThreadEvent::STARTED | InternalThreadEvent::RESUMED,
    ///         move |event: &InternalThreadEvent| match event.flag() {
    ///             InternalThreadEvent::STARTED => {
    ///                 started.fetch_add(1, Ordering::SeqCst);
    ///             }
    ///             InternalThreadEvent::RESUMED => {
    ///                 resumed.fetch_add(1, Ordering::SeqCst);
    ///             }
    ///             _ => unreachable!(),
    ///         },
    ///     )
    /// };
    ///
    /// VM::eval("Thread.new { 1 + 1 }.join").unwrap();
    ///
    /// if cfg!(windows) {
    ///     assert!(hook.is_none());
    /// } else {
    ///     assert!(hook.is_some());
    ///     assert_eq!(started.load(Ordering::SeqCst), 1);
    ///     assert!(resumed.load(Ordering::SeqCst) >= 1);
    /// }
    ///
    /// // Unregistered: no more calls.
    /// drop(hook);
    /// VM::eval("Thread.new { 1 + 1 }.join").unwrap();
    /// assert!(started.load(Ordering::SeqCst) <= 1);
    /// ```
    pub fn add_internal_event_hook<F>(events: u32, func: F) -> Option<InternalThreadEventHook>
    where
        F: Fn(&InternalThreadEvent) + Send + Sync + 'static,
    {
        let callback = move |flag: u32, data: *const rubysys_thread::InternalThreadEventData| {
            let event = InternalThreadEvent::new(flag, data);

            func(&event)
        };

        thread::internal_thread_add_event_hook(events, callback).map(|(hook, data)| {
            InternalThreadEventHook {
                remove: Box::new(move || unsafe {
                    thread::internal_thread_remove_event_hook(hook, data);
                }),
                _not_send: PhantomData,
            }
        })
    }

    /// Returns the data stored for `key` on this thread (`NULL` until set)
    /// (`rb_internal_thread_specific_get`, Ruby 3.3+).
    ///
    /// Async signal safe and thread safe, and fine to call without the GVL,
    /// for instance in a hook added with
    /// [`Thread::add_internal_event_hook`](#method.add_internal_event_hook).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{InternalThreadSpecificKey, Thread, VM};
    /// # VM::init();
    ///
    /// let key = InternalThreadSpecificKey::new().unwrap();
    /// let thread = Thread::current();
    ///
    /// assert!(thread.internal_specific(key).is_null());
    ///
    /// let counter = Box::into_raw(Box::new(0u64));
    /// thread.set_internal_specific(key, counter as *mut _);
    /// assert_eq!(thread.internal_specific(key) as *mut u64, counter);
    ///
    /// thread.set_internal_specific(key, std::ptr::null_mut());
    /// # drop(unsafe { Box::from_raw(counter) });
    /// ```
    pub fn internal_specific(&self, key: InternalThreadSpecificKey) -> *mut c_void {
        thread::internal_thread_specific_get(self.value(), key.0)
    }

    /// Stores `data` for `key` on this thread
    /// (`rb_internal_thread_specific_set`, Ruby 3.3+). Ruby does not free
    /// or otherwise use it.
    ///
    /// Async signal safe and thread safe, and fine to call without the GVL.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{InternalThreadSpecificKey, Object, Thread, VM};
    /// # VM::init();
    ///
    /// let key = InternalThreadSpecificKey::new().unwrap();
    ///
    /// let worker = VM::eval("Thread.new { sleep }").unwrap().try_convert_to::<Thread>().unwrap();
    /// let main = Thread::current();
    ///
    /// // Each thread has its own slot.
    /// worker.set_internal_specific(key, 1 as *mut _);
    /// main.set_internal_specific(key, 2 as *mut _);
    ///
    /// assert_eq!(worker.internal_specific(key) as usize, 1);
    /// assert_eq!(main.internal_specific(key) as usize, 2);
    /// # worker.kill();
    /// # worker.join().unwrap();
    /// ```
    pub fn set_internal_specific(&self, key: InternalThreadSpecificKey, data: *mut c_void) {
        thread::internal_thread_specific_set(self.value(), key.0, data)
    }
}

/// A thread event passed to a hook added with
/// [`Thread::add_internal_event_hook`](struct.Thread.html#method.add_internal_event_hook)
/// (Ruby 3.2+).
///
/// # Examples
///
/// ```
/// use std::sync::{Arc, Mutex};
///
/// use rutie::{InternalThreadEvent, Thread, VM};
/// # VM::init();
///
/// let seen = Arc::new(Mutex::new(Vec::new()));
///
/// let hook = {
///     let seen = seen.clone();
///
///     Thread::add_internal_event_hook(InternalThreadEvent::ALL, move |event| {
///         seen.lock().unwrap().push(event.flag());
///     })
/// };
///
/// VM::eval("Thread.new { 1 + 1 }.join").unwrap();
/// drop(hook);
///
/// let seen = seen.lock().unwrap();
/// if cfg!(not(windows)) {
///     assert!(seen.contains(&InternalThreadEvent::STARTED));
///     assert!(seen.contains(&InternalThreadEvent::READY));
///     assert!(seen.contains(&InternalThreadEvent::RESUMED));
///     assert!(seen.contains(&InternalThreadEvent::SUSPENDED));
/// }
/// assert!(seen.iter().all(|flag| flag & InternalThreadEvent::ALL == *flag));
/// ```
#[derive(Debug, Clone, Copy)]
pub struct InternalThreadEvent {
    flag: u32,
    thread: Value,
}

impl InternalThreadEvent {
    /// A thread started.
    pub const STARTED: u32 = rubysys_thread::RUBY_INTERNAL_THREAD_EVENT_STARTED;
    /// A thread is about to acquire the GVL (the hook runs without it).
    pub const READY: u32 = rubysys_thread::RUBY_INTERNAL_THREAD_EVENT_READY;
    /// A thread acquired the GVL (the hook runs with it).
    pub const RESUMED: u32 = rubysys_thread::RUBY_INTERNAL_THREAD_EVENT_RESUMED;
    /// A thread released the GVL (the hook runs without it).
    pub const SUSPENDED: u32 = rubysys_thread::RUBY_INTERNAL_THREAD_EVENT_SUSPENDED;
    /// A thread is terminating (the hook runs without the GVL).
    pub const EXITED: u32 = rubysys_thread::RUBY_INTERNAL_THREAD_EVENT_EXITED;
    /// Every thread event.
    pub const ALL: u32 = rubysys_thread::RUBY_INTERNAL_THREAD_EVENT_MASK;

    fn new(flag: u32, data: *const rubysys_thread::InternalThreadEventData) -> Self {
        InternalThreadEvent {
            flag,
            thread: unsafe { (*data).thread },
        }
    }

    /// Returns which event this is: one of the `InternalThreadEvent` flags.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::atomic::{AtomicU32, Ordering};
    /// use std::sync::Arc;
    ///
    /// use rutie::{InternalThreadEvent, Thread, VM};
    /// # VM::init();
    ///
    /// let flags = Arc::new(AtomicU32::new(0));
    ///
    /// let hook = {
    ///     let flags = flags.clone();
    ///
    ///     Thread::add_internal_event_hook(InternalThreadEvent::STARTED, move |event| {
    ///         flags.fetch_or(event.flag(), Ordering::SeqCst);
    ///     })
    /// };
    ///
    /// VM::eval("Thread.new {}.join").unwrap();
    /// drop(hook);
    ///
    /// let expected = if cfg!(windows) { 0 } else { InternalThreadEvent::STARTED };
    /// assert_eq!(flags.load(Ordering::SeqCst), expected);
    /// ```
    pub fn flag(&self) -> u32 {
        self.flag
    }

    /// Returns the Ruby thread the event is about (Ruby 3.3+), which is not
    /// necessarily the native thread the hook runs on.
    ///
    /// Without the GVL, only use it with
    /// [`Thread::internal_specific`](struct.Thread.html#method.internal_specific),
    /// [`Thread::set_internal_specific`](struct.Thread.html#method.set_internal_specific)
    /// or to compare it with other threads (`Thread::value`).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    ///
    /// use rutie::{InternalThreadEvent, Object, Thread, VM};
    /// # VM::init();
    ///
    /// let started = Arc::new(Mutex::new(Vec::new()));
    ///
    /// let hook = {
    ///     let started = started.clone();
    ///
    ///     Thread::add_internal_event_hook(InternalThreadEvent::STARTED, move |event| {
    ///         started.lock().unwrap().push(event.thread().value());
    ///     })
    /// };
    ///
    /// let thread = VM::eval("Thread.new {}.tap(&:join)").unwrap().try_convert_to::<Thread>().unwrap();
    /// drop(hook);
    ///
    /// if cfg!(not(windows)) {
    ///     assert_eq!(*started.lock().unwrap(), vec![thread.value()]);
    /// }
    /// ```
    pub fn thread(&self) -> Thread {
        Thread::from(self.thread)
    }
}

/// A hook added with
/// [`Thread::add_internal_event_hook`](struct.Thread.html#method.add_internal_event_hook)
/// (Ruby 3.2+). Dropping it unregisters the hook, waiting for calls of it
/// that are running on other threads to finish, and drops its closure.
///
/// It cannot be sent to other threads, so a hook cannot remove itself. To
/// keep a hook for good, `std::mem::forget` the handle.
///
/// # Examples
///
/// ```
/// use std::sync::atomic::{AtomicUsize, Ordering};
/// use std::sync::Arc;
///
/// use rutie::{InternalThreadEvent, Thread, VM};
/// # VM::init();
///
/// let calls = Arc::new(AtomicUsize::new(0));
///
/// let hook = {
///     let calls = calls.clone();
///
///     Thread::add_internal_event_hook(InternalThreadEvent::EXITED, move |_| {
///         calls.fetch_add(1, Ordering::SeqCst);
///     })
/// };
///
/// drop(hook);
/// VM::eval("Thread.new {}.join").unwrap();
/// assert_eq!(calls.load(Ordering::SeqCst), 0);
/// // The closure was dropped with the hook.
/// assert_eq!(Arc::strong_count(&calls), 1);
/// ```
pub struct InternalThreadEventHook {
    remove: Box<dyn FnMut()>,
    // Removing a hook from a hook deadlocks.
    _not_send: PhantomData<*mut ()>,
}

impl Drop for InternalThreadEventHook {
    fn drop(&mut self) {
        (self.remove)()
    }
}

impl fmt::Debug for InternalThreadEventHook {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("InternalThreadEventHook")
    }
}

/// A key for per-thread data that tools can read and write without the
/// GVL (Ruby 3.3+); see
/// [`Thread::internal_specific`](struct.Thread.html#method.internal_specific).
///
/// Ruby allows only 8 keys per process.
///
/// # Examples
///
/// ```
/// use rutie::{InternalThreadSpecificKey, Thread, VM};
/// # VM::init();
///
/// let first = InternalThreadSpecificKey::new().unwrap();
/// let second = InternalThreadSpecificKey::new().unwrap();
/// assert_ne!(first, second);
///
/// Thread::current().set_internal_specific(first, 7 as *mut _);
/// assert!(Thread::current().internal_specific(second).is_null());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InternalThreadSpecificKey(rubysys_thread::InternalThreadSpecificKey);

impl InternalThreadSpecificKey {
    /// Creates a key (`rb_internal_thread_specific_key_create`). Returns the
    /// `ThreadError` once the process has 8 keys, or when the first key is
    /// created while there are several Ractors.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Exception, InternalThreadSpecificKey, Object, VM};
    /// # VM::init();
    ///
    /// let keys: Vec<_> = (0..8).map(|_| InternalThreadSpecificKey::new()).collect();
    /// assert!(keys.iter().all(Result::is_ok));
    ///
    /// let error = InternalThreadSpecificKey::new().unwrap_err();
    /// assert_eq!(error.class().name().unwrap().to_str(), "ThreadError");
    /// ```
    pub fn new() -> Result<Self, AnyException> {
        let mut key = 0;
        let created = vm::protect_value(|| {
            key = thread::internal_thread_specific_key_create();

            NilClass::new().value()
        });

        match created {
            Err(exception) => Err(AnyException::from(exception)),
            // Ruby 3.3 and 3.4 return one key past their table before
            // raising.
            Ok(_) if key >= rubysys_thread::RB_INTERNAL_THREAD_SPECIFIC_KEY_MAX => Err(
                AnyException::new("ThreadError", Some("too many thread specific keys")),
            ),
            Ok(_) => Ok(InternalThreadSpecificKey(key)),
        }
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

            // Ruby's own descriptors, on every platform.
            let fds = VM::eval("r, w = IO.pipe; $rutie_fd_pipe = [r, w]; [r.fileno, w.fileno]")
                .unwrap()
                .try_convert_to::<crate::Array>()
                .unwrap();
            let fd = |i| {
                fds.at(i)
                    .try_convert_to::<crate::Fixnum>()
                    .unwrap()
                    .to_i32()
            };
            #[allow(deprecated)]
            Thread::wait_fd_writable(fd(1));
            VM::eval("$rutie_fd_pipe[1].write('x')").unwrap();
            // Returns once the reader has data.
            #[allow(deprecated)]
            Thread::wait_fd(fd(0));
            VM::eval("$rutie_fd_pipe.each(&:close); $rutie_fd_pipe = nil").unwrap();

            #[cfg(unix)]
            {
                use std::io::Write;
                use std::os::unix::io::AsRawFd;
                use std::os::unix::net::UnixStream;

                let (mut writer, reader) = UnixStream::pair().unwrap();
                #[allow(deprecated)]
                Thread::wait_fd_writable(writer.as_raw_fd());
                writer.write_all(b"x").unwrap();
                // Returns once the reader has data.
                #[allow(deprecated)]
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

    #[test]
    fn test_internal_event_hook() {
        use crate::InternalThreadEvent;
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };

        crate::on_ruby_thread(|| {
            let counts: Arc<Vec<AtomicUsize>> =
                Arc::new((0..5).map(|_| AtomicUsize::new(0)).collect());
            let count = |i: usize| counts[i].load(Ordering::SeqCst);

            let odd_flags = Arc::new(AtomicUsize::new(0));
            let panicked = Arc::new(std::sync::atomic::AtomicBool::new(false));

            let hook = {
                let (counts, odd_flags, panicked) =
                    (counts.clone(), odd_flags.clone(), panicked.clone());

                Thread::add_internal_event_hook(InternalThreadEvent::ALL, move |event| {
                    let flag = event.flag();
                    if flag.count_ones() != 1 || flag > InternalThreadEvent::EXITED {
                        odd_flags.fetch_add(1, Ordering::SeqCst);
                        return;
                    }
                    counts[flag.trailing_zeros() as usize].fetch_add(1, Ordering::SeqCst);

                    // A panic stays in the hook.
                    if flag == InternalThreadEvent::EXITED && !panicked.swap(true, Ordering::SeqCst)
                    {
                        panic!("a panic in a thread event hook is ignored");
                    }
                })
            };

            VM::eval("2.times.map { Thread.new { Thread.pass; 1 } }.each(&:join)").unwrap();
            // Ruby keeps running after a hook panicked.
            VM::eval("Thread.new {}.join").unwrap();

            if cfg!(windows) {
                assert!(hook.is_none());
                // The closure was dropped.
                assert_eq!(Arc::strong_count(&counts), 1);
                return;
            }

            let hook = hook.unwrap();
            assert_eq!(odd_flags.load(Ordering::SeqCst), 0);
            assert_eq!(count(0), 3, "started");
            assert!(count(1) >= 3, "ready");
            assert!(count(2) >= 3, "resumed");
            assert!(count(3) >= 3, "suspended");
            assert_eq!(format!("{:?}", hook), "InternalThreadEventHook");

            // A second hook sees events too; removing it leaves the first.
            let second_calls = Arc::new(AtomicUsize::new(0));
            let second = {
                let second_calls = second_calls.clone();

                Thread::add_internal_event_hook(InternalThreadEvent::STARTED, move |_| {
                    second_calls.fetch_add(1, Ordering::SeqCst);
                })
                .unwrap()
            };
            VM::eval("Thread.new {}.join").unwrap();
            assert_eq!(second_calls.load(Ordering::SeqCst), 1);
            assert_eq!(count(0), 4);

            drop(second);
            assert_eq!(Arc::strong_count(&second_calls), 1);
            VM::eval("Thread.new {}.join").unwrap();
            assert_eq!(second_calls.load(Ordering::SeqCst), 1);
            assert_eq!(count(0), 5);

            drop(hook);
            assert_eq!(Arc::strong_count(&counts), 1);
            VM::eval("Thread.new {}.join").unwrap();
            assert_eq!(count(0), 5);
        });
    }

    #[test]
    fn test_internal_specific_and_event_thread() {
        use crate::{InternalThreadEvent, InternalThreadSpecificKey};
        use std::sync::{Arc, Mutex};

        crate::on_ruby_thread(|| {
            // Only 8 keys exist per process; this is the unit tests' only one.
            let key = InternalThreadSpecificKey::new().unwrap();

            let main = Thread::current();
            assert!(main.internal_specific(key).is_null());
            main.set_internal_specific(key, 42 as *mut crate::types::c_void);
            assert_eq!(main.internal_specific(key) as usize, 42);

            // Hooks read the slot of the thread an event is about.
            let seen = Arc::new(Mutex::new(Vec::new()));
            let hook = {
                let seen = seen.clone();

                Thread::add_internal_event_hook(InternalThreadEvent::RESUMED, move |event| {
                    let thread = event.thread();
                    seen.lock()
                        .unwrap()
                        .push((thread.value(), thread.internal_specific(key) as usize));
                })
            };

            let worker = VM::eval("q = $rutie_specific_queue = Queue.new; Thread.new { q.pop }")
                .unwrap()
                .try_convert_to::<Thread>()
                .unwrap();
            worker.set_internal_specific(key, 7 as *mut crate::types::c_void);
            VM::eval("$rutie_specific_queue << 1; $rutie_specific_queue = nil").unwrap();
            worker.join().unwrap();
            drop(hook);

            assert_eq!(worker.internal_specific(key) as usize, 7);
            assert_eq!(main.internal_specific(key) as usize, 42);
            main.set_internal_specific(key, std::ptr::null_mut());

            // Ruby has no thread event hooks on Windows.
            if cfg!(not(windows)) {
                let seen = seen.lock().unwrap();
                assert!(seen.contains(&(main.value(), 42)));
                assert!(seen.iter().any(|&(thread, _)| thread == worker.value()));
            }
        });
    }

    #[test]
    fn test_profile_frames_of_another_thread() {
        crate::on_ruby_thread(|| {
            let worker = VM::eval(
                "$rutie_profile_queue = Queue.new
                 t = Thread.new { def rutie_parked(q) = q.pop; rutie_parked($rutie_profile_queue) }
                 Thread.pass until t.status == 'sleep'
                 t",
            )
            .unwrap()
            .try_convert_to::<Thread>()
            .unwrap();

            let frames = worker.profile_frames(0, 10);
            let labels: Vec<String> = frames
                .iter()
                .map(|frame| frame.full_label().unwrap().to_string())
                .collect();
            assert_eq!(labels[0], "Thread::Queue#pop");
            assert_eq!(labels[1], "Object#rutie_parked");
            assert!(frames[1].line() > 0);
            assert_eq!(
                worker.profile_frames(1, 1)[0]
                    .full_label()
                    .unwrap()
                    .to_str(),
                "Object#rutie_parked"
            );
            assert!(worker.profile_frames(0, 0).is_empty());

            VM::eval("$rutie_profile_queue << 1; $rutie_profile_queue = nil").unwrap();
            worker.join().unwrap();
            // A finished thread has no frames.
            assert!(worker.profile_frames(0, 10).is_empty());
        });
    }
}
