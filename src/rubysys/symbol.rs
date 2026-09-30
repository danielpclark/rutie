use crate::rubysys::types::{c_char, c_int, c_long, Id, Value};

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
}
