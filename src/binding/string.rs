use crate::{
    rubysys::{encoding, string},
    types::{c_char, c_int, c_long, Value},
    util,
};

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

pub fn new_from_bytes(bytes: &[u8], enc: Value) -> Value {
    let bts = bytes.as_ptr() as *const c_char;
    let len = bytes.len() as c_long;

    unsafe { string::rb_enc_str_new(bts, len, encoding::rb_to_encoding(enc)) }
}

pub fn new_frozen(value: Value) -> Value {
    unsafe { string::rb_str_new_frozen(value) }
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
        let len = string::rstring_len(value) as usize;

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
    unsafe { string::rstring_len(value) as i64 }
}

pub fn count_chars(value: Value) -> i64 {
    unsafe { string::rb_str_strlen(value) as i64 }
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

pub fn freeze(value: Value) -> Value {
    unsafe { string::rb_str_freeze(value) }
}

// An empty UTF-8 string with room for `capacity` bytes.
pub fn with_capacity(capacity: usize) -> Value {
    unsafe {
        let value = string::rb_str_buf_new(capacity as c_long);
        encoding::rb_enc_associate_index(value, encoding::rb_utf8_encindex());

        value
    }
}

pub fn capacity(value: Value) -> usize {
    unsafe { string::rb_str_capacity(value) as usize }
}

pub fn compare(value: Value, other: Value) -> i32 {
    unsafe { string::rb_str_cmp(value, other) as i32 }
}

pub fn dup(value: Value) -> Value {
    unsafe { string::rb_str_dup(value) }
}

pub fn ellipsize(value: Value, len: usize) -> Value {
    unsafe { string::rb_str_ellipsize(value, len as c_long) }
}

pub fn plus(value: Value, other: Value) -> Value {
    unsafe { string::rb_str_plus(value, other) }
}

pub fn replace(value: Value, other: Value) -> Value {
    unsafe { string::rb_str_replace(value, other) }
}

// Only shrinks: growing through `rb_str_resize` would expose uninitialized bytes.
pub fn truncate(value: Value, len: usize) {
    if (len as i64) < bytesize(value) {
        unsafe { string::rb_str_resize(value, len as c_long) };
    }
}

pub fn scrub(value: Value, replacement: Value) -> Value {
    unsafe { string::rb_str_scrub(value, replacement) }
}

pub fn split(value: Value, separator: &str) -> Value {
    let separator = util::str_to_cstring(separator);

    unsafe { string::rb_str_split(value, separator.as_ptr()) }
}

// `None` unless `begin..begin + len` is within the string's bytes.
pub fn byte_slice(value: Value, begin: usize, len: usize) -> Option<Value> {
    let size = bytesize(value) as usize;

    match begin.checked_add(len) {
        Some(end) if end <= size => {
            Some(unsafe { string::rb_str_subseq(value, begin as c_long, len as c_long) })
        }
        _ => None,
    }
}

pub fn substr(value: Value, begin: i64, len: i64) -> Value {
    unsafe { string::rb_str_substr(value, begin as c_long, len as c_long) }
}

pub fn times(value: Value, times: Value) -> Value {
    unsafe { string::rb_str_times(value, times) }
}

pub fn to_f64(value: Value, strict: bool) -> f64 {
    unsafe { string::rb_str_to_dbl(value, util::bool_to_c_int(strict)) }
}

pub fn to_integer(value: Value, base: u32, strict: bool) -> Value {
    unsafe { string::rb_str_to_inum(value, base as c_int, util::bool_to_c_int(strict)) }
}

pub fn coderange(value: Value) -> c_int {
    unsafe { encoding::rb_enc_str_coderange(value) }
}
