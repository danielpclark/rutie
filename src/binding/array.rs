use crate::{
    rubysys::array,
    types::{c_long, Value},
};

pub fn new() -> Value {
    unsafe { array::rb_ary_new().into() }
}

pub fn with_capacity(capacity: usize) -> Value {
    unsafe { array::rb_ary_new_capa(capacity as c_long).into() }
}

pub fn entry(array: Value, offset: i64) -> Value {
    unsafe { array::rb_ary_entry(array.into(), offset as c_long).into() }
}

pub fn join(array: Value, separator: Value) -> Value {
    unsafe { array::rb_ary_join(array.into(), separator.into()).into() }
}

pub fn len(array: Value) -> i64 {
    unsafe { array::rb_ary_len(array) as i64 }
}

pub fn push(array: Value, item: Value) -> Value {
    unsafe { array::rb_ary_push(array.into(), item.into()).into() }
}

pub fn store(array: Value, offset: i64, item: Value) {
    unsafe { array::rb_ary_store(array.into(), offset as c_long, item.into()) }
}

pub fn pop(array: Value) -> Value {
    unsafe { array::rb_ary_pop(array.into()).into() }
}

pub fn unshift(array: Value, item: Value) -> Value {
    unsafe { array::rb_ary_unshift(array.into(), item.into()).into() }
}

pub fn shift(array: Value) -> Value {
    unsafe { array::rb_ary_shift(array.into()).into() }
}

pub fn dup(array: Value) -> Value {
    unsafe { array::rb_ary_dup(array.into()).into() }
}

pub fn to_s(array: Value) -> Value {
    unsafe { array::rb_ary_to_s(array.into()).into() }
}

pub fn reverse_bang(array: Value) -> Value {
    unsafe { array::rb_ary_reverse(array.into()).into() }
}

pub fn concat(array: Value, other_array: Value) -> Value {
    unsafe { array::rb_ary_concat(array.into(), other_array.into()).into() }
}

pub fn sort(array: Value) -> Value {
    unsafe { array::rb_ary_sort(array.into()).into() }
}

pub fn sort_bang(array: Value) -> Value {
    unsafe { array::rb_ary_sort_bang(array.into()).into() }
}
