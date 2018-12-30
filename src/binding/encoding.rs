use rubysys::{encoding, string, vm};
use types::{c_char, size_t, c_int, Value, EncodingIndex, ValueType};
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

// best str1/str2 encoding or nil if incompatible
pub fn compatible_encoding(str1: Value, str2: Value) -> Value {
    unsafe { encoding::rb_enc_from_encoding(encoding::rb_enc_compatible(str1, str2)) }
}

pub fn is_compatible_encoding(str1: Value, str2: Value) -> bool {
    compatible_encoding(str1, str2).ty() != ValueType::Nil
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

// ptr - pointer for current point in string starting from the beginning
// end - pointer for the end of the string
// len_p - a mutable integer pointer for Ruby to give us how much we need to add on to `ptr`
// enc - the encoding the codepoints will be based on
pub fn next_codepoint(ptr: *const c_char, end: *const c_char, len_p: *mut c_int, enc: Value) -> usize {
    unsafe {
        encoding::rb_enc_codepoint_len(
            ptr,
            end,
            len_p,
            encoding::rb_to_encoding(enc)
        )
    }
}
