use crate::rubysys::{
    libc,
    types::{SignedValue, Value},
};

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_int2inum(intptr_t n)
    pub fn rb_int2inum(num: libc::intptr_t) -> Value;
    // VALUE
    // rb_uint2inum(uintptr_t n)
    pub fn rb_uint2inum(num: libc::uintptr_t) -> Value;
    // VALUE
    // rb_ll2inum(LONG_LONG n)
    pub fn rb_ll2inum(num: libc::c_longlong) -> Value;
    // VALUE
    // rb_ull2inum(unsigned LONG_LONG n)
    pub fn rb_ull2inum(num: libc::c_ulonglong) -> Value;

    // short
    // rb_num2short(VALUE val)
    pub fn rb_num2short(num: Value) -> libc::c_short;
    // unsigned short
    // rb_num2ushort(VALUE val)
    pub fn rb_num2ushort(num: Value) -> libc::c_ushort;
    // long
    // rb_num2int(VALUE val)
    pub fn rb_num2int(num: Value) -> libc::c_long;
    // unsigned long
    // rb_num2uint(VALUE val)
    //
    // Only defined where `int` is smaller than `long`; elsewhere (Windows,
    // 32-bit platforms) `NUM2UINT` is `rb_num2ulong`.
    #[cfg(not(any(windows, target_pointer_width = "32")))]
    pub fn rb_num2uint(num: Value) -> libc::c_ulong;
    // long
    // rb_num2long(VALUE val)
    pub fn rb_num2long(num: Value) -> libc::c_long;
    // unsigned long
    // rb_num2ulong(VALUE val)
    pub fn rb_num2ulong(num: Value) -> libc::c_ulong;
    // LONG_LONG
    // rb_num2ll(VALUE val)
    pub fn rb_num2ll(num: Value) -> libc::c_longlong;
    // unsigned LONG_LONG
    // rb_num2ull(VALUE val)
    pub fn rb_num2ull(num: Value) -> libc::c_ulonglong;
    // VALUE
    // rb_int2big(intptr_t i)
    //
    // Always a Bignum, even for a value that fits a Fixnum.
    pub fn rb_int2big(num: libc::intptr_t) -> Value;
    // VALUE
    // rb_uint2big(uintptr_t i)
    //
    // Always a Bignum, even for a value that fits a Fixnum.
    pub fn rb_uint2big(num: libc::uintptr_t) -> Value;

    // long
    // rb_fix2int(VALUE num)
    pub fn rb_fix2int(num: Value) -> libc::c_long;
    // unsigned long
    // rb_fix2uint(VALUE num)
    //
    // Like `rb_num2uint`, only defined where `int` is smaller than `long`;
    // elsewhere `FIX2UINT` is `FIX2ULONG`.
    #[cfg(not(any(windows, target_pointer_width = "32")))]
    pub fn rb_fix2uint(num: Value) -> libc::c_ulong;
    // short
    // rb_fix2short(VALUE num)
    pub fn rb_fix2short(num: Value) -> libc::c_short;
    // unsigned short
    // rb_fix2ushort(VALUE num)
    pub fn rb_fix2ushort(num: Value) -> libc::c_ushort;
    // void
    // rb_out_of_int(SIGNED_VALUE num)
    //
    // Raises `RangeError`. Only defined where `int` is smaller than `long`.
    #[cfg(not(any(windows, target_pointer_width = "32")))]
    pub fn rb_out_of_int(num: SignedValue) -> !;
}
