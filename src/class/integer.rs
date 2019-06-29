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

impl From<i32> for Integer {
    fn from(num: i32) -> Self {
        Integer { value: fixnum::i32_to_num(num) }
    }
}

impl Into<i32> for Integer {
    fn into(self) -> i32 {
        fixnum::num_to_i32(self.value())
    }
}

impl From<u32> for Integer {
    fn from(num: u32) -> Self {
        Integer { value: fixnum::u32_to_num(num) }
    }
}

impl Into<u32> for Integer {
    fn into(self) -> u32 {
        fixnum::num_to_u32(self.value())
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
        let ty = object.value().ty();
        ty == ValueType::Fixnum || ty == ValueType::Bignum
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

#[cfg(test)]
mod tests {
    use super::super::super::{LOCK_FOR_TEST, AnyException, Object, Integer, VM, NilClass};
    use super::super::super::types::Value;

    #[test]
    fn test_i32() {
        let _guard = LOCK_FOR_TEST.write().unwrap();
        VM::init();

        let nil = NilClass::new();

        let num = str_to_num("1").unwrap();
        assert_eq!(1, num.to_i32());

        let num = str_to_num("-1").unwrap();
        assert_eq!(-1, num.to_i32());

        let num = str_to_num("2 ** 31 - 1").unwrap();
        assert_eq!(::std::i32::MAX, num.to_i32());

        let num = str_to_num("2 ** 31").unwrap();
        let result = VM::protect(|| { num.to_i32(); nil.into() });
        assert!(result.is_err());

        let num = str_to_num("-1 * 2 ** 31").unwrap();
        assert_eq!(::std::i32::MIN, num.to_i32());

        let num = str_to_num("-1 * 2 ** 31 - 1").unwrap();
        let result = VM::protect(|| { num.to_i32(); nil.into() });
        assert!(result.is_err());
    }

    #[test]
    fn test_i64() {
        let _guard = LOCK_FOR_TEST.write().unwrap();
        VM::init();

        let nil = NilClass::new();

        let num = str_to_num("2 ** 63 - 1").unwrap();
        assert_eq!(::std::i64::MAX, num.to_i64());

        let num = str_to_num("2 ** 63").unwrap();
        let result = VM::protect(|| { num.to_i64(); nil.into() });
        assert!(result.is_err());

        let num = str_to_num("-1 * 2 ** 63").unwrap();
        assert_eq!(::std::i64::MIN, num.to_i64());

        let num = str_to_num("-1 * 2 ** 63 - 1").unwrap();
        let result = VM::protect(|| { num.to_i64(); nil.into() });
        assert!(result.is_err());
    }

    #[test]
    fn test_u64() {
        let _guard = LOCK_FOR_TEST.write().unwrap();
        VM::init();

        let nil = NilClass::new();

        let num = str_to_num("2 ** 64 - 1").unwrap();
        assert_eq!(::std::u64::MAX, num.to_u64());

        let num = str_to_num("2 ** 64").unwrap();
        let result = VM::protect(|| { num.to_u64(); nil.into() });
        assert!(result.is_err());

        let num = str_to_num("0").unwrap();
        assert_eq!(::std::u64::MIN, num.to_u64());

        // // Current Ruby implementation does not raise an exception
        // let num = str_to_num("-1").unwrap();
        // let result = VM::protect(|| { num.to_u64(); nil.into() });
        // assert!(result.is_err());
    }

    fn str_to_num(code: &str) -> Result<Integer, AnyException> {
        VM::eval(code).and_then(|x| x.try_convert_to::<Integer>())
    }
}
