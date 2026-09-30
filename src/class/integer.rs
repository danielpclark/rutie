use std::{
    cmp::Ordering,
    convert::{From, TryFrom},
};

use crate::{
    binding::{fixnum, float, numeric, vm},
    types::{Value, ValueType},
    AnyException, AnyObject, Exception, Fixnum, Object, RString, VerifiedObject,
};

/// `Integer`
#[derive(Debug)]
#[repr(C)]
pub struct Integer {
    value: Value,
}

impl Integer {
    /// Converts `object` to a `Integer` the way Ruby's `Integer(object)`
    /// (`Kernel#Integer`, `rb_Integer`) does, parsing strings strictly (with `0x`/`0b`/`0o` prefixes) and calling `to_int` or `to_i`. Returns the exception when
    /// it cannot be converted.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Exception, Float, Integer, Object, RString, VM};
    /// # VM::init();
    ///
    /// let parsed = Integer::convert(&RString::new_utf8("0x1A")).unwrap();
    /// assert_eq!(parsed.to_i64(), 26);
    ///
    /// let truncated = Integer::convert(&Float::new(3.99)).unwrap();
    /// assert_eq!(truncated.to_i64(), 3);
    ///
    /// let error = Integer::convert(&RString::new_utf8("12abc")).unwrap_err();
    /// assert!(error.message().contains("invalid value for Integer"));
    /// ```
    pub fn convert<T: Object>(object: &T) -> Result<Self, AnyException> {
        let object = object.value();

        crate::binding::vm::protect_value(|| crate::binding::object::to_integer(object))
            .map(Self::from)
            .map_err(AnyException::from)
    }

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

    /// Retrieves a `u32` value from `Integer`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// let integer = Integer::new(1);
    ///
    /// assert_eq!(integer.to_u32(), 1);
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

    /// Parses `string` as an integer in `base` like Ruby's
    /// `Integer(string, base)` (`rb_str_to_inum`), returning the
    /// `ArgumentError` when it is not a valid integer.
    ///
    /// A `base` of `0` accepts the `0b`, `0o`, `0` and `0x` prefixes.
    /// Numbers of any size work.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, Object, VM};
    /// # VM::init();
    ///
    /// let big = Integer::from_str_radix("123456789012345678901234567890", 10).unwrap();
    ///
    /// assert!(big.is_bignum());
    /// assert_eq!(Integer::from_str_radix("-ff", 16).unwrap().to_i64(), -255);
    /// assert!(Integer::from_str_radix("12z", 10).is_err());
    /// ```
    pub fn from_str_radix(string: &str, base: u32) -> Result<Integer, AnyException> {
        // `rb_cstr_to_inum` passes no length, which disables prefix detection
        // for base 0; `rb_str_to_inum` (through `RString`) has the length.
        RString::new_utf8(string).parse_integer(base)
    }

    /// Returns the integer written in `base` (Ruby's `to_s(base)`,
    /// `rb_big2str`).
    ///
    /// # Panics
    ///
    /// If `base` is not between 2 and 36.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Integer::new(255).to_s_radix(16).to_str(), "ff");
    /// assert_eq!(Integer::from(u128::MAX).to_s_radix(36).to_str(), "f5lxx1zz5pnorynqglhzmsp33");
    /// ```
    pub fn to_s_radix(&self, base: u32) -> RString {
        assert!((2..=36).contains(&base), "invalid radix {}", base);

        RString::from(numeric::integer_to_s(self.value(), base))
    }

    /// Returns `true` if the integer is too big for a Fixnum and is stored
    /// as a Bignum.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// assert!(!Integer::new(1).is_bignum());
    /// assert!(Integer::from(i128::MAX).is_bignum());
    /// ```
    pub fn is_bignum(&self) -> bool {
        self.value().ty() == ValueType::Bignum
    }

    /// Returns the integer as an `i128`, or `None` if it does not fit
    /// (`rb_integer_pack`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Integer::from(i128::MIN).to_i128(), Some(i128::MIN));
    /// assert_eq!(Integer::new(-5).to_i128(), Some(-5));
    /// assert_eq!(Integer::from(u128::MAX).to_i128(), None);
    /// ```
    pub fn to_i128(&self) -> Option<i128> {
        numeric::integer_to_i128(self.value())
    }

    /// Returns the integer as a `u128`, or `None` if it is negative or does
    /// not fit (`rb_integer_pack`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Integer::from(u128::MAX).to_u128(), Some(u128::MAX));
    /// assert_eq!(Integer::new(-1).to_u128(), None);
    /// ```
    pub fn to_u128(&self) -> Option<u128> {
        numeric::integer_to_u128(self.value())
    }

    /// Returns the integer as the nearest `f64` (Ruby's `to_f`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Integer::new(3).to_f64(), 3.0);
    /// assert_eq!(Integer::from(1u128 << 100).to_f64(), 2f64.powi(100));
    /// ```
    pub fn to_f64(&self) -> f64 {
        float::num_to_float(self.value())
    }

    /// Returns `self + other` (Ruby's `+`), of any size.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// let sum = Integer::from(u64::MAX).add(&Integer::new(1));
    ///
    /// assert_eq!(sum.to_u128(), Some(u64::MAX as u128 + 1));
    /// ```
    pub fn add(&self, other: &Integer) -> Integer {
        Integer::from(vm::call_method(self.value(), "+", &[other.value()]))
    }

    /// Returns `self - other` (Ruby's `-`), of any size.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Integer::new(1).sub(&Integer::new(3)).to_i64(), -2);
    /// ```
    pub fn sub(&self, other: &Integer) -> Integer {
        Integer::from(vm::call_method(self.value(), "-", &[other.value()]))
    }

    /// Returns `self * other` (Ruby's `*`), of any size.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// let product = Integer::from(u64::MAX).mul(&Integer::from(u64::MAX));
    ///
    /// assert_eq!(product.to_u128(), Some(u64::MAX as u128 * u64::MAX as u128));
    /// ```
    pub fn mul(&self, other: &Integer) -> Integer {
        Integer::from(vm::call_method(self.value(), "*", &[other.value()]))
    }

    /// Returns `self / other` rounded towards negative infinity (Ruby's
    /// `/`), or the `ZeroDivisionError` when `other` is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Integer::new(-7).div(&Integer::new(2)).unwrap().to_i64(), -4);
    /// assert!(Integer::new(1).div(&Integer::new(0)).is_err());
    /// ```
    pub fn div(&self, other: &Integer) -> Result<Integer, AnyException> {
        let (integer, other) = (self.value(), other.value());

        vm::protect_value(|| vm::call_method(integer, "/", &[other]))
            .map(Integer::from)
            .map_err(AnyException::from)
    }

    /// Returns `self` modulo `other`, with the sign of `other` (Ruby's `%`),
    /// or the `ZeroDivisionError` when `other` is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Integer::new(-7).modulo(&Integer::new(2)).unwrap().to_i64(), 1);
    /// assert!(Integer::new(1).modulo(&Integer::new(0)).is_err());
    /// ```
    pub fn modulo(&self, other: &Integer) -> Result<Integer, AnyException> {
        let (integer, other) = (self.value(), other.value());

        vm::protect_value(|| vm::call_method(integer, "%", &[other]))
            .map(Integer::from)
            .map_err(AnyException::from)
    }

    /// Returns `self` raised to `exponent` (Ruby's `**`).
    ///
    /// The result is an `Integer`, except that Ruby gives `Infinity` (a
    /// `Float`, with a warning) when the result would be enormous.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, Object, VM};
    /// # VM::init();
    ///
    /// let power = Integer::new(2).pow(100).try_convert_to::<Integer>().unwrap();
    ///
    /// assert_eq!(power.to_u128(), Some(1 << 100));
    /// ```
    pub fn pow(&self, exponent: u32) -> AnyObject {
        let exponent = Integer::from(exponent);

        AnyObject::from(vm::call_method(self.value(), "**", &[exponent.value()]))
    }

    /// Compares two integers of any size.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// use std::cmp::Ordering;
    /// # VM::init();
    ///
    /// let big = Integer::from(u128::MAX);
    ///
    /// assert_eq!(Integer::new(1).compare(&big), Ordering::Less);
    /// assert!(big > Integer::new(0));
    /// ```
    pub fn compare(&self, other: &Integer) -> Ordering {
        let result = vm::call_method(self.value(), "<=>", &[other.value()]);

        Fixnum::from(result).to_i64().cmp(&0)
    }
}

impl From<Value> for Integer {
    fn from(value: Value) -> Self {
        Integer { value }
    }
}

impl From<i64> for Integer {
    fn from(num: i64) -> Self {
        Integer {
            value: fixnum::i64_to_num(num),
        }
    }
}

impl Into<i64> for Integer {
    fn into(self) -> i64 {
        fixnum::num_to_i64(self.value())
    }
}

impl From<u64> for Integer {
    fn from(num: u64) -> Self {
        Integer {
            value: fixnum::u64_to_num(num),
        }
    }
}

impl Into<u64> for Integer {
    fn into(self) -> u64 {
        fixnum::num_to_u64(self.value())
    }
}

impl From<i32> for Integer {
    fn from(num: i32) -> Self {
        Integer {
            value: fixnum::i32_to_num(num),
        }
    }
}

impl Into<i32> for Integer {
    fn into(self) -> i32 {
        fixnum::num_to_i32(self.value())
    }
}

impl From<u32> for Integer {
    fn from(num: u32) -> Self {
        Integer {
            value: fixnum::u32_to_num(num),
        }
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

impl PartialOrd for Integer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.compare(other))
    }
}

impl From<i128> for Integer {
    fn from(number: i128) -> Self {
        Integer::from(numeric::i128_to_integer(number))
    }
}

impl From<u128> for Integer {
    fn from(number: u128) -> Self {
        Integer::from(numeric::u128_to_integer(number))
    }
}

/// Converts an `f64` to an `Integer`, truncating towards zero (Ruby's
/// `Float#to_i`); `NaN` and infinities are a `FloatDomainError`.
///
/// # Examples
///
/// ```
/// use rutie::{Integer, VM};
/// use std::convert::TryFrom;
/// # VM::init();
///
/// assert_eq!(Integer::try_from(-2.9).unwrap().to_i64(), -2);
/// assert!(Integer::try_from(1e30).unwrap().is_bignum());
/// assert!(Integer::try_from(f64::NAN).is_err());
/// ```
impl TryFrom<f64> for Integer {
    type Error = AnyException;

    fn try_from(number: f64) -> Result<Self, Self::Error> {
        if number.is_finite() {
            Ok(Integer::from(numeric::f64_to_integer(number)))
        } else {
            let message = format!("{}", number);

            Err(AnyException::new("FloatDomainError", Some(&message)))
        }
    }
}

/// Converts an `Integer` to an `i128`; a `RangeError` if it does not fit.
///
/// # Examples
///
/// ```
/// use rutie::{Integer, VM};
/// use std::convert::TryFrom;
/// # VM::init();
///
/// assert_eq!(i128::try_from(Integer::new(-7)).unwrap(), -7);
/// assert!(i128::try_from(Integer::from(u128::MAX)).is_err());
/// ```
impl TryFrom<Integer> for i128 {
    type Error = AnyException;

    fn try_from(integer: Integer) -> Result<Self, Self::Error> {
        integer
            .to_i128()
            .ok_or_else(|| AnyException::new("RangeError", Some("integer too big for i128")))
    }
}

/// Converts an `Integer` to a `u128`; a `RangeError` if it is negative or
/// does not fit.
///
/// # Examples
///
/// ```
/// use rutie::{Integer, VM};
/// use std::convert::TryFrom;
/// # VM::init();
///
/// assert_eq!(u128::try_from(Integer::new(7)).unwrap(), 7);
/// assert!(u128::try_from(Integer::new(-7)).is_err());
/// ```
impl TryFrom<Integer> for u128 {
    type Error = AnyException;

    fn try_from(integer: Integer) -> Result<Self, Self::Error> {
        integer
            .to_u128()
            .ok_or_else(|| AnyException::new("RangeError", Some("integer out of range for u128")))
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::{types::Value, AnyException, Integer, NilClass, Object, VM};

    #[cfg(target_os = "macos")]
    #[test]
    fn test_github_issue_113_darwin_os() {
        crate::on_ruby_thread(|| {
            let num: Integer = Integer::new(i64::MIN);
            assert_eq!(num.to_i64(), i64::MIN);

            let num: Integer = Integer::new(i64::MAX);
            assert_eq!(num.to_i64(), i64::MAX);

            let num: i64 = i64::MIN + u32::MAX as i64;
            assert_eq!(Integer::new(num).to_i64(), -9223372032559808513);

            let num: Integer = Integer::new((i32::MIN as i64).pow(2));
            assert_eq!(num.to_i64(), 4611686018427387904);

            let num: Integer = Integer::new((i32::MIN as i64).pow(2) * -1 - 1);
            assert_eq!(num.to_i64(), -4611686018427387905)
        });
    }

    #[test]
    fn test_i32() {
        crate::on_ruby_thread(|| {
            let nil = NilClass::new();

            let num = str_to_num("1").unwrap();
            assert_eq!(1, num.to_i32());

            let num = str_to_num("-1").unwrap();
            assert_eq!(-1, num.to_i32());

            let num = str_to_num("2 ** 31 - 1").unwrap();
            assert_eq!(i32::MAX, num.to_i32());

            let num = str_to_num("2 ** 31").unwrap();
            let result = VM::protect(|| {
                num.to_i32();
                nil.into()
            });
            assert!(result.is_err());

            let num = str_to_num("-1 * 2 ** 31").unwrap();
            assert_eq!(i32::MIN, num.to_i32());

            let num = str_to_num("-1 * 2 ** 31 - 1").unwrap();
            let result = VM::protect(|| {
                num.to_i32();
                nil.into()
            });
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_u32() {
        crate::on_ruby_thread(|| {
            let nil = NilClass::new();

            let num = str_to_num("1").unwrap();
            assert_eq!(1, num.to_u32());

            let num = str_to_num("-1").unwrap();
            assert_eq!(u32::MAX, num.to_u32());

            let num = str_to_num("2 ** 32 - 1").unwrap();
            assert_eq!(u32::MAX, num.to_u32());

            let num = str_to_num("2 ** 32").unwrap();
            let result = VM::protect(|| {
                num.to_u32();
                nil.into()
            });
            assert!(result.is_err());

            let num = str_to_num("0").unwrap();
            assert_eq!(u32::MIN, num.to_u32());
        });
    }

    #[test]
    fn test_i64() {
        crate::on_ruby_thread(|| {
            let nil = NilClass::new();

            let num = str_to_num("2 ** 63 - 1").unwrap();
            assert_eq!(i64::MAX, num.to_i64());

            let num = str_to_num("2 ** 63").unwrap();
            let result = VM::protect(|| {
                num.to_i64();
                nil.into()
            });
            assert!(result.is_err());

            let num = str_to_num("-1 * 2 ** 63").unwrap();
            assert_eq!(i64::MIN, num.to_i64());

            let num = str_to_num("-1 * 2 ** 63 - 1").unwrap();
            let result = VM::protect(|| {
                num.to_i64();
                nil.into()
            });
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_u64() {
        crate::on_ruby_thread(|| {
            let nil = NilClass::new();

            let num = str_to_num("2 ** 64 - 1").unwrap();
            assert_eq!(u64::MAX, num.to_u64());

            let num = str_to_num("2 ** 64").unwrap();
            let result = VM::protect(|| {
                num.to_u64();
                nil.into()
            });
            assert!(result.is_err());

            let num = str_to_num("0").unwrap();
            assert_eq!(u64::MIN, num.to_u64());

            // // Current Ruby implementation does not raise an exception
            // let num = str_to_num("-1").unwrap();
            // let result = VM::protect(|| { num.to_u64(); nil.into() });
            // assert!(result.is_err());
        });
    }

    fn str_to_num(code: &str) -> Result<Integer, AnyException> {
        VM::eval(code).and_then(|x| x.try_convert_to::<Integer>())
    }

    #[test]
    fn test_i128_and_u128_round_trips() {
        crate::on_ruby_thread(|| {
            use std::convert::TryFrom;

            for &number in &[
                0i128,
                1,
                -1,
                i64::MAX as i128,
                i64::MIN as i128,
                i64::MAX as i128 + 1,
                i128::MAX,
                i128::MIN,
                i128::MIN + 1,
            ] {
                let integer = Integer::from(number);

                assert_eq!(integer.to_i128(), Some(number), "{}", number);
                assert_eq!(integer.to_s_radix(10).to_str(), number.to_string());
            }

            for &number in &[0u128, u64::MAX as u128 + 1, u128::MAX] {
                assert_eq!(Integer::from(number).to_u128(), Some(number));
            }

            let too_big = Integer::from(u128::MAX).add(&Integer::new(1));
            assert_eq!(too_big.to_u128(), None);
            assert_eq!(too_big.to_i128(), None);
            assert!(u128::try_from(too_big).is_err());

            let below_min = Integer::from(i128::MIN).sub(&Integer::new(1));
            assert_eq!(below_min.to_i128(), None);
            assert_eq!(
                Integer::from(i128::MAX).add(&Integer::new(1)).to_i128(),
                None
            );
            assert_eq!(Integer::new(-1).to_u128(), None);

            // Ruby-side arithmetic agrees with Rust's.
            let ruby = VM::eval("2**100 - 3")
                .unwrap()
                .try_convert_to::<Integer>()
                .unwrap();
            assert_eq!(ruby.to_u128(), Some((1u128 << 100) - 3));
        });
    }

    #[test]
    fn test_integer_parsing_and_arithmetic() {
        crate::on_ruby_thread(|| {
            use std::{cmp::Ordering, convert::TryFrom};

            assert_eq!(Integer::from_str_radix("0b1010", 0).unwrap().to_i64(), 10);
            assert!(Integer::from_str_radix("1\0", 10).is_err());
            assert!(Integer::from_str_radix("1", 37).is_err());

            let a = Integer::from(u64::MAX);
            let b = Integer::new(2);
            assert_eq!(a.mul(&b).div(&b).unwrap(), a);
            assert_eq!(a.modulo(&b).unwrap().to_i64(), 1);
            assert!(a.div(&Integer::new(0)).is_err());
            assert_eq!(b.compare(&a), Ordering::Less);
            assert_eq!(
                b.pow(10).try_convert_to::<Integer>().unwrap().to_i64(),
                1024
            );

            assert_eq!(
                Integer::try_from(1e20).unwrap().to_u128(),
                Some(100_000_000_000_000_000_000)
            );
            assert!(Integer::try_from(f64::INFINITY).is_err());
            assert_eq!(Integer::new(-3).to_f64(), -3.0);
        });
    }

    #[test]
    fn test_integer_is_bignum() {
        crate::on_ruby_thread(|| {
            assert!(!Integer::new(1).is_bignum());
            assert!(Integer::from(u64::MAX).is_bignum());
            let big = VM::eval("2 ** 100")
                .unwrap()
                .try_convert_to::<Integer>()
                .unwrap();
            assert!(big.is_bignum());
        });
    }
}
