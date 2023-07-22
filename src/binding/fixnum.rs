use crate::{rubysys::fixnum, types::Value};

pub fn i32_to_num(num: i32) -> Value {
    unsafe { fixnum::rb_int2inum(num as isize) }
}

pub fn u32_to_num(num: u32) -> Value {
    unsafe { fixnum::rb_uint2inum(num as usize) }
}

#[allow(dead_code)]
pub fn isize_to_num(num: isize) -> Value {
    unsafe { fixnum::rb_int2inum(num) }
}

#[allow(dead_code)]
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

pub fn num_to_u32(num: Value) -> u32 {
    unsafe { fixnum::rb_num2long(num) as u32 }
}

#[allow(dead_code)]
pub fn num_to_isize(num: Value) -> isize {
    unsafe { fixnum::rb_num2long(num) as isize }
}

#[allow(dead_code)]
pub fn num_to_usize(num: Value) -> usize {
    unsafe { fixnum::rb_num2ulong(num) as usize }
}

pub fn num_to_i64(num: Value) -> i64 {
    unsafe { fixnum::rb_num2ll(num) }
}

pub fn num_to_u64(num: Value) -> u64 {
    unsafe { fixnum::rb_num2ull(num) }
}
