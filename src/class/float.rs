use std::convert::From;

use crate::{
    binding::float,
    types::{Value, ValueType},
    AnyException, AnyObject, Object, VerifiedObject,
};

/// `Float`
#[derive(Debug)]
#[repr(C)]
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

impl From<Float> for Value {
    fn from(val: Float) -> Self {
        val.value
    }
}

impl From<Float> for AnyObject {
    fn from(val: Float) -> Self {
        AnyObject::from(val.value)
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
