use rubysys::types::{c_int, c_long, SignedValue, Value};

extern "C" {
    // VALUE
    // rb_int2inum(intptr_t n)
    pub fn rb_int2inum(num: SignedValue) -> Value;
    // long
    // rb_num2int(VALUE val)
    pub fn rb_num2int(num: Value) -> c_int;
    // long
    // rb_num2long(VALUE val)
    pub fn rb_num2long(num: Value) -> c_long;
}
