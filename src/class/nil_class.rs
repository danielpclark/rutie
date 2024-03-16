use std::{convert::From, default::Default};

use crate::{
    binding::global::RubySpecialConsts,
    types::{InternalValue, Value, ValueType},
    AnyObject, Object, VerifiedObject,
};

/// `NilClass`
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct NilClass {
    value: Value,
}

impl NilClass {
    /// Creates a new instance of `NilClass` (`nil`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{NilClass, Object, VM};
    /// # VM::init();
    ///
    /// assert!(NilClass::new().is_nil());
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// nil.nil? == true
    /// ```
    pub fn new() -> Self {
        Self::from(Value::from(RubySpecialConsts::Nil as InternalValue))
    }
}

impl Default for NilClass {
    fn default() -> Self {
        NilClass::new()
    }
}

impl From<Value> for NilClass {
    fn from(value: Value) -> Self {
        NilClass { value }
    }
}

impl From<NilClass> for Value {
    fn from(val: NilClass) -> Self {
        val.value
    }
}

impl From<NilClass> for AnyObject {
    fn from(val: NilClass) -> Self {
        AnyObject::from(val.value)
    }
}

impl Object for NilClass {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for NilClass {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.value().ty() == ValueType::Nil
    }

    fn error_message() -> &'static str {
        "Error converting to NilClass"
    }
}

impl PartialEq for NilClass {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}
