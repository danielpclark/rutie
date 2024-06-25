use crate::{rubysys::types::Value, RString};

extern "C" {
    // VALUE
    // rb_marshal_dump(VALUE obj, VALUE port)
    pub fn rb_marshal_dump(val: Value, port: Value) -> Value;
    // VALUE
    // rb_marshal_load(VALUE port)
    pub fn rb_marshal_load(port: RString) -> Value;
}
