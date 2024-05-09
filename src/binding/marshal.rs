use crate::{rubysys::marshal, types::Value};

pub fn marshal_dump(val: Value, port: Value) -> Value {
    unsafe { marshal::rb_marshal_dump(val, port) }
}

pub fn marshal_load(port: Value) -> Value {
    unsafe { marshal::rb_marshal_load(port) }
}
