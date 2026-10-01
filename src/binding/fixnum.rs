use crate::{rubysys::fixnum, types::Value};

pub fn i32_to_num(num: i32) -> Value {
    unsafe { fixnum::rb_int2inum(num as isize) }
}

pub fn u32_to_num(num: u32) -> Value {
    unsafe { fixnum::rb_uint2inum(num as usize) }
}

pub fn isize_to_num(num: isize) -> Value {
    unsafe { fixnum::rb_int2inum(num) }
}

pub fn usize_to_num(num: usize) -> Value {
    unsafe { fixnum::rb_uint2inum(num) }
}

pub fn i64_to_num(num: i64) -> Value {
    unsafe { fixnum::rb_ll2inum(num) }
}

pub fn u64_to_num(num: u64) -> Value {
    unsafe { fixnum::rb_ull2inum(num) }
}

pub fn num_to_i32(num: Value) -> i32 {
    unsafe { fixnum::rb_num2int(num) as i32 }
}

#[cfg(not(any(windows, target_pointer_width = "32")))]
pub fn num_to_u32(num: Value) -> u32 {
    unsafe { fixnum::rb_num2uint(num) as u32 }
}

// `int` and `long` are the same size here, and Ruby's `NUM2UINT` is
// `rb_num2ulong` (there is no `rb_num2uint`).
#[cfg(any(windows, target_pointer_width = "32"))]
pub fn num_to_u32(num: Value) -> u32 {
    unsafe { fixnum::rb_num2ulong(num) as u32 }
}

// On 64-bit Windows (LLP64) `long` is 32 bits while `isize` is 64, so
// `rb_num2long` would raise `RangeError` for values that fit an `isize`.
#[cfg(not(all(windows, target_pointer_width = "64")))]
pub fn num_to_isize(num: Value) -> isize {
    unsafe { fixnum::rb_num2long(num) as isize }
}

#[cfg(all(windows, target_pointer_width = "64"))]
pub fn num_to_isize(num: Value) -> isize {
    unsafe { fixnum::rb_num2ll(num) as isize }
}

#[cfg(not(all(windows, target_pointer_width = "64")))]
pub fn num_to_usize(num: Value) -> usize {
    unsafe { fixnum::rb_num2ulong(num) as usize }
}

#[cfg(all(windows, target_pointer_width = "64"))]
pub fn num_to_usize(num: Value) -> usize {
    unsafe { fixnum::rb_num2ull(num) as usize }
}

pub fn num_to_i64(num: Value) -> i64 {
    unsafe { fixnum::rb_num2ll(num) }
}

pub fn num_to_u64(num: Value) -> u64 {
    unsafe { fixnum::rb_num2ull(num) }
}

// The `fix_to_*` conversions raise `RangeError` when the number does not fit.
pub fn fix_to_i16(num: Value) -> i16 {
    unsafe { fixnum::rb_fix2short(num) as i16 }
}

pub fn fix_to_u16(num: Value) -> u16 {
    unsafe { fixnum::rb_fix2ushort(num) as u16 }
}

pub fn fix_to_i32(num: Value) -> i32 {
    unsafe { fixnum::rb_fix2int(num) as i32 }
}

#[cfg(not(any(windows, target_pointer_width = "32")))]
pub fn fix_to_u32(num: Value) -> u32 {
    unsafe { fixnum::rb_fix2uint(num) as u32 }
}

// No `rb_fix2uint` here (see `num_to_u32`).
#[cfg(any(windows, target_pointer_width = "32"))]
pub fn fix_to_u32(num: Value) -> u32 {
    unsafe { fixnum::rb_num2ulong(num) as u32 }
}
