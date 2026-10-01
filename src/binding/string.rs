use std::ffi::CStr;

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

fn bytes_ptr(bytes: &[u8]) -> (*const c_char, c_long) {
    (bytes.as_ptr() as *const c_char, bytes.len() as c_long)
}

pub fn new_external(bytes: &[u8]) -> Value {
    let (ptr, len) = bytes_ptr(bytes);

    unsafe { string::rb_external_str_new(ptr, len) }
}

pub fn new_external_with_encoding(bytes: &[u8], enc: Value) -> Value {
    let (ptr, len) = bytes_ptr(bytes);

    unsafe { string::rb_external_str_new_with_enc(ptr, len, encoding::rb_to_encoding(enc)) }
}

pub fn new_locale(bytes: &[u8]) -> Value {
    let (ptr, len) = bytes_ptr(bytes);

    unsafe { string::rb_locale_str_new(ptr, len) }
}

pub fn new_filesystem(bytes: &[u8]) -> Value {
    let (ptr, len) = bytes_ptr(bytes);

    unsafe { string::rb_filesystem_str_new(ptr, len) }
}

// Ruby uses the bytes in place, so they must live for the whole program,
// and reads the byte after them (the NUL). Wider terminators (UTF-16, ...)
// would be read past it, so ASCII-incompatible encodings get a copy.
pub fn new_static(string: &'static CStr, enc: Value) -> Value {
    let (ptr, len) = bytes_ptr(string.to_bytes());

    unsafe {
        let enc = encoding::rb_to_encoding(enc);

        if encoding::enc_asciicompat(enc) {
            string::rb_enc_str_new_static(ptr, len, enc)
        } else {
            string::rb_enc_str_new(ptr, len, enc)
        }
    }
}

pub fn new_static_utf8(string: &'static CStr) -> Value {
    let (ptr, len) = bytes_ptr(string.to_bytes());

    unsafe { string::rb_utf8_str_new_static(ptr, len) }
}

pub fn interned(bytes: &[u8], enc: Value) -> Value {
    let (ptr, len) = bytes_ptr(bytes);

    unsafe { string::rb_enc_interned_str(ptr, len, encoding::rb_to_encoding(enc)) }
}

pub fn to_interned(value: Value) -> Value {
    unsafe { string::rb_str_to_interned_str(value) }
}

pub fn append(value: Value, other: Value) -> Value {
    unsafe { string::rb_str_append(value, other) }
}

// `other` is a String or an Integer code point.
pub fn concat_object(value: Value, other: Value) -> Value {
    unsafe { string::rb_str_concat(value, other) }
}

pub fn is_comparable(value: Value, other: Value) -> bool {
    util::c_int_to_bool(unsafe { string::rb_str_comparable(value, other) })
}

pub fn is_eql(value: Value, other: Value) -> bool {
    unsafe { string::rb_str_hash_cmp(value, other) == 0 }
}

pub fn modify_expand(value: Value, additional: usize) {
    let additional = additional.min(c_long::MAX as usize) as c_long;

    unsafe { string::rb_str_modify_expand(value, additional) }
}

pub fn drop_bytes(value: Value, count: usize) -> Value {
    // Ruby stops at the end of the string.
    let count = count.min(bytesize(value) as usize) as c_long;

    unsafe { string::rb_str_drop_bytes(value, count) }
}

// Raises `IndexError` when `start` is outside the string.
pub fn update(value: Value, start: i64, len: usize, other: Value) {
    let len = len.min(c_long::MAX as usize) as c_long;

    unsafe { string::rb_str_update(value, start as c_long, len, other) }
}

pub fn byte_offset(value: Value, char_index: usize) -> usize {
    // A string has at most as many characters as bytes.
    let char_index = char_index.min(bytesize(value) as usize) as c_long;

    unsafe { string::rb_str_offset(value, char_index) as usize }
}

pub fn char_index(value: Value, byte_offset: usize) -> usize {
    // `rb_str_sublen` reads `byte_offset` bytes without checking the length.
    let byte_offset = byte_offset.min(bytesize(value) as usize) as c_long;

    unsafe { string::rb_str_sublen(value, byte_offset) as usize }
}

pub fn succ(value: Value) -> Value {
    unsafe { string::rb_str_succ(value) }
}

pub fn dump(value: Value) -> Value {
    unsafe { string::rb_str_dump(value) }
}

pub fn must_ascii_compatible(value: Value) {
    unsafe { string::rb_must_asciicompat(value) }
}

pub fn export(value: Value) -> Value {
    unsafe { string::rb_str_export(value) }
}

// Raises `TypeError` when `value` has no `to_str`.
pub fn to_str(value: Value) -> Value {
    unsafe { string::rb_str_to_str(value) }
}

// `Kernel#format`: `arguments[0]` is the format string.
pub fn format(arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { string::rb_f_sprintf(argc, argv) }
}

// The byte range of the last path component (without trailing separators)
// of `bytes`, and of that component without its extension.
pub fn path_basename(bytes: &[u8], enc: Value) -> Option<(usize, usize, usize)> {
    if bytes.is_empty() {
        return None;
    }

    // Ruby reads up to a NUL.
    let mut name = bytes.to_vec();
    name.push(0);

    let start = name.as_ptr() as *const c_char;
    let mut baselen: c_long = 0;
    let mut alllen: c_long = bytes.len() as c_long;

    let found = unsafe {
        encoding::ruby_enc_find_basename(
            start,
            &mut baselen,
            &mut alllen,
            encoding::rb_to_encoding(enc),
        )
    };

    // Only separators: Ruby points at the last one (`alllen` is -1).
    if alllen < 0 {
        alllen = baselen;
    }

    let offset = (found as isize).wrapping_sub(start as isize);

    if found.is_null()
        || offset < 0
        || baselen < 0
        || offset as usize + alllen as usize > bytes.len()
        || baselen > alllen
    {
        return None;
    }

    Some((offset as usize, alllen as usize, baselen as usize))
}

// The byte range of the extension (with its dot) of `bytes`.
pub fn path_extname(bytes: &[u8], enc: Value) -> Option<(usize, usize)> {
    let mut name = bytes.to_vec();
    name.push(0);

    let start = name.as_ptr() as *const c_char;
    let mut len: c_long = bytes.len() as c_long;

    let found =
        unsafe { encoding::ruby_enc_find_extname(start, &mut len, encoding::rb_to_encoding(enc)) };

    let offset = (found as isize).wrapping_sub(start as isize);

    if found.is_null() || len <= 0 || offset < 0 || offset as usize + len as usize > bytes.len() {
        return None;
    }

    Some((offset as usize, len as usize))
}
