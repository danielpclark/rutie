use super::types::Value;
use libc::{c_long, c_longlong, c_short, c_ulong, c_ulonglong, c_ushort, intptr_t, uintptr_t};

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
}
