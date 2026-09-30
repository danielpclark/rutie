use crate::{
    binding::string,
    rubysys::regexp,
    types::{c_int, Value},
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
