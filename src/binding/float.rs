use rubysys::float;

use types::Value;

pub fn float_to_num(num: f64) -> Value {
    unsafe { float::rb_float_new(num) }
}

pub fn num_to_float(num: Value) -> f64 {
    unsafe { float::rb_num2dbl(num) as f64 }
}
