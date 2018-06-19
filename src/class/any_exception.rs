use ::{Object, VerifiedObject, Exception};
use ::types::{Value, ValueType};

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

impl VerifiedObject for AnyException {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.value().ty() == ValueType::Class &&
          object.respond_to("set_backtrace")
    }

    fn error_message() -> &'static str {
        "Error converting to AnyException"
    }
}
