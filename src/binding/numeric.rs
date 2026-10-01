use std::{cmp::Ordering, ffi::CStr, mem, ptr};

use crate::{
    binding::{fixnum, symbol},
    rubysys::{
        self,
        numeric::{
            self, INTEGER_PACK_2COMP, INTEGER_PACK_LSWORD_FIRST, INTEGER_PACK_NATIVE_BYTE_ORDER,
        },
    },
    types::{c_int, c_long, c_ulong, c_void, Value},
};

const PACK_FLAGS: c_int = INTEGER_PACK_LSWORD_FIRST | INTEGER_PACK_NATIVE_BYTE_ORDER;

pub fn i128_to_integer(number: i128) -> Value {
    unsafe {
        numeric::rb_integer_unpack(
            &number as *const i128 as *const c_void,
            1,
            mem::size_of::<i128>(),
            0,
            PACK_FLAGS | INTEGER_PACK_2COMP,
        )
    }
}

pub fn u128_to_integer(number: u128) -> Value {
    unsafe {
        numeric::rb_integer_unpack(
            &number as *const u128 as *const c_void,
            1,
            mem::size_of::<u128>(),
            0,
            PACK_FLAGS,
        )
    }
}

// `None` when `integer` does not fit in an `i128`.
pub fn integer_to_i128(integer: Value) -> Option<i128> {
    let mut number: i128 = 0;
    let sign = unsafe {
        numeric::rb_integer_pack(
            integer,
            &mut number as *mut i128 as *mut c_void,
            1,
            mem::size_of::<i128>(),
            0,
            PACK_FLAGS | INTEGER_PACK_2COMP,
        )
    };

    // 128-bit two's complement holds -2**128 <= n < 2**128; i128 only the
    // half of that whose sign bit agrees with the sign Ruby reports.
    match sign {
        0 => Some(0),
        1 if number > 0 => Some(number),
        -1 if number < 0 => Some(number),
        _ => None,
    }
}

// `None` when `integer` is negative or does not fit in a `u128`.
pub fn integer_to_u128(integer: Value) -> Option<u128> {
    let mut number: u128 = 0;
    let sign = unsafe {
        numeric::rb_integer_pack(
            integer,
            &mut number as *mut u128 as *mut c_void,
            1,
            mem::size_of::<u128>(),
            0,
            PACK_FLAGS,
        )
    };

    match sign {
        0 | 1 => Some(number),
        _ => None,
    }
}

// Raises `ArgumentError` for an invalid string, so the caller owns `string`.
// With no length, base 0 does not detect prefixes; prefer `string::to_integer`.
pub fn cstr_to_integer(string: &CStr, base: u32) -> Value {
    unsafe { numeric::rb_cstr_to_inum(string.as_ptr(), base as c_int, 1) }
}

// Fixnums and Bignums; `base` must be 2..=36.
pub fn integer_to_s(integer: Value, base: u32) -> Value {
    unsafe { numeric::rb_big2str(integer, base as c_int) }
}

pub fn f64_to_integer(number: f64) -> Value {
    unsafe { numeric::rb_dbl2big(number) }
}

pub fn float_rationalize(float: Value) -> Value {
    unsafe { numeric::rb_flt_rationalize(float) }
}

pub fn rational_new(numerator: Value, denominator: Value) -> Value {
    unsafe { numeric::rb_rational_new(numerator, denominator) }
}

pub fn rational_numerator(rational: Value) -> Value {
    unsafe { numeric::rb_rational_num(rational) }
}

pub fn rational_denominator(rational: Value) -> Value {
    unsafe { numeric::rb_rational_den(rational) }
}

pub fn to_rational(object: Value) -> Value {
    unsafe { numeric::rb_Rational(object, fixnum::i64_to_num(1)) }
}

pub fn complex_new(real: Value, imaginary: Value) -> Value {
    unsafe { numeric::rb_complex_new(real, imaginary) }
}

pub fn complex_polar(abs: Value, arg: Value) -> Value {
    unsafe { numeric::rb_complex_new_polar(abs, arg) }
}

pub fn to_complex(object: Value) -> Value {
    unsafe { numeric::rb_Complex(object, fixnum::i64_to_num(0)) }
}

pub fn complex_real(complex: Value) -> Value {
    unsafe { numeric::rb_complex_real(complex) }
}

pub fn complex_imaginary(complex: Value) -> Value {
    unsafe { numeric::rb_complex_imag(complex) }
}

pub fn complex_abs(complex: Value) -> Value {
    unsafe { numeric::rb_complex_abs(complex) }
}

pub fn complex_arg(complex: Value) -> Value {
    unsafe { numeric::rb_complex_arg(complex) }
}

pub fn coerce_bin(x: Value, y: Value, operator: &str) -> Value {
    unsafe { numeric::rb_num_coerce_bin(x, y, symbol::internal_id(operator)) }
}

pub fn coerce_cmp(x: Value, y: Value, operator: &str) -> Value {
    unsafe { numeric::rb_num_coerce_cmp(x, y, symbol::internal_id(operator)) }
}

pub fn coerce_relop(x: Value, y: Value, operator: &str) -> Value {
    unsafe { numeric::rb_num_coerce_relop(x, y, symbol::internal_id(operator)) }
}

pub fn complex_from_parts(real: Value, imaginary: Value) -> Value {
    unsafe { numeric::rb_Complex(real, imaginary) }
}

// `x ** y`; `x` must not be `c_long::MIN`, which Ruby negates.
pub fn int_positive_pow(x: c_long, y: c_ulong) -> Value {
    unsafe { numeric::rb_int_positive_pow(x, y) }
}

// The `rb_big_*` operations read their receiver as a Bignum, so a Fixnum is
// first copied into one (`rb_int2big`). Their results are normalised.
fn to_bignum(integer: Value) -> Value {
    if integer.is_fixnum() {
        unsafe { rubysys::fixnum::rb_int2big(fixnum::num_to_isize(integer)) }
    } else {
        integer
    }
}

pub fn integer_and(integer: Value, other: Value) -> Value {
    unsafe { numeric::rb_big_and(to_bignum(integer), other) }
}

pub fn integer_or(integer: Value, other: Value) -> Value {
    unsafe { numeric::rb_big_or(to_bignum(integer), other) }
}

pub fn integer_xor(integer: Value, other: Value) -> Value {
    unsafe { numeric::rb_big_xor(to_bignum(integer), other) }
}

// Raises `RangeError` (or `NoMemoryError`) when the result is too big.
pub fn integer_lshift(integer: Value, bits: i64) -> Value {
    unsafe { numeric::rb_big_lshift(to_bignum(integer), fixnum::i64_to_num(bits)) }
}

// A negative `bits` shifts left, so this can raise like `integer_lshift`.
pub fn integer_rshift(integer: Value, bits: i64) -> Value {
    unsafe { numeric::rb_big_rshift(to_bignum(integer), fixnum::i64_to_num(bits)) }
}

// `[quotient, modulus]`; raises `ZeroDivisionError`.
pub fn integer_divmod(integer: Value, other: Value) -> Value {
    unsafe { numeric::rb_big_divmod(to_bignum(integer), other) }
}

// `None` when `word_bits` is 0 or the count does not fit a `size_t`.
pub fn integer_abs_num_words(integer: Value, word_bits: usize) -> Option<usize> {
    let words = unsafe { numeric::rb_absint_numwords(integer, word_bits, ptr::null_mut()) };

    if words == usize::MAX {
        None
    } else {
        Some(words)
    }
}

pub fn integer_abs_is_single_bit(integer: Value) -> bool {
    unsafe { numeric::rb_absint_singlebit_p(integer) != 0 }
}

// The low `words.len()` words of the two's complement, least significant first.
pub fn integer_pack_longs(integer: Value, words: &mut [c_ulong]) {
    if !words.is_empty() {
        unsafe { numeric::rb_big_pack(integer, words.as_mut_ptr(), words.len() as c_long) }
    }
}

pub fn integer_unpack_longs(words: &[c_ulong]) -> Value {
    if words.is_empty() {
        return fixnum::i64_to_num(0);
    }

    // `rb_big_unpack` only reads the buffer.
    unsafe { numeric::rb_big_unpack(words.as_ptr() as *mut c_ulong, words.len() as c_long) }
}

// Raises `FloatDomainError` for `NaN` and infinities.
pub fn float_rationalize_with_precision(float: Value, precision: Value) -> Value {
    unsafe { numeric::rb_flt_rationalize_with_prec(float, precision) }
}

// `None` when either is `NaN`.
pub fn compare_f64(lhs: f64, rhs: f64) -> Option<Ordering> {
    let result = unsafe { numeric::rb_dbl_cmp(lhs, rhs) };

    if result.is_nil() {
        None
    } else {
        Some(fixnum::num_to_i64(result).cmp(&0))
    }
}

pub fn complex_from_f64(real: f64, imaginary: f64) -> Value {
    unsafe { numeric::rb_dbl_complex_new(real, imaginary) }
}

pub fn complex_plus(complex: Value, other: Value) -> Value {
    unsafe { numeric::rb_complex_plus(complex, other) }
}

pub fn complex_minus(complex: Value, other: Value) -> Value {
    unsafe { numeric::rb_complex_minus(complex, other) }
}

pub fn complex_mul(complex: Value, other: Value) -> Value {
    unsafe { numeric::rb_complex_mul(complex, other) }
}

pub fn complex_div(complex: Value, other: Value) -> Value {
    unsafe { numeric::rb_complex_div(complex, other) }
}

pub fn complex_pow(complex: Value, exponent: Value) -> Value {
    unsafe { numeric::rb_complex_pow(complex, exponent) }
}

pub fn complex_uminus(complex: Value) -> Value {
    unsafe { numeric::rb_complex_uminus(complex) }
}

pub fn complex_conjugate(complex: Value) -> Value {
    unsafe { numeric::rb_complex_conjugate(complex) }
}

pub fn coerce_bit(x: Value, y: Value, operator: &str) -> Value {
    unsafe { numeric::rb_num_coerce_bit(x, y, symbol::internal_id(operator)) }
}
