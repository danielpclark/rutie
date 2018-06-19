use rubysys::fixnum;

use types::{SignedValue, Value};

pub fn int_to_num(num: i64) -> Value {
    unsafe { fixnum::rb_int2inum(num as SignedValue) }
}

pub fn num_to_int(num: Value) -> i64 {
    unsafe { fixnum::rb_num2int(num) as i64 }
}
