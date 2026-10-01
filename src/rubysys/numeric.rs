use crate::rubysys::{
    libc::c_ulong,
    types::{c_char, c_double, c_int, c_long, c_void, size_t, ssize_t, Id, Value},
};

pub const INTEGER_PACK_MSWORD_FIRST: c_int = 0x01;
pub const INTEGER_PACK_LSWORD_FIRST: c_int = 0x02;
pub const INTEGER_PACK_MSBYTE_FIRST: c_int = 0x10;
pub const INTEGER_PACK_LSBYTE_FIRST: c_int = 0x20;
pub const INTEGER_PACK_NATIVE_BYTE_ORDER: c_int = 0x40;
pub const INTEGER_PACK_2COMP: c_int = 0x80;
pub const INTEGER_PACK_FORCE_BIGNUM: c_int = 0x100;
pub const INTEGER_PACK_NEGATIVE: c_int = 0x200;

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // size_t
    // rb_absint_size(VALUE val, int *nlz_bits_ret)
    pub fn rb_absint_size(value: Value, nlz_bits: *mut c_int) -> size_t;
    // VALUE
    // rb_big2str(VALUE x, int base)
    //
    // Also accepts a Fixnum.
    pub fn rb_big2str(integer: Value, base: c_int) -> Value;
    // double
    // rb_big2dbl(VALUE x)
    //
    // The `rb_big*` functions below take a Bignum receiver only; a Fixnum
    // receiver is undefined behaviour.
    pub fn rb_big2dbl(bignum: Value) -> c_double;
    // LONG_LONG
    // rb_big2ll(VALUE x)
    pub fn rb_big2ll(bignum: Value) -> i64;
    // unsigned LONG_LONG
    // rb_big2ull(VALUE x)
    pub fn rb_big2ull(bignum: Value) -> u64;
    // VALUE
    // rb_big_cmp(VALUE x, VALUE y)
    pub fn rb_big_cmp(bignum: Value, other: Value) -> Value;
    // VALUE
    // rb_big_div(VALUE x, VALUE y)
    pub fn rb_big_div(bignum: Value, other: Value) -> Value;
    // VALUE
    // rb_big_eq(VALUE x, VALUE y)
    pub fn rb_big_eq(bignum: Value, other: Value) -> Value;
    // VALUE
    // rb_big_minus(VALUE x, VALUE y)
    pub fn rb_big_minus(bignum: Value, other: Value) -> Value;
    // VALUE
    // rb_big_modulo(VALUE x, VALUE y)
    pub fn rb_big_modulo(bignum: Value, other: Value) -> Value;
    // VALUE
    // rb_big_mul(VALUE x, VALUE y)
    pub fn rb_big_mul(bignum: Value, other: Value) -> Value;
    // VALUE
    // rb_big_plus(VALUE x, VALUE y)
    pub fn rb_big_plus(bignum: Value, other: Value) -> Value;
    // VALUE
    // rb_big_pow(VALUE x, VALUE y)
    pub fn rb_big_pow(bignum: Value, other: Value) -> Value;
    // VALUE
    // rb_Complex(VALUE x, VALUE y)
    pub fn rb_Complex(real: Value, imaginary: Value) -> Value;
    // VALUE
    // rb_complex_abs(VALUE z)
    pub fn rb_complex_abs(complex: Value) -> Value;
    // VALUE
    // rb_complex_arg(VALUE z)
    pub fn rb_complex_arg(complex: Value) -> Value;
    // VALUE
    // rb_complex_imag(VALUE z)
    pub fn rb_complex_imag(complex: Value) -> Value;
    // VALUE
    // rb_complex_new(VALUE x, VALUE y)
    pub fn rb_complex_new(real: Value, imaginary: Value) -> Value;
    // VALUE
    // rb_complex_new_polar(VALUE abs, VALUE arg)
    //
    // `rb_complex_polar` is the same function, deprecated since Ruby 3.0.
    pub fn rb_complex_new_polar(abs: Value, arg: Value) -> Value;
    // VALUE
    // rb_complex_raw(VALUE x, VALUE y)
    pub fn rb_complex_raw(real: Value, imaginary: Value) -> Value;
    // VALUE
    // rb_complex_real(VALUE z)
    pub fn rb_complex_real(complex: Value) -> Value;
    // VALUE
    // rb_cstr_to_inum(const char *str, int base, int badcheck)
    pub fn rb_cstr_to_inum(string: *const c_char, base: c_int, badcheck: c_int) -> Value;
    // VALUE
    // rb_dbl2big(double d)
    pub fn rb_dbl2big(number: c_double) -> Value;
    // VALUE
    // rb_fix2str(VALUE x, int base)
    pub fn rb_fix2str(fixnum: Value, base: c_int) -> Value;
    // VALUE
    // rb_flt_rationalize(VALUE flt)
    pub fn rb_flt_rationalize(float: Value) -> Value;
    // int
    // rb_integer_pack(VALUE val, void *words, size_t numwords, size_t wordsize,
    //                 size_t nails, int flags)
    pub fn rb_integer_pack(
        value: Value,
        words: *mut c_void,
        numwords: size_t,
        wordsize: size_t,
        nails: size_t,
        flags: c_int,
    ) -> c_int;
    // VALUE
    // rb_integer_unpack(const void *words, size_t numwords, size_t wordsize,
    //                   size_t nails, int flags)
    pub fn rb_integer_unpack(
        words: *const c_void,
        numwords: size_t,
        wordsize: size_t,
        nails: size_t,
        flags: c_int,
    ) -> Value;
    // VALUE
    // rb_num2fix(VALUE val)
    pub fn rb_num2fix(value: Value) -> Value;
    // VALUE
    // rb_num_coerce_bin(VALUE x, VALUE y, ID func)
    pub fn rb_num_coerce_bin(x: Value, y: Value, func: Id) -> Value;
    // VALUE
    // rb_num_coerce_cmp(VALUE x, VALUE y, ID func)
    pub fn rb_num_coerce_cmp(x: Value, y: Value, func: Id) -> Value;
    // VALUE
    // rb_num_coerce_relop(VALUE x, VALUE y, ID func)
    pub fn rb_num_coerce_relop(x: Value, y: Value, func: Id) -> Value;
    // VALUE
    // rb_Rational(VALUE x, VALUE y)
    pub fn rb_Rational(numerator: Value, denominator: Value) -> Value;
    // VALUE
    // rb_rational_den(VALUE rat)
    pub fn rb_rational_den(rational: Value) -> Value;
    // VALUE
    // rb_rational_new(VALUE x, VALUE y)
    pub fn rb_rational_new(numerator: Value, denominator: Value) -> Value;
    // VALUE
    // rb_rational_num(VALUE rat)
    pub fn rb_rational_num(rational: Value) -> Value;
    // VALUE
    // rb_rational_raw(VALUE x, VALUE y)
    pub fn rb_rational_raw(numerator: Value, denominator: Value) -> Value;
    // VALUE
    // rb_str2inum(VALUE str, int base)
    pub fn rb_str2inum(string: Value, base: c_int) -> Value;
    // VALUE
    // rb_int_positive_pow(long x, unsigned long y)
    //
    // `x ** y`; a `Float` (`Infinity`, with a warning) when the result is
    // enormous. Negates `x` internally, so `x` must not be `LONG_MIN`.
    pub fn rb_int_positive_pow(x: c_long, y: c_ulong) -> Value;
}

// `ruby/util.h`
#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // RUBY_EXTERN const char ruby_hexdigits[];
    //
    // `"0123456789abcdef0123456789ABCDEF"`, with its terminating NUL: the
    // lowercase digits, then the uppercase ones from index 16.
    pub static ruby_hexdigits: [c_char; 33];
    // unsigned long
    // ruby_scan_digits(const char *str, ssize_t len, int base, size_t *retlen, int *overflow)
    //
    // Parses the digits of `base` (2 to 36) at `str`, stopping at the first
    // other byte or after `len` bytes (a negative `len` has no limit). Stores
    // the number of digits read in `*retlen`, and sets `*overflow` when the
    // value does not fit.
    pub fn ruby_scan_digits(
        str: *const c_char,
        len: ssize_t,
        base: c_int,
        retlen: *mut size_t,
        overflow: *mut c_int,
    ) -> c_ulong;
}

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use super::*;

    #[test]
    fn test_ruby_hexdigits() {
        let digits = unsafe { CStr::from_ptr(ruby_hexdigits.as_ptr()) };

        assert_eq!(digits.to_str(), Ok("0123456789abcdef0123456789ABCDEF"));
    }

    #[test]
    fn test_ruby_scan_digits() {
        crate::on_ruby_thread(|| {
            let scan = |text: &[u8], len: ssize_t, base: c_int| {
                let (mut read, mut overflow) = (0, 0);
                let value = unsafe {
                    ruby_scan_digits(
                        text.as_ptr() as *const c_char,
                        len,
                        base,
                        &mut read,
                        &mut overflow,
                    )
                };

                (value as u64, read, overflow)
            };

            assert_eq!(scan(b"ffz\0", -1, 16), (255, 2, 0));
            assert_eq!(scan(b"1234\0", 2, 10), (12, 2, 0));
            assert_eq!(scan(b"777\0", -1, 8), (511, 3, 0));
            assert_eq!(scan(b"9\0", -1, 8), (0, 0, 0));
            assert_eq!(scan(b"zz\0", -1, 36), (35 * 36 + 35, 2, 0));

            let (_, read, overflow) = scan(b"999999999999999999999999999999\0", -1, 10);

            assert_eq!((read, overflow), (30, 1));
        });
    }
}
