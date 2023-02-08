use super::types::Value;
use libc::{c_long, c_longlong, c_short, c_ulong, c_ulonglong, c_ushort, intptr_t, uintptr_t};
pub use rb_sys::{
    // VALUE
    // rb_int2inum(intptr_t n)
    rb_int2inum,
    // VALUE
    // rb_ll2inum(LONG_LONG n)
    rb_ll2inum,
    // long
    // rb_num2int(VALUE val)
    rb_num2int,
    // LONG_LONG
    // rb_num2ll(VALUE val)
    rb_num2ll,
    // long
    // rb_num2long(VALUE val)
    rb_num2long,
    // short
    // rb_num2short(VALUE val)
    rb_num2short,
    // unsigned long
    // rb_num2uint(VALUE val)
    rb_num2uint,
    // unsigned LONG_LONG
    // rb_num2ull(VALUE val)
    rb_num2ull,
    // unsigned long
    // rb_num2ulong(VALUE val)
    rb_num2ulong,
    // unsigned short
    // rb_num2ushort(VALUE val)
    rb_num2ushort,
    // VALUE
    // rb_uint2inum(uintptr_t n)
    rb_uint2inum,
    // VALUE
    // rb_ull2inum(unsigned LONG_LONG n)
    rb_ull2inum,
};
