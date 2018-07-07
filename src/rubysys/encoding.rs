use rubysys::types::{c_int, c_char, Value};

extern "C" {
    // VALUE
    // rb_enc_associate_index(VALUE obj, int idx)
    pub fn rb_enc_associate_index(obj: Value, idx: c_int) -> Value;
    // int
    // rb_enc_find_index(const char *name)
    pub fn rb_enc_find_index(name: *const c_char ) -> c_int;
    // int
    // rb_enc_get_index(VALUE obj)
    pub fn rb_enc_get_index(obj: Value) -> c_int;
    // void
    // rb_enc_set_index(VALUE obj, int idx)
    pub fn rb_enc_set_index(obj: Value, encindex: c_int);
    // int
    // rb_filesystem_encindex(void)
    pub fn rb_filesystem_encindex() -> c_int;
    // int
    // rb_locale_encindex(void)
    pub fn rb_locale_encindex() -> c_int;
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
