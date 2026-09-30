use crate::{
    rubysys::array,
    types::{c_long, Value},
};

pub fn new() -> Value {
    unsafe { array::rb_ary_new() }
}

pub fn with_capacity(capacity: usize) -> Value {
    unsafe { array::rb_ary_new_capa(capacity as c_long) }
}

pub fn entry(array: Value, offset: i64) -> Value {
    unsafe { array::rb_ary_entry(array, offset as c_long) }
}

pub fn join(array: Value, separator: Value) -> Value {
    unsafe { array::rb_ary_join(array, separator) }
}

pub fn len(array: Value) -> i64 {
    unsafe { array::rb_ary_len(array) as i64 }
}

pub fn push(array: Value, item: Value) -> Value {
    unsafe { array::rb_ary_push(array, item) }
}

pub fn store(array: Value, offset: i64, item: Value) {
    unsafe { array::rb_ary_store(array, offset as c_long, item) }
}

pub fn pop(array: Value) -> Value {
    unsafe { array::rb_ary_pop(array) }
}

pub fn unshift(array: Value, item: Value) -> Value {
    unsafe { array::rb_ary_unshift(array, item) }
}

pub fn shift(array: Value) -> Value {
    unsafe { array::rb_ary_shift(array) }
}

pub fn dup(array: Value) -> Value {
    unsafe { array::rb_ary_dup(array) }
}

pub fn to_s(array: Value) -> Value {
    unsafe { array::rb_ary_to_s(array) }
}

pub fn reverse_bang(array: Value) -> Value {
    unsafe { array::rb_ary_reverse(array) }
}

pub fn concat(array: Value, other_array: Value) -> Value {
    unsafe { array::rb_ary_concat(array, other_array) }
}

pub fn sort(array: Value) -> Value {
    unsafe { array::rb_ary_sort(array) }
}

pub fn sort_bang(array: Value) -> Value {
    unsafe { array::rb_ary_sort_bang(array) }
}

pub fn freeze(array: Value) -> Value {
    unsafe { array::rb_ary_freeze(array) }
}

pub fn assoc(array: Value, key: Value) -> Value {
    unsafe { array::rb_ary_assoc(array, key) }
}

pub fn clear(array: Value) {
    unsafe { array::rb_ary_clear(array) };
}

pub fn compare(array: Value, other: Value) -> Value {
    unsafe { array::rb_ary_cmp(array, other) }
}

pub fn delete(array: Value, item: Value) -> Value {
    unsafe { array::rb_ary_delete(array, item) }
}

pub fn delete_at(array: Value, index: i64) -> Value {
    unsafe { array::rb_ary_delete_at(array, index as c_long) }
}

pub fn includes(array: Value, item: Value) -> bool {
    unsafe { array::rb_ary_includes(array, item) }.is_true()
}

pub fn plus(array: Value, other: Value) -> Value {
    unsafe { array::rb_ary_plus(array, other) }
}

pub fn rassoc(array: Value, value: Value) -> Value {
    unsafe { array::rb_ary_rassoc(array, value) }
}

pub fn replace(array: Value, other: Value) -> Value {
    unsafe { array::rb_ary_replace(array, other) }
}

pub fn resize(array: Value, len: usize) -> Value {
    unsafe { array::rb_ary_resize(array, len as c_long) }
}

pub fn rotate(array: Value, count: i64) {
    unsafe { array::rb_ary_rotate(array, count as c_long) };
}

pub fn subseq(array: Value, begin: usize, len: usize) -> Value {
    unsafe { array::rb_ary_subseq(array, begin as c_long, len as c_long) }
}

pub fn check_array_type(object: Value) -> Value {
    unsafe { array::rb_check_array_type(object) }
}
