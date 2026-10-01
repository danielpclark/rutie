use std::convert::From;

use crate::{
    binding::{fixnum, vm},
    types::{Value, ValueType},
    AnyException, AnyObject, NilClass, Object, VerifiedObject,
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

    /// Returns the number as an `i16`, or the `RangeError` when it does
    /// not fit (`rb_fix2short`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Fixnum::new(-300).try_to_i16(), Ok(-300));
    /// assert!(Fixnum::new(40_000).try_to_i16().is_err());
    /// ```
    pub fn try_to_i16(&self) -> Result<i16, AnyException> {
        protect_conversion(self.value(), fixnum::fix_to_i16)
    }

    /// Returns the number as a `u16`, or the `RangeError` when it does not
    /// fit (`rb_fix2ushort`).
    ///
    /// Like C's conversion, a negative number down to `-32768` wraps around.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Fixnum::new(40_000).try_to_u16(), Ok(40_000));
    /// assert_eq!(Fixnum::new(-1).try_to_u16(), Ok(u16::MAX));
    /// assert!(Fixnum::new(70_000).try_to_u16().is_err());
    /// ```
    pub fn try_to_u16(&self) -> Result<u16, AnyException> {
        protect_conversion(self.value(), fixnum::fix_to_u16)
    }

    /// Returns the number as an `i32`, or the `RangeError` when it does
    /// not fit (`rb_fix2int`). Unlike [`to_i32`](#method.to_i32), the
    /// exception is returned instead of raised.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Fixnum::new(i32::MIN as i64).try_to_i32(), Ok(i32::MIN));
    /// assert!(Fixnum::new(i32::MAX as i64 + 1).try_to_i32().is_err());
    /// ```
    pub fn try_to_i32(&self) -> Result<i32, AnyException> {
        protect_conversion(self.value(), fixnum::fix_to_i32)
    }

    /// Returns the number as a `u32`, or the `RangeError` when it does not
    /// fit (`rb_fix2uint`). Unlike [`to_u32`](#method.to_u32), the
    /// exception is returned instead of raised.
    ///
    /// Like C's conversion, a negative number wraps around.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Fixnum::new(u32::MAX as i64).try_to_u32(), Ok(u32::MAX));
    /// assert_eq!(Fixnum::new(-1).try_to_u32(), Ok(u32::MAX));
    /// assert!(Fixnum::new(u32::MAX as i64 + 1).try_to_u32().is_err());
    /// ```
    pub fn try_to_u32(&self) -> Result<u32, AnyException> {
        protect_conversion(self.value(), fixnum::fix_to_u32)
    }
}

// Runs a conversion that raises `RangeError`, returning the exception.
fn protect_conversion<T: Default>(
    value: Value,
    convert: fn(Value) -> T,
) -> Result<T, AnyException> {
    let mut result = T::default();

    vm::protect_value(|| {
        result = convert(value);

        NilClass::new().value()
    })
    .map(|_| result)
    .map_err(AnyException::from)
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

    #[test]
    fn test_fixnum_checked_conversions() {
        crate::on_ruby_thread(|| {
            assert_eq!(Fixnum::new(i16::MIN as i64).try_to_i16(), Ok(i16::MIN));
            assert_eq!(Fixnum::new(i16::MAX as i64).try_to_i16(), Ok(i16::MAX));
            assert!(Fixnum::new(i16::MIN as i64 - 1).try_to_i16().is_err());
            assert!(Fixnum::new(i16::MAX as i64 + 1).try_to_i16().is_err());

            assert_eq!(Fixnum::new(u16::MAX as i64).try_to_u16(), Ok(u16::MAX));
            assert!(Fixnum::new(u16::MAX as i64 + 1).try_to_u16().is_err());

            assert_eq!(Fixnum::new(i32::MAX as i64).try_to_i32(), Ok(i32::MAX));
            assert!(Fixnum::new(i32::MIN as i64 - 1).try_to_i32().is_err());

            assert_eq!(Fixnum::new(0).try_to_u32(), Ok(0));
            assert_eq!(Fixnum::new(u32::MAX as i64).try_to_u32(), Ok(u32::MAX));
            let error = Fixnum::new(u32::MAX as i64 + 1).try_to_u32().unwrap_err();
            assert_eq!(error.class().name().unwrap().to_str(), "RangeError");

            // The error is cleared: Ruby code still runs normally.
            assert_eq!(
                VM::eval("1 + 1").unwrap().try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(2))
            );
        });
    }
}
