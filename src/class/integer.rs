use std::convert::From;

use binding::fixnum;
use types::{Value, ValueType};

use {Object, VerifiedObject, Fixnum, AnyObject};

/// `Integer`
#[derive(Debug)]
pub struct Integer {
    value: Value,
}

impl Integer {
    /// Creates a new `Integer`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// let integer = Integer::new(1);
    ///
    /// assert_eq!(integer.to_i64(), 1);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// 1 == 1
    /// ```
    pub fn new(num: i64) -> Self {
        Self::from(num)
    }

    /// Retrieves an `i64` value from `Integer`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// let integer = Integer::new(1);
    ///
    /// assert_eq!(integer.to_i64(), 1);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// 1 == 1
    /// ```
    pub fn to_i64(&self) -> i64 {
        fixnum::num_to_i64(self.value())
    }

    /// Retrieves an `u64` value from `Integer`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// let integer = Integer::new(1);
    ///
    /// assert_eq!(integer.to_u64(), 1);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// 1 == 1
    /// ```
    pub fn to_u64(&self) -> u64 {
        fixnum::num_to_u64(self.value())
    }

    /// Retrieves an `i32` value from `Integer`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// let integer = Integer::new(1);
    ///
    /// assert_eq!(integer.to_i32(), 1);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// 1 == 1
    /// ```
    pub fn to_i32(&self) -> i32 {
        fixnum::num_to_i32(self.value())
    }
}

impl From<Value> for Integer {
    fn from(value: Value) -> Self {
        Integer { value: value }
    }
}

impl From<i64> for Integer {
    fn from(num: i64) -> Self {
        Integer { value: fixnum::i64_to_num(num) }
    }
}

impl Into<i64> for Integer {
    fn into(self) -> i64 {
        fixnum::num_to_i64(self.value())
    }
}

impl From<u64> for Integer {
    fn from(num: u64) -> Self {
        Integer { value: fixnum::u64_to_num(num) }
    }
}

impl Into<u64> for Integer {
    fn into(self) -> u64 {
        fixnum::num_to_u64(self.value())
    }
}

impl Into<i32> for Integer {
    fn into(self) -> i32 {
        fixnum::num_to_i32(self.value())
    }
}

impl From<Fixnum> for Integer {
    fn from(num: Fixnum) -> Self {
        Integer { value: num.value() }
    }
}

impl Into<Value> for Integer {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Integer {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for Integer {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Integer {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.value().ty() == ValueType::Fixnum
    }

    fn error_message() -> &'static str {
        "Error converting to Integer"
    }
}

impl PartialEq for Integer {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}
