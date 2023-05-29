use std::convert::From;

use crate::{binding::rproc, types::Value, AnyObject, Class, Object, VerifiedObject};

/// `Integer`
#[derive(Debug)]
pub struct Binding {
    value: Value,
}

impl Binding {
    /// Creates a new `Binding`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Binding, VM};
    /// # VM::init();
    ///
    /// let _ = Binding::new();
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// binding
    /// ```
    pub fn new() -> Self {
        Binding {
            value: rproc::binding_new(),
        }
    }
}

impl From<Value> for Binding {
    fn from(value: Value) -> Self {
        Binding { value }
    }
}

impl Into<Value> for Binding {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Binding {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for Binding {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Binding {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        Class::from_existing("Binding").case_equals(object)
    }

    fn error_message() -> &'static str {
        "Error converting to Integer"
    }
}

impl PartialEq for Binding {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}
