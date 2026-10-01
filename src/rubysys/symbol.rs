use crate::rubysys::types::{c_char, c_int, c_long, EncodingType, Id, Value};

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_id2sym(ID x)
    pub fn rb_id2sym(id: Id) -> Value;
    // const char *
    // rb_id2name(ID id)
    pub fn rb_id2name(id: Id) -> *const c_char;
    // ID
    // rb_sym2id(VALUE sym)
    pub fn rb_sym2id(id: Value) -> Id;
    // ID
    // rb_intern(const char *name)
    pub fn rb_intern(name: *const c_char) -> Id;
    // ID
    // rb_intern2(const char *name, long len)
    pub fn rb_intern2(name: *const c_char, len: c_long) -> Id;
    // ID
    // rb_check_id(volatile VALUE *namep)
    //
    // Returns 0, without creating a symbol, when the name was never interned.
    pub fn rb_check_id(name: *mut Value) -> Id;
    // ID
    // rb_intern_str(VALUE str)
    pub fn rb_intern_str(string: Value) -> Id;
    // int
    // rb_is_class_id(ID id)
    pub fn rb_is_class_id(id: Id) -> c_int;
    // int
    // rb_is_const_id(ID id)
    pub fn rb_is_const_id(id: Id) -> c_int;
    // int
    // rb_is_instance_id(ID id)
    pub fn rb_is_instance_id(id: Id) -> c_int;
    // VALUE
    // rb_sym2str(VALUE sym)
    pub fn rb_sym2str(symbol: Value) -> Value;
    // ID
    // rb_to_id(VALUE name)
    pub fn rb_to_id(name: Value) -> Id;
    // VALUE
    // rb_to_symbol(VALUE name)
    pub fn rb_to_symbol(name: Value) -> Value;
    // VALUE
    // rb_id2str(ID id)
    //
    // The frozen name of `id`, or 0 for an internal ID without one.
    pub fn rb_id2str(id: Id) -> Value;
    // VALUE
    // rb_check_symbol(volatile VALUE *namep)
    //
    // `Qnil`, without creating a symbol, when the name was never interned.
    pub fn rb_check_symbol(namep: *mut Value) -> Value;
    // ID
    // rb_intern3(const char *name, long len, rb_encoding *enc)
    pub fn rb_intern3(name: *const c_char, len: c_long, enc: EncodingType) -> Id;
    // ID
    // rb_check_id_cstr(const char *ptr, long len, rb_encoding *enc)
    pub fn rb_check_id_cstr(ptr: *const c_char, len: c_long, enc: EncodingType) -> Id;
    // VALUE
    // rb_check_symbol_cstr(const char *ptr, long len, rb_encoding *enc)
    pub fn rb_check_symbol_cstr(ptr: *const c_char, len: c_long, enc: EncodingType) -> Value;
    // int
    // rb_enc_symname_p(const char *str, rb_encoding *enc)
    pub fn rb_enc_symname_p(str: *const c_char, enc: EncodingType) -> c_int;
    // int
    // rb_enc_symname2_p(const char *name, long len, rb_encoding *enc)
    pub fn rb_enc_symname2_p(name: *const c_char, len: c_long, enc: EncodingType) -> c_int;
    // int
    // rb_symname_p(const char *str)
    pub fn rb_symname_p(str: *const c_char) -> c_int;
    // ID
    // rb_id_attrset(ID id)
    pub fn rb_id_attrset(id: Id) -> Id;
    // int
    // rb_is_attrset_id(ID id)
    pub fn rb_is_attrset_id(id: Id) -> c_int;
    // int
    // rb_is_global_id(ID id)
    pub fn rb_is_global_id(id: Id) -> c_int;
    // int
    // rb_is_junk_id(ID)
    pub fn rb_is_junk_id(id: Id) -> c_int;
    // int
    // rb_is_local_id(ID id)
    pub fn rb_is_local_id(id: Id) -> c_int;
    // VALUE
    // rb_sym_all_symbols(void)
    pub fn rb_sym_all_symbols() -> Value;
    // VALUE
    // rb_lastline_get(void)
    //
    // `$_` of the current Ruby frame.
    pub fn rb_lastline_get() -> Value;
    // void
    // rb_lastline_set(VALUE str)
    pub fn rb_lastline_set(str: Value);
}
