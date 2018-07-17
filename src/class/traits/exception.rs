use ::{AnyObject, Object, RString, Array, Class};
use binding::vm;
use util;

/// Descendants of class Exception are used to communicate between Kernel#raise
/// and rescue statements in `begin ... end` blocks. Exception objects carry
/// information about the exception – its type (the exception's class name), an
/// optional descriptive string, and optional traceback information. Exception
/// subclasses may add additional information like NameError#name.
///
/// Programs may make subclasses of Exception, typically of StandardError or
/// RuntimeError, to provide custom classes and add additional information.
/// See the subclass list below for defaults for `raise` and `rescue`.
pub trait Exception: Object {
    /// Construct a new Exception object, optionally passing in a message.
    ///
    /// # Examples
    /// ```
    /// use rutie::{AnyException, Exception, Object, VM};
    /// # VM::init();
    ///
    /// assert_eq!(
    ///   AnyException::new("StandardError", None).to_s(),
    ///   "StandardError"
    /// );
    /// ```
    fn new(class: &str, msg: Option<&str>) -> Self {
        let class = Class::from_existing(class);
        let msg = msg.map(|s| RString::new(s).value());

        Self::from(vm::call_method(class.value(), "new", util::option_to_slice(&msg)))
    }

    /// With no argument, or if the argument is the same as the receiver,
    /// return the receiver. Otherwise, create a new exception object of
    /// the same class as the receiver, but with a message equal
    /// to `string.to_str`.
    ///
    /// # Examples
    /// ```
    /// use rutie::{AnyException, Exception, Object, VM};
    /// # VM::init();
    ///
    /// assert_eq!(
    ///   AnyException::new("StandardError", Some("something went wrong")).exception(None),
    ///   AnyException::new("StandardError", Some("something went wrong"))
    /// );
    /// ```
    fn exception(&self, string: Option<&str>) -> Self {
        let string = string.map(|s| RString::new(s).value());

        Self::from(vm::call_method(self.value(), "exception", util::option_to_slice(&string)))
    }

    /// Returns any backtrace associated with the exception. The
    /// backtrace is an array of strings, each containing either
    /// “filename:lineNo: in `method''' or “filename:lineNo.''
    ///
    /// # Examples
    /// ```
    /// use rutie::{AnyException, Exception, Object, VM, RString};
    /// # VM::init();
    ///
    /// let x = AnyException::new("StandardError", Some("something went wrong"));
    ///
    /// assert!(x.backtrace().is_none());
    /// ```
    fn backtrace(&self) -> Option<Array> {
        let result = vm::call_method(self.value(), "backtrace", &[]);

        if result.is_nil() {
            return None;
        }

        Some(Array::from(result))
    }

    /// Returns any backtrace associated with the exception. This
    /// method is similar to #backtrace, but the backtrace is an
    /// array of Thread::Backtrace::Location.
    ///
    /// Now, this method is not affected by #set_backtrace.
    ///
    /// # Examples
    /// ```
    /// use rutie::{AnyException, Exception, Object, VM, RString};
    /// # VM::init();
    ///
    /// let x = AnyException::new("StandardError", Some("something went wrong"));
    ///
    /// assert!(x.backtrace_locations().is_none());
    /// ```
    fn backtrace_locations(&self) -> Option<Array> {
        let result = vm::call_method(self.value(), "backtrace_locations", &[]);

        if result.is_nil() {
            return None;
        }

        Some(Array::from(result))
    }

    /// Returns the previous exception at the time this
    /// exception was raised. This is useful for wrapping exceptions
    /// and retaining the original exception information.
    ///
    /// # Examples
    /// ```
    /// use rutie::{AnyException, Exception, Object, VM, RString};
    /// # VM::init();
    ///
    /// let x = AnyException::new("StandardError", Some("something went wrong"));
    ///
    /// assert!(x.cause().is_none());
    /// ```
    fn cause(&self) -> Option<Self> {
        let result = vm::call_method(self.value(), "cause", &[]);

        if result.is_nil() {
            return None;
        }

        Some(Self::from(result))
    }

    /// Return this exception's class name and message
    ///
    /// # Examples
    /// ```
    /// use rutie::{AnyException, Exception, Object, VM};
    /// # VM::init();
    ///
    /// assert_eq!(
    ///   AnyException::new("StandardError", Some("oops")).inspect(),
    ///   "#<StandardError: oops>"
    /// );
    /// ```
    fn inspect(&self) -> String {
        RString::from(vm::call_method(self.value(), "inspect", &[])).to_string()
    }

    /// Returns the result of invoking `exception.to_s`. Normally this
    /// returns the exception's message or name.
    ///
    /// # Examples
    /// ```
    /// use rutie::{AnyException, Exception, Object, VM};
    /// # VM::init();
    ///
    /// assert_eq!(
    ///   AnyException::new("StandardError", Some("oops")).message(),
    ///   "oops"
    /// );
    /// ```
    fn message(&self) -> String {
        RString::from(vm::call_method(self.value(), "message", &[])).to_string()
    }

    /// Sets the backtrace information associated with exc. The backtrace
    /// must be an array of String objects or a single String in the format
    /// described in #backtrace.
    ///
    /// # Examples
    /// ```
    /// use rutie::{AnyException, Exception, Object, VM, RString, Array};
    /// # VM::init();
    ///
    /// let x = AnyException::new("StandardError", Some("something went wrong"));
    ///
    /// let mut arr = Array::new();
    /// arr.push(RString::new("prog.rb:10"));
    ///
    /// x.set_backtrace(arr.to_any_object());
    ///
    /// assert_eq!(
    ///   x.backtrace().
    ///     unwrap().
    ///     pop().
    ///     try_convert_to::<RString>().
    ///     unwrap().
    ///     to_string(),
    ///   "prog.rb:10"
    /// );
    /// ```
    fn set_backtrace(&self, backtrace: AnyObject) -> Option<Array> {
        let result = vm::call_method(self.value(), "set_backtrace", &[backtrace.value()]);

        if result.is_nil() {
            return None;
        }

        Some(Array::from(result))
    }

    /// Returns exception's message (or the name of the exception if no message is set).
    ///
    /// # Examples
    /// ```
    /// use rutie::{AnyException, Exception, Object, VM};
    /// # VM::init();
    ///
    /// assert_eq!(
    ///   AnyException::new("StandardError", Some("oops")).to_s(),
    ///   "oops"
    /// );
    /// ```
    fn to_s(&self) -> String {
        RString::from(vm::call_method(self.value(), "to_s", &[])).to_string()
    }
}
