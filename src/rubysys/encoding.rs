use super::{
    constant::{FL_USER_8, FL_USER_9},
    types::{c_char, c_int, size_t, EncodingIndex, EncodingType, InternalValue, RBasic, Value},
};

pub const ENC_DUMMY_FLAG: isize = 1 << 24;
pub const ENC_INDEX_MASK: isize = !(!0 << 24);
pub const ENC_CODERANGE_UNKNOWN: isize = 0;
pub const ENC_CODERANGE_7BIT: isize = FL_USER_8;
pub const ENC_CODERANGE_VALID: isize = FL_USER_9;
pub const ENC_CODERANGE_BROKEN: isize = FL_USER_8 | FL_USER_9;
pub const ENC_CODERANGE_MASK: isize =
    ENC_CODERANGE_7BIT | ENC_CODERANGE_VALID | ENC_CODERANGE_BROKEN;

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
    // VALUE
    // rb_str_export_to_enc(VALUE str, rb_encoding *enc)
    pub fn rb_str_export_to_enc(str: Value, enc: EncodingType) -> Value;
    // VALUE
    // rb_str_encode(VALUE str, VALUE to, int ecflags, VALUE ecopts)
    pub fn rb_str_encode(str: Value, to: Value, ecflags: c_int, ecopts: Value) -> Value;
    // int
    // rb_econv_prepare_opts(VALUE opthash, VALUE *opts)
    pub fn rb_econv_prepare_opts(opthash: Value, opts: *const Value) -> c_int;
    // unsigned int
    // rb_enc_codepoint_len(const char *p, const char *e, int *len_p, rb_encoding *enc)
    pub fn rb_enc_codepoint_len(
        ptr: *const c_char,
        end: *const c_char,
        len_p: *mut c_int,
        enc: EncodingType,
    ) -> size_t;
}

pub unsafe fn coderange_set(obj: Value, code_range: InternalValue) {
    let basic: *mut RBasic = obj.value as _;
    (*basic).flags = ((*basic).flags & !(ENC_CODERANGE_MASK as InternalValue)) | code_range
}

pub unsafe fn coderange_clear(obj: Value) {
    coderange_set(obj, 0)
}
