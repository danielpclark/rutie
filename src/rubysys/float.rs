use rubysys::types::{c_double, Value};

extern "C" {
    // VALUE
    // rb_float_new(double d)
    pub fn rb_float_new(num: f64) -> Value;
    // double
    // rb_num2dbl(VALUE val)
    pub fn rb_num2dbl(num: Value) -> c_double;
}
