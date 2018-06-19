use ::{AnyObject, Object, RString, Array, Class};
use binding::util as binding_util;

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
        let arguments = msg.map(|s| vec![RString::new(s).value()]);

        Self::from(binding_util::call_method(class.value(), "new", arguments))
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
    ///   AnyException::new("StandardError", Some("something went wrong")).to_s(),
    ///   "something went wrong"
    /// );
    /// ```
    fn exception(&self, string: Option<&str>) -> Self {
        let arguments = string.map(|s| vec![RString::new(s).value()]);

        Self::from(binding_util::call_method(self.value(), "exception", arguments))
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
        let result = binding_util::call_method(self.value(), "backtrace", None);

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
        let result = binding_util::call_method(self.value(), "backtrace_locations", None);

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
        let result = binding_util::call_method(self.value(), "cause", None);

        if result.is_nil() {
            return None;
        }

        Some(Self::from(result))
    }

    // TODO: calling `full_string` panics in Rust
    // /// Returns formatted string of exception. The returned string is
    // /// formatted using the same format that Ruby uses when printing an
    // /// uncaught exceptions to stderr. So it may differ by `$stderr.tty?`
    // /// at the timing of a call.
    // ///
    // /// # Examples
    // /// ```
    // /// use rutie::{AnyException, Exception, Object, VM};
    // /// # VM::init();
    // ///
    // /// assert_eq!(
    // ///   AnyException::new("StandardError", Some("something went wrong")).full_string(),
    // ///   "StandardError: something went wrong"
    // /// );
    // /// ```
    // fn full_string(&self) -> String {
    //     RString::from(binding_util::call_method(self.value(), "full_string", None)).to_string()
    // }

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
        RString::from(binding_util::call_method(self.value(), "inspect", None)).to_string()
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
        RString::from(binding_util::call_method(self.value(), "message", None)).to_string()
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
    /// x.set_backtrace(RString::new("prog.rb:10").to_any_object());
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
        let result = binding_util::call_method(self.value(), "set_backtrace", Some(vec![backtrace.value()])); 

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
        RString::from(binding_util::call_method(self.value(), "to_s", None)).to_string()
    }
}
