use crate::{
    binding::exception,
    types::{Value, ValueType},
    AnyObject, Class, Exception, NilClass, Object, TryConvert, VerifiedObject,
};
use std::{
    borrow::Borrow,
    fmt,
    fmt::{Display, Formatter},
    ops::Deref,
};

pub struct AnyException {
    value: Value,
}

impl AnyException {
    /// Creates an exception of `class` with `message`, without looking the
    /// class up by name (`rb_exc_new_str`).
    ///
    /// Unlike [`Exception::new`](trait.Exception.html#method.new), this
    /// cannot fail on an unknown class name.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{AnyException, Class, Exception, Object, VM};
    /// # VM::init();
    ///
    /// let error = AnyException::from_class(&Class::key_error(), "no such key");
    ///
    /// assert!(Class::key_error().case_equals(&error));
    /// assert_eq!(error.message(), "no such key");
    ///
    /// let result = VM::protect(|| {
    ///     VM::raise_ex(AnyException::from_class(&Class::argument_error(), "bad"));
    ///     rutie::NilClass::new().into()
    /// });
    /// assert!(result.is_err());
    /// assert_eq!(VM::error_pop().unwrap().message(), "bad");
    /// ```
    pub fn from_class(class: &Class, message: &str) -> Self {
        AnyException::from(exception::new(class.value(), message))
    }

    /// Creates the `SystemCallError` subclass (`Errno::*`) for the OS error
    /// number `errno`, with `message` added to its description
    /// (`rb_syserr_new`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{AnyException, Class, Exception, Object, VM};
    /// # VM::init();
    ///
    /// let error = AnyException::from_errno(2, "config.yml");
    ///
    /// assert!(Class::from_existing("Errno").get_nested_class("ENOENT").case_equals(&error));
    /// assert!(error.message().contains("config.yml"));
    /// ```
    pub fn from_errno(errno: i32, message: &str) -> Self {
        AnyException::from(exception::syserr_new(errno, message))
    }

    /// Creates the `SystemCallError` subclass for a Rust `std::io::Error`
    /// that came from the OS, or a plain `IOError` for any other kind of
    /// error, with `message` added to its description.
    ///
    /// On Windows the OS error is a Win32 error code, which is mapped to an
    /// `errno` the way Ruby maps it (`ERROR_ACCESS_DENIED` is `Errno::EACCES`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{AnyException, Class, Exception, Object, VM};
    /// use std::fs::File;
    /// # VM::init();
    ///
    /// let io_error = File::open("/no/such/file/for/rutie").unwrap_err();
    /// let error = AnyException::from_io_error(&io_error, "/no/such/file/for/rutie");
    ///
    /// assert!(Class::system_call_error().case_equals(&error));
    /// assert!(Class::from_existing("Errno").get_nested_class("ENOENT").case_equals(&error));
    ///
    /// let custom = std::io::Error::new(std::io::ErrorKind::Other, "custom");
    /// let error = AnyException::from_io_error(&custom, "while reading");
    ///
    /// assert!(Class::io_error().case_equals(&error));
    /// assert!(error.message().contains("custom"));
    /// ```
    pub fn from_io_error(error: &std::io::Error, message: &str) -> Self {
        match error.raw_os_error() {
            Some(code) => AnyException::from_errno(exception::os_error_to_errno(code), message),
            None => {
                let message = format!("{} - {}", error, message);

                AnyException::from_class(&Class::io_error(), &message)
            }
        }
    }
}

impl From<Value> for AnyException {
    fn from(value: Value) -> Self {
        AnyException { value }
    }
}

impl Into<Value> for AnyException {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for AnyException {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Borrow<Value> for AnyException {
    fn borrow(&self) -> &Value {
        &self.value
    }
}

impl AsRef<Value> for AnyException {
    fn as_ref(&self) -> &Value {
        &self.value
    }
}

impl AsRef<AnyException> for AnyException {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Object for AnyException {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl Deref for AnyException {
    type Target = Value;

    fn deref(&self) -> &Value {
        &self.value
    }
}

impl Exception for AnyException {}

impl TryConvert<AnyObject> for AnyException {
    type Nil = NilClass;

    fn try_convert(obj: AnyObject) -> Result<Self, NilClass> {
        obj.try_convert_to::<AnyException>()
            .map_err(|_| NilClass::new())
    }
}

impl VerifiedObject for AnyException {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        Class::exception().case_equals(object)
    }

    fn error_message() -> &'static str {
        "Error converting to AnyException"
    }
}

impl Display for AnyException {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.inspect())
    }
}

impl fmt::Debug for AnyException {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inspect())
    }
}

impl PartialEq for AnyException {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{AnyException, Class, Exception, Object, VM};

    #[test]
    fn test_exception_constructors() {
        crate::on_ruby_thread(|| {
            let error = AnyException::from_class(&Class::runtime_error(), "100% %s");
            assert_eq!(error.message(), "100% %s");
            assert!(error.try_convert_to::<AnyException>().is_ok());

            let enoent = AnyException::from_errno(2, "path");
            assert!(Class::system_call_error().case_equals(&enoent));
            let errno = unsafe { enoent.send("errno", &[]) };
            assert_eq!(errno.try_convert_to::<crate::Fixnum>().unwrap().to_i64(), 2);

            // An unknown errno still gives a SystemCallError.
            let unknown = AnyException::from_errno(99_999, "odd");
            assert!(Class::system_call_error().case_equals(&unknown));

            // A permission error: EACCES, or ERROR_ACCESS_DENIED on Windows.
            let code = if cfg!(windows) { 5 } else { 13 };
            let io = std::io::Error::from_raw_os_error(code);
            let eacces = AnyException::from_io_error(&io, "secret");
            assert!(Class::from_path("Errno::EACCES")
                .unwrap()
                .case_equals(&eacces));

            // Raised and rescued from Ruby like any exception.
            let result = VM::protect(|| {
                VM::raise_ex(AnyException::from_class(&Class::type_error(), "typed"));
                crate::NilClass::new().into()
            });
            assert!(result.is_err());
            let raised = VM::error_pop().unwrap();
            assert!(Class::type_error().case_equals(&raised));
            assert!(raised.backtrace().is_some());
        });
    }
}
