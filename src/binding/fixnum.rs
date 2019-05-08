use rubysys::fixnum;
use types::Value;

pub fn i32_to_num(num: i32) -> Value {
    unsafe { fixnum::rb_int2inum(num as isize) }
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

pub fn num_to_i64(num: Value) -> i64 {
    unsafe { fixnum::rb_num2ll(num) }
}

pub fn num_to_u64(num: Value) -> u64 {
    unsafe { fixnum::rb_num2ull(num) }
}
