use crate::{
    binding::string,
    rubysys::{encoding, regexp},
    types::{c_char, c_int, c_long, Value},
    util,
};

pub fn new(pattern: &str, options: i32) -> Value {
    unsafe { regexp::rb_reg_new_str(string::new_utf8(pattern), options as c_int) }
}

pub fn options(regexp: Value) -> i32 {
    unsafe { regexp::rb_reg_options(regexp) as i32 }
}

// Character offset of the first match, or `nil`; sets `$~`.
pub fn match_offset(regexp: Value, string: Value) -> Value {
    unsafe { regexp::rb_reg_match(regexp, string) }
}

pub fn last_match_data() -> Value {
    unsafe { regexp::rb_backref_get() }
}

pub fn set_last_match_data(match_data: Value) {
    unsafe { regexp::rb_backref_set(match_data) }
}

pub fn nth_match(match_data: Value, nth: i32) -> Value {
    unsafe { regexp::rb_reg_nth_match(nth as c_int, match_data) }
}

// Raises `IndexError` for an unknown group name.
pub fn backref_number(match_data: Value, name: &str) -> i32 {
    unsafe { regexp::rb_reg_backref_number(match_data, string::new_utf8(name)) as i32 }
}

pub fn matched(match_data: Value) -> Value {
    unsafe { regexp::rb_reg_last_match(match_data) }
}

pub fn pre_match(match_data: Value) -> Value {
    unsafe { regexp::rb_reg_match_pre(match_data) }
}

pub fn post_match(match_data: Value) -> Value {
    unsafe { regexp::rb_reg_match_post(match_data) }
}

// Raises `RegexpError` for an invalid pattern.
pub fn new_with_encoding(pattern: &[u8], enc: Value, options: i32) -> Value {
    unsafe {
        regexp::rb_enc_reg_new(
            pattern.as_ptr() as *const c_char,
            pattern.len() as c_long,
            encoding::rb_to_encoding(enc),
            options as c_int,
        )
    }
}

pub fn quote(string: Value) -> Value {
    unsafe { regexp::rb_reg_quote(string) }
}

// Byte offset of the match, or `None`; sets `$~`.
pub fn search(regexp: Value, string: Value, byte_offset: usize, reverse: bool) -> Option<usize> {
    // Ruby returns -1 for a start past the end.
    let byte_offset = byte_offset.min(c_long::MAX as usize) as c_long;
    let found =
        unsafe { regexp::rb_reg_search(regexp, string, byte_offset, util::bool_to_c_int(reverse)) };

    if found < 0 {
        None
    } else {
        Some(found as usize)
    }
}

pub fn last_group(match_data: Value) -> Value {
    unsafe { regexp::rb_reg_match_last(match_data) }
}
