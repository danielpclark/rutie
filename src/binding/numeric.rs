use std::{ffi::CStr, mem};

use crate::{
    binding::{fixnum, symbol},
    rubysys::numeric::{
        self, INTEGER_PACK_2COMP, INTEGER_PACK_LSWORD_FIRST, INTEGER_PACK_NATIVE_BYTE_ORDER,
    },
    types::{c_int, c_void, Value},
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
    unsafe { numeric::rb_complex_polar(abs, arg) }
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
