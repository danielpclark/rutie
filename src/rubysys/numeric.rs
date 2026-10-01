use crate::rubysys::{
    libc::{c_long, c_ulong, ssize_t},
    types::{c_char, c_double, c_int, c_void, size_t, Id, Value},
};

pub const INTEGER_PACK_MSWORD_FIRST: c_int = 0x01;
pub const INTEGER_PACK_LSWORD_FIRST: c_int = 0x02;
pub const INTEGER_PACK_MSBYTE_FIRST: c_int = 0x10;
pub const INTEGER_PACK_LSBYTE_FIRST: c_int = 0x20;
pub const INTEGER_PACK_NATIVE_BYTE_ORDER: c_int = 0x40;
pub const INTEGER_PACK_2COMP: c_int = 0x80;
pub const INTEGER_PACK_FORCE_BIGNUM: c_int = 0x100;
pub const INTEGER_PACK_NEGATIVE: c_int = 0x200;
pub const INTEGER_PACK_FORCE_GENERIC_IMPLEMENTATION: c_int = 0x400;
pub const INTEGER_PACK_LITTLE_ENDIAN: c_int = INTEGER_PACK_LSWORD_FIRST | INTEGER_PACK_LSBYTE_FIRST;
pub const INTEGER_PACK_BIG_ENDIAN: c_int = INTEGER_PACK_MSWORD_FIRST | INTEGER_PACK_MSBYTE_FIRST;

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

    // ruby/internal/intern/bignum.h and ruby/internal/core/rbignum.h. Unless
    // noted, the `VALUE x`/`num` receivers below must be Bignums.

    // size_t
    // rb_absint_numwords(VALUE val, size_t word_numbits, size_t *nlz_bits_ret)
    //
    // Accepts any Integer; `(size_t)-1` when `word_numbits` is 0 or on overflow.
    pub fn rb_absint_numwords(value: Value, word_numbits: size_t, nlz_bits: *mut size_t) -> size_t;
    // int
    // rb_absint_singlebit_p(VALUE val)
    //
    // Accepts any Integer.
    pub fn rb_absint_singlebit_p(value: Value) -> c_int;
    // long
    // rb_big2long(VALUE x)
    pub fn rb_big2long(bignum: Value) -> c_long;
    // unsigned long
    // rb_big2ulong(VALUE x)
    pub fn rb_big2ulong(bignum: Value) -> c_ulong;
    // void
    // rb_big_2comp(VALUE num)
    pub fn rb_big_2comp(bignum: Value);
    // VALUE
    // rb_big_and(VALUE x, VALUE y)
    pub fn rb_big_and(bignum: Value, other: Value) -> Value;
    // VALUE
    // rb_big_clone(VALUE num)
    pub fn rb_big_clone(bignum: Value) -> Value;
    // VALUE
    // rb_big_divmod(VALUE x, VALUE y)
    pub fn rb_big_divmod(bignum: Value, other: Value) -> Value;
    // VALUE
    // rb_big_eql(VALUE lhs, VALUE rhs)
    pub fn rb_big_eql(bignum: Value, other: Value) -> Value;
    // VALUE
    // rb_big_idiv(VALUE x, VALUE y)
    pub fn rb_big_idiv(bignum: Value, other: Value) -> Value;
    // VALUE
    // rb_big_lshift(VALUE x, VALUE y)
    pub fn rb_big_lshift(bignum: Value, bits: Value) -> Value;
    // VALUE
    // rb_big_new(size_t len, int sign)
    //
    // The digits are uninitialised; `sign` is nonzero for a positive number.
    pub fn rb_big_new(len: size_t, sign: c_int) -> Value;
    // VALUE
    // rb_big_norm(VALUE x)
    pub fn rb_big_norm(bignum: Value) -> Value;
    // VALUE
    // rb_big_or(VALUE x, VALUE y)
    pub fn rb_big_or(bignum: Value, other: Value) -> Value;
    // void
    // rb_big_pack(VALUE val, unsigned long *buf, long num_longs)
    //
    // Accepts any Integer (or object with `to_int`).
    pub fn rb_big_pack(value: Value, buf: *mut c_ulong, num_longs: c_long);
    // void
    // rb_big_resize(VALUE big, size_t len)
    pub fn rb_big_resize(bignum: Value, len: size_t);
    // VALUE
    // rb_big_rshift(VALUE x, VALUE y)
    pub fn rb_big_rshift(bignum: Value, bits: Value) -> Value;
    // int
    // rb_big_sign(VALUE num)
    pub fn rb_big_sign(bignum: Value) -> c_int;
    // VALUE
    // rb_big_unpack(unsigned long *buf, long num_longs)
    pub fn rb_big_unpack(buf: *mut c_ulong, num_longs: c_long) -> Value;
    // VALUE
    // rb_big_xor(VALUE x, VALUE y)
    pub fn rb_big_xor(bignum: Value, other: Value) -> Value;
    // int
    // rb_bigzero_p(VALUE x)
    pub fn rb_bigzero_p(bignum: Value) -> c_int;
    // VALUE
    // rb_cstr2inum(const char *str, int base)
    pub fn rb_cstr2inum(string: *const c_char, base: c_int) -> Value;
    // int
    // rb_uv_to_utf8(char buf[6], unsigned long uv)
    pub fn rb_uv_to_utf8(buf: *mut c_char, uv: c_ulong) -> c_int;

    // ruby/internal/intern/complex.h. `x`, `z` and `base` must be Complex.

    // VALUE
    // rb_complex_conjugate(VALUE z)
    pub fn rb_complex_conjugate(complex: Value) -> Value;
    // VALUE
    // rb_complex_div(VALUE x, VALUE y)
    pub fn rb_complex_div(complex: Value, other: Value) -> Value;
    // VALUE
    // rb_complex_minus(VALUE x, VALUE y)
    pub fn rb_complex_minus(complex: Value, other: Value) -> Value;
    // VALUE
    // rb_complex_mul(VALUE x, VALUE y)
    pub fn rb_complex_mul(complex: Value, other: Value) -> Value;
    // VALUE
    // rb_complex_plus(VALUE x, VALUE y)
    //
    // `rb_complex_add` is a macro for this function.
    pub fn rb_complex_plus(complex: Value, other: Value) -> Value;
    // VALUE
    // rb_complex_pow(VALUE base, VALUE exp)
    pub fn rb_complex_pow(complex: Value, exponent: Value) -> Value;
    // VALUE
    // rb_complex_uminus(VALUE z)
    pub fn rb_complex_uminus(complex: Value) -> Value;
    // VALUE
    // rb_dbl_complex_new(double real, double imag)
    pub fn rb_dbl_complex_new(real: c_double, imaginary: c_double) -> Value;

    // ruby/internal/intern/rational.h

    // VALUE
    // rb_flt_rationalize_with_prec(VALUE flt, VALUE prec)
    pub fn rb_flt_rationalize_with_prec(float: Value, precision: Value) -> Value;

    // ruby/internal/intern/numeric.h

    // VALUE
    // rb_dbl_cmp(double lhs, double rhs)
    pub fn rb_dbl_cmp(lhs: c_double, rhs: c_double) -> Value;
    // VALUE
    // rb_int_positive_pow(long x, unsigned long y)
    pub fn rb_int_positive_pow(x: c_long, y: c_ulong) -> Value;
    // VALUE
    // rb_num_coerce_bit(VALUE x, VALUE y, ID func)
    pub fn rb_num_coerce_bit(x: Value, y: Value, func: Id) -> Value;

    // ruby/util.h and ruby/internal/ctype.h

    // RUBY_EXTERN const char ruby_hexdigits[];
    //
    // `"0123456789abcdef0123456789ABCDEF"`, with its terminating NUL: the
    // lowercase digits, then the uppercase ones from index 16.
    pub static ruby_hexdigits: [c_char; 33];
    // unsigned long
    // ruby_scan_digits(const char *str, ssize_t len, int base, size_t *retlen,
    //                  int *overflow)
    pub fn ruby_scan_digits(
        string: *const c_char,
        len: ssize_t,
        base: c_int,
        retlen: *mut size_t,
        overflow: *mut c_int,
    ) -> c_ulong;
    // unsigned long
    // ruby_scan_hex(const char *str, size_t len, size_t *ret)
    pub fn ruby_scan_hex(string: *const c_char, len: size_t, consumed: *mut size_t) -> c_ulong;
    // unsigned long
    // ruby_scan_oct(const char *str, size_t len, size_t *consumed)
    pub fn ruby_scan_oct(string: *const c_char, len: size_t, consumed: *mut size_t) -> c_ulong;
    // double
    // ruby_strtod(const char *str, char **endptr)
    pub fn ruby_strtod(string: *const c_char, endptr: *mut *mut c_char) -> c_double;
    // unsigned long
    // ruby_strtoul(const char *str, char **endptr, int base)
    pub fn ruby_strtoul(string: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_ulong;
}
