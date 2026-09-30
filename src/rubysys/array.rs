use crate::rubysys::{
    constant::{
        FL_USER_1, FL_USER_3, FL_USER_4, FL_USER_5, FL_USER_6, FL_USER_7, FL_USER_8, FL_USER_9,
        FL_USHIFT,
    },
    libc::size_t,
    types::{c_int, c_long, InternalValue, RBasic, Value},
};

use std::mem;

extern "C" {
    // VALUE
    // rb_ary_concat(VALUE x, VALUE y)
    pub fn rb_ary_concat(array: Value, other_array: Value) -> Value;
    // VALUE
    // rb_ary_dup(VALUE ary)
    pub fn rb_ary_dup(array: Value) -> Value;
    // VALUE
    // rb_ary_freeze(VALUE ary)
    pub fn rb_ary_freeze(array: Value) -> Value;
    // VALUE
    // rb_ary_aref(int argc, const VALUE *argv, VALUE ary)
    pub fn rb_ary_aref(argc: c_int, argv: *const Value, array: Value) -> Value;
    // VALUE
    // rb_ary_assoc(VALUE ary, VALUE key)
    pub fn rb_ary_assoc(array: Value, key: Value) -> Value;
    // VALUE
    // rb_ary_clear(VALUE ary)
    pub fn rb_ary_clear(array: Value) -> Value;
    // VALUE
    // rb_ary_cmp(VALUE ary1, VALUE ary2)
    pub fn rb_ary_cmp(array: Value, other: Value) -> Value;
    // VALUE
    // rb_ary_delete(VALUE ary, VALUE item)
    pub fn rb_ary_delete(array: Value, item: Value) -> Value;
    // VALUE
    // rb_ary_delete_at(VALUE ary, long pos)
    pub fn rb_ary_delete_at(array: Value, position: c_long) -> Value;
    // VALUE
    // rb_ary_includes(VALUE ary, VALUE item)
    pub fn rb_ary_includes(array: Value, item: Value) -> Value;
    // VALUE
    // rb_ary_plus(VALUE x, VALUE y)
    pub fn rb_ary_plus(array: Value, other: Value) -> Value;
    // VALUE
    // rb_ary_rassoc(VALUE ary, VALUE value)
    pub fn rb_ary_rassoc(array: Value, value: Value) -> Value;
    // VALUE
    // rb_ary_replace(VALUE copy, VALUE orig)
    pub fn rb_ary_replace(copy: Value, original: Value) -> Value;
    // VALUE
    // rb_ary_resize(VALUE ary, long len)
    pub fn rb_ary_resize(array: Value, len: c_long) -> Value;
    // VALUE
    // rb_ary_rotate(VALUE ary, long cnt)
    pub fn rb_ary_rotate(array: Value, count: c_long) -> Value;
    // VALUE
    // rb_ary_subseq(VALUE ary, long beg, long len)
    //
    // `Qnil` when out of range.
    pub fn rb_ary_subseq(array: Value, begin: c_long, len: c_long) -> Value;
    // VALUE
    // rb_ary_to_ary(VALUE obj)
    pub fn rb_ary_to_ary(object: Value) -> Value;
    // VALUE
    // rb_check_array_type(VALUE ary)
    pub fn rb_check_array_type(object: Value) -> Value;
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

// #[link_name = "ruby_rarray_flags"]
#[derive(Debug, PartialEq)]
#[repr(C)]
enum RArrayEmbed {
    Flag = FL_USER_1,
    // Ruby 3.2 (`USE_RVARGC`) embeds longer arrays and widens the mask.
    #[cfg(not(ruby_gte_3_2))]
    LenMask = FL_USER_4 | FL_USER_3,
    #[cfg(ruby_gte_3_2)]
    LenMask = FL_USER_9 | FL_USER_8 | FL_USER_7 | FL_USER_6 | FL_USER_5 | FL_USER_4 | FL_USER_3,
    LenShift = FL_USHIFT + 3,
}

#[repr(C)]
struct RArrayAs {
    heap: RArrayHeap,
}

#[repr(C)]
struct RArrayHeap {
    len: c_long,
    // Really, this is a union but value is the largest item.
    value: InternalValue,
    ptr: InternalValue,
}

#[repr(C)]
struct RArray {
    basic: RBasic,
    as_: RArrayAs,
}

pub unsafe fn rb_ary_len(value: Value) -> c_long {
    let rarray: *const RArray = mem::transmute(value.value);
    let flags = (*rarray).basic.flags;

    if flags & (RArrayEmbed::Flag as size_t) == 0 {
        (*rarray).as_.heap.len
    } else {
        ((flags as i64 >> RArrayEmbed::LenShift as i64)
            & (RArrayEmbed::LenMask as i64 >> RArrayEmbed::LenShift as i64)) as c_long
    }
}
