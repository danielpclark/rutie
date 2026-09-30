use std::convert::From;

use crate::{
    binding::float,
    types::{Value, ValueType},
    AnyException, AnyObject, Object, Rational, VerifiedObject, VM,
};

/// `Float`
#[derive(Debug)]
#[repr(C)]
pub struct Float {
    value: Value,
}

impl Float {
    /// Returns the simplest `Rational` that rounds to this float (Ruby's
    /// `rationalize`, `rb_flt_rationalize`), or the `FloatDomainError` for
    /// `NaN` and infinities.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Float, VM};
    /// # VM::init();
    ///
    /// let third = Float::new(0.333333333333333333).rationalize().unwrap();
    ///
    /// assert_eq!(third.numerator().to_i64(), 1);
    /// assert_eq!(third.denominator().to_i64(), 3);
    /// assert!(Float::new(f64::NAN).rationalize().is_err());
    /// ```
    pub fn rationalize(&self) -> Result<Rational, AnyException> {
        let float = self.value();

        crate::binding::vm::protect_value(|| crate::binding::numeric::float_rationalize(float))
            .map(Rational::from)
            .map_err(AnyException::from)
    }

    /// Converts `object` to a `Float` the way Ruby's `Float(object)`
    /// (`Kernel#Float`, `rb_Float`) does, parsing strings strictly and calling `to_f`. Returns the exception when
    /// it cannot be converted.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Float, Object, RString, VM};
    /// # VM::init();
    ///
    /// let parsed = Float::convert(&RString::new_utf8("2.5")).unwrap();
    /// assert_eq!(parsed.to_f64(), 2.5);
    ///
    /// let widened = Float::convert(&Fixnum::new(3)).unwrap();
    /// assert_eq!(widened.to_f64(), 3.0);
    ///
    /// assert!(Float::convert(&RString::new_utf8("2.5x")).is_err());
    /// ```
    pub fn convert<T: Object>(object: &T) -> Result<Self, AnyException> {
        let object = object.value();

        crate::binding::vm::protect_value(|| crate::binding::object::to_float(object))
            .map(Self::from)
            .map_err(AnyException::from)
    }

    /// Creates a new `Float`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Float, VM};
    /// # VM::init();
    ///
    /// let float = Float::new(1.23);
    ///
    /// assert_eq!(float.to_f64(), 1.23);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// 1.23 == 1.23
    /// ```
    pub fn new(num: f64) -> Self {
        Self::from(float::float_to_num(num))
    }

    /// Retrieves an `f64` value from `Float`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Float, VM};
    /// # VM::init();
    ///
    /// let float = Float::new(1.23);
    ///
    /// assert_eq!(float.to_f64(), 1.23);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// 1.23 == 1.23
    /// ```
    pub fn to_f64(&self) -> f64 {
        float::num_to_float(self.value())
    }

    /// Cast any object to a `Float` implicitly, otherwise
    /// returns an `AnyException`
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, Float, Object, VM};
    /// # VM::init();
    ///
    /// let integer = Integer::new(3);
    ///
    /// assert_eq!(Float::implicit_to_f(integer), Ok(Float::new(3.0)));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// Float(3) == 3.0
    /// ```
    pub fn implicit_to_f(object: impl Object) -> Result<Float, AnyException> {
        float::implicit_to_f(object.value())
    }
}

impl From<Value> for Float {
    fn from(value: Value) -> Self {
        Float { value }
    }
}

impl Into<Value> for Float {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Float {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for Float {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Float {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.value().ty() == ValueType::Float
    }

    fn error_message() -> &'static str {
        "Error converting to Float"
    }
}

impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{AnyObject, Fixnum, Float, Object, RString, VerifiedObject, VM};

    #[test]
    fn test_float() {
        crate::on_ruby_thread(|| {
            let float = Float::new(1.5);
            assert_eq!(float.to_f64(), 1.5);

            let any: AnyObject = Float::new(1.5).into();
            assert!(Float::is_correct_type(&any));
            assert!(!Float::is_correct_type(&Fixnum::new(1)));
            assert_eq!(any.try_convert_to::<Float>(), Ok(Float::new(1.5)));
            assert_ne!(Float::new(1.5), Float::new(2.5));

            // `implicit_to_f` accepts numerics (`rb_to_float`), not strings.
            assert_eq!(Float::implicit_to_f(Fixnum::new(3)).unwrap().to_f64(), 3.0);
            assert!(Float::implicit_to_f(RString::new_utf8("3")).is_err());

            // `convert` is Kernel#Float, which also parses strings.
            assert_eq!(
                Float::convert(&RString::new_utf8("2.25")).unwrap().to_f64(),
                2.25
            );
            assert!(Float::convert(&RString::new_utf8("nope")).is_err());

            let rational = Float::new(0.5).rationalize().unwrap();
            assert_eq!(rational.inspect_object().to_str(), "(1/2)");

            // Infinity cannot be a Rational.
            assert!(Float::new(f64::INFINITY).rationalize().is_err());

            let parsed = VM::eval("0.1 + 0.2")
                .unwrap()
                .try_convert_to::<Float>()
                .unwrap();
            assert!((parsed.to_f64() - 0.3).abs() < 1e-9);
        });
    }
}
