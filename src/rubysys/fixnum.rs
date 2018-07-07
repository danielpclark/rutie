use rubysys::types::{c_long, SignedValue, Value};

extern "C" {
    // VALUE
    // rb_int2inum(intptr_t n)
    pub fn rb_int2inum(num: SignedValue) -> Value;
    // long
    // rb_num2int(VALUE val)
    pub fn rb_num2int(num: Value) -> c_long;
}
