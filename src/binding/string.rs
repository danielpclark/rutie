use rubysys::string;

use types::{c_char, c_long, Value};
use util;

pub fn new(string: &str) -> Value {
    let str = string.as_ptr() as *const c_char;
    let len = string.len() as c_long;

    unsafe { string::rb_str_new(str, len) }
}

pub fn new_utf8(string: &str) -> Value {
    let str = string.as_ptr() as *const c_char;
    let len = string.len() as c_long;

    unsafe { string::rb_utf8_str_new(str, len) }
}

// Returns RString Value or NilClass Value
// same as method `String.try_convert`
pub fn method_to_str(str: Value) -> Value {
    unsafe { string::rb_check_string_type(str) }
}

pub fn value_to_string(value: Value) -> String {
    unsafe {
        let str = string::rb_string_value_cstr(&value);

        util::cstr_to_string(str)
    }
}

pub fn value_to_string_unchecked(value: Value) -> String {
    unsafe {
        let vec = value_to_bytes_unchecked(value).to_vec();

        String::from_utf8_unchecked(vec)
    }
}

pub fn value_to_str<'a>(value: Value) -> &'a str {
    unsafe {
        let str = string::rb_string_value_cstr(&value);

        util::cstr_to_str(str)
    }
}

pub fn value_to_bytes_unchecked<'a>(value: Value) -> &'a [u8] {
    unsafe {
        let str = string::rb_string_value_ptr(&value) as *const u8;
        let len = string::rb_str_len(value) as usize;

        ::std::slice::from_raw_parts(str, len)
    }
}

pub fn value_to_str_unchecked<'a>(value: Value) -> &'a str {
    unsafe {
        let slice = value_to_bytes_unchecked(value);

        ::std::str::from_utf8_unchecked(slice)
    }
}

pub fn bytesize(value: Value) -> i64 {
    unsafe { string::rb_str_len(value) as i64 }
}

pub fn concat(value: Value, bytes: &[u8]) -> Value {
    let str = bytes.as_ptr() as *const c_char;
    let len = bytes.len() as c_long;

    unsafe { string::rb_str_cat(value, str, len) }
}

pub fn is_lockedtmp(str: Value) -> bool {
    unsafe { string::is_lockedtmp(str) }
}

pub fn locktmp(str: Value) -> Value {
    unsafe { string::rb_str_locktmp(str) }
}

pub fn unlocktmp(str: Value) -> Value {
    unsafe { string::rb_str_unlocktmp(str) }
}
