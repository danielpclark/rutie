use crate::{binding::symbol, rubysys::rstruct, types::Value};

pub fn alloc(klass: Value, values: Value) -> Value {
    unsafe { rstruct::rb_struct_alloc(klass, values) }
}

pub fn aref(object: Value, index: Value) -> Value {
    unsafe { rstruct::rb_struct_aref(object, index) }
}

pub fn aset(object: Value, index: Value, value: Value) -> Value {
    unsafe { rstruct::rb_struct_aset(object, index, value) }
}

pub fn get_member(object: Value, name: &str) -> Value {
    unsafe { rstruct::rb_struct_getmember(object, symbol::internal_id(name)) }
}

pub fn members(object: Value) -> Value {
    unsafe { rstruct::rb_struct_members(object) }
}

pub fn class_members(klass: Value) -> Value {
    unsafe { rstruct::rb_struct_s_members(klass) }
}

pub fn size(object: Value) -> Value {
    unsafe { rstruct::rb_struct_size(object) }
}
