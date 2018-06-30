use rubysys::libc::size_t;
use std::mem;

use rubysys::constant::{FL_USHIFT, FL_USER_1, FL_USER_2, FL_USER_3, FL_USER_4, FL_USER_5, FL_USER_6};
use rubysys::types::{c_char, c_long, InternalValue, RBasic, Value};

extern "C" {
    pub fn rb_str_new(str: *const c_char, len: c_long) -> Value;
    pub fn rb_str_new_cstr(str: *const c_char) -> Value;
    pub fn rb_utf8_str_new(str: *const c_char, len: c_long) -> Value;
    pub fn rb_string_value_cstr(str: *const Value) -> *const c_char;
    pub fn rb_string_value_ptr(str: *const Value) -> *const c_char;
    pub fn rb_str_ascii_only_p(str: Value) -> bool;
    pub fn rb_str_export_locale(str: Value) -> Value;
    pub fn rb_str_valid_encoding_p(str: Value) -> bool;
    pub fn rb_str_cat(str: Value, ptr: *const c_char, len: c_long) -> Value;
}

#[repr(C)]
enum RStringEmbed {
    NoEmbed = FL_USER_1,
    LenMask = FL_USER_2 | FL_USER_3 | FL_USER_4 | FL_USER_5 | FL_USER_6,
    LenShift = FL_USHIFT + 2,
}

#[repr(C)]
struct RStringAs {
    heap: RStringHeap,
}

#[repr(C)]
struct RStringHeap {
    len: c_long,
    // Really, this is a union but value is the largest item.
    value: InternalValue,
    ptr: InternalValue,
}

#[repr(C)]
struct RString {
    basic: RBasic,
    as_: RStringAs,
}

pub unsafe fn rb_str_len(value: Value) -> c_long {
    let rstring: *const RString = mem::transmute(value.value);
    let flags = (*rstring).basic.flags;

    if flags & (RStringEmbed::NoEmbed as size_t) == 0 {
        ((flags as i64 >> RStringEmbed::LenShift as i64) &
         (RStringEmbed::LenMask as i64 >> RStringEmbed::LenShift as i64)) as c_long
    } else {
        (*rstring).as_.heap.len
    }
}
