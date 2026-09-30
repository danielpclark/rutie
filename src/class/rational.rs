use std::convert::From;

use crate::{
    binding::{float, numeric, vm},
    types::{Value, ValueType},
    AnyException, AnyObject, Integer, Object, VerifiedObject,
};

/// `Rational`, an exact fraction of two integers.
#[derive(Debug)]
#[repr(C)]
pub struct Rational {
    value: Value,
}

impl Rational {
    /// Creates the fraction `numerator / denominator` in lowest terms, or
    /// returns the `ZeroDivisionError` when `denominator` is zero
    /// (`rb_rational_new`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Rational, VM};
    /// # VM::init();
    ///
    /// let half = Rational::new(3, 6).unwrap();
    ///
    /// assert_eq!(half.numerator().to_i64(), 1);
    /// assert_eq!(half.denominator().to_i64(), 2);
    /// assert!(Rational::new(1, 0).is_err());
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// Rational(3, 6) # => (1/2)
    /// ```
    pub fn new(numerator: i64, denominator: i64) -> Result<Self, AnyException> {
        Self::from_integers(&Integer::new(numerator), &Integer::new(denominator))
    }

    /// Like [`Rational::new`](#method.new), for integers of any size.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, Rational, VM};
    /// # VM::init();
    ///
    /// let tiny = Rational::from_integers(&Integer::new(1), &Integer::from(u128::MAX)).unwrap();
    ///
    /// assert_eq!(tiny.denominator().to_u128(), Some(u128::MAX));
    /// ```
    pub fn from_integers(numerator: &Integer, denominator: &Integer) -> Result<Self, AnyException> {
        let (numerator, denominator) = (numerator.value(), denominator.value());

        vm::protect_value(|| numeric::rational_new(numerator, denominator))
            .map(Rational::from)
            .map_err(AnyException::from)
    }

    /// Converts `object` to a `Rational` the way Ruby's `Rational(object)`
    /// (`rb_Rational`) does, parsing strings such as `"3/4"` or `"0.75"`.
    /// Returns the exception when it cannot be converted.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Float, Object, RString, Rational, VM};
    /// # VM::init();
    ///
    /// let parsed = Rational::convert(&RString::new_utf8("3/4")).unwrap();
    ///
    /// assert_eq!(parsed.to_f64(), 0.75);
    /// assert_eq!(Rational::convert(&Float::new(0.5)).unwrap().denominator().to_i64(), 2);
    /// assert!(Rational::convert(&RString::new_utf8("x")).is_err());
    /// ```
    pub fn convert<T: Object>(object: &T) -> Result<Self, AnyException> {
        let object = object.value();

        vm::protect_value(|| numeric::to_rational(object))
            .map(Rational::from)
            .map_err(AnyException::from)
    }

    /// Returns the numerator, in lowest terms (`rb_rational_num`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Rational, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Rational::new(-4, 6).unwrap().numerator().to_i64(), -2);
    /// ```
    pub fn numerator(&self) -> Integer {
        Integer::from(numeric::rational_numerator(self.value()))
    }

    /// Returns the denominator, in lowest terms and always positive
    /// (`rb_rational_den`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Rational, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Rational::new(4, -6).unwrap().denominator().to_i64(), 3);
    /// ```
    pub fn denominator(&self) -> Integer {
        Integer::from(numeric::rational_denominator(self.value()))
    }

    /// Returns the nearest `f64` (Ruby's `to_f`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Rational, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Rational::new(1, 4).unwrap().to_f64(), 0.25);
    /// ```
    pub fn to_f64(&self) -> f64 {
        float::num_to_float(self.value())
    }
}

impl From<Value> for Rational {
    fn from(value: Value) -> Self {
        Rational { value }
    }
}

impl Into<Value> for Rational {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Rational {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for Rational {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Rational {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.value().ty() == ValueType::Rational
    }

    fn error_message() -> &'static str {
        "Error converting to Rational"
    }
}

impl PartialEq for Rational {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Float, Integer, Object, RString, Rational, VM};

    #[test]
    fn test_rational() {
        crate::on_ruby_thread(|| {
            let third = Rational::new(2, 6).unwrap();
            assert_eq!(third.numerator().to_i64(), 1);
            assert_eq!(third.denominator().to_i64(), 3);
            assert_eq!(third, Rational::convert(&RString::new_utf8("1/3")).unwrap());

            let from_ruby = VM::eval("Rational(5, 10)")
                .unwrap()
                .try_convert_to::<Rational>()
                .unwrap();
            assert_eq!(from_ruby.to_f64(), 0.5);

            assert!(VM::eval("1.5")
                .unwrap()
                .try_convert_to::<Rational>()
                .is_err());
            assert!(Rational::from_integers(&Integer::new(1), &Integer::new(0)).is_err());
            assert_eq!(
                Float::new(0.75).rationalize().unwrap(),
                Rational::new(3, 4).unwrap()
            );
        });
    }
}
