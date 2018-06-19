use rubysys::types::{c_double, Value};

extern "C" {
    pub fn rb_float_new(num: f64) -> Value;
    pub fn rb_num2dbl(num: Value) -> c_double;
}
