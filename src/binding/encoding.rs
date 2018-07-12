use rubysys::{encoding, string};
use types::{c_int, Value};
use std::ffi::CString;
use util;

pub fn default_external() -> Value {
    unsafe { encoding::rb_enc_default_external() }
}

pub fn default_internal() -> Value {
    unsafe { encoding::rb_enc_default_internal() }
}

pub fn force_encoding(s: Value, enc: Value) -> Value {
    unsafe { encoding::rb_enc_associate(s, encoding::rb_to_encoding(enc)) }
}

pub fn from_encoding_index(idx: c_int) -> Value {
    unsafe { encoding::rb_enc_from_encoding(encoding::rb_enc_from_index(idx)) }
}

pub fn usascii_encoding() -> Value {
    unsafe { from_encoding_index(encoding::rb_usascii_encindex()) }
}

pub fn utf8_encoding() -> Value {
    unsafe { from_encoding_index(encoding::rb_utf8_encindex()) }
}

pub fn enc_get_index(s: Value) -> c_int {
    unsafe { encoding::rb_enc_get_index(s) }
}

pub fn find_encoding_index(name: &str) -> c_int {
    let cstr = CString::new(name).unwrap();

    unsafe { encoding::rb_enc_find_index(cstr.as_ptr()) }
}
