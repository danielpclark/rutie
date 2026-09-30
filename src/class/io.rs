use std::convert::From;

use crate::{
    binding::{io, vm},
    types::Value,
    util, AnyException, AnyObject, Class, Exception, Fixnum, NilClass, Object, RString,
    VerifiedObject,
};

fn protect<F>(func: F) -> Result<Value, AnyException>
where
    F: FnOnce() -> Value,
{
    vm::protect_value(func).map_err(AnyException::from)
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
    /// assert_eq!(std::fs::read_to_string(&path).unwrap(), "a\n1\n");
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

    /// Returns `true` if the stream is closed (Ruby's `closed?`).
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
        vm::call_method(self.value(), "closed?", &[]).is_true()
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
    /// assert_eq!(File::expand_path("../b", Some("/tmp/a")).unwrap().to_str(), "/tmp/b");
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
    /// assert_eq!(File::absolute_path("~/x", Some("/tmp")).unwrap().to_str(), "/tmp/~/x");
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
    /// assert_eq!(cwd.to_str(), std::env::current_dir().unwrap().to_str().unwrap());
    /// ```
    pub fn current_directory() -> Result<RString, AnyException> {
        protect(io::getwd).map(RString::from)
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
    use crate::{Class, File, Fixnum, Object, RString, IO, VM};

    #[test]
    fn test_file_round_trip() {
        crate::on_ruby_thread(|| {
            let path =
                std::env::temp_dir().join(format!("rutie_io_unit_test_{}.txt", std::process::id()));
            let path_str = path.to_str().unwrap();

            let file = File::open(path_str, "w").unwrap();
            file.write(&RString::new_utf8("first\n")).unwrap();
            file.puts(&[Fixnum::new(2).into()]).unwrap();
            file.print(&[RString::new_utf8("three").into()]).unwrap();
            file.flush().unwrap();
            assert!(file.is_eof().is_err()); // not open for reading
            file.close().unwrap();

            let file = File::open(path_str, "r").unwrap();
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
            assert_eq!(File::expand_path("x", Some("/y")).unwrap().to_str(), "/y/x");
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

            let absolute = File::absolute_path("child", Some("/base")).unwrap();
            assert_eq!(absolute.to_str(), "/base/child");
            // Unlike expand_path, `~` is not expanded.
            let tilde = File::absolute_path("~", Some("/base")).unwrap();
            assert_eq!(tilde.to_str(), "/base/~");
        });
    }
}
