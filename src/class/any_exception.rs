use crate::{
    types::Value, AnyObject, Class, Exception, NilClass, Object, TryConvert, VerifiedObject,
};
use std::{
    borrow::Borrow,
    fmt,
    fmt::{Display, Formatter},
    ops::Deref,
};

pub struct AnyException {
    value: Value,
}

impl From<Value> for AnyException {
    fn from(value: Value) -> Self {
        AnyException { value }
    }
}

impl Into<Value> for AnyException {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for AnyException {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Borrow<Value> for AnyException {
    fn borrow(&self) -> &Value {
        &self.value
    }
}

impl AsRef<Value> for AnyException {
    fn as_ref(&self) -> &Value {
        &self.value
    }
}

impl AsRef<AnyException> for AnyException {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Object for AnyException {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl Deref for AnyException {
    type Target = Value;

    fn deref(&self) -> &Value {
        &self.value
    }
}

impl Exception for AnyException {}

impl TryConvert<AnyObject> for AnyException {
    type Nil = NilClass;

    fn try_convert(obj: AnyObject) -> Result<Self, NilClass> {
        obj.try_convert_to::<AnyException>()
            .map_err(|_| NilClass::new())
    }
}

impl VerifiedObject for AnyException {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        Class::from_existing("Exception").case_equals(object)
    }

    fn error_message() -> &'static str {
        "Error converting to AnyException"
    }
}

impl Display for AnyException {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.inspect())
    }
}

impl fmt::Debug for AnyException {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inspect())
    }
}

impl PartialEq for AnyException {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}
