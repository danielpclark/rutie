use std::{
    cmp::Ordering,
    convert::{From, TryFrom},
};

use crate::{
    binding::{array, fixnum, float, numeric, vm},
    types::{c_long, c_ulong, Value, ValueType},
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

    /// Returns `base` raised to the power `exponent`, computed by Ruby
    /// without creating an `Integer` for `base` first (`rb_int_positive_pow`).
    ///
    /// Like [`pow`](#method.pow), the result is an `Integer`, except that
    /// Ruby gives `Infinity` (a `Float`, with a warning) when it would be
    /// enormous. A `base` that does not fit a C `long` (on Windows) or is its
    /// minimum is computed with `**` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, Object, VM};
    /// # VM::init();
    ///
    /// let power = Integer::positive_pow(-3, 3).try_convert_to::<Integer>().unwrap();
    ///
    /// assert_eq!(power.to_i64(), -27);
    ///
    /// let big = Integer::positive_pow(2, 100).try_convert_to::<Integer>().unwrap();
    ///
    /// assert_eq!(big.to_u128(), Some(1 << 100));
    /// assert_eq!(Integer::positive_pow(7, 0).try_convert_to::<Integer>().unwrap().to_i64(), 1);
    /// ```
    pub fn positive_pow(base: i64, exponent: u32) -> AnyObject {
        match c_long::try_from(base) {
            Ok(base) if base != c_long::MIN => {
                AnyObject::from(numeric::int_positive_pow(base, exponent.into()))
            }
            _ => Integer::new(base).pow(exponent),
        }
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

    /// Returns `self & other`, the bitwise and of the two's complements
    /// (Ruby's `&`, `rb_big_and`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Integer::new(0b1100).bit_and(&Integer::new(0b1010)).to_i64(), 0b1000);
    ///
    /// let big = Integer::from(u128::MAX);
    /// assert_eq!(big.bit_and(&Integer::new(-256)).to_u128(), Some(u128::MAX - 255));
    /// ```
    pub fn bit_and(&self, other: &Integer) -> Integer {
        Integer::from(numeric::integer_and(self.value(), other.value()))
    }

    /// Returns `self | other`, the bitwise or of the two's complements
    /// (Ruby's `|`, `rb_big_or`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Integer::new(0b1100).bit_or(&Integer::new(0b1010)).to_i64(), 0b1110);
    ///
    /// let high = Integer::from(1u128 << 100);
    /// assert_eq!(high.bit_or(&Integer::new(1)).to_u128(), Some((1 << 100) | 1));
    /// ```
    pub fn bit_or(&self, other: &Integer) -> Integer {
        Integer::from(numeric::integer_or(self.value(), other.value()))
    }

    /// Returns `self ^ other`, the bitwise exclusive or of the two's
    /// complements (Ruby's `^`, `rb_big_xor`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Integer::new(0b1100).bit_xor(&Integer::new(0b1010)).to_i64(), 0b0110);
    ///
    /// assert_eq!(Integer::new(-1).bit_xor(&Integer::new(5)).to_i64(), -6);
    ///
    /// let big = Integer::from(u128::MAX);
    /// assert_eq!(big.bit_xor(&big).to_i64(), 0);
    /// ```
    pub fn bit_xor(&self, other: &Integer) -> Integer {
        Integer::from(numeric::integer_xor(self.value(), other.value()))
    }

    /// Returns `self` shifted left by `bits` (Ruby's `<<`, `rb_big_lshift`);
    /// a negative `bits` shifts right. Returns the exception, a
    /// `NoMemoryError`, when the result is too big to allocate.
    ///
    /// Ruby treats a `NoMemoryError` caught outside Ruby code as still
    /// being raised, so a second one in the same process aborts it; avoid
    /// shifts by absurd amounts rather than relying on the error.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Integer::new(1).shift_left(100).unwrap().to_u128(), Some(1 << 100));
    /// assert_eq!(Integer::new(-12).shift_left(-2).unwrap().to_i64(), -3);
    /// ```
    pub fn shift_left(&self, bits: i64) -> Result<Integer, AnyException> {
        let integer = self.value();

        vm::protect_value(|| numeric::integer_lshift(integer, bits))
            .map(Integer::from)
            .map_err(AnyException::from)
    }

    /// Returns `self` shifted right by `bits`, rounding towards negative
    /// infinity (Ruby's `>>`, `rb_big_rshift`). A negative `bits` shifts
    /// left, which returns the exception when the result is too big, as
    /// [`shift_left`](#method.shift_left) does.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Integer::from(1u128 << 100).shift_right(98).unwrap().to_i64(), 4);
    /// assert_eq!(Integer::new(-5).shift_right(1).unwrap().to_i64(), -3);
    /// assert_eq!(Integer::new(-5).shift_right(1000).unwrap().to_i64(), -1);
    /// ```
    pub fn shift_right(&self, bits: i64) -> Result<Integer, AnyException> {
        let integer = self.value();

        vm::protect_value(|| numeric::integer_rshift(integer, bits))
            .map(Integer::from)
            .map_err(AnyException::from)
    }

    /// Returns the quotient rounded towards negative infinity and the
    /// modulus with the sign of `other` (Ruby's `divmod`, `rb_big_divmod`),
    /// or the `ZeroDivisionError` when `other` is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// let (quotient, modulus) = Integer::new(5).divmod(&Integer::new(-3)).unwrap();
    /// assert_eq!((quotient.to_i64(), modulus.to_i64()), (-2, -1));
    ///
    /// let (quotient, modulus) = Integer::from(u128::MAX).divmod(&Integer::from(u64::MAX)).unwrap();
    /// assert_eq!(quotient.to_u128(), Some(u64::MAX as u128 + 2));
    /// assert_eq!(modulus.to_i64(), 0);
    ///
    /// assert!(Integer::new(1).divmod(&Integer::new(0)).is_err());
    /// ```
    pub fn divmod(&self, other: &Integer) -> Result<(Integer, Integer), AnyException> {
        let (integer, other) = (self.value(), other.value());

        let pair = vm::protect_value(|| numeric::integer_divmod(integer, other))
            .map_err(AnyException::from)?;

        Ok((
            Integer::from(array::entry(pair, 0)),
            Integer::from(array::entry(pair, 1)),
        ))
    }

    /// Returns how many `word_bits`-bit words the absolute value needs
    /// (`rb_absint_numwords`), or `None` when `word_bits` is zero or the
    /// count does not fit a `usize`. Zero needs no words.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Integer::new(255).abs_num_words(8), Some(1));
    /// assert_eq!(Integer::new(-256).abs_num_words(8), Some(2));
    /// assert_eq!(Integer::from(u128::MAX).abs_num_words(1), Some(128));
    /// assert_eq!(Integer::new(0).abs_num_words(64), Some(0));
    /// assert_eq!(Integer::new(1).abs_num_words(0), None);
    /// ```
    pub fn abs_num_words(&self, word_bits: usize) -> Option<usize> {
        numeric::integer_abs_num_words(self.value(), word_bits)
    }

    /// Returns `true` if the absolute value is a power of two, that is
    /// has exactly one bit set (`rb_absint_singlebit_p`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, VM};
    /// # VM::init();
    ///
    /// assert!(Integer::new(-8).abs_is_power_of_two());
    /// assert!(Integer::from(1u128 << 100).abs_is_power_of_two());
    /// assert!(!Integer::new(6).abs_is_power_of_two());
    /// assert!(!Integer::new(0).abs_is_power_of_two());
    /// ```
    pub fn abs_is_power_of_two(&self) -> bool {
        numeric::integer_abs_is_single_bit(self.value())
    }

    /// Returns the lowest `count` C `unsigned long` words of the integer's
    /// two's complement, least significant first (`rb_big_pack`). Bits that
    /// do not fit are dropped.
    ///
    /// A C `long` is 32 bits on Windows and 32-bit platforms, 64 bits
    /// elsewhere.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{types::c_ulong, Integer, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Integer::new(5).to_long_words(2), vec![5, 0]);
    /// assert_eq!(Integer::new(-1).to_long_words(2), vec![c_ulong::MAX; 2]);
    ///
    /// let wide = Integer::from(1u128 << 64).to_long_words(4);
    /// assert_eq!(Integer::from_long_words(&wide).to_u128(), Some(1 << 64));
    /// ```
    pub fn to_long_words(&self, count: usize) -> Vec<c_ulong> {
        let mut words = vec![0; count];

        numeric::integer_pack_longs(self.value(), &mut words);

        words
    }

    /// Creates the integer whose two's complement is `words`, C `unsigned
    /// long`s least significant first (`rb_big_unpack`). The most
    /// significant bit of the last word is the sign.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{types::c_ulong, Integer, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Integer::from_long_words(&[7, 0]).to_i64(), 7);
    /// assert_eq!(Integer::from_long_words(&[c_ulong::MAX, c_ulong::MAX]).to_i64(), -1);
    /// assert_eq!(Integer::from_long_words(&[]).to_i64(), 0);
    /// ```
    pub fn from_long_words(words: &[c_ulong]) -> Integer {
        Integer::from(numeric::integer_unpack_longs(words))
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

    #[test]
    fn test_positive_pow() {
        crate::on_ruby_thread(|| {
            let pow = |base: i64, exponent: u32| {
                Integer::positive_pow(base, exponent)
                    .try_convert_to::<Integer>()
                    .unwrap()
            };

            assert_eq!(pow(0, 0).to_i64(), 1);
            assert_eq!(pow(5, 1).to_i64(), 5);
            assert_eq!(pow(-2, 63).to_i128(), Some(-(1 << 63)));
            assert_eq!(pow(-2, 64).to_i128(), Some(1 << 64));
            assert_eq!(pow(i64::MAX, 1).to_i64(), i64::MAX);
            assert_eq!(pow(i64::MIN, 1).to_i64(), i64::MIN);
            assert_eq!(
                pow(i64::MIN, 2).to_u128(),
                Some((i64::MIN as i128 * i64::MIN as i128) as u128)
            );
            assert!(pow(10, 40).is_bignum());
        });
    }

    #[test]
    fn test_integer_bit_operations() {
        crate::on_ruby_thread(|| {
            let ruby = |code: &str| str_to_num(code).unwrap();
            let operands = ["0", "5", "-6", "2**70 + 3", "-(2**70) - 5", "2**64 - 1"];

            for x in operands.iter() {
                for y in operands.iter() {
                    let (a, b) = (ruby(x), ruby(y));

                    for (op, result) in [
                        ("&", a.bit_and(&b)),
                        ("|", a.bit_or(&b)),
                        ("^", a.bit_xor(&b)),
                    ] {
                        assert_eq!(
                            result,
                            ruby(&format!("({}) {} ({})", x, op, y)),
                            "{} {} {}",
                            x,
                            op,
                            y
                        );
                    }

                    if b.to_i128() != Some(0) {
                        let (quotient, modulus) = a.divmod(&b).unwrap();
                        assert_eq!(quotient, ruby(&format!("({}).div({})", x, y)));
                        assert_eq!(modulus, ruby(&format!("({}) % ({})", x, y)));
                    } else {
                        assert!(a.divmod(&b).is_err());
                    }
                }

                let a = ruby(x);
                for &bits in &[0i64, 1, 63, 64, 100, -1, -64, -200] {
                    assert_eq!(
                        a.shift_left(bits).unwrap(),
                        ruby(&format!("({}) << {}", x, bits))
                    );
                    assert_eq!(
                        a.shift_right(bits).unwrap(),
                        ruby(&format!("({}) >> {}", x, bits))
                    );
                }

                let words = a.to_long_words(4);
                assert_eq!(Integer::from_long_words(&words), a, "{}", x);
            }

            // Fixnum results are normalised back to Fixnums.
            let big = ruby("2**70 + 3");
            let small = big.bit_and(&Integer::new(0xff));
            assert!(!small.is_bignum());
            assert_eq!(small.to_i64(), 3);

            assert_eq!(Integer::new(-1).shift_right(i64::MAX).unwrap().to_i64(), -1);
        });
    }

    #[test]
    fn test_integer_words() {
        crate::on_ruby_thread(|| {
            use crate::types::c_ulong;

            let bits = c_ulong::BITS as i64;
            let two_words = str_to_num(&format!("2**{} + 5", bits)).unwrap();
            assert_eq!(two_words.to_long_words(3), vec![5, 1, 0]);
            assert_eq!(two_words.to_long_words(1), vec![5]);
            assert_eq!(two_words.to_long_words(0), Vec::<c_ulong>::new());
            assert_eq!(
                Integer::new(-2).to_long_words(2),
                vec![c_ulong::MAX - 1, c_ulong::MAX]
            );
            assert_eq!(Integer::from_long_words(&[5, 1]), two_words);
            assert_eq!(Integer::from_long_words(&[c_ulong::MAX]).to_i64(), -1);
            assert_eq!(
                Integer::from_long_words(&[c_ulong::MAX, 0]).to_u64(),
                c_ulong::MAX as u64
            );

            assert_eq!(two_words.abs_num_words(c_ulong::BITS as usize), Some(2));
            assert_eq!(two_words.abs_num_words(1), Some(bits as usize + 1));
            assert_eq!(Integer::new(-1).abs_num_words(1), Some(1));
            assert_eq!(Integer::new(1).abs_num_words(0), None);

            assert!(Integer::new(1).abs_is_power_of_two());
            assert!(Integer::new(-1).abs_is_power_of_two());
            assert!(str_to_num("-(2**200)").unwrap().abs_is_power_of_two());
            assert!(!str_to_num("2**200 + 1").unwrap().abs_is_power_of_two());
        });
    }

    #[test]
    fn test_bignum_raw_functions() {
        crate::on_ruby_thread(|| {
            use crate::{
                rubysys::{fixnum::*, numeric::*},
                types::c_ulong,
            };
            use std::ffi::CString;

            unsafe {
                let big = rb_int2big(5);
                assert_eq!(big.ty(), crate::types::ValueType::Bignum);
                assert_eq!(rb_big_sign(big), 1);
                assert_eq!(rb_bigzero_p(big), 0);
                assert_eq!(rb_big2long(big), 5);
                assert_eq!(rb_big2ulong(big), 5);
                assert!(rb_big_norm(big).is_fixnum());

                let negative = rb_int2big(-5);
                assert_eq!(rb_big_sign(negative), 0);
                assert_eq!(Integer::from(rb_big_norm(negative)).to_i64(), -5);

                let unsigned = rb_uint2big(usize::MAX);
                assert_eq!(Integer::from(unsigned).to_u64(), usize::MAX as u64);

                let clone = rb_big_clone(big);
                assert!(rb_big_eql(big, clone).is_true());
                assert!(!rb_big_eql(big, Integer::new(5).value()).is_true());
                assert_eq!(
                    Integer::from(rb_big_idiv(big, Integer::new(2).value())).to_i64(),
                    2
                );

                // A zero-filled, frozen Bignum: write nothing, then normalise.
                let zero = rb_big_new(0, 1);
                assert_eq!(rb_bigzero_p(zero), 1);
                assert_eq!(Integer::from(rb_big_norm(zero)).to_i64(), 0);

                // Keeping only the least significant digit of 2**100 + 5.
                let copy = rb_big_clone(str_to_num("2**100 + 5").unwrap().value());
                rb_big_resize(copy, 1);
                assert_eq!(Integer::from(rb_big_norm(rb_big_clone(copy))).to_i64(), 5);
                rb_big_2comp(copy);
                assert!(Integer::from(copy) > Integer::new(5));

                let mut words: [c_ulong; 2] = [7, 0];
                let unpacked = rb_big_unpack(words.as_mut_ptr(), 2);
                rb_big_pack(Integer::new(-1).value(), words.as_mut_ptr(), 2);
                assert_eq!(Integer::from(unpacked).to_i64(), 7);
                assert_eq!(words, [c_ulong::MAX; 2]);

                let digits = CString::new("ff").unwrap();
                assert_eq!(
                    Integer::from(rb_cstr2inum(digits.as_ptr(), 16)).to_i64(),
                    255
                );

                let mut buffer = [0 as crate::types::c_char; 6];
                assert_eq!(rb_uv_to_utf8(buffer.as_mut_ptr(), 0x20ac), 3);
                let bytes: Vec<u8> = buffer[..3].iter().map(|&b| b as u8).collect();
                assert_eq!(bytes, "\u{20ac}".as_bytes());

                assert_eq!(Integer::from(rb_int_positive_pow(3, 4)).to_i64(), 81);
                assert_eq!(
                    Integer::from(rb_int_positive_pow(2, 100)).to_u128(),
                    Some(1 << 100)
                );

                let id = crate::binding::symbol::internal_id("&");
                let and = rb_num_coerce_bit(Integer::new(6).value(), Integer::new(3).value(), id);
                assert_eq!(Integer::from(and).to_i64(), 2);
            }
        });
    }

    #[test]
    fn test_numeric_parsing_raw_functions() {
        crate::on_ruby_thread(|| {
            use crate::rubysys::numeric::*;
            use std::{ffi::CString, ptr};

            unsafe {
                let text = CString::new("0x1p4 rest").unwrap();
                let mut end = ptr::null_mut();
                assert_eq!(ruby_strtod(text.as_ptr(), &mut end), 16.0);
                assert_eq!(end as usize - text.as_ptr() as usize, 5);

                let text = CString::new("  -12z").unwrap();
                let mut end = ptr::null_mut();
                assert_eq!(ruby_strtoul(text.as_ptr(), &mut end, 10) as i64 as i32, -12);
                assert_eq!(*end as u8, b'z');
                assert_eq!(
                    ruby_strtoul(text.as_ptr(), ptr::null_mut(), 36) as i64 as i32,
                    -1403
                );

                let mut consumed = 0;
                assert_eq!(ruby_scan_hex(b"fFg".as_ptr() as _, 3, &mut consumed), 255);
                assert_eq!(consumed, 2);
                assert_eq!(ruby_scan_oct(b"778".as_ptr() as _, 3, &mut consumed), 63);
                assert_eq!(consumed, 2);

                let mut overflow = 0;
                let text = b"zz!";
                assert_eq!(
                    ruby_scan_digits(text.as_ptr() as _, 3, 36, &mut consumed, &mut overflow),
                    35 * 36 + 35
                );
                assert_eq!((consumed, overflow), (2, 0));
                let text = CString::new("1".repeat(100)).unwrap();
                ruby_scan_digits(text.as_ptr(), -1, 10, &mut consumed, &mut overflow);
                assert_eq!((consumed, overflow), (100, 1));
            }
        });
    }
}
