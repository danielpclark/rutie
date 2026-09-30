use std::mem;

use crate::rubysys::{
    constant::{
        FL_USER_1, FL_USER_17, FL_USER_2, FL_USER_3, FL_USER_4, FL_USER_5, FL_USER_6, FL_USER_7,
        FL_USHIFT,
    },
    libc::size_t,
    types::{
        c_char, c_double, c_int, c_long, CallbackPtr, EncodingType, InternalValue, RBasic, Value,
    },
};

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
    // VALUE
    // rb_str_freeze(VALUE str)
    pub fn rb_str_freeze(string: Value) -> Value;
    // VALUE
    // rb_str_buf_append(VALUE str, VALUE str2)
    pub fn rb_str_buf_append(string: Value, other: Value) -> Value;
    // VALUE
    // rb_str_buf_new(long capa)
    pub fn rb_str_buf_new(capacity: c_long) -> Value;
    // size_t
    // rb_str_capacity(VALUE str)
    pub fn rb_str_capacity(string: Value) -> size_t;
    // int
    // rb_str_cmp(VALUE str1, VALUE str2)
    pub fn rb_str_cmp(string: Value, other: Value) -> c_int;
    // VALUE
    // rb_str_conv_enc(VALUE str, rb_encoding *from, rb_encoding *to)
    pub fn rb_str_conv_enc(string: Value, from: EncodingType, to: EncodingType) -> Value;
    // VALUE
    // rb_str_dup(VALUE str)
    pub fn rb_str_dup(string: Value) -> Value;
    // VALUE
    // rb_str_ellipsize(VALUE str, long len)
    pub fn rb_str_ellipsize(string: Value, len: c_long) -> Value;
    // VALUE
    // rb_str_equal(VALUE str1, VALUE str2)
    pub fn rb_str_equal(string: Value, other: Value) -> Value;
    // st_index_t
    // rb_str_hash(VALUE str)
    pub fn rb_str_hash(string: Value) -> size_t;
    // VALUE
    // rb_str_inspect(VALUE str)
    pub fn rb_str_inspect(string: Value) -> Value;
    // VALUE
    // rb_str_intern(VALUE str)
    pub fn rb_str_intern(string: Value) -> Value;
    // VALUE
    // rb_str_length(VALUE str)
    pub fn rb_str_length(string: Value) -> Value;
    // void
    // rb_str_modify(VALUE str)
    pub fn rb_str_modify(string: Value);
    // VALUE
    // rb_str_plus(VALUE str1, VALUE str2)
    pub fn rb_str_plus(string: Value, other: Value) -> Value;
    // VALUE
    // rb_str_replace(VALUE str, VALUE str2)
    pub fn rb_str_replace(string: Value, other: Value) -> Value;
    // VALUE
    // rb_str_resize(VALUE str, long len)
    //
    // Growing leaves the new bytes uninitialized.
    pub fn rb_str_resize(string: Value, len: c_long) -> Value;
    // VALUE
    // rb_str_scrub(VALUE str, VALUE repl)
    //
    // Returns `Qnil` when `str` has no invalid byte sequences.
    pub fn rb_str_scrub(string: Value, replacement: Value) -> Value;
    // void
    // rb_str_set_len(VALUE str, long len)
    pub fn rb_str_set_len(string: Value, len: c_long);
    // VALUE
    // rb_str_split(VALUE str, const char *sep0)
    pub fn rb_str_split(string: Value, separator: *const c_char) -> Value;
    // VALUE
    // rb_str_subseq(VALUE str, long beg, long len)
    //
    // Byte offsets, not bounds-checked.
    pub fn rb_str_subseq(string: Value, begin: c_long, len: c_long) -> Value;
    // VALUE
    // rb_str_substr(VALUE str, long beg, long len)
    //
    // Character offsets; `Qnil` when out of range.
    pub fn rb_str_substr(string: Value, begin: c_long, len: c_long) -> Value;
    // VALUE
    // rb_str_times(VALUE str, VALUE times)
    pub fn rb_str_times(string: Value, times: Value) -> Value;
    // double
    // rb_str_to_dbl(VALUE str, int badcheck)
    pub fn rb_str_to_dbl(string: Value, badcheck: c_int) -> c_double;
    // VALUE
    // rb_str_to_inum(VALUE str, int base, int badcheck)
    pub fn rb_str_to_inum(string: Value, base: c_int, badcheck: c_int) -> Value;
}

// #[link_name = "ruby_rstring_flags"]
#[derive(Debug, PartialEq)]
#[repr(C)]
enum RStringEmbed {
    NoEmbed = FL_USER_1,
    LenMask = FL_USER_2 | FL_USER_3 | FL_USER_4 | FL_USER_5 | FL_USER_6,
    LenShift = FL_USHIFT + 2,
    LenMax = (mem::size_of::<Value>() as isize * 3) / mem::size_of::<c_char>() as isize - 1,
    Fstr = FL_USER_17,
}

#[derive(Copy, Clone)]
#[repr(C)]
union RStringAs {
    heap: RStringHeap,
    ary: [c_char; RStringEmbed::LenMax as usize + 1],
}

#[derive(Copy, Clone)]
#[repr(C)]
union RStringAux {
    capa: c_long,
    value: InternalValue,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct RStringHeap {
    len: c_long,
    ptr: *const c_char,
    aux: RStringAux,
}

#[repr(C)]
struct RString {
    basic: RBasic,
    as_: RStringAs,
}

unsafe fn rstring_and_flags(value: Value) -> (*const RString, InternalValue) {
    let rstring: *const RString = value.value as *const RString;
    let flags = (*rstring).basic.flags;

    (rstring, flags)
}

unsafe fn embed_check(flags: InternalValue) -> bool {
    flags & (RStringEmbed::NoEmbed as size_t) == 0
}

pub unsafe fn rstring_embed_len(value: Value) -> c_long {
    let (_rstring, flags) = rstring_and_flags(value);

    ((flags as i64 >> RStringEmbed::LenShift as i64)
        & (RStringEmbed::LenMask as i64 >> RStringEmbed::LenShift as i64)) as c_long
}

pub unsafe fn rstring_len(value: Value) -> c_long {
    let (rstring, flags) = rstring_and_flags(value);

    if embed_check(flags) {
        rstring_embed_len(value)
    } else {
        (*rstring).as_.heap.len
    }
}

pub unsafe fn rstring_ptr(value: Value) -> *const c_char {
    let (rstring, flags) = rstring_and_flags(value);

    if embed_check(flags) {
        (*rstring).as_.ary.as_ptr()
    } else {
        (*rstring).as_.heap.ptr
    }
}

pub unsafe fn rstring_end(value: Value) -> *const c_char {
    let (rstring, flags) = rstring_and_flags(value);

    if embed_check(flags) {
        (*rstring)
            .as_
            .ary
            .as_ptr()
            .add(rstring_embed_len(value) as usize)
    } else {
        (*rstring)
            .as_
            .heap
            .ptr
            .add((*rstring).as_.heap.len as usize)
    }
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
    let (_rstring, flags) = rstring_and_flags(value);

    flags & STR_TMPLOCK as size_t != 0
}
