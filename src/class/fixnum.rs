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

impl Into<Value> for Fixnum {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Fixnum {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
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

#[cfg(test)]
mod tests {
    use crate::{AnyObject, Fixnum, Float, Object, VerifiedObject, VM};

    #[test]
    fn test_fixnum() {
        crate::on_ruby_thread(|| {
            let number = Fixnum::new(-42);

            assert_eq!(number.to_i64(), -42);
            assert_eq!(number.to_i32(), -42);
            assert_eq!(Fixnum::new(42).to_u64(), 42);
            assert_eq!(Fixnum::new(42).to_u32(), 42);
            assert_eq!(Fixnum::new(i32::MAX as i64).to_i32(), i32::MAX);

            // Out of range for i32: Ruby raises RangeError.
            let result = VM::protect(|| {
                Fixnum::new(i64::from(i32::MAX) + 1).to_i32();
                crate::NilClass::new().into()
            });
            assert!(result.is_err());
            VM::clear_error_info();

            let any: AnyObject = Fixnum::new(-42).into();
            assert!(Fixnum::is_correct_type(&any));
            assert!(!Fixnum::is_correct_type(&Float::new(1.0)));
            assert_eq!(any.try_convert_to::<Fixnum>(), Ok(Fixnum::new(-42)));
            assert!(Float::new(1.0).try_convert_to::<Fixnum>().is_err());

            let sum = VM::eval("20 + 22")
                .unwrap()
                .try_convert_to::<Fixnum>()
                .unwrap();
            assert_eq!(sum, Fixnum::new(42));
            assert_ne!(sum, number);
        });
    }
}
