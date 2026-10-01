use std::{convert::From, time::Duration};

use crate::{
    binding::{float, io, vm},
    types::Value,
    util, AnyException, AnyObject, Class, Exception, Fixnum, Float, NilClass, Object, RString,
    VerifiedObject,
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
    /// `rb_io_closed_p` on Ruby 3.3+).
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
    #[cfg(ruby_gte_3_1)]
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
    /// Ruby 3.1+.
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
    #[cfg(ruby_gte_3_1)]
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
    /// (or `errno` is `EINTR`), `Ok(false)` on timeout or for an error that
    /// waiting does not help with.
    ///
    /// Ruby 3.1+.
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
    #[cfg(ruby_gte_3_1)]
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
    /// Ruby 3.1+.
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
    #[cfg(ruby_gte_3_1)]
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
    #[cfg(ruby_gte_3_2)]
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
    #[cfg(ruby_gte_3_2)]
    pub fn set_timeout(&self, timeout: Option<Duration>) -> Result<(), AnyException> {
        let io_value = self.value();
        let timeout = timeout_value(timeout);

        protect(|| io::set_timeout(io_value, timeout)).map(|_| ())
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
    /// Ruby 3.1+.
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
    #[cfg(ruby_gte_3_1)]
    pub fn size(&self) -> Result<u64, AnyException> {
        let file = self.value();
        let mut size = 0;

        protect(|| {
            size = io::file_size(file);

            NilClass::new().value()
        })
        .map(|_| size as u64)
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
    #[cfg(ruby_gte_3_1)]
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
            assert_eq!(
                reader.maybe_wait_readable(errno("EAGAIN"), short),
                Ok(false)
            );

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

    #[cfg(ruby_gte_3_2)]
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
    #[cfg(ruby_gte_3_1)]
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
}
