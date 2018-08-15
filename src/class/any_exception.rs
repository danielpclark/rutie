use ::{Object, VerifiedObject, Exception, NilClass, AnyObject, Class, TryConvert};
use ::types::{Value, ValueType};
use std::fmt::{Display, Formatter};
use std::fmt;

pub struct AnyException {
    value: Value
}

impl From<Value> for AnyException {
    fn from(value: Value) -> Self {
        AnyException { value: value }
    }
}

impl Object for AnyException {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl Exception for AnyException {}

impl TryConvert<AnyObject> for AnyException {
    type Nil = NilClass;

    fn try_convert(obj: AnyObject) -> Result<Self, NilClass> {
        obj.try_convert_to::<AnyException>().map_err(|_| NilClass::new() )
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
