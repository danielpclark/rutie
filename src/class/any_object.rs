use crate::{
    types::{InternalValue, Value},
    Object, VerifiedObject,
};
use std::{borrow::Borrow, convert::AsRef, ops::Deref};

/// Representation of any Ruby object while its type is unknown
///
/// As Ruby is a dynamically typed language, at some points Rutie does not know the exact Ruby type
/// of the object, for example:
///
/// - Retrieving an object from array;
///
/// - Retrieving an object from hash;
///
/// - Receiving arguments to a method;
///
/// - Initializing a new instance of a non-built-in class.
///
/// In these cases you should cast `AnyObject` to the required type.
///
/// # Examples
///
/// ### Retrieving an object from `Array`
///
/// ```
/// use rutie::{Array, Fixnum, Object, VM};
/// # VM::init();
///
/// let array = Array::new().push(Fixnum::new(1));
/// let value = array.at(0).try_convert_to::<Fixnum>(); // `Array::at()` returns `AnyObject`
///
/// assert_eq!(value, Ok(Fixnum::new(1)));
/// ```
///
/// ### Retrieving an object from `Hash`
///
/// ```
/// use rutie::{Fixnum, Hash, Object, Symbol, VM};
/// # VM::init();
///
/// let mut hash = Hash::new();
///
/// hash.store(Symbol::new("key"), Fixnum::new(1));
///
/// // `Hash::at()` returns `AnyObject`
/// let value = hash.at(&Symbol::new("key")).try_convert_to::<Fixnum>();
///
/// assert_eq!(value, Ok(Fixnum::new(1)));
/// ```
///
/// You can find more examples in `Class`, `Object` and `VerifiedObject` documentation.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct AnyObject {
    value: Value,
}

impl From<Value> for AnyObject {
    fn from(value: Value) -> Self {
        AnyObject { value }
    }
}

impl From<InternalValue> for AnyObject {
    fn from(value: InternalValue) -> Self {
        AnyObject {
            value: Value::from(value),
        }
    }
}

impl Into<Value> for AnyObject {
    fn into(self) -> Value {
        self.value
    }
}

impl Borrow<Value> for AnyObject {
    fn borrow(&self) -> &Value {
        &self.value
    }
}

impl AsRef<Value> for AnyObject {
    fn as_ref(&self) -> &Value {
        &self.value
    }
}

impl AsRef<AnyObject> for AnyObject {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T: Object> From<&T> for AnyObject {
    fn from(value: &T) -> Self {
        value.to_any_object()
    }
}

impl Object for AnyObject {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl Deref for AnyObject {
    type Target = Value;

    fn deref(&self) -> &Value {
        &self.value
    }
}

// Any object can be safely converted to `AnyObject` :)
impl VerifiedObject for AnyObject {
    fn is_correct_type<T: Object>(_: &T) -> bool {
        true
    }

    fn error_message() -> &'static str {
        unreachable!()
    }
}

impl PartialEq for AnyObject {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}
