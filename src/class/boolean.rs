use std::convert::From;

use crate::{types::Value, util, AnyObject, Object, VerifiedObject};

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

#[cfg(test)]
mod tests {
    use crate::{AnyObject, Boolean, NilClass, Object, VerifiedObject, VM};

    #[test]
    fn test_boolean() {
        crate::on_ruby_thread(|| {
            assert!(Boolean::new(true).to_bool());
            assert!(!Boolean::new(false).to_bool());
            assert_eq!(Boolean::new(true), Boolean::new(true));
            assert_ne!(Boolean::new(true), Boolean::new(false));

            let any: AnyObject = Boolean::new(false).into();
            assert!(Boolean::is_correct_type(&any));
            assert!(!Boolean::is_correct_type(&NilClass::new()));
            assert!(NilClass::new().try_convert_to::<Boolean>().is_err());

            let result = VM::eval("1 < 2")
                .unwrap()
                .try_convert_to::<Boolean>()
                .unwrap();
            assert!(result.to_bool());
            assert!(Boolean::new(true).value().is_true());
            assert!(Boolean::new(false).value().is_false());
        });
    }
}
