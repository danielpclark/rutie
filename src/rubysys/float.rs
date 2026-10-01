use crate::rubysys::types::{c_double, Value};

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_float_new(double d)
    pub fn rb_float_new(num: f64) -> Value;
    // VALUE
    // rb_float_new_in_heap(double d)
    //
    // Never a flonum.
    pub fn rb_float_new_in_heap(num: c_double) -> Value;
    // double
    // rb_float_value(VALUE num)
    //
    // `num` must be a Float.
    pub fn rb_float_value(num: Value) -> c_double;
    // double
    // rb_num2dbl(VALUE val)
    pub fn rb_num2dbl(num: Value) -> c_double;
    // VALUE
    // rb_to_float(VALUE val)
    pub fn rb_to_float(num: Value) -> Value;
}
