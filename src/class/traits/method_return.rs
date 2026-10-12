use crate::{AnyException, Object};

/// The return type of a method written with `methods!` or `unsafe_methods!`:
/// any `Object`, or a `Result` whose `Err` is raised as a Ruby exception.
///
/// Raising from Rust (`VM::raise`, `VM::raise_ex`, or a Ruby method called
/// with `send` that raises) jumps straight back into Ruby with `longjmp`,
/// so the destructors of the Rust values in scope never run: a `String`,
/// `Vec` or `Box` leaks, and a lock guard, file or socket is never released
/// (#159). A method returning `Err(exception)` instead raises `exception`
/// after its body has returned and dropped its values.
///
/// # Examples
///
/// ```
/// #[macro_use]
/// extern crate rutie;
///
/// use rutie::{AnyException, Class, Exception, Fixnum, Object, RString, VM};
///
/// class!(Parser);
///
/// methods!(
///     Parser,
///     rtself,
///
///     fn parse(input: RString) -> Result<Fixnum, AnyException> {
///         // An argument of the wrong type is raised as its `TypeError`.
///         let input = input?.to_string();
///
///         match input.trim().parse::<i64>() {
///             Ok(number) => Ok(Fixnum::new(number)),
///             // `input` is dropped before the exception is raised.
///             Err(error) => Err(AnyException::new("ArgumentError", Some(&error.to_string()))),
///         }
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     Class::new("Parser", None).define(|klass| {
///         klass.def("parse", parse);
///     });
///
///     let number = VM::eval("Parser.new.parse(' 42 ')").unwrap();
///     assert_eq!(number.try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));
///
///     let error = VM::eval("Parser.new.parse('forty-two')").unwrap_err();
///     assert!(Class::argument_error().case_equals(&error));
///     assert_eq!(error.message(), "invalid digit found in string");
///
///     let error = VM::eval("Parser.new.parse(42)").unwrap_err();
///     assert!(Class::type_error().case_equals(&error));
/// }
/// ```
pub trait MethodReturn {
    /// The `Object` handed back to Ruby.
    type Output: Object;

    /// The object to return to Ruby, or the exception to raise.
    fn into_method_result(self) -> Result<Self::Output, AnyException>;
}

impl<T: Object> MethodReturn for T {
    type Output = T;

    #[inline]
    fn into_method_result(self) -> Result<T, AnyException> {
        Ok(self)
    }
}

impl<T: Object, E: Into<AnyException>> MethodReturn for Result<T, E> {
    type Output = T;

    #[inline]
    fn into_method_result(self) -> Result<T, AnyException> {
        self.map_err(Into::into)
    }
}
