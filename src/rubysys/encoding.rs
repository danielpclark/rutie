use rubysys::types::{c_int, c_char, Value};

extern "C" {
    pub fn rb_enc_associate_index(obj: Value, idx: c_int) -> Value;
    pub fn rb_enc_find_index(name: *const c_char ) -> c_int;
    pub fn rb_enc_get_index(obj: Value) -> c_int;
    pub fn rb_enc_set_index(obj: Value, encindex: c_int);
    pub fn rb_filesystem_encindex() -> c_int;
    pub fn rb_locale_encindex() -> c_int;
    pub fn rb_to_encoding_index(obj: Value) -> c_int;
    pub fn rb_usascii_encindex() -> c_int;
    pub fn rb_utf8_encindex() -> c_int;
}
