use std::convert::From;

use binding::float;
use types::{Value, ValueType};

use {Object, VerifiedObject, AnyObject};

/// `Float`
#[derive(Debug)]
pub struct Float {
    value: Value,
}

impl Float {
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
}

impl From<Value> for Float {
    fn from(value: Value) -> Self {
        Float { value: value }
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
        let ty = object.value().ty();
        ty == ValueType::Float
            || ty == ValueType::Fixnum
            || ty == ValueType::Bignum
            || ty == ValueType::Rational
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
    use super::super::super::types::Value;
    use super::super::super::{AnyException, Float, Object, LOCK_FOR_TEST, VM};

    #[test]
    fn test_numeric_conversion() {
        let _guard = LOCK_FOR_TEST.write().unwrap();
        VM::init();

        let fixnum = eval_to_float("-81927").unwrap();
        assert_eq!(fixnum.to_f64(), -81927.0);

        let bignum = eval_to_float("2 ** 62").unwrap();
        assert_eq!(bignum.to_f64(), 4611686018427387904.0);

        let float = eval_to_float("123456789.0").unwrap();
        assert_eq!(float.to_f64(), 123456789.0);

        let rational = eval_to_float("1/2r").unwrap();
        assert_eq!(rational.to_f64(), 0.5);

        let rational_repeating = eval_to_float("-1/3r").unwrap();
        assert_eq!(rational_repeating.to_f64(), -0.3333333333333333);

        let string = eval_to_float("'5.0'");
        assert!(string.is_err());

        VM::exit(0);
    }

    fn eval_to_float(code: &str) -> Result<Float, AnyException> {
        VM::eval(code).and_then(|x| x.try_convert_to::<Float>())
    }
}
