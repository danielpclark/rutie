use crate::rubysys::{
    constant::{FL_USER_8, FL_USER_9},
    types::{
        c_char, c_int, c_long, c_uint, size_t, CallbackPtr, EncodingIndex, EncodingType,
        InternalValue, RBasic, Value,
    },
};
use std::mem;

pub const ENC_DUMMY_FLAG: isize = 1 << 24;
pub const ENC_INDEX_MASK: isize = !(!0 << 24);
pub const ENC_CODERANGE_UNKNOWN: isize = 0;
pub const ENC_CODERANGE_7BIT: isize = FL_USER_8;
pub const ENC_CODERANGE_VALID: isize = FL_USER_9;
pub const ENC_CODERANGE_BROKEN: isize = FL_USER_8 | FL_USER_9;
pub const ENC_CODERANGE_MASK: isize =
    ENC_CODERANGE_7BIT | ENC_CODERANGE_VALID | ENC_CODERANGE_BROKEN;

// `rb_econv_t`, Ruby's transcoder (opaque).
#[repr(C)]
pub struct RbEconv {
    _private: [u8; 0],
}

// `rb_econv_result_t`, what `rb_econv_convert` stopped at.
pub type EconvResult = c_int;

pub const ECONV_RESULT_INVALID_BYTE_SEQUENCE: EconvResult = 0;
pub const ECONV_RESULT_UNDEFINED_CONVERSION: EconvResult = 1;
pub const ECONV_RESULT_DESTINATION_BUFFER_FULL: EconvResult = 2;
pub const ECONV_RESULT_SOURCE_BUFFER_EMPTY: EconvResult = 3;
pub const ECONV_RESULT_FINISHED: EconvResult = 4;
pub const ECONV_RESULT_AFTER_OUTPUT: EconvResult = 5;
pub const ECONV_RESULT_INCOMPLETE_INPUT: EconvResult = 6;

// `enum ruby_econv_flag_type`
pub const ECONV_ERROR_HANDLER_MASK: c_int = 0x000000ff;
pub const ECONV_INVALID_MASK: c_int = 0x0000000f;
pub const ECONV_INVALID_REPLACE: c_int = 0x00000002;
pub const ECONV_UNDEF_MASK: c_int = 0x000000f0;
pub const ECONV_UNDEF_REPLACE: c_int = 0x00000020;
pub const ECONV_UNDEF_HEX_CHARREF: c_int = 0x00000030;
pub const ECONV_DECORATOR_MASK: c_int = 0x0001ff00;
pub const ECONV_NEWLINE_DECORATOR_MASK: c_int = 0x00007f00;
pub const ECONV_NEWLINE_DECORATOR_READ_MASK: c_int = 0x00000f00;
pub const ECONV_NEWLINE_DECORATOR_WRITE_MASK: c_int = 0x00007000;
pub const ECONV_UNIVERSAL_NEWLINE_DECORATOR: c_int = 0x00000100;
pub const ECONV_CRLF_NEWLINE_DECORATOR: c_int = 0x00001000;
pub const ECONV_CR_NEWLINE_DECORATOR: c_int = 0x00002000;
pub const ECONV_LF_NEWLINE_DECORATOR: c_int = 0x00004000;
pub const ECONV_XML_TEXT_DECORATOR: c_int = 0x00008000;
pub const ECONV_XML_ATTR_CONTENT_DECORATOR: c_int = 0x00010000;
pub const ECONV_STATEFUL_DECORATOR_MASK: c_int = 0x00f00000;
pub const ECONV_XML_ATTR_QUOTE_DECORATOR: c_int = 0x00100000;
#[cfg(windows)]
pub const ECONV_DEFAULT_NEWLINE_DECORATOR: c_int = ECONV_CRLF_NEWLINE_DECORATOR;
#[cfg(not(windows))]
pub const ECONV_DEFAULT_NEWLINE_DECORATOR: c_int = 0;
pub const ECONV_PARTIAL_INPUT: c_int = 0x00020000;
pub const ECONV_AFTER_OUTPUT: c_int = 0x00040000;

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_enc_associate(VALUE obj, rb_encoding *enc)
    pub fn rb_enc_associate(obj: Value, enc: EncodingType) -> Value;
    // VALUE
    // rb_enc_associate_index(VALUE obj, int idx)
    pub fn rb_enc_associate_index(obj: Value, idx: c_int) -> Value;
    // rb_encoding*
    // rb_enc_compatible(VALUE str1, VALUE str2)
    pub fn rb_enc_compatible(str1: Value, str2: Value) -> EncodingType;
    // VALUE
    // rb_enc_default_external(void)
    pub fn rb_enc_default_external() -> Value;
    // VALUE
    // rb_enc_default_internal(void)
    pub fn rb_enc_default_internal() -> Value;
    // int
    // rb_enc_find_index(const char *name)
    pub fn rb_enc_find_index(name: *const c_char) -> EncodingIndex;
    // ------------------------------------------------------
    // LINKER CANNOT FIND
    // // static VALUE
    // // rb_enc_from_encoding_index(int idx)
    // pub fn rb_enc_from_encoding_index(idx: c_int) -> Value;
    // ------------------------------------------------------
    // VALUE
    // rb_enc_from_encoding(rb_encoding *encoding)
    pub fn rb_enc_from_encoding(encoding: EncodingType) -> Value;
    // rb_encoding *
    // rb_enc_from_index(int index)
    pub fn rb_enc_from_index(index: EncodingIndex) -> EncodingType;
    // int
    // rb_enc_get_index(VALUE obj)
    pub fn rb_enc_get_index(obj: Value) -> EncodingIndex;
    // void
    // rb_enc_set_index(VALUE obj, int idx)
    pub fn rb_enc_set_index(obj: Value, encindex: EncodingIndex);
    // void
    // rb_enc_set_default_external(VALUE encoding)
    pub fn rb_enc_set_default_external(encoding: Value);
    // void
    // rb_enc_set_default_internal(VALUE encoding)
    pub fn rb_enc_set_default_internal(encoding: Value);
    // int
    // rb_filesystem_encindex(void)
    pub fn rb_filesystem_encindex() -> EncodingIndex;
    // int
    // rb_locale_encindex(void)
    pub fn rb_locale_encindex() -> EncodingIndex;
    // VALUE
    // rb_obj_encoding(VALUE obj)
    pub fn rb_obj_encoding(obj: Value) -> Value;
    // rb_encoding *
    // rb_to_encoding(VALUE enc)
    pub fn rb_to_encoding(enc: Value) -> EncodingType;
    // int
    // rb_to_encoding_index(VALUE enc)
    pub fn rb_to_encoding_index(obj: Value) -> EncodingIndex;
    // int
    // rb_usascii_encindex(void)
    pub fn rb_usascii_encindex() -> EncodingIndex;
    // int
    // rb_utf8_encindex(void)
    pub fn rb_utf8_encindex() -> EncodingIndex;
    // rb_encoding *
    // rb_ascii8bit_encoding(void)
    pub fn rb_ascii8bit_encoding() -> EncodingType;
    // int
    // rb_enc_ascget(const char *p, const char *e, int *len, rb_encoding *enc)
    pub fn rb_enc_ascget(
        p: *const c_char,
        e: *const c_char,
        len: *mut c_int,
        enc: EncodingType,
    ) -> c_int;
    // rb_encoding *
    // rb_enc_check(VALUE str1, VALUE str2)
    pub fn rb_enc_check(str1: Value, str2: Value) -> EncodingType;
    // int
    // rb_enc_codelen(int code, rb_encoding *enc)
    pub fn rb_enc_codelen(code: c_int, enc: EncodingType) -> c_int;
    // void
    // rb_enc_copy(VALUE obj1, VALUE obj2)
    pub fn rb_enc_copy(destination: Value, source: Value);
    // rb_encoding *
    // rb_enc_find(const char *name)
    pub fn rb_enc_find(name: *const c_char) -> EncodingType;
    // rb_encoding *
    // rb_enc_get(VALUE obj)
    //
    // Null for objects without an encoding.
    pub fn rb_enc_get(object: Value) -> EncodingType;
    // int
    // rb_enc_precise_mbclen(const char *p, const char *e, rb_encoding *enc)
    pub fn rb_enc_precise_mbclen(p: *const c_char, e: *const c_char, enc: EncodingType) -> c_int;
    // VALUE
    // rb_enc_str_buf_cat(VALUE str, const char *ptr, long len, rb_encoding *ptr_enc)
    pub fn rb_enc_str_buf_cat(
        string: Value,
        ptr: *const c_char,
        len: c_long,
        encoding: EncodingType,
    ) -> Value;
    // int
    // rb_enc_to_index(rb_encoding *enc)
    pub fn rb_enc_to_index(encoding: EncodingType) -> c_int;
    // VALUE
    // rb_enc_uint_chr(unsigned int code, rb_encoding *enc)
    pub fn rb_enc_uint_chr(code: u32, encoding: EncodingType) -> Value;
    // rb_encoding *
    // rb_filesystem_encoding(void)
    pub fn rb_filesystem_encoding() -> EncodingType;
    // rb_encoding *
    // rb_locale_encoding(void)
    pub fn rb_locale_encoding() -> EncodingType;
    // rb_encoding *
    // rb_usascii_encoding(void)
    pub fn rb_usascii_encoding() -> EncodingType;
    // rb_encoding *
    // rb_utf8_encoding(void)
    pub fn rb_utf8_encoding() -> EncodingType;
    // int
    // rb_enc_mbclen(const char *p, const char *e, rb_encoding *enc)
    pub fn rb_enc_mbclen(p: *const c_char, e: *const c_char, enc: EncodingType) -> c_int;
    // char *
    // rb_enc_nth(const char *p, const char *e, long nth, rb_encoding *enc)
    pub fn rb_enc_nth(
        p: *const c_char,
        e: *const c_char,
        nth: c_long,
        enc: EncodingType,
    ) -> *const c_char;
    // int
    // rb_enc_str_coderange(VALUE str)
    //
    // Computes and caches the coderange; returns one of `ENC_CODERANGE_*`.
    pub fn rb_enc_str_coderange(string: Value) -> c_int;
    // VALUE
    // rb_str_export_to_enc(VALUE str, rb_encoding *enc)
    pub fn rb_str_export_to_enc(str: Value, enc: EncodingType) -> Value;
    // VALUE
    // rb_str_encode(VALUE str, VALUE to, int ecflags, VALUE ecopts)
    pub fn rb_str_encode(str: Value, to: Value, ecflags: c_int, ecopts: Value) -> Value;
    // int
    // rb_econv_prepare_opts(VALUE opthash, VALUE *opts)
    pub fn rb_econv_prepare_opts(opthash: Value, opts: *mut Value) -> c_int;
    // unsigned int
    // rb_enc_codepoint_len(const char *p, const char *e, int *len_p, rb_encoding *enc)
    pub fn rb_enc_codepoint_len(
        ptr: *const c_char,
        end: *const c_char,
        len_p: *mut c_int,
        enc: EncodingType,
    ) -> c_uint;
    // int
    // rb_ascii8bit_encindex(void)
    pub fn rb_ascii8bit_encindex() -> EncodingIndex;
    // int
    // rb_char_to_option_kcode(int c, int *option, int *kcode)
    pub fn rb_char_to_option_kcode(c: c_int, option: *mut c_int, kcode: *mut c_int) -> c_int;
    // rb_encoding *
    // rb_default_external_encoding(void)
    pub fn rb_default_external_encoding() -> EncodingType;
    // rb_encoding *
    // rb_default_internal_encoding(void)
    //
    // Null when no default internal encoding is set.
    pub fn rb_default_internal_encoding() -> EncodingType;
    // int
    // rb_define_dummy_encoding(const char *name)
    pub fn rb_define_dummy_encoding(name: *const c_char) -> EncodingIndex;
    // int
    // rb_enc_alias(const char *alias, const char *orig)
    //
    // -1 when `orig` is not an encoding.
    pub fn rb_enc_alias(alias: *const c_char, orig: *const c_char) -> c_int;
    // int
    // rb_enc_capable(VALUE obj)
    pub fn rb_enc_capable(obj: Value) -> c_int;
    // int
    // rb_enc_dummy_p(rb_encoding *enc)
    pub fn rb_enc_dummy_p(enc: EncodingType) -> c_int;
    // int
    // rb_enc_fast_mbclen(const char *p, const char *e, rb_encoding *enc)
    pub fn rb_enc_fast_mbclen(p: *const c_char, e: *const c_char, enc: EncodingType) -> c_int;
    // int
    // rb_enc_unicode_p(rb_encoding *enc)
    pub fn rb_enc_unicode_p(enc: EncodingType) -> c_int;
    // rb_encoding *
    // rb_find_encoding(VALUE obj)
    //
    // Null, rather than raising, for an unknown encoding.
    pub fn rb_find_encoding(obj: Value) -> EncodingType;
    // VALUE
    // rb_locale_charmap(VALUE klass)
    pub fn rb_locale_charmap(klass: Value) -> Value;
    // int
    // rb_enc_tolower(int c, rb_encoding *enc)
    pub fn rb_enc_tolower(c: c_int, enc: EncodingType) -> c_int;
    // int
    // rb_enc_toupper(int c, rb_encoding *enc)
    pub fn rb_enc_toupper(c: c_int, enc: EncodingType) -> c_int;
    // char *
    // rb_enc_path_next(const char *path, const char *end, rb_encoding *enc)
    pub fn rb_enc_path_next(
        path: *const c_char,
        end: *const c_char,
        enc: EncodingType,
    ) -> *mut c_char;
    // char *
    // rb_enc_path_skip_prefix(const char *path, const char *end, rb_encoding *enc)
    pub fn rb_enc_path_skip_prefix(
        path: *const c_char,
        end: *const c_char,
        enc: EncodingType,
    ) -> *mut c_char;
    // char *
    // rb_enc_path_last_separator(const char *path, const char *end, rb_encoding *enc)
    pub fn rb_enc_path_last_separator(
        path: *const c_char,
        end: *const c_char,
        enc: EncodingType,
    ) -> *mut c_char;
    // char *
    // rb_enc_path_end(const char *path, const char *end, rb_encoding *enc)
    pub fn rb_enc_path_end(
        path: *const c_char,
        end: *const c_char,
        enc: EncodingType,
    ) -> *mut c_char;
    // const char *
    // ruby_enc_find_basename(const char *name, long *baselen, long *alllen, rb_encoding *enc)
    //
    // `name` must be NUL-terminated.
    pub fn ruby_enc_find_basename(
        name: *const c_char,
        baselen: *mut c_long,
        alllen: *mut c_long,
        enc: EncodingType,
    ) -> *const c_char;
    // const char *
    // ruby_enc_find_extname(const char *name, long *len, rb_encoding *enc)
    //
    // `name` must be NUL-terminated; null when there is no extension.
    pub fn ruby_enc_find_extname(
        name: *const c_char,
        len: *mut c_long,
        enc: EncodingType,
    ) -> *const c_char;
    // int
    // rb_econv_has_convpath_p(const char* from_encoding, const char* to_encoding)
    pub fn rb_econv_has_convpath_p(
        from_encoding: *const c_char,
        to_encoding: *const c_char,
    ) -> c_int;
    // int
    // rb_econv_prepare_options(VALUE opthash, VALUE *ecopts, int ecflags)
    pub fn rb_econv_prepare_options(opthash: Value, ecopts: *mut Value, ecflags: c_int) -> c_int;
    // rb_econv_t *
    // rb_econv_open(const char *source_encoding, const char *destination_encoding, int ecflags)
    //
    // Null when there is no conversion path.
    pub fn rb_econv_open(
        source_encoding: *const c_char,
        destination_encoding: *const c_char,
        ecflags: c_int,
    ) -> *mut RbEconv;
    // rb_econv_t *
    // rb_econv_open_opts(const char *source_encoding, const char *destination_encoding, int ecflags, VALUE ecopts)
    pub fn rb_econv_open_opts(
        source_encoding: *const c_char,
        destination_encoding: *const c_char,
        ecflags: c_int,
        ecopts: Value,
    ) -> *mut RbEconv;
    // VALUE
    // rb_econv_open_exc(const char *senc, const char *denc, int ecflags)
    pub fn rb_econv_open_exc(senc: *const c_char, denc: *const c_char, ecflags: c_int) -> Value;
    // rb_econv_result_t
    // rb_econv_convert(rb_econv_t *ec,
    //     const unsigned char **source_buffer_ptr, const unsigned char *source_buffer_end,
    //     unsigned char **destination_buffer_ptr, unsigned char *destination_buffer_end,
    //     int flags)
    //
    // Returns one of `ECONV_RESULT_*` (a C enum).
    pub fn rb_econv_convert(
        ec: *mut RbEconv,
        source_buffer_ptr: *mut *const u8,
        source_buffer_end: *const u8,
        destination_buffer_ptr: *mut *mut u8,
        destination_buffer_end: *mut u8,
        flags: c_int,
    ) -> EconvResult;
    // void
    // rb_econv_close(rb_econv_t *ec)
    pub fn rb_econv_close(ec: *mut RbEconv);
    // int
    // rb_econv_set_replacement(rb_econv_t *ec, const unsigned char *str, size_t len, const char *encname)
    pub fn rb_econv_set_replacement(
        ec: *mut RbEconv,
        str: *const u8,
        len: size_t,
        encname: *const c_char,
    ) -> c_int;
    // int
    // rb_econv_decorate_at_first(rb_econv_t *ec, const char *decorator_name)
    pub fn rb_econv_decorate_at_first(ec: *mut RbEconv, decorator_name: *const c_char) -> c_int;
    // int
    // rb_econv_decorate_at_last(rb_econv_t *ec, const char *decorator_name)
    pub fn rb_econv_decorate_at_last(ec: *mut RbEconv, decorator_name: *const c_char) -> c_int;
    // int
    // rb_econv_insert_output(rb_econv_t *ec,
    //     const unsigned char *str, size_t len, const char *str_encoding)
    pub fn rb_econv_insert_output(
        ec: *mut RbEconv,
        str: *const u8,
        len: size_t,
        str_encoding: *const c_char,
    ) -> c_int;
    // const char *
    // rb_econv_encoding_to_insert_output(rb_econv_t *ec)
    pub fn rb_econv_encoding_to_insert_output(ec: *mut RbEconv) -> *const c_char;
    // void
    // rb_econv_check_error(rb_econv_t *ec)
    pub fn rb_econv_check_error(ec: *mut RbEconv);
    // VALUE
    // rb_econv_make_exception(rb_econv_t *ec)
    //
    // `Qnil` when the last conversion did not fail.
    pub fn rb_econv_make_exception(ec: *mut RbEconv) -> Value;
    // int
    // rb_econv_putbackable(rb_econv_t *ec)
    pub fn rb_econv_putbackable(ec: *mut RbEconv) -> c_int;
    // void
    // rb_econv_putback(rb_econv_t *ec, unsigned char *p, int n)
    pub fn rb_econv_putback(ec: *mut RbEconv, p: *mut u8, n: c_int);
    // const char *
    // rb_econv_asciicompat_encoding(const char *encname)
    //
    // Null when `encname` is ASCII compatible or unknown.
    pub fn rb_econv_asciicompat_encoding(encname: *const c_char) -> *const c_char;
    // VALUE
    // rb_econv_str_convert(rb_econv_t *ec, VALUE src, int flags)
    pub fn rb_econv_str_convert(ec: *mut RbEconv, src: Value, flags: c_int) -> Value;
    // VALUE
    // rb_econv_substr_convert(rb_econv_t *ec, VALUE src, long byteoff, long bytesize, int flags)
    pub fn rb_econv_substr_convert(
        ec: *mut RbEconv,
        src: Value,
        byteoff: c_long,
        bytesize: c_long,
        flags: c_int,
    ) -> Value;
    // VALUE
    // rb_econv_str_append(rb_econv_t *ec, VALUE src, VALUE dst, int flags)
    pub fn rb_econv_str_append(ec: *mut RbEconv, src: Value, dst: Value, flags: c_int) -> Value;
    // VALUE
    // rb_econv_substr_append(rb_econv_t *ec, VALUE src, long byteoff, long bytesize, VALUE dst, int flags)
    pub fn rb_econv_substr_append(
        ec: *mut RbEconv,
        src: Value,
        byteoff: c_long,
        bytesize: c_long,
        dst: Value,
        flags: c_int,
    ) -> Value;
    // VALUE
    // rb_econv_append(rb_econv_t *ec, const char *bytesrc, long bytesize, VALUE dst, int flags)
    pub fn rb_econv_append(
        ec: *mut RbEconv,
        bytesrc: *const c_char,
        bytesize: c_long,
        dst: Value,
        flags: c_int,
    ) -> Value;
    // void
    // rb_econv_binmode(rb_econv_t *ec)
    pub fn rb_econv_binmode(ec: *mut RbEconv);
}

pub unsafe fn coderange_set(obj: Value, code_range: InternalValue) {
    let basic: *mut RBasic = obj.value as *mut RBasic;
    (*basic).flags = ((*basic).flags & !(ENC_CODERANGE_MASK as InternalValue)) | code_range
}

pub unsafe fn coderange_clear(obj: Value) {
    coderange_set(obj, 0)
}
