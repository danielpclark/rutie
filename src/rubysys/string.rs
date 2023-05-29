use std::mem;

use super::{
    constant::FL_USER_7,
    types::{c_char, c_long, EncodingType, InternalValue, Value},
};
use rb_sys::RString;
pub const STR_TMPLOCK: isize = FL_USER_7;

extern "C" {
    // VALUE
    // rb_str_new(const char *ptr, long len)
    pub fn rb_str_new(str: *const c_char, len: c_long) -> Value;
    // VALUE
    // rb_str_new_cstr(const char *ptr)
    pub fn rb_str_new_cstr(str: *const c_char) -> Value;
    // VALUE
    // rb_utf8_str_new(const char *ptr, long len)
    pub fn rb_utf8_str_new(str: *const c_char, len: c_long) -> Value;
    // VALUE
    // rb_utf8_str_new_cstr(const char *ptr)
    pub fn rb_utf8_str_new_cstr(str: *const c_char) -> Value;
    // char *
    // rb_string_value_cstr(volatile VALUE *ptr)
    pub fn rb_string_value_cstr(str: *const Value) -> *const c_char;
    // char *
    // rb_string_value_ptr(volatile VALUE *ptr)
    pub fn rb_string_value_ptr(str: *const Value) -> *const c_char;
    // long
    // rb_str_strlen(VALUE str)
    pub fn rb_str_strlen(str: Value) -> c_long;
    // int
    // rb_enc_str_asciionly_p(VALUE str)
    pub fn rb_enc_str_asciionly_p(str: Value) -> bool;
    // VALUE
    // rb_enc_str_new(const char *ptr, long len, rb_encoding *enc)
    pub fn rb_enc_str_new(str: *const c_char, len: c_long, enc: EncodingType) -> Value;
    // VALUE
    // rb_str_export_locale(VALUE str)
    pub fn rb_str_export_locale(str: Value) -> Value;
    // static VALUE
    // rb_str_valid_encoding_p(VALUE str)
    pub fn rb_str_valid_encoding_p(str: Value) -> bool;
    // VALUE
    // rb_str_cat(VALUE str, const char *ptr, long len)
    pub fn rb_str_cat(str: Value, ptr: *const c_char, len: c_long) -> Value;
    // VALUE
    // rb_check_string_type(VALUE str)
    pub fn rb_check_string_type(str: Value) -> Value;
    //-------------------------------------------------------------
    // LINKER CANNOT FIND
    // //
    // //  call-seq:
    // //     str.force_encoding(encoding)   -> str
    // //
    // //  Changes the encoding to +encoding+ and returns self.
    // //
    // // static VALUE
    // // rb_str_force_encoding(VALUE str, VALUE enc)
    // pub fn rb_str_force_encoding(s: Value, enc: Value) -> Value;
    //-------------------------------------------------------------
    // VALUE
    // rb_str_locktmp(VALUE str)
    pub fn rb_str_locktmp(str: Value) -> Value;
    // VALUE
    // rb_str_unlocktmp(VALUE str)
    pub fn rb_str_unlocktmp(str: Value) -> Value;
    // VALUE
    // rb_str_new_frozen(VALUE orig)
    pub fn rb_str_new_frozen(orig: Value) -> Value;
}

unsafe fn rstring_and_flags(value: Value) -> (*const RString, InternalValue) {
    let rstring: *const RString = mem::transmute(value.value);
    let flags = (*rstring).basic.flags;

    (rstring, flags)
}

pub unsafe fn rstring_len(value: Value) -> c_long {
    rb_sys::RSTRING_LEN(value.into())
}

pub unsafe fn rstring_ptr(value: Value) -> *const c_char {
    rb_sys::RSTRING_PTR(value.into())
}

pub unsafe fn rstring_end(value: Value) -> *const c_char {
    let ptr = rstring_ptr(value) as *const c_char;
    let len = rstring_len(value) as usize;

    &*ptr.add(len)
}

// ```
// use rutie::VM;
// # VM::init();
//
// use rutie::binding::string::*; // binding not public
//
// let word = new_utf8("word");
// unsafe {
//     assert!(!is_locktmp(word), "word should not be locktmp but is");
//     locktmp(word);
//     assert!(is_locktmp(word), "word should be locktmp but is not");
//     unlocktmp(word);
//     assert!(!is_locktmp(word), "word should not be locktmp but is");
// }
// ```
pub unsafe fn is_lockedtmp(value: Value) -> bool {
    let (_, flags) = rstring_and_flags(value);

    flags & STR_TMPLOCK as u64 != 0
}
