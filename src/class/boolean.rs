use std::convert::From;

use crate::{
    types::{InternalValue, Value},
    util, AnyObject, Object, VerifiedObject,
};

/// `TrueClass` and `FalseClass`
#[derive(Debug)]
#[repr(C)]
pub struct Boolean {
    value: Value,
}

impl Boolean {
    /// Creates a new instance boolean value from `bool`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Boolean, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Boolean::new(true).to_bool(), true);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// true == true
    /// ```
    pub fn new(state: bool) -> Self {
        Self::from(util::bool_to_value(state))
    }

    /// Retrieves a `bool` value from `Boolean`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Boolean, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Boolean::new(true).to_bool(), true);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// true == true
    /// ```
    pub fn to_bool(&self) -> bool {
        self.value().is_true()
    }
}

impl From<Value> for Boolean {
    fn from(value: Value) -> Self {
        Boolean { value }
    }
}

impl From<InternalValue> for Boolean {
    fn from(internal_value: InternalValue) -> Self {
        Boolean {
            value: Value::from(internal_value),
        }
    }
}

impl Into<Value> for Boolean {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Boolean {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for Boolean {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Boolean {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        let value = object.value();

        value.is_true() || value.is_false()
    }

    fn error_message() -> &'static str {
        "Error converting to Boolean"
    }
}

impl PartialEq for Boolean {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}
