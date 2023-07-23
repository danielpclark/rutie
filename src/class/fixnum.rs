use std::convert::From;

use crate::{
    binding::fixnum,
    types::{Value, ValueType},
    AnyObject, Object, VerifiedObject,
};

/// `Fixnum`
#[derive(Debug)]
#[repr(C)]
pub struct Fixnum {
    value: Value,
}

impl Fixnum {
    /// Creates a new `Fixnum`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, VM};
    /// # VM::init();
    ///
    /// let fixnum = Fixnum::new(1);
    ///
    /// assert_eq!(fixnum.to_i64(), 1);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// 1 == 1
    /// ```
    pub fn new(num: i64) -> Self {
        Self::from(fixnum::i64_to_num(num))
    }

    /// Retrieves an `i64` value from `Fixnum`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, VM};
    /// # VM::init();
    ///
    /// let fixnum = Fixnum::new(1);
    ///
    /// assert_eq!(fixnum.to_i64(), 1);
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

    /// Retrieves an `u64` value from `Fixnum`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, VM};
    /// # VM::init();
    ///
    /// let fixnum = Fixnum::new(1);
    ///
    /// assert_eq!(fixnum.to_u64(), 1);
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

    /// Retrieves an `i32` value from `Fixnum`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, VM};
    /// # VM::init();
    ///
    /// let fixnum = Fixnum::new(1);
    ///
    /// assert_eq!(fixnum.to_i32(), 1);
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

    /// Retrieves a `u32` value from `Fixnum`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, VM};
    /// # VM::init();
    ///
    /// let fixnum = Fixnum::new(1);
    ///
    /// assert_eq!(fixnum.to_u32(), 1);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// 1 == 1
    /// ```
    pub fn to_u32(&self) -> u32 {
        fixnum::num_to_u32(self.value())
    }
}

impl From<Value> for Fixnum {
    fn from(value: Value) -> Self {
        Fixnum { value }
    }
}

impl From<Fixnum> for Value {
    fn from(val: Fixnum) -> Self {
        val.value
    }
}

impl From<Fixnum> for AnyObject {
    fn from(val: Fixnum) -> Self {
        AnyObject::from(val.value)
    }
}

impl Object for Fixnum {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Fixnum {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.value().ty() == ValueType::Fixnum
    }

    fn error_message() -> &'static str {
        "Error converting to Fixnum"
    }
}

impl PartialEq for Fixnum {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}
