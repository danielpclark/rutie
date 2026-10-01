use std::{convert::From, ffi::CString, time::Duration};

use crate::{
    binding::{float, io, vm},
    types::Value,
    util, AnyException, AnyObject, Array, Class, Exception, Fixnum, Float, NilClass, Object,
    RString, VerifiedObject,
};

#[cfg(any(unix, windows))]
use crate::types::RawFd;

fn protect<F>(func: F) -> Result<Value, AnyException>
where
    F: FnOnce() -> Value,
{
    vm::protect_value(func).map_err(AnyException::from)
}

// `None` is `nil`.
fn timeout_value(timeout: Option<Duration>) -> Value {
    match timeout {
        Some(timeout) => Float::new(timeout.as_secs_f64()).value(),
        None => NilClass::new().value(),
    }
}

fn mode_cstring(mode: &str) -> Result<CString, AnyException> {
    CString::new(mode)
        .map_err(|_| AnyException::new("ArgumentError", Some("mode contains a NUL byte")))
}

fn glob_results(paths: Value) -> Vec<RString> {
    Array::from(paths)
        .into_iter()
        .map(|path| RString::from(path.value()))
        .collect()
}

/// `IO`, including `File` objects.
///
/// Every operation that can raise (a closed stream, a stream not open for
/// reading or writing, an OS error) returns the exception as `Err`.
#[derive(Debug)]
#[repr(C)]
pub struct IO {
    value: Value,
}

impl IO {
    /// The event of a stream being readable (`RUBY_IO_READABLE`, Ruby's
    /// `IO::READABLE`), for [`IO::maybe_wait`](#method.maybe_wait).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, IO, VM};
    /// # VM::init();
    ///
    /// let readable = VM::eval("IO::READABLE").unwrap().try_convert_to::<Fixnum>().unwrap();
    ///
    /// assert_eq!(readable.to_i64(), IO::READABLE as i64);
    /// ```
    pub const READABLE: i32 = io::RUBY_IO_READABLE;

    /// The event of priority data being readable (`RUBY_IO_PRIORITY`, Ruby's
    /// `IO::PRIORITY`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, IO, VM};
    /// # VM::init();
    ///
    /// let priority = VM::eval("IO::PRIORITY").unwrap().try_convert_to::<Fixnum>().unwrap();
    ///
    /// assert_eq!(priority.to_i64(), IO::PRIORITY as i64);
    /// ```
    pub const PRIORITY: i32 = io::RUBY_IO_PRIORITY;

    /// The event of a stream being writable (`RUBY_IO_WRITABLE`, Ruby's
    /// `IO::WRITABLE`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, IO, VM};
    /// # VM::init();
    ///
    /// let writable = VM::eval("IO::WRITABLE").unwrap().try_convert_to::<Fixnum>().unwrap();
    ///
    /// assert_eq!(writable.to_i64(), IO::WRITABLE as i64);
    /// ```
    pub const WRITABLE: i32 = io::RUBY_IO_WRITABLE;

    /// Returns `$stdin` (`rb_stdin`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IO, VM};
    /// # VM::init();
    ///
    /// assert!(!IO::stdin().is_closed());
    /// ```
    pub fn stdin() -> Self {
        IO::from(io::stdin())
    }

    /// Returns `$stdout` (`rb_stdout`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IO, RString, VM};
    /// # VM::init();
    ///
    /// assert_eq!(IO::stdout().write(&RString::new_utf8("")).unwrap(), 0);
    /// ```
    pub fn stdout() -> Self {
        IO::from(io::stdout())
    }

    /// Returns `$stderr` (`rb_stderr`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IO, VM};
    /// # VM::init();
    ///
    /// assert!(IO::stderr().flush().is_ok());
    /// ```
    pub fn stderr() -> Self {
        IO::from(io::stderr())
    }

    /// Writes `string` and returns the number of bytes written (Ruby's
    /// `write`, `rb_io_write`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{File, RString, VM};
    /// # VM::init();
    ///
    /// let path = std::env::temp_dir().join(format!("rutie_io_write_example_{}.txt", std::process::id()));
    /// let file = File::open(path.to_str().unwrap(), "w").unwrap();
    ///
    /// assert_eq!(file.write(&RString::new_utf8("héllo")).unwrap(), 6);
    /// file.close().unwrap();
    ///
    /// assert!(file.write(&RString::new_utf8("x")).is_err());
    /// # std::fs::remove_file(path).unwrap();
    /// ```
    pub fn write(&self, string: &RString) -> Result<usize, AnyException> {
        let (io_value, string) = (self.value(), string.value());

        protect(|| io::write(io_value, string))
            .map(|written| Fixnum::from(written).to_i64() as usize)
    }

    /// Writes each object's `to_s` followed by a newline (Ruby's `puts`,
    /// `rb_io_puts`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, File, RString, VM};
    /// # VM::init();
    ///
    /// let path = std::env::temp_dir().join(format!("rutie_io_puts_example_{}.txt", std::process::id()));
    /// let file = File::open(path.to_str().unwrap(), "w").unwrap();
    ///
    /// file.puts(&[RString::new_utf8("a").into(), Fixnum::new(1).into()]).unwrap();
    /// file.close().unwrap();
    ///
    /// // Lines end in "\r\n" on Windows, where "w" is text mode.
    /// let contents = std::fs::read_to_string(&path).unwrap();
    /// assert_eq!(contents.lines().collect::<Vec<_>>(), ["a", "1"]);
    /// # std::fs::remove_file(path).unwrap();
    /// ```
    pub fn puts(&self, objects: &[AnyObject]) -> Result<(), AnyException> {
        let io_value = self.value();
        let objects = util::arguments_to_values(objects);

        protect(|| io::puts(io_value, &objects)).map(|_| ())
    }

    /// Writes each object's `to_s` with no separator (Ruby's `print`,
    /// `rb_io_print`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, File, RString, VM};
    /// # VM::init();
    ///
    /// let path = std::env::temp_dir().join(format!("rutie_io_print_example_{}.txt", std::process::id()));
    /// let file = File::open(path.to_str().unwrap(), "w").unwrap();
    ///
    /// file.print(&[RString::new_utf8("a").into(), Fixnum::new(1).into()]).unwrap();
    /// file.close().unwrap();
    ///
    /// assert_eq!(std::fs::read_to_string(&path).unwrap(), "a1");
    /// # std::fs::remove_file(path).unwrap();
    /// ```
    pub fn print(&self, objects: &[AnyObject]) -> Result<(), AnyException> {
        let io_value = self.value();
        let objects = util::arguments_to_values(objects);

        protect(|| io::print(io_value, &objects)).map(|_| ())
    }

    /// Reads the next line, including its newline, or `None` at the end
    /// (Ruby's `gets`, `rb_io_gets`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{File, VM};
    /// # VM::init();
    ///
    /// let path = std::env::temp_dir().join(format!("rutie_io_gets_example_{}.txt", std::process::id()));
    /// std::fs::write(&path, "one\ntwo").unwrap();
    ///
    /// let file = File::open(path.to_str().unwrap(), "r").unwrap();
    ///
    /// assert_eq!(file.gets().unwrap().unwrap().to_str(), "one\n");
    /// assert_eq!(file.gets().unwrap().unwrap().to_str(), "two");
    /// assert!(file.gets().unwrap().is_none());
    /// # file.close().unwrap();
    /// # std::fs::remove_file(path).unwrap();
    /// ```
    pub fn gets(&self) -> Result<Option<RString>, AnyException> {
        let io_value = self.value();

        protect(|| io::gets(io_value)).map(|line| {
            if line.is_nil() {
                None
            } else {
                Some(RString::from(line))
            }
        })
    }

    /// Reads the next byte, or `None` at the end (Ruby's `getbyte`,
    /// `rb_io_getbyte`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{File, VM};
    /// # VM::init();
    ///
    /// let path = std::env::temp_dir().join(format!("rutie_io_getbyte_example_{}.txt", std::process::id()));
    /// std::fs::write(&path, [7u8]).unwrap();
    ///
    /// let file = File::open(path.to_str().unwrap(), "rb").unwrap();
    ///
    /// assert_eq!(file.getbyte().unwrap(), Some(7));
    /// assert_eq!(file.getbyte().unwrap(), None);
    /// # file.close().unwrap();
    /// # std::fs::remove_file(path).unwrap();
    /// ```
    pub fn getbyte(&self) -> Result<Option<u8>, AnyException> {
        let io_value = self.value();

        protect(|| io::getbyte(io_value)).map(|byte| {
            if byte.is_nil() {
                None
            } else {
                Some(Fixnum::from(byte).to_i64() as u8)
            }
        })
    }

    /// Flushes buffered output (Ruby's `flush`, `rb_io_flush`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IO, VM};
    /// # VM::init();
    ///
    /// assert!(IO::stdout().flush().is_ok());
    /// ```
    pub fn flush(&self) -> Result<(), AnyException> {
        let io_value = self.value();

        protect(|| io::flush(io_value)).map(|_| ())
    }

    /// Closes the stream; closing it again does nothing (Ruby's `close`,
    /// `rb_io_close`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{File, VM};
    /// # VM::init();
    ///
    /// let path = std::env::temp_dir().join(format!("rutie_io_close_example_{}.txt", std::process::id()));
    /// let file = File::open(path.to_str().unwrap(), "w").unwrap();
    ///
    /// file.close().unwrap();
    ///
    /// assert!(file.is_closed());
    /// assert!(file.close().is_ok());
    /// # std::fs::remove_file(path).unwrap();
    /// ```
    pub fn close(&self) -> Result<(), AnyException> {
        let io_value = self.value();

        protect(|| io::close(io_value)).map(|_| ())
    }

    /// Returns `true` if the stream is closed (Ruby's `closed?`,
    /// `rb_io_closed_p`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IO, VM};
    /// # VM::init();
    ///
    /// assert!(!IO::stdout().is_closed());
    /// ```
    pub fn is_closed(&self) -> bool {
        io::is_closed(self.value())
    }

    /// Returns `true` at the end of the stream, waiting for input on pipes
    /// and terminals (Ruby's `eof?`, `rb_io_eof`). An error for a stream not
    /// open for reading.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{File, VM};
    /// # VM::init();
    ///
    /// let path = std::env::temp_dir().join(format!("rutie_io_eof_example_{}.txt", std::process::id()));
    /// std::fs::write(&path, "x").unwrap();
    ///
    /// let file = File::open(path.to_str().unwrap(), "r").unwrap();
    ///
    /// assert!(!file.is_eof().unwrap());
    /// file.getbyte().unwrap();
    /// assert!(file.is_eof().unwrap());
    /// # file.close().unwrap();
    /// # std::fs::remove_file(path).unwrap();
    /// ```
    pub fn is_eof(&self) -> Result<bool, AnyException> {
        let io_value = self.value();
        let mut eof = false;

        protect(|| {
            eof = io::is_eof(io_value);

            NilClass::new().value()
        })
        .map(|_| eof)
    }

    /// Switches the stream to binary mode and sets its external encoding
    /// to ASCII-8BIT (Ruby's `binmode`, `rb_io_ascii8bit_binmode`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, EncodingSupport, File, VM};
    /// # VM::init();
    ///
    /// let path = std::env::temp_dir().join(format!("rutie_io_binmode_example_{}.txt", std::process::id()));
    /// std::fs::write(&path, "x").unwrap();
    ///
    /// let file = File::open(path.to_str().unwrap(), "r").unwrap();
    /// file.binmode().unwrap();
    ///
    /// assert_eq!(file.gets().unwrap().unwrap().encoding().name(), "ASCII-8BIT");
    /// # file.close().unwrap();
    /// # std::fs::remove_file(path).unwrap();
    /// ```
    pub fn binmode(&self) -> Result<(), AnyException> {
        let io_value = self.value();

        protect(|| io::binmode(io_value)).map(|_| ())
    }

    /// Returns the file descriptor (Ruby's `fileno`, `rb_io_descriptor`), or
    /// `IOError` for a closed stream.
    ///
    /// On Windows this is a C runtime descriptor, not a `HANDLE`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, IO, VM};
    /// # VM::init();
    ///
    /// assert_eq!(IO::stderr().descriptor().unwrap(), 2);
    ///
    /// let pipe = VM::eval("IO.pipe").unwrap().try_convert_to::<rutie::Array>().unwrap();
    /// let reader = pipe.at(0).try_convert_to::<IO>().unwrap();
    /// let fileno = unsafe { reader.send("fileno", &[]) };
    ///
    /// assert_eq!(fileno.try_convert_to::<rutie::Fixnum>().unwrap().to_i64(), reader.descriptor().unwrap() as i64);
    ///
    /// reader.close().unwrap();
    /// assert!(Class::io_error().case_equals(&reader.descriptor().unwrap_err()));
    /// # pipe.at(1).try_convert_to::<IO>().unwrap().close().unwrap();
    /// ```
    #[cfg(any(unix, windows))]
    pub fn descriptor(&self) -> Result<RawFd, AnyException> {
        let io_value = self.value();
        let mut fd = -1;

        protect(|| {
            fd = io::descriptor(io_value);

            NilClass::new().value()
        })
        .map(|_| fd as RawFd)
    }

    /// After an operation on the stream failed with the OS error `errno`,
    /// waits for `events` (a combination of [`IO::READABLE`](#associatedconstant.READABLE),
    /// [`IO::PRIORITY`](#associatedconstant.PRIORITY) and
    /// [`IO::WRITABLE`](#associatedconstant.WRITABLE)) if retrying makes
    /// sense (`rb_io_maybe_wait`).
    ///
    /// For `EAGAIN`/`EWOULDBLOCK` it waits like
    /// [`Thread::wait_readable`](struct.Thread.html#method.wait_readable) and
    /// returns the ready events, or `None` on timeout. For `EINTR` it returns
    /// `events` right away, and for any other error `None`, so the operation
    /// can be retried in a loop while this returns `Some`. `timeout` is in
    /// seconds; `None` waits without a limit (on Ruby 3.2+, up to the
    /// stream's [`timeout`](#method.timeout), if one is set).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use rutie::{Fixnum, Object, RString, IO, VM};
    /// # VM::init();
    ///
    /// let errno = |name: &str| {
    ///     VM::eval(&format!("Errno::{}::Errno", name)).unwrap().try_convert_to::<Fixnum>().unwrap().to_i64() as i32
    /// };
    ///
    /// let pipe = VM::eval("IO.pipe").unwrap().try_convert_to::<rutie::Array>().unwrap();
    /// let reader = pipe.at(0).try_convert_to::<IO>().unwrap();
    /// let writer = pipe.at(1).try_convert_to::<IO>().unwrap();
    ///
    /// // Nothing to read yet: a read would have failed with EAGAIN.
    /// let short = Some(Duration::from_millis(10));
    /// assert_eq!(reader.maybe_wait(errno("EAGAIN"), IO::READABLE, short), Ok(None));
    ///
    /// writer.write(&RString::new_utf8("x")).unwrap();
    /// assert_eq!(reader.maybe_wait(errno("EAGAIN"), IO::READABLE, None), Ok(Some(IO::READABLE)));
    ///
    /// // Errors that retrying can't fix.
    /// assert_eq!(reader.maybe_wait(errno("EBADF"), IO::READABLE, None), Ok(None));
    /// # reader.close().unwrap();
    /// # writer.close().unwrap();
    /// ```
    pub fn maybe_wait(
        &self,
        errno: i32,
        events: i32,
        timeout: Option<Duration>,
    ) -> Result<Option<i32>, AnyException> {
        let io_value = self.value();
        let timeout = timeout_value(timeout);

        protect(|| io::maybe_wait(errno, io_value, events, timeout)).map(|ready| {
            if ready.is_false() || ready.is_nil() {
                None
            } else {
                Some(Fixnum::from(ready).to_i64() as i32)
            }
        })
    }

    /// Like [`IO::maybe_wait`](#method.maybe_wait) for
    /// [`IO::READABLE`](#associatedconstant.READABLE)
    /// (`rb_io_maybe_wait_readable`): `Ok(true)` when the stream is readable
    /// (or `errno` is `EINTR`), and `Ok(false)` for an error that waiting
    /// does not help with. On timeout it is `Err` with `IO::TimeoutError`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, RString, IO, VM};
    /// # VM::init();
    ///
    /// let errno = |name: &str| {
    ///     VM::eval(&format!("Errno::{}::Errno", name)).unwrap().try_convert_to::<Fixnum>().unwrap().to_i64() as i32
    /// };
    ///
    /// let pipe = VM::eval("IO.pipe").unwrap().try_convert_to::<rutie::Array>().unwrap();
    /// let reader = pipe.at(0).try_convert_to::<IO>().unwrap();
    /// let writer = pipe.at(1).try_convert_to::<IO>().unwrap();
    ///
    /// writer.write(&RString::new_utf8("x")).unwrap();
    ///
    /// assert_eq!(reader.maybe_wait_readable(errno("EAGAIN"), None), Ok(true));
    /// assert_eq!(reader.maybe_wait_readable(errno("EPIPE"), None), Ok(false));
    /// # reader.close().unwrap();
    /// # writer.close().unwrap();
    /// ```
    pub fn maybe_wait_readable(
        &self,
        errno: i32,
        timeout: Option<Duration>,
    ) -> Result<bool, AnyException> {
        let io_value = self.value();
        let timeout = timeout_value(timeout);
        let mut ready = 0;

        protect(|| {
            ready = io::maybe_wait_readable(errno, io_value, timeout);

            NilClass::new().value()
        })
        .map(|_| ready != 0)
    }

    /// Like [`IO::maybe_wait`](#method.maybe_wait) for
    /// [`IO::WRITABLE`](#associatedconstant.WRITABLE)
    /// (`rb_io_maybe_wait_writable`); see
    /// [`IO::maybe_wait_readable`](#method.maybe_wait_readable).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, IO, VM};
    /// # VM::init();
    ///
    /// let errno = |name: &str| {
    ///     VM::eval(&format!("Errno::{}::Errno", name)).unwrap().try_convert_to::<Fixnum>().unwrap().to_i64() as i32
    /// };
    ///
    /// let pipe = VM::eval("IO.pipe").unwrap().try_convert_to::<rutie::Array>().unwrap();
    /// let writer = pipe.at(1).try_convert_to::<IO>().unwrap();
    ///
    /// // An empty pipe has room.
    /// assert_eq!(writer.maybe_wait_writable(errno("EAGAIN"), None), Ok(true));
    /// assert_eq!(writer.maybe_wait_writable(errno("ENOSPC"), None), Ok(false));
    ///
    /// writer.close().unwrap();
    /// assert!(writer.maybe_wait_writable(errno("EAGAIN"), None).is_err());
    /// # pipe.at(0).try_convert_to::<IO>().unwrap().close().unwrap();
    /// ```
    pub fn maybe_wait_writable(
        &self,
        errno: i32,
        timeout: Option<Duration>,
    ) -> Result<bool, AnyException> {
        let io_value = self.value();
        let timeout = timeout_value(timeout);
        let mut ready = 0;

        protect(|| {
            ready = io::maybe_wait_writable(errno, io_value, timeout);

            NilClass::new().value()
        })
        .map(|_| ready != 0)
    }

    /// Returns the stream's timeout, or `None` if it has none (Ruby's
    /// `IO#timeout`, `rb_io_timeout`). Ruby 3.2+.
    ///
    /// Blocking operations that take longer raise `IO::TimeoutError`
    /// ([`Class::io_timeout_error`](struct.Class.html#method.io_timeout_error)).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use rutie::{Object, IO, VM};
    /// # VM::init();
    ///
    /// let pipe = VM::eval("IO.pipe").unwrap().try_convert_to::<rutie::Array>().unwrap();
    /// let reader = pipe.at(0).try_convert_to::<IO>().unwrap();
    ///
    /// assert_eq!(reader.timeout().unwrap(), None);
    ///
    /// reader.set_timeout(Some(Duration::from_millis(1500))).unwrap();
    /// assert_eq!(reader.timeout().unwrap(), Some(Duration::from_millis(1500)));
    ///
    /// // Set from Ruby as an Integer.
    /// unsafe { reader.send("timeout=", &[rutie::Fixnum::new(2).into()]) };
    /// assert_eq!(reader.timeout().unwrap(), Some(Duration::from_secs(2)));
    /// # reader.close().unwrap();
    /// # pipe.at(1).try_convert_to::<IO>().unwrap().close().unwrap();
    /// ```
    pub fn timeout(&self) -> Result<Option<Duration>, AnyException> {
        let io_value = self.value();
        let mut seconds = None;

        protect(|| {
            let timeout = io::timeout(io_value);

            if !timeout.is_nil() {
                seconds = Some(float::num_to_float(timeout));
            }

            NilClass::new().value()
        })
        .map(|_| seconds.map(Duration::from_secs_f64))
    }

    /// Sets the stream's timeout, or removes it with `None` (Ruby's
    /// `IO#timeout=`, `rb_io_set_timeout`). Ruby 3.2+.
    ///
    /// Blocking operations on the stream that take longer than `timeout`
    /// raise `IO::TimeoutError` (a best-effort limit).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use rutie::{Class, Object, IO, VM};
    /// # VM::init();
    ///
    /// let pipe = VM::eval("IO.pipe").unwrap().try_convert_to::<rutie::Array>().unwrap();
    /// let reader = pipe.at(0).try_convert_to::<IO>().unwrap();
    ///
    /// reader.set_timeout(Some(Duration::from_millis(10))).unwrap();
    ///
    /// // Nothing is ever written, so reading times out.
    /// let error = reader.gets().unwrap_err();
    /// assert!(Class::io_timeout_error().case_equals(&error));
    ///
    /// reader.set_timeout(None).unwrap();
    /// assert_eq!(reader.timeout().unwrap(), None);
    /// # reader.close().unwrap();
    /// # pipe.at(1).try_convert_to::<IO>().unwrap().close().unwrap();
    /// ```
    pub fn set_timeout(&self, timeout: Option<Duration>) -> Result<(), AnyException> {
        let io_value = self.value();
        let timeout = timeout_value(timeout);

        protect(|| io::set_timeout(io_value, timeout)).map(|_| ())
    }

    /// Wraps the open file descriptor `fd` in a new `IO`
    /// (`rb_io_open_descriptor`). Ruby 3.3+.
    ///
    /// `readable` and `writable` say what the stream may be used for. With
    /// `autoclose`, the `IO` owns `fd`: closing it, or the garbage collector
    /// freeing it, closes `fd`. Without it (`FMODE_EXTERNAL`,
    /// `IO#autoclose?` false), `fd` stays open and is still the caller's to
    /// close.
    ///
    /// # Safety
    ///
    /// `fd` must be an open descriptor that allows the reads and writes
    /// `readable` and `writable` ask for. With `autoclose`, ownership of `fd`
    /// moves to Ruby, so nothing else may close it or use it after the `IO`
    /// closes it. Without `autoclose`, `fd` must stay open, referring to the
    /// same file, for as long as the `IO` is used.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, IO, VM};
    /// # VM::init();
    ///
    /// let pipe = VM::eval("IO.pipe").unwrap().try_convert_to::<rutie::Array>().unwrap();
    /// let reader = pipe.at(0).try_convert_to::<IO>().unwrap();
    /// let writer = pipe.at(1).try_convert_to::<IO>().unwrap();
    ///
    /// // A second IO for the writer's descriptor, which `writer` keeps owning.
    /// let borrowed = unsafe { IO::from_raw_fd(writer.descriptor().unwrap(), false, true, false) };
    ///
    /// borrowed.write(&RString::new_utf8("via fd\n")).unwrap();
    /// borrowed.flush().unwrap();
    /// borrowed.close().unwrap();
    ///
    /// // Closing `borrowed` left the descriptor open.
    /// writer.write(&RString::new_utf8("via writer\n")).unwrap();
    ///
    /// assert_eq!(reader.gets().unwrap().unwrap().to_str(), "via fd\n");
    /// assert_eq!(reader.gets().unwrap().unwrap().to_str(), "via writer\n");
    /// # reader.close().unwrap();
    /// # writer.close().unwrap();
    /// ```
    #[cfg(any(unix, windows))]
    pub unsafe fn from_raw_fd(fd: RawFd, readable: bool, writable: bool, autoclose: bool) -> Self {
        use crate::rubysys::io::{FMODE_EXTERNAL, FMODE_READABLE, FMODE_WRITABLE};

        let mut mode = 0;

        if readable {
            mode |= FMODE_READABLE;
        }

        if writable {
            mode |= FMODE_WRITABLE;
        }

        if !autoclose {
            mode |= FMODE_EXTERNAL;
        }

        IO::from(io::open_descriptor(fd as _, mode))
    }

    /// Converts `object` with its `to_io` method, returning `None` when it
    /// has none (Ruby's `IO.try_convert`, `rb_io_check_io`), or the
    /// `TypeError` when `to_io` returns something else than an `IO`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, IO, VM};
    /// # VM::init();
    ///
    /// assert_eq!(IO::try_convert(&IO::stdout()).unwrap(), Some(IO::stdout()));
    /// assert_eq!(IO::try_convert(&Fixnum::new(1)).unwrap(), None);
    ///
    /// let wrapper = VM::eval("o = Object.new; def o.to_io = $stderr; o").unwrap();
    /// assert_eq!(IO::try_convert(&wrapper).unwrap(), Some(IO::stderr()));
    ///
    /// let broken = VM::eval("o = Object.new; def o.to_io = 1; o").unwrap();
    /// assert!(IO::try_convert(&broken).is_err());
    /// ```
    pub fn try_convert<T: Object>(object: &T) -> Result<Option<IO>, AnyException> {
        let object = object.value();

        protect(|| io::check_io(object)).map(|io| {
            if io.is_nil() {
                None
            } else {
                Some(IO::from(io))
            }
        })
    }

    /// Like [`IO::try_convert`](#method.try_convert), but an object without
    /// `to_io` is a `TypeError` too (`rb_io_get_io`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Fixnum, IO, Object, VM};
    /// # VM::init();
    ///
    /// assert_eq!(IO::convert(&IO::stdin()).unwrap(), IO::stdin());
    ///
    /// let error = IO::convert(&Fixnum::new(1)).unwrap_err();
    /// assert!(Class::type_error().case_equals(&error));
    /// ```
    pub fn convert<T: Object>(object: &T) -> Result<IO, AnyException> {
        let object = object.value();

        protect(|| io::get_io(object)).map(IO::from)
    }

    /// Creates a pipe and returns its reading and writing ends (like Ruby's
    /// `IO.pipe`, made with `rb_pipe` and `rb_io_fdopen`). Both descriptors
    /// are close-on-exec, and each `IO` closes its descriptor when closed or
    /// garbage collected.
    ///
    /// Returns the `SystemCallError` when the OS has no pipe to give (such as
    /// `Errno::EMFILE`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IO, RString, VM};
    /// # VM::init();
    ///
    /// let (reader, writer) = IO::pipe().unwrap();
    ///
    /// writer.write(&RString::new_utf8("through the pipe\n")).unwrap();
    /// writer.close().unwrap();
    ///
    /// assert_eq!(reader.gets().unwrap().unwrap().to_str(), "through the pipe\n");
    /// assert_eq!(reader.gets().unwrap(), None);
    /// reader.close().unwrap();
    /// ```
    pub fn pipe() -> Result<(IO, IO), AnyException> {
        let mut writer = None;

        let reader = protect(|| {
            let (reader, write_end) = io::pipe();
            writer = Some(write_end);

            reader
        })?;

        Ok((IO::from(reader), IO::from(writer.unwrap())))
    }

    /// Wraps the file descriptor `fd` in a new `IO` (`rb_io_fdopen`). `flags`
    /// are the `open(2)` flags `fd` was opened with (`O_RDONLY` is 0,
    /// `O_WRONLY` 1 and `O_RDWR` 2), which decide whether the IO can be read
    /// and written; `path` is only for messages and `inspect` (a `path`
    /// other than `"-"` makes it a `File`).
    ///
    /// # Safety
    ///
    /// The `IO` owns `fd`: it closes it when closed or garbage collected.
    /// `fd` must be an open descriptor that nothing else closes or wraps.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, IO, Object, RString, VM};
    /// # VM::init();
    ///
    /// let (reader, writer) = IO::pipe().unwrap();
    ///
    /// // Hand the writing end's descriptor over to a second IO.
    /// let fd = unsafe { writer.send("fileno", &[]) }.try_convert_to::<Fixnum>().unwrap().to_i32();
    /// unsafe { writer.send("autoclose=", &[rutie::Boolean::new(false).into()]) };
    ///
    /// let wrapped = unsafe { IO::fdopen(fd, 1, None) }.unwrap();
    /// wrapped.write(&RString::new_utf8("x")).unwrap();
    /// wrapped.close().unwrap();
    ///
    /// assert_eq!(reader.getbyte().unwrap(), Some(b'x'));
    /// # reader.close().unwrap();
    /// ```
    #[cfg(any(unix, windows))]
    pub unsafe fn fdopen(fd: RawFd, flags: i32, path: Option<&str>) -> Result<IO, AnyException> {
        let path = match path.map(CString::new) {
            Some(Ok(path)) => Some(path),
            Some(Err(_)) => {
                return Err(AnyException::new(
                    "ArgumentError",
                    Some("path contains a NUL byte"),
                ))
            }
            None => None,
        };

        protect(|| io::fdopen(fd, flags, path.as_deref())).map(IO::from)
    }

    /// Pushes `byte` back onto the stream, to be read next (Ruby's
    /// `ungetbyte`, `rb_io_ungetbyte`), or returns the error when the stream
    /// is not open for reading.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IO, VM};
    /// # VM::init();
    ///
    /// let (reader, writer) = IO::pipe().unwrap();
    ///
    /// reader.ungetbyte(b'!').unwrap();
    /// assert_eq!(reader.getbyte().unwrap(), Some(b'!'));
    ///
    /// assert!(writer.ungetbyte(b'!').is_err());
    /// # reader.close().unwrap();
    /// # writer.close().unwrap();
    /// ```
    pub fn ungetbyte(&self, byte: u8) -> Result<(), AnyException> {
        let io_value = self.value();

        protect(|| io::ungetbyte(io_value, Fixnum::new(i64::from(byte)).value())).map(|_| ())
    }

    /// Writes `arguments` formatted with `format` (Ruby's `printf`,
    /// `rb_io_printf`; the format is the one of `Kernel#format`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, IO, RString, VM};
    /// # VM::init();
    ///
    /// let (reader, writer) = IO::pipe().unwrap();
    ///
    /// writer.printf("%s=%03d\n", &[RString::new_utf8("x").into(), Fixnum::new(7).into()]).unwrap();
    ///
    /// assert_eq!(reader.gets().unwrap().unwrap().to_str(), "x=007\n");
    /// assert!(writer.printf("%d", &[RString::new_utf8("x").into()]).is_err());
    /// # reader.close().unwrap();
    /// # writer.close().unwrap();
    /// ```
    pub fn printf(&self, format: &str, arguments: &[AnyObject]) -> Result<(), AnyException> {
        let io_value = self.value();
        let mut values = vec![RString::new_utf8(format).value()];
        values.extend(util::arguments_to_values(arguments));

        protect(|| io::printf(io_value, &values)).map(|_| ())
    }

    /// Writes `bytes` through the stream's write buffer and returns how many
    /// were written (`rb_io_bufwrite`), or the error: the stream is closed
    /// or not open for writing, or the write failed (`Errno::*`).
    ///
    /// Unlike [`write`](#method.write), the bytes are not converted to the
    /// stream's encoding and need not be a Ruby String.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IO, VM};
    /// # VM::init();
    ///
    /// let (reader, writer) = IO::pipe().unwrap();
    ///
    /// assert_eq!(writer.write_bytes(&[b'o', b'k', 0xff]).unwrap(), 3);
    /// writer.flush().unwrap();
    ///
    /// assert_eq!(reader.getbyte().unwrap(), Some(b'o'));
    /// assert!(reader.write_bytes(b"x").is_err());
    /// # reader.close().unwrap();
    /// # writer.close().unwrap();
    /// ```
    pub fn write_bytes(&self, bytes: &[u8]) -> Result<usize, AnyException> {
        let io_value = self.value();
        let mut written = 0;

        protect(|| {
            written = io::bufwrite(io_value, bytes);

            NilClass::new().value()
        })
        .map(|_| written)
    }

    /// Makes the stream write to `write_io` instead of itself (or to itself
    /// again for `None`), as a duplex stream like a pipe opened with
    /// `IO.popen` does, and returns the IO it wrote to before, if it was
    /// another one (`rb_io_set_write_io`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IO, RString, VM};
    /// # VM::init();
    ///
    /// let (reader, writer) = IO::pipe().unwrap();
    /// let (other_reader, other_writer) = IO::pipe().unwrap();
    ///
    /// // `reader` now writes to `other_writer`.
    /// assert_eq!(reader.set_write_io(Some(&other_writer)).unwrap(), None);
    /// reader.write(&RString::new_utf8("y")).unwrap();
    /// assert_eq!(other_reader.getbyte().unwrap(), Some(b'y'));
    ///
    /// assert_eq!(reader.set_write_io(None).unwrap(), Some(other_writer));
    /// # for io in [reader, writer, other_reader] { io.close().unwrap(); }
    /// ```
    pub fn set_write_io(&self, write_io: Option<&IO>) -> Result<Option<IO>, AnyException> {
        let io_value = self.value();
        let write_io = write_io.map_or_else(|| NilClass::new().value(), Object::value);

        protect(|| io::set_write_io(io_value, write_io)).map(|previous| {
            if previous.is_nil() {
                None
            } else {
                Some(IO::from(previous))
            }
        })
    }

    /// Returns Ruby's `FMODE_*` flags for a mode string such as `"r+b"`
    /// (`rb_io_modestr_fmode`), or the `ArgumentError` for an invalid one.
    ///
    /// `FMODE_READABLE` is 0x1, `FMODE_WRITABLE` 0x2, `FMODE_BINMODE` 0x4,
    /// `FMODE_APPEND` 0x40, `FMODE_CREATE` 0x80, `FMODE_EXCL` 0x400,
    /// `FMODE_TRUNC` 0x800 and `FMODE_TEXTMODE` 0x1000.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IO, VM};
    /// # VM::init();
    ///
    /// assert_eq!(IO::fmode("r").unwrap(), 0x1);
    /// assert_eq!(IO::fmode("w+b").unwrap(), 0x1 | 0x2 | 0x4 | 0x80 | 0x800);
    /// assert!(IO::fmode("q").is_err());
    /// ```
    pub fn fmode(mode: &str) -> Result<i32, AnyException> {
        let mode = mode_cstring(mode)?;
        let mut flags = 0;

        protect(|| {
            flags = io::modestr_fmode(&mode);

            NilClass::new().value()
        })
        .map(|_| flags)
    }

    /// Returns the `open(2)` flags (`O_*`) for a mode string
    /// (`rb_io_modestr_oflags`), or the `ArgumentError` for an invalid one.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IO, VM};
    /// # VM::init();
    ///
    /// // O_RDONLY is 0 and O_WRONLY 1 everywhere.
    /// assert_eq!(IO::oflags("r").unwrap() & 3, 0);
    /// assert_eq!(IO::oflags("w").unwrap() & 3, 1);
    /// assert!(IO::oflags("rw").is_err());
    /// ```
    pub fn oflags(mode: &str) -> Result<i32, AnyException> {
        let mode = mode_cstring(mode)?;
        let mut flags = 0;

        protect(|| {
            flags = io::modestr_oflags(&mode);

            NilClass::new().value()
        })
        .map(|_| flags)
    }

    /// Converts `open(2)` flags (`O_*`) to Ruby's `FMODE_*` flags
    /// (`rb_io_oflags_fmode`); see [`IO::fmode`](#method.fmode).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IO, VM};
    /// # VM::init();
    ///
    /// let oflags = IO::oflags("a+").unwrap();
    ///
    /// assert_eq!(IO::fmode_from_oflags(oflags), IO::fmode("a+").unwrap());
    /// ```
    pub fn fmode_from_oflags(oflags: i32) -> i32 {
        io::oflags_fmode(oflags)
    }

    /// Returns whether Ruby uses the file descriptor `fd` itself, such as
    /// for its timer thread (`rb_reserved_fd_p`); a C extension must not
    /// close those.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IO, VM};
    /// # VM::init();
    ///
    /// assert!(!IO::is_reserved_fd(0));
    /// ```
    #[cfg(any(unix, windows))]
    pub fn is_reserved_fd(fd: RawFd) -> bool {
        io::is_reserved_fd(fd)
    }
}

impl From<Value> for IO {
    fn from(value: Value) -> Self {
        IO { value }
    }
}

impl Into<Value> for IO {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for IO {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for IO {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for IO {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        Class::io().case_equals(object)
    }

    fn error_message() -> &'static str {
        "Error converting to IO"
    }
}

impl PartialEq for IO {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

/// `File`, an `IO` for a file; the `IO` methods are available through
/// `Deref`.
#[derive(Debug)]
#[repr(C)]
pub struct File {
    value: Value,
}

impl File {
    /// Opens the file at `path` with a Ruby mode string such as `"r"`,
    /// `"w"`, `"a+"` or `"rb"` (Ruby's `File.open`, `rb_file_open_str`), or
    /// returns the error (usually `Errno::*`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, File, Object, VM};
    /// # VM::init();
    ///
    /// let error = File::open("/no/such/dir/rutie.txt", "r").unwrap_err();
    ///
    /// assert!(Class::system_call_error().case_equals(&error));
    /// assert!(File::open("/tmp", "q").is_err());
    /// ```
    pub fn open(path: &str, mode: &str) -> Result<Self, AnyException> {
        let mode = ::std::ffi::CString::new(mode)
            .map_err(|_| AnyException::new("ArgumentError", Some("mode contains a NUL byte")))?;
        let path = RString::new_utf8(path);

        protect(|| io::file_open(path.value(), &mode)).map(File::from)
    }

    /// Returns `path` as an absolute path, expanding `~` and relative
    /// components against `directory` or the current directory (Ruby's
    /// `File.expand_path`, `rb_file_expand_path`), or the error (for
    /// example an unknown `~user`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{File, VM};
    /// # VM::init();
    ///
    /// let path = File::expand_path("../b", Some("/tmp/a")).unwrap();
    ///
    /// // "/tmp/b" ("C:/tmp/b" on Windows, which adds the current drive).
    /// assert!(path.to_str().ends_with("/tmp/b"));
    /// assert!(File::expand_path("~no_such_rutie_user/x", None).is_err());
    /// ```
    pub fn expand_path(path: &str, directory: Option<&str>) -> Result<RString, AnyException> {
        let path = RString::new_utf8(path);
        let directory = directory.map(RString::new_utf8);

        protect(|| io::expand_path(path.value(), directory.as_ref().map(Object::value)))
            .map(RString::from)
    }

    /// Like [`File::expand_path`](#method.expand_path), but `~` is not
    /// expanded (Ruby's `File.absolute_path`, `rb_file_absolute_path`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{File, VM};
    /// # VM::init();
    ///
    /// let path = File::absolute_path("~/x", Some("/tmp")).unwrap();
    ///
    /// // "/tmp/~/x" ("C:/tmp/~/x" on Windows, which adds the current drive).
    /// assert!(path.to_str().ends_with("/tmp/~/x"));
    /// ```
    pub fn absolute_path(path: &str, directory: Option<&str>) -> Result<RString, AnyException> {
        let path = RString::new_utf8(path);
        let directory = directory.map(RString::new_utf8);

        protect(|| io::absolute_path(path.value(), directory.as_ref().map(Object::value)))
            .map(RString::from)
    }

    /// Returns everything but the last component of `path` (Ruby's
    /// `File.dirname`, `rb_file_dirname`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{File, VM};
    /// # VM::init();
    ///
    /// assert_eq!(File::dirname("/usr/lib/ruby").to_str(), "/usr/lib");
    /// assert_eq!(File::dirname("file.rb").to_str(), ".");
    /// ```
    pub fn dirname(path: &str) -> RString {
        RString::from(io::dirname(RString::new_utf8(path).value()))
    }

    /// Returns the current working directory (Ruby's `Dir.pwd`,
    /// `rb_dir_getwd`), or the OS error.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{File, VM};
    /// # VM::init();
    ///
    /// let cwd = File::current_directory().unwrap();
    ///
    /// // Ruby separates path components with "/", on Windows too.
    /// let expected = std::env::current_dir().unwrap().to_str().unwrap().replace('\\', "/");
    ///
    /// assert_eq!(cwd.to_str(), expected);
    /// ```
    pub fn current_directory() -> Result<RString, AnyException> {
        protect(io::getwd).map(RString::from)
    }

    /// Returns the file's size in bytes, after flushing buffered output
    /// (Ruby's `File#size`, `rb_file_size`), or the error, such as
    /// `IOError` for a closed file.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{File, RString, VM};
    /// # VM::init();
    ///
    /// let path = std::env::temp_dir().join(format!("rutie_file_size_example_{}.txt", std::process::id()));
    /// let file = File::open(path.to_str().unwrap(), "wb").unwrap();
    ///
    /// assert_eq!(file.size().unwrap(), 0);
    ///
    /// // Counts output still in Ruby's buffer.
    /// file.write(&RString::new_utf8("12345")).unwrap();
    /// assert_eq!(file.size().unwrap(), 5);
    ///
    /// file.close().unwrap();
    /// assert!(file.size().is_err());
    /// # std::fs::remove_file(path).unwrap();
    /// ```
    pub fn size(&self) -> Result<u64, AnyException> {
        let file = self.value();
        let mut size = 0;

        protect(|| {
            size = io::file_size(file);

            NilClass::new().value()
        })
        .map(|_| size as u64)
    }

    /// Returns whether `path` is a directory, or a symbolic link to one
    /// (Ruby's `File.directory?`, `rb_file_directory_p`), or the error for a
    /// path with a NUL byte.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{File, VM};
    /// # VM::init();
    ///
    /// let directory = std::env::temp_dir();
    ///
    /// assert!(File::is_directory(directory.to_str().unwrap()).unwrap());
    /// assert!(!File::is_directory("/no/such/rutie/dir").unwrap());
    /// assert!(File::is_directory("nul\0byte").is_err());
    /// ```
    pub fn is_directory(path: &str) -> Result<bool, AnyException> {
        let path = RString::new_utf8(path);
        let mut is_directory = false;

        protect(|| {
            is_directory = io::is_directory(path.value());

            NilClass::new().value()
        })
        .map(|_| is_directory)
    }

    /// Returns whether `path` is absolute (`rb_is_absolute_path`): it starts
    /// with `/`, or on Windows with a drive letter and a separator or with
    /// two separators (a UNC path). A path with a NUL byte is not.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{File, VM};
    /// # VM::init();
    ///
    /// assert!(!File::is_absolute_path("relative/path"));
    /// assert!(!File::is_absolute_path("~/x"));
    ///
    /// if cfg!(windows) {
    ///     assert!(File::is_absolute_path("C:/Windows"));
    /// } else {
    ///     assert!(File::is_absolute_path("/usr"));
    /// }
    /// ```
    pub fn is_absolute_path(path: &str) -> bool {
        match CString::new(path) {
            Ok(path) => io::is_absolute_path(&path),
            Err(_) => false,
        }
    }

    /// Returns `path` converted to the encoding of the OS's file names where
    /// there is one, UTF-8 on Windows and macOS, and as it is elsewhere
    /// (`rb_str_encode_ospath`); or the error when it cannot be converted.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{File, RString, VM};
    /// # VM::init();
    ///
    /// let path = File::encode_ospath(&RString::new_utf8("dir/fïle")).unwrap();
    ///
    /// assert_eq!(path.to_str(), "dir/fïle");
    /// ```
    pub fn encode_ospath(path: &RString) -> Result<RString, AnyException> {
        let path = path.value();

        protect(|| io::encode_ospath(path)).map(RString::from)
    }

    /// Converts `object` to a path the way Ruby's file methods do: with its
    /// `to_path` method if it has one, then as a String, rejecting NUL bytes
    /// and encodings that are not ASCII-compatible (`rb_get_path`,
    /// `FilePathValue`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, File, Fixnum, Object, RString, VM};
    /// # VM::init();
    ///
    /// assert_eq!(File::get_path(&RString::new_utf8("a/b")).unwrap().to_str(), "a/b");
    ///
    /// VM::require("pathname");
    /// let pathname = VM::eval("Pathname.new('/c/d')").unwrap();
    /// assert_eq!(File::get_path(&pathname).unwrap().to_str(), "/c/d");
    ///
    /// let error = File::get_path(&Fixnum::new(1)).unwrap_err();
    /// assert!(Class::type_error().case_equals(&error));
    /// assert!(File::get_path(&RString::new_utf8("a\0b")).is_err());
    /// ```
    pub fn get_path<T: Object>(object: &T) -> Result<RString, AnyException> {
        let object = object.value();

        protect(|| io::get_path(object)).map(RString::from)
    }

    /// Returns the paths matching the glob `pattern`, like Ruby's
    /// `Dir.glob(pattern)` (`rb_glob`), or the error Ruby raised while
    /// matching.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{File, VM};
    /// # VM::init();
    ///
    /// let directory = std::env::temp_dir().join(format!("rutie_glob_example_{}", std::process::id()));
    /// std::fs::create_dir_all(&directory).unwrap();
    /// for name in ["a.rb", "b.rb", "c.txt"] {
    ///     std::fs::write(directory.join(name), "").unwrap();
    /// }
    ///
    /// let pattern = format!("{}/*.rb", directory.to_str().unwrap().replace('\\', "/"));
    /// let mut names: Vec<String> = File::glob(&pattern).unwrap().iter()
    ///     .map(|path| path.to_str().rsplit('/').next().unwrap().to_string())
    ///     .collect();
    /// names.sort();
    ///
    /// assert_eq!(names, ["a.rb", "b.rb"]);
    /// # std::fs::remove_dir_all(directory).unwrap();
    /// ```
    pub fn glob(pattern: &str) -> Result<Vec<RString>, AnyException> {
        let pattern = CString::new(pattern)
            .map_err(|_| AnyException::new("ArgumentError", Some("pattern contains a NUL byte")))?;

        protect(|| io::glob(&pattern)).map(glob_results)
    }

    /// Like [`File::glob`](#method.glob), with `flags` (`File::FNM_*`, such
    /// as `File::FNM_DOTMATCH`, 4, to match names starting with a dot), and
    /// never raising (`ruby_glob`). A pattern with a NUL byte matches
    /// nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{File, VM};
    /// # VM::init();
    ///
    /// let directory = std::env::temp_dir().join(format!("rutie_glob_flags_example_{}", std::process::id()));
    /// std::fs::create_dir_all(&directory).unwrap();
    /// for name in ["a.rb", "b.txt", ".hidden.rb"] {
    ///     std::fs::write(directory.join(name), "").unwrap();
    /// }
    /// let directory_str = directory.to_str().unwrap().replace('\\', "/");
    ///
    /// let names = |paths: Vec<rutie::RString>| {
    ///     let mut names: Vec<String> = paths.iter()
    ///         .map(|path| path.to_str().rsplit('/').next().unwrap().to_string())
    ///         .collect();
    ///     names.sort();
    ///     names
    /// };
    ///
    /// let pattern = format!("{}/*.rb", directory_str);
    /// assert_eq!(names(File::glob_with_flags(&pattern, 0)), ["a.rb"]);
    /// assert_eq!(names(File::glob_with_flags(&pattern, 4)), [".hidden.rb", "a.rb"]);
    ///
    /// let pattern = format!("{}/*.{{rb,txt}}", directory_str);
    /// assert_eq!(names(File::glob_with_flags(&pattern, 0)), ["a.rb", "b.txt"]);
    /// # std::fs::remove_dir_all(directory).unwrap();
    /// ```
    pub fn glob_with_flags(pattern: &str, flags: i32) -> Vec<RString> {
        match CString::new(pattern) {
            Ok(pattern) => glob_results(io::glob_with_flags(&pattern, flags)),
            Err(_) => Vec::new(),
        }
    }
}

impl ::std::ops::Deref for File {
    type Target = IO;

    fn deref(&self) -> &IO {
        // `File` and `IO` are both `#[repr(C)]` wrappers around a `Value`.
        unsafe { &*(self as *const File as *const IO) }
    }
}

impl From<Value> for File {
    fn from(value: Value) -> Self {
        File { value }
    }
}

impl Into<Value> for File {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for File {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for File {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for File {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        Class::file().case_equals(object)
    }

    fn error_message() -> &'static str {
        "Error converting to File"
    }
}

impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Array, Class, File, Fixnum, Object, RString, IO, VM};
    use std::time::Duration;

    fn errno(name: &str) -> i32 {
        VM::eval(&format!("Errno::{}::Errno", name))
            .unwrap()
            .try_convert_to::<Fixnum>()
            .unwrap()
            .to_i64() as i32
    }

    fn pipe() -> (IO, IO) {
        let pipe = VM::eval("IO.pipe")
            .unwrap()
            .try_convert_to::<Array>()
            .unwrap();

        (
            pipe.at(0).try_convert_to::<IO>().unwrap(),
            pipe.at(1).try_convert_to::<IO>().unwrap(),
        )
    }

    #[test]
    fn test_io_descriptor_and_maybe_wait() {
        crate::on_ruby_thread(|| {
            let (reader, writer) = pipe();
            let short = Some(Duration::from_millis(10));

            assert!(reader.descriptor().unwrap() > 2);
            assert_ne!(reader.descriptor(), writer.descriptor());

            // EINTR: retry right away.
            assert_eq!(
                reader.maybe_wait(errno("EINTR"), IO::READABLE, None),
                Ok(Some(IO::READABLE))
            );
            assert_eq!(reader.maybe_wait_readable(errno("EINTR"), None), Ok(true));

            // EAGAIN with nothing to read: times out.
            assert_eq!(
                reader.maybe_wait(errno("EAGAIN"), IO::READABLE, short),
                Ok(None)
            );
            // A timeout raises `IO::TimeoutError`.
            let error = reader
                .maybe_wait_readable(errno("EAGAIN"), short)
                .unwrap_err();
            assert_eq!(error.class().name().unwrap().to_str(), "IO::TimeoutError");

            assert_eq!(
                writer.maybe_wait(errno("EAGAIN"), IO::WRITABLE, None),
                Ok(Some(IO::WRITABLE))
            );
            assert_eq!(
                writer.maybe_wait(errno("EACCES"), IO::WRITABLE, None),
                Ok(None)
            );

            reader.close().unwrap();
            writer.close().unwrap();
            assert!(reader.descriptor().is_err());
            assert!(reader
                .maybe_wait(errno("EAGAIN"), IO::READABLE, None)
                .is_err());
            assert!(reader.maybe_wait_readable(errno("EAGAIN"), None).is_err());
        });
    }

    #[test]
    fn test_io_timeout() {
        crate::on_ruby_thread(|| {
            let (reader, writer) = pipe();

            assert_eq!(reader.timeout(), Ok(None));
            reader
                .set_timeout(Some(Duration::from_millis(250)))
                .unwrap();
            assert_eq!(reader.timeout(), Ok(Some(Duration::from_millis(250))));
            let ruby_timeout = unsafe { reader.send("timeout", &[]) };
            assert_eq!(
                ruby_timeout
                    .try_convert_to::<crate::Float>()
                    .unwrap()
                    .to_f64(),
                0.25
            );

            reader.set_timeout(Some(Duration::from_millis(5))).unwrap();
            let error = reader.getbyte().unwrap_err();
            assert!(Class::io_timeout_error().case_equals(&error));
            assert!(Class::io_timeout_error()
                .inherits(&Class::io_error())
                .unwrap());

            reader.set_timeout(None).unwrap();
            assert_eq!(reader.timeout(), Ok(None));
            reader.close().unwrap();
            writer.close().unwrap();
        });
    }

    #[test]
    fn test_io_from_raw_fd_owning() {
        crate::on_ruby_thread(|| {
            let (reader, writer) = pipe();
            let fd = writer.descriptor().unwrap();

            // Let go of the descriptor, then hand it to a new IO that owns it.
            unsafe { writer.send("autoclose=", &[crate::Boolean::new(false).into()]) };
            writer.close().unwrap();

            let owner = unsafe { IO::from_raw_fd(fd, false, true, true) };
            assert!(unsafe { owner.send("autoclose?", &[]) }.value().is_true());
            assert_eq!(owner.descriptor(), Ok(fd));
            owner.write(&RString::new_utf8("owned\n")).unwrap();
            owner.close().unwrap();

            // Closing the owner closed the descriptor: the pipe is at its end.
            assert_eq!(reader.gets().unwrap().unwrap().to_str(), "owned\n");
            assert!(reader.gets().unwrap().is_none());
            reader.close().unwrap();

            // An IO that may not write.
            let (reader, writer) = pipe();
            let read_only =
                unsafe { IO::from_raw_fd(reader.descriptor().unwrap(), true, false, false) };
            assert!(read_only.write(&RString::new_utf8("x")).is_err());
            read_only.close().unwrap();
            reader.close().unwrap();
            writer.close().unwrap();
        });
    }

    #[test]
    fn test_process_status_wait() {
        crate::on_ruby_thread(|| {
            VM::require("rbconfig");
            let pid = VM::eval("Process.spawn(RbConfig.ruby, '-e', 'exit 3')")
                .unwrap()
                .try_convert_to::<Fixnum>()
                .unwrap()
                .to_i64();

            let status = unsafe { crate::rubysys::io::rb_process_status_wait(pid as _, 0) };
            let status = crate::AnyObject::from(status);

            assert!(Class::from_existing("Process")
                .get_nested_class("Status")
                .case_equals(&status));
            assert_eq!(
                unsafe { status.send("exitstatus", &[]) }.try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(3))
            );
            assert_eq!(
                unsafe { status.send("pid", &[]) }.try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(pid))
            );

            // Still running: nothing to reap with WNOHANG.
            let pid = VM::eval("Process.spawn(RbConfig.ruby, '-e', 'sleep 30')")
                .unwrap()
                .try_convert_to::<Fixnum>()
                .unwrap()
                .to_i64();
            let wnohang = VM::eval("Process::WNOHANG")
                .unwrap()
                .try_convert_to::<Fixnum>()
                .unwrap()
                .to_i64();

            let status =
                unsafe { crate::rubysys::io::rb_process_status_wait(pid as _, wnohang as _) };
            assert!(status.is_nil());

            VM::eval(&format!("Process.kill(:KILL, {})", pid)).unwrap();
            let status = unsafe { crate::rubysys::io::rb_process_status_wait(pid as _, 0) };
            assert_eq!(
                unsafe { crate::AnyObject::from(status).send("pid", &[]) }
                    .try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(pid))
            );
        });
    }

    #[test]
    fn test_file_size() {
        crate::on_ruby_thread(|| {
            let path =
                std::env::temp_dir().join(format!("rutie_file_size_unit_{}", std::process::id()));
            std::fs::write(&path, "abc").unwrap();

            let file = File::open(path.to_str().unwrap(), "ab").unwrap();
            assert_eq!(file.size(), Ok(3));
            file.write(&RString::new_utf8("de")).unwrap();
            assert_eq!(file.size(), Ok(5));
            file.close().unwrap();

            assert!(Class::io_error().case_equals(&file.size().unwrap_err()));
            std::fs::remove_file(path).unwrap();
        });
    }

    #[test]
    fn test_file_round_trip() {
        crate::on_ruby_thread(|| {
            let path =
                std::env::temp_dir().join(format!("rutie_io_unit_test_{}.txt", std::process::id()));
            let path_str = path.to_str().unwrap();

            // Binary mode: in text mode Ruby on Windows writes "\r\n".
            let file = File::open(path_str, "wb").unwrap();
            file.write(&RString::new_utf8("first\n")).unwrap();
            file.puts(&[Fixnum::new(2).into()]).unwrap();
            file.print(&[RString::new_utf8("three").into()]).unwrap();
            file.flush().unwrap();
            assert!(file.is_eof().is_err()); // not open for reading
            file.close().unwrap();

            let file = File::open(path_str, "rb").unwrap();
            assert!(file.write(&RString::new_utf8("x")).is_err()); // not open for writing
            assert_eq!(file.gets().unwrap().unwrap().to_str(), "first\n");
            assert_eq!(file.gets().unwrap().unwrap().to_str(), "2\n");
            assert_eq!(file.getbyte().unwrap(), Some(b't'));
            file.close().unwrap();
            assert!(file.gets().is_err());

            // The object Ruby sees is a File.
            assert!(Class::file().case_equals(&file.to_any_object()));
            assert!(file.to_any_object().try_convert_to::<IO>().is_ok());
            assert!(IO::stdout()
                .to_any_object()
                .try_convert_to::<File>()
                .is_err());

            std::fs::remove_file(path).unwrap();

            assert_eq!(File::dirname("/a/b/c.rb").to_str(), "/a/b");
            // An absolute directory on Windows has a drive.
            let base = if cfg!(windows) { "C:/y" } else { "/y" };
            assert_eq!(
                File::expand_path("x", Some(base)).unwrap().to_str(),
                format!("{}/x", base)
            );
            assert!(File::open("/definitely/missing/rutie", "r").is_err());
            assert!(File::open("x", "bad\0mode").is_err());
            assert!(!File::current_directory().unwrap().to_str().is_empty());

            let _ = VM::eval("1");
        });
    }

    #[test]
    fn test_io_streams_and_paths() {
        crate::on_ruby_thread(|| {
            let stdin = IO::stdin();
            let stderr = IO::stderr();
            assert!(!stdin.is_closed());
            assert!(!stderr.is_closed());
            assert_eq!(
                unsafe { stderr.send("fileno", &[]) }.try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(2))
            );
            assert_eq!(
                unsafe { stdin.send("fileno", &[]) }.try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(0))
            );

            let path =
                std::env::temp_dir().join(format!("rutie_io_binmode_{}", std::process::id()));
            let path_str = path.to_str().unwrap();
            let file = File::open(path_str, "w").unwrap();
            file.binmode().unwrap();
            let binmode = unsafe { file.send("binmode?", &[]) };
            assert!(binmode.value().is_true());
            file.close().unwrap();
            assert!(file.is_closed());
            // Operations on a closed stream are errors.
            assert!(file.binmode().is_err());
            std::fs::remove_file(&path).unwrap();

            // An absolute directory on Windows has a drive.
            let base = if cfg!(windows) { "C:/base" } else { "/base" };
            let absolute = File::absolute_path("child", Some(base)).unwrap();
            assert_eq!(absolute.to_str(), format!("{}/child", base));
            // Unlike expand_path, `~` is not expanded.
            let tilde = File::absolute_path("~", Some(base)).unwrap();
            assert_eq!(tilde.to_str(), format!("{}/~", base));
        });
    }

    #[test]
    fn test_raw_io_mode() {
        use crate::rubysys::io::{rb_io_mode, FMODE_READABLE, FMODE_WRITABLE};

        crate::on_ruby_thread(|| {
            let stdout = VM::eval("$stdout").unwrap();
            let stdin = VM::eval("$stdin").unwrap();

            let mode = unsafe { rb_io_mode(stdout.value()) };
            assert_eq!(mode & (FMODE_READABLE | FMODE_WRITABLE), FMODE_WRITABLE);

            let mode = unsafe { rb_io_mode(stdin.value()) };
            assert_eq!(mode & (FMODE_READABLE | FMODE_WRITABLE), FMODE_READABLE);
        });
    }

    #[test]
    fn test_pipes_and_descriptors() {
        crate::on_ruby_thread(|| {
            let (reader, writer) = IO::pipe().unwrap();

            // The write end is unbuffered, like `IO.pipe`'s.
            writer
                .printf(
                    "%d-%s\n",
                    &[Fixnum::new(4).into(), RString::new_utf8("x").into()],
                )
                .unwrap();
            assert_eq!(reader.gets().unwrap().unwrap().to_str(), "4-x\n");

            assert_eq!(writer.write_bytes(b"ab").unwrap(), 2);
            assert_eq!(reader.getbyte().unwrap(), Some(b'a'));
            reader.ungetbyte(b'z').unwrap();
            assert_eq!(reader.getbyte().unwrap(), Some(b'z'));
            assert_eq!(reader.getbyte().unwrap(), Some(b'b'));
            assert!(writer.ungetbyte(b'z').is_err());
            assert!(reader.write_bytes(b"z").is_err());
            assert!(writer
                .printf("%d", &[RString::new_utf8("no").into()])
                .is_err());

            assert_eq!(
                IO::try_convert(&reader).unwrap(),
                Some(IO::from(reader.value()))
            );
            assert_eq!(IO::try_convert(&Fixnum::new(1)).unwrap(), None);
            assert!(IO::convert(&Fixnum::new(1)).is_err());
            assert!(IO::convert(&writer).is_ok());

            let (other_reader, other_writer) = IO::pipe().unwrap();
            assert_eq!(reader.set_write_io(Some(&other_writer)).unwrap(), None);
            reader.write(&RString::new_utf8("w")).unwrap();
            assert_eq!(other_reader.getbyte().unwrap(), Some(b'w'));
            assert_eq!(
                reader.set_write_io(None).unwrap(),
                Some(IO::from(other_writer.value()))
            );
            assert!(reader.write(&RString::new_utf8("w")).is_err());

            let fd = unsafe { other_writer.send("fileno", &[]) }
                .try_convert_to::<Fixnum>()
                .unwrap()
                .to_i32();
            assert!(!IO::is_reserved_fd(fd));
            unsafe { other_writer.send("autoclose=", &[crate::Boolean::new(false).into()]) };
            let wrapped = unsafe { IO::fdopen(fd, 1, Some("pipe-end")) }.unwrap();
            // A path other than "-" makes a File.
            assert!(Class::file().case_equals(&wrapped));
            wrapped.write(&RString::new_utf8("v")).unwrap();
            wrapped.flush().unwrap();
            assert_eq!(other_reader.getbyte().unwrap(), Some(b'v'));
            assert!(unsafe { IO::fdopen(fd, 1, Some("nul\0")) }.is_err());

            for io in [reader, writer, other_reader, wrapped] {
                io.close().unwrap();
            }

            assert_eq!(IO::fmode("r+").unwrap(), 0x3);
            assert_eq!(IO::fmode("ab").unwrap() & 0x46, 0x46);
            assert!(IO::fmode("x").is_err());
            assert!(IO::fmode("r\0").is_err());
            assert_eq!(IO::fmode_from_oflags(IO::oflags("r+").unwrap()), 0x3);
            assert!(IO::oflags("").is_err());
        });
    }

    #[test]
    fn test_file_paths_and_globs() {
        crate::on_ruby_thread(|| {
            let directory =
                std::env::temp_dir().join(format!("rutie_glob_unit_test_{}", std::process::id()));
            std::fs::create_dir_all(&directory).unwrap();
            for name in ["one.rb", "two.rs", ".three.rb"] {
                std::fs::write(directory.join(name), "").unwrap();
            }
            let base = directory.to_str().unwrap().replace('\\', "/");

            assert!(File::is_directory(&base).unwrap());
            assert!(!File::is_directory(&format!("{}/one.rb", base)).unwrap());
            assert!(File::is_absolute_path(&base));
            assert!(!File::is_absolute_path("one.rb"));
            assert!(!File::is_absolute_path("/nul\0"));

            let names = |paths: Vec<RString>| {
                let mut names: Vec<String> = paths
                    .iter()
                    .map(|path| path.to_str().rsplit('/').next().unwrap().to_string())
                    .collect();
                names.sort();
                names
            };

            assert_eq!(
                names(File::glob(&format!("{}/*.rb", base)).unwrap()),
                ["one.rb"]
            );
            assert!(File::glob("nul\0").is_err());
            let braces = format!("{}/*.{{rb,rs}}", base);
            assert_eq!(names(File::glob(&braces).unwrap()), ["one.rb", "two.rs"]);
            assert_eq!(
                names(File::glob_with_flags(&braces, 0)),
                ["one.rb", "two.rs"]
            );
            assert_eq!(
                names(File::glob_with_flags(&format!("{}/*.rb", base), 4)),
                [".three.rb", "one.rb"]
            );
            assert!(File::glob_with_flags("nul\0", 0).is_empty());

            let path = RString::new_utf8("a/b");
            assert_eq!(File::encode_ospath(&path).unwrap().to_str(), "a/b");
            assert_eq!(File::get_path(&path).unwrap().to_str(), "a/b");
            assert!(File::get_path(&Fixnum::new(1)).is_err());

            std::fs::remove_dir_all(directory).unwrap();
        });
    }
}
