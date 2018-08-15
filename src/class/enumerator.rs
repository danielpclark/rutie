use std::convert::From;
use std::default::Default;

use types::{Value, ValueType};

use {
    AnyObject,
    AnyException,
    Array,
    Class,
    Fixnum,
    Object,
    VerifiedObject,
};

/// `Enumerator`
#[derive(Debug, PartialEq)]
pub struct Enumerator {
    value: Value,
}

impl Enumerator {
    pub fn count(&self) -> i64 {
        self.send("count", None).try_convert_to::<Fixnum>().unwrap().to_i64()
    }
}

impl From<Value> for Enumerator {
    fn from(value: Value) -> Self {
        Enumerator { value: value }
    }
}

impl Object for Enumerator {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Enumerator {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        Class::from_existing("Enumerator").case_equals(object)
    }

    fn error_message() -> &'static str {
        "Error converting to Enumerator"
    }
}

pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    fn next_values(&mut self) -> Option<Array>;
    fn peek(&self) -> Option<Self::Item>;
    fn peek_values(&self) -> Option<Array>;
    fn rewind(&mut self) -> &mut Self;
    fn feed(&mut self, object: AnyObject) -> Result<(), AnyException>;
}

impl Iterator for Enumerator {
    type Item = AnyObject;

    fn next(&mut self) -> Option<AnyObject> {
        self.protect_send("next", None).ok()
    }

    fn next_values(&mut self) -> Option<Array> {
        self.protect_send("next_values", None).
            map(|v| Array::from(v.value()) ).ok()
    }

    fn peek(&self) -> Option<AnyObject> {
        self.protect_send("peek", None).ok()
    }

    fn peek_values(&self) -> Option<Array> {
        self.protect_send("peek_values", None).
            map(|v| Array::from(v.value()) ).ok()
    }

    fn rewind(&mut self) -> &mut Self {
        self.send("rewind", None);
        self
    }

    fn feed(&mut self, object: AnyObject) -> Result<(), AnyException> {
        self.protect_send("feed", Some(&[object])).map(|_|())
    }
}
