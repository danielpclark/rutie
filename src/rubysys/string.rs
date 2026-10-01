use std::mem;

use crate::rubysys::{
    constant::{FL_USER_1, FL_USER_17, FL_USER_7},
    libc::size_t,
    types::{
        c_char, c_double, c_int, c_long, c_void, CallbackPtr, EncodingType, InternalValue, RBasic,
        Value,
    },
};

pub const STR_TMPLOCK: isize = FL_USER_7;

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
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
    pub fn rb_enc_str_asciionly_p(str: Value) -> c_int;
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
    // VALUE
    // rb_external_str_new(const char *ptr, long len)
    pub fn rb_external_str_new(ptr: *const c_char, len: c_long) -> Value;
    // VALUE
    // rb_external_str_new_cstr(const char *ptr)
    pub fn rb_external_str_new_cstr(ptr: *const c_char) -> Value;
    // VALUE
    // rb_locale_str_new(const char *ptr, long len)
    pub fn rb_locale_str_new(ptr: *const c_char, len: c_long) -> Value;
    // VALUE
    // rb_locale_str_new_cstr(const char *ptr)
    pub fn rb_locale_str_new_cstr(ptr: *const c_char) -> Value;
    // VALUE
    // rb_filesystem_str_new(const char *ptr, long len)
    pub fn rb_filesystem_str_new(ptr: *const c_char, len: c_long) -> Value;
    // VALUE
    // rb_filesystem_str_new_cstr(const char *ptr)
    pub fn rb_filesystem_str_new_cstr(ptr: *const c_char) -> Value;
    // VALUE
    // rb_usascii_str_new(const char *ptr, long len)
    pub fn rb_usascii_str_new(ptr: *const c_char, len: c_long) -> Value;
    // VALUE
    // rb_usascii_str_new_cstr(const char *ptr)
    pub fn rb_usascii_str_new_cstr(ptr: *const c_char) -> Value;
    // VALUE
    // rb_str_new_static(const char *ptr, long len)
    //
    // Uses `ptr` without copying it; it must outlive the string.
    pub fn rb_str_new_static(ptr: *const c_char, len: c_long) -> Value;
    // VALUE
    // rb_usascii_str_new_static(const char *ptr, long len)
    pub fn rb_usascii_str_new_static(ptr: *const c_char, len: c_long) -> Value;
    // VALUE
    // rb_utf8_str_new_static(const char *ptr, long len)
    pub fn rb_utf8_str_new_static(ptr: *const c_char, len: c_long) -> Value;
    // VALUE
    // rb_str_new_with_class(VALUE obj, const char *ptr, long len)
    //
    // A string of the same class as `obj`.
    pub fn rb_str_new_with_class(obj: Value, ptr: *const c_char, len: c_long) -> Value;
    // VALUE
    // rb_str_new_shared(VALUE str)
    pub fn rb_str_new_shared(str: Value) -> Value;
    // VALUE
    // rb_str_resurrect(VALUE str)
    pub fn rb_str_resurrect(str: Value) -> Value;
    // VALUE
    // rb_str_tmp_new(long len)
    //
    // A hidden (class-less) string of `len` uninitialized bytes.
    pub fn rb_str_tmp_new(len: c_long) -> Value;
    // VALUE
    // rb_str_buf_new_cstr(const char *ptr)
    pub fn rb_str_buf_new_cstr(ptr: *const c_char) -> Value;
    // VALUE
    // rb_interned_str(const char *ptr, long len)
    //
    // A frozen, deduplicated US-ASCII (ASCII-8BIT for non-ASCII bytes) string.
    pub fn rb_interned_str(ptr: *const c_char, len: c_long) -> Value;
    // VALUE
    // rb_interned_str_cstr(const char *ptr)
    pub fn rb_interned_str_cstr(ptr: *const c_char) -> Value;
    // VALUE
    // rb_str_to_interned_str(VALUE str)
    pub fn rb_str_to_interned_str(str: Value) -> Value;
    // void
    // rb_str_free(VALUE str)
    pub fn rb_str_free(str: Value);
    // void
    // rb_str_shared_replace(VALUE dst, VALUE src)
    //
    // `dst` takes over `src`'s buffer.
    pub fn rb_str_shared_replace(dst: Value, src: Value);
    // VALUE
    // rb_str_buf_cat(VALUE, const char*, long)
    pub fn rb_str_buf_cat(dst: Value, src: *const c_char, len: c_long) -> Value;
    // VALUE
    // rb_str_buf_cat2(VALUE, const char*)
    pub fn rb_str_buf_cat2(dst: Value, src: *const c_char) -> Value;
    // VALUE
    // rb_str_cat2(VALUE, const char*)
    pub fn rb_str_cat2(dst: Value, src: *const c_char) -> Value;
    // VALUE
    // rb_str_cat_cstr(VALUE dst, const char *src)
    pub fn rb_str_cat_cstr(dst: Value, src: *const c_char) -> Value;
    // VALUE
    // rb_str_buf_cat_ascii(VALUE dst, const char *src)
    //
    // Appends ASCII text, converting it to `dst`'s encoding.
    pub fn rb_str_buf_cat_ascii(dst: Value, src: *const c_char) -> Value;
    // VALUE
    // rb_str_append(VALUE dst, VALUE src)
    pub fn rb_str_append(dst: Value, src: Value) -> Value;
    // VALUE
    // rb_str_concat(VALUE dst, VALUE src)
    //
    // `src` may be an Integer code point.
    pub fn rb_str_concat(dst: Value, src: Value) -> Value;
    // st_index_t
    // rb_memhash(const void *ptr, long len)
    pub fn rb_memhash(ptr: *const c_void, len: c_long) -> size_t;
    // st_index_t
    // rb_hash_start(st_index_t i)
    pub fn rb_hash_start(i: size_t) -> size_t;
    // int
    // rb_str_hash_cmp(VALUE str1, VALUE str2)
    //
    // 0 when the strings are `eql?`.
    pub fn rb_str_hash_cmp(str1: Value, str2: Value) -> c_int;
    // int
    // rb_str_comparable(VALUE str1, VALUE str2)
    pub fn rb_str_comparable(str1: Value, str2: Value) -> c_int;
    // VALUE
    // rb_str_dup_frozen(VALUE)
    pub fn rb_str_dup_frozen(str: Value) -> Value;
    // void
    // rb_str_modify_expand(VALUE str, long capa)
    //
    // Makes room for `capa` more bytes (without changing the length).
    pub fn rb_str_modify_expand(str: Value, capa: c_long);
    // VALUE
    // rb_str_drop_bytes(VALUE str, long len)
    pub fn rb_str_drop_bytes(str: Value, len: c_long) -> Value;
    // void
    // rb_str_update(VALUE dst, long beg, long len, VALUE src)
    //
    // `dst[beg, len] = src`, character offsets.
    pub fn rb_str_update(dst: Value, beg: c_long, len: c_long, src: Value);
    // long
    // rb_str_offset(VALUE str, long pos)
    //
    // Byte offset of character `pos`.
    pub fn rb_str_offset(str: Value, pos: c_long) -> c_long;
    // long
    // rb_str_sublen(VALUE str, long pos)
    //
    // Characters in the first `pos` bytes; `pos` is not bounds-checked.
    pub fn rb_str_sublen(str: Value, pos: c_long) -> c_long;
    // char *
    // rb_str_subpos(VALUE str, long beg, long *len)
    pub fn rb_str_subpos(str: Value, beg: c_long, len: *mut c_long) -> *mut c_char;
    // VALUE
    // rb_str_succ(VALUE orig)
    pub fn rb_str_succ(orig: Value) -> Value;
    // VALUE
    // rb_str_dump(VALUE str)
    pub fn rb_str_dump(str: Value) -> Value;
    // void
    // rb_must_asciicompat(VALUE obj)
    pub fn rb_must_asciicompat(obj: Value);
    // VALUE
    // rb_sym_to_s(VALUE sym)
    pub fn rb_sym_to_s(sym: Value) -> Value;
    // VALUE
    // rb_str_export(VALUE obj)
    pub fn rb_str_export(obj: Value) -> Value;
    // VALUE
    // rb_str_to_str(VALUE obj)
    pub fn rb_str_to_str(obj: Value) -> Value;
    // VALUE
    // rb_string_value(volatile VALUE *ptr)
    pub fn rb_string_value(ptr: *mut Value) -> Value;
    // void
    // rb_debug_rstring_null_ptr(const char *func)
    pub fn rb_debug_rstring_null_ptr(func: *const c_char);
    // VALUE
    // rb_enc_str_new_cstr(const char *ptr, rb_encoding *enc)
    pub fn rb_enc_str_new_cstr(ptr: *const c_char, enc: EncodingType) -> Value;
    // VALUE
    // rb_enc_str_new_static(const char *ptr, long len, rb_encoding *enc)
    pub fn rb_enc_str_new_static(ptr: *const c_char, len: c_long, enc: EncodingType) -> Value;
    // VALUE
    // rb_enc_interned_str(const char *ptr, long len, rb_encoding *enc)
    pub fn rb_enc_interned_str(ptr: *const c_char, len: c_long, enc: EncodingType) -> Value;
    // VALUE
    // rb_enc_interned_str_cstr(const char *ptr, rb_encoding *enc)
    pub fn rb_enc_interned_str_cstr(ptr: *const c_char, enc: EncodingType) -> Value;
    // VALUE
    // rb_external_str_new_with_enc(const char *ptr, long len, rb_encoding *enc)
    pub fn rb_external_str_new_with_enc(
        ptr: *const c_char,
        len: c_long,
        enc: EncodingType,
    ) -> Value;
    // long
    // rb_enc_strlen(const char *head, const char *tail, rb_encoding *enc)
    pub fn rb_enc_strlen(head: *const c_char, tail: *const c_char, enc: EncodingType) -> c_long;
    // long
    // rb_memsearch(const void *x, long m, const void *y, long n, rb_encoding *enc)
    //
    // Byte offset of `x` in `y`, or -1.
    pub fn rb_memsearch(
        x: *const c_void,
        m: c_long,
        y: *const c_void,
        n: c_long,
        enc: EncodingType,
    ) -> c_long;
    // long
    // rb_str_coderange_scan_restartable(const char *str, const char *end, rb_encoding *enc, int *cr)
    pub fn rb_str_coderange_scan_restartable(
        str: *const c_char,
        end: *const c_char,
        enc: EncodingType,
        cr: *mut c_int,
    ) -> c_long;
    // VALUE
    // rb_str_conv_enc_opts(VALUE str, rb_encoding *from, rb_encoding *to, int ecflags, VALUE ecopts)
    pub fn rb_str_conv_enc_opts(
        str: Value,
        from: EncodingType,
        to: EncodingType,
        ecflags: c_int,
        ecopts: Value,
    ) -> Value;
    // VALUE
    // rb_sprintf(const char *fmt, ...)
    pub fn rb_sprintf(fmt: *const c_char, ...) -> Value;
    // VALUE
    // rb_str_catf(VALUE dst, const char *fmt, ...)
    pub fn rb_str_catf(dst: Value, fmt: *const c_char, ...) -> Value;
    // VALUE
    // rb_f_sprintf(int argc, const VALUE *argv)
    //
    // `Kernel#format`: `argv[0]` is the format string.
    pub fn rb_f_sprintf(argc: c_int, argv: *const Value) -> Value;
    // VALUE
    // rb_enc_sprintf(rb_encoding *enc, const char *fmt, ...)
    pub fn rb_enc_sprintf(enc: EncodingType, fmt: *const c_char, ...) -> Value;
    // void
    // rb_enc_raise(rb_encoding *enc, VALUE exc, const char *fmt, ...)
    pub fn rb_enc_raise(enc: EncodingType, exc: Value, fmt: *const c_char, ...) -> !;
}

// #[link_name = "ruby_rstring_flags"]
#[derive(Debug, PartialEq)]
#[repr(C)]
enum RStringEmbed {
    NoEmbed = FL_USER_1,
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

// The length is at the top level for every string (Ruby 3.3 and later);
// `as.heap` keeps `ptr` and `aux`, and an embedded string's bytes start at
// `as`.
#[derive(Copy, Clone)]
#[repr(C)]
struct RStringHeap {
    ptr: *const c_char,
    aux: RStringAux,
}

#[repr(C)]
struct RString {
    basic: RBasic,
    len: c_long,
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
    let (rstring, _flags) = rstring_and_flags(value);

    (*rstring).len
}

// An embedded string's `as.embed.ary` is all of `as`.
unsafe fn rstring_embed_ptr(rstring: *const RString) -> *const c_char {
    &(*rstring).as_ as *const RStringAs as *const c_char
}

pub unsafe fn rstring_len(value: Value) -> c_long {
    let (rstring, _flags) = rstring_and_flags(value);

    (*rstring).len
}

pub unsafe fn rstring_ptr(value: Value) -> *const c_char {
    let (rstring, flags) = rstring_and_flags(value);

    if embed_check(flags) {
        rstring_embed_ptr(rstring)
    } else {
        (*rstring).as_.heap.ptr
    }
}

pub unsafe fn rstring_end(value: Value) -> *const c_char {
    rstring_ptr(value).add(rstring_len(value) as usize)
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

#[cfg(test)]
mod tests {
    use super::{rstring_end, rstring_len, rstring_ptr};
    use crate::{Array, Fixnum, Object, RString, VM};

    // Strings of every length across the embedded/heap boundary (the slot
    // size), made in several ways, read directly and compared with what
    // Ruby reports.
    #[test]
    fn test_direct_rstring_reads() {
        crate::on_ruby_thread(|| {
            let strings = VM::eval(
                "(0..700).step(1).flat_map do |n|
                   s = (('a'..'z').to_a.join * 30)[0, n]
                   [s, s.dup, String.new(s, capacity: 1000), \"#{s}\", (\"_\" + s)[1..], s.b]
                 end",
            )
            .unwrap();
            let strings = Array::from(strings.value());

            for i in 0..strings.length() {
                let string = strings.at(i as i64);
                let value = string.value();
                let bytesize = unsafe { string.send("bytesize", &[]) };
                let expected = bytesize.try_convert_to::<Fixnum>().unwrap().to_i64();
                let len = unsafe { rstring_len(value) };

                assert_eq!(len as i64, expected, "length of string #{}", i);

                let bytes = unsafe {
                    std::slice::from_raw_parts(rstring_ptr(value) as *const u8, len as usize)
                };
                let alphabet: Vec<u8> = (b'a'..=b'z').cycle().take(len as usize).collect();
                assert_eq!(bytes, &alphabet[..], "bytes of string #{}", i);

                assert_eq!(
                    unsafe { rstring_end(value).offset_from(rstring_ptr(value)) },
                    len as isize
                );
            }

            let empty = RString::new_utf8("");
            assert_eq!(unsafe { rstring_len(empty.value()) }, 0);
        });
    }
}
