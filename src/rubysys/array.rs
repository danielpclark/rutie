use super::{
    constant::{FL_USER_1, FL_USER_3, FL_USER_4, FL_USHIFT},
    types::{c_long, InternalValue, RBasic, Value},
};

use libc::size_t;
use std::mem;

extern "C" {
    // VALUE
    // rb_ary_concat(VALUE x, VALUE y)
    pub fn rb_ary_concat(array: Value, other_array: Value) -> Value;
    // VALUE
    // rb_ary_dup(VALUE ary)
    pub fn rb_ary_dup(array: Value) -> Value;
    // VALUE
    // rb_ary_entry(VALUE ary, long offset)
    pub fn rb_ary_entry(array: Value, offset: c_long) -> Value;
    // VALUE
    // rb_ary_join(VALUE ary, VALUE sep)
    pub fn rb_ary_join(array: Value, separator: Value) -> Value;
    // VALUE
    // rb_ary_new(void)
    pub fn rb_ary_new() -> Value;
    // VALUE
    // rb_ary_new_from_values(long n, const VALUE *elts)
    pub fn rb_ary_new_from_values(count: c_long, elements: *const Value) -> Value;
    // VALUE
    // rb_ary_new_capa(long capa)
    pub fn rb_ary_new_capa(capacity: c_long) -> Value;
    // VALUE
    // rb_ary_pop(VALUE ary)
    pub fn rb_ary_pop(array: Value) -> Value;
    // VALUE
    // rb_ary_push(VALUE ary, VALUE item)
    pub fn rb_ary_push(array: Value, item: Value) -> Value;
    // VALUE
    // rb_ary_reverse(VALUE ary)
    pub fn rb_ary_reverse(array: Value) -> Value;
    // VALUE
    // rb_ary_shift(VALUE ary)
    pub fn rb_ary_shift(array: Value) -> Value;
    // VALUE
    // rb_ary_sort_bang(VALUE ary)
    pub fn rb_ary_sort_bang(array: Value) -> Value;
    // VALUE
    // rb_ary_sort(VALUE ary)
    pub fn rb_ary_sort(array: Value) -> Value;
    // void
    // rb_ary_store(VALUE ary, long idx, VALUE val)
    pub fn rb_ary_store(array: Value, index: c_long, item: Value);
    // VALUE
    // rb_ary_to_s(VALUE ary)
    pub fn rb_ary_to_s(array: Value) -> Value;
    // VALUE
    // rb_ary_unshift(VALUE ary, VALUE item)
    pub fn rb_ary_unshift(array: Value, item: Value) -> Value;
}

pub unsafe fn rb_ary_len(value: Value) -> c_long {
    rb_sys::RARRAY_LEN(value) as c_long
}
