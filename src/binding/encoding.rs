use rubysys::{encoding, string, vm};
use types::{c_int, Value, EncodingIndex};
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

pub fn coderange_clear(obj: Value) {
    unsafe { encoding::coderange_clear(obj) }
}

pub fn from_encoding_index(idx: EncodingIndex) -> Value {
    unsafe { encoding::rb_enc_from_encoding(encoding::rb_enc_from_index(idx.0)) }
}

pub fn usascii_encoding() -> Value {
    unsafe { from_encoding_index(EncodingIndex(encoding::rb_usascii_encindex())) }
}

pub fn utf8_encoding() -> Value {
    unsafe { from_encoding_index(EncodingIndex(encoding::rb_utf8_encindex())) }
}

pub fn enc_get_index(s: Value) -> EncodingIndex {
    let idx = unsafe { encoding::rb_enc_get_index(s) };

    EncodingIndex(idx)
}

pub fn find_encoding_index(name: &str) -> EncodingIndex {
    let cstr = CString::new(name).unwrap();
    let idx = unsafe { encoding::rb_enc_find_index(cstr.as_ptr()) };

    EncodingIndex(idx)
}

pub fn encode(str: Value, to: Value, ecflags: c_int, ecopts: Value) -> Value {
    unsafe { encoding::rb_str_encode(str, to, ecflags, ecopts) }
}

pub fn econv_prepare_opts(opthash: Value, opts: *const Value) -> c_int {
    unsafe { encoding::rb_econv_prepare_opts(opthash, opts) }
}
