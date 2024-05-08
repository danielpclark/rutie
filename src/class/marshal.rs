use crate::{
    binding::marshal,
    types::Value,
};

/// `Marshal`
#[derive(Debug)]
#[repr(C)]
pub struct Marshal {
    value: Value,
}

impl Marshal {
    pub fn load(port: Value) -> Value {
        marshal::marshal_load(port)
    }

    pub fn dump(val: Value, port: Value) -> Value {
        marshal::marshal_dump(val, port)
    }
}
