use rubysys::types::{c_int, c_char, Value, CallbackPtr};

pub const ENC_DUMMY_FLAG: isize = (1<<24);
pub const ENC_INDEX_MASK: isize = (!(!0<<24));

extern "C" {
    // VALUE
    // rb_enc_associate(VALUE obj, rb_encoding *enc)
    pub fn rb_enc_associate(obj: Value, enc: CallbackPtr) -> Value;
    // VALUE
    // rb_enc_associate_index(VALUE obj, int idx)
    pub fn rb_enc_associate_index(obj: Value, idx: c_int) -> Value;
    // VALUE
    // rb_enc_default_external(void)
    pub fn rb_enc_default_external() -> Value;
    // VALUE
    // rb_enc_default_internal(void)
    pub fn rb_enc_default_internal() -> Value;
    // int
    // rb_enc_find_index(const char *name)
    pub fn rb_enc_find_index(name: *const c_char) -> c_int;
    // ------------------------------------------------------
    // LINKER CANNOT FIND
    // // static VALUE
    // // rb_enc_from_encoding_index(int idx)
    // pub fn rb_enc_from_encoding_index(idx: c_int) -> Value;
    // ------------------------------------------------------
    // VALUE
    // rb_enc_from_encoding(rb_encoding *encoding)
    pub fn rb_enc_from_encoding(encoding: CallbackPtr) -> Value;
    // rb_encoding *
    // rb_enc_from_index(int index)
    pub fn rb_enc_from_index(index: c_int) -> CallbackPtr;
    // int
    // rb_enc_get_index(VALUE obj)
    pub fn rb_enc_get_index(obj: Value) -> c_int;
    // void
    // rb_enc_set_index(VALUE obj, int idx)
    pub fn rb_enc_set_index(obj: Value, encindex: c_int);
    // void
    // rb_enc_set_default_external(VALUE encoding)
    pub fn rb_enc_set_default_external(encoding: Value);
    // void
    // rb_enc_set_default_internal(VALUE encoding)
    pub fn rb_enc_set_default_internal(encoding: Value);
    // int
    // rb_filesystem_encindex(void)
    pub fn rb_filesystem_encindex() -> c_int;
    // int
    // rb_locale_encindex(void)
    pub fn rb_locale_encindex() -> c_int;
    // VALUE
    // rb_obj_encoding(VALUE obj)
    pub fn rb_obj_encoding(obj: Value) -> Value;
    // rb_encoding *
    // rb_to_encoding(VALUE enc)
    pub fn rb_to_encoding(enc: Value) -> CallbackPtr;
    // int
    // rb_to_encoding_index(VALUE enc)
    pub fn rb_to_encoding_index(obj: Value) -> c_int;
    // int
    // rb_usascii_encindex(void)
    pub fn rb_usascii_encindex() -> c_int;
    // int
    // rb_utf8_encindex(void)
    pub fn rb_utf8_encindex() -> c_int;
}
