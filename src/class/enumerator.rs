use std::convert::From;
use std::default::Default;
use std::iter::{FromIterator, IntoIterator, Iterator};

use types::{Value, ValueType};

use {AnyObject, Object, VerifiedObject, Class, Fixnum, Array};

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

impl Iterator for Enumerator {
    type Item = AnyObject;

    fn next(&mut self) -> Option<AnyObject> {
        self.protect_send("next", None).ok()
    }
}

pub trait IteratorValues {
    type Item;
    fn next_values(&mut self) -> Option<Self::Item>;
}

impl IteratorValues for Enumerator {
    type Item = Array;

    fn next_values(&mut self) -> Option<Array> {
        self.protect_send("next_values", None).
            map(|v| Array::from(v.value()) ).ok()
    }
}

impl ExactSizeIterator for Enumerator {
    fn len(&self) -> usize {
        self.count() as usize
    }
}
