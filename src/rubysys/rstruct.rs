use crate::rubysys::types::{c_char, Id, Value};

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    pub static rb_cStruct: Value;
}

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_struct_alloc(VALUE klass, VALUE values)
    pub fn rb_struct_alloc(klass: Value, values: Value) -> Value;
    // VALUE
    // rb_struct_aref(VALUE s, VALUE idx)
    pub fn rb_struct_aref(object: Value, index: Value) -> Value;
    // VALUE
    // rb_struct_aset(VALUE s, VALUE idx, VALUE val)
    pub fn rb_struct_aset(object: Value, index: Value, value: Value) -> Value;
    // VALUE
    // rb_struct_define(const char *name, ...)
    //
    // Member names are `const char *`, terminated by a null pointer.
    pub fn rb_struct_define(name: *const c_char, ...) -> Value;
    // VALUE
    // rb_struct_define_under(VALUE outer, const char *name, ...)
    pub fn rb_struct_define_under(outer: Value, name: *const c_char, ...) -> Value;
    // VALUE
    // rb_struct_getmember(VALUE obj, ID id)
    pub fn rb_struct_getmember(object: Value, name: Id) -> Value;
    // VALUE
    // rb_struct_members(VALUE s)
    pub fn rb_struct_members(object: Value) -> Value;
    // VALUE
    // rb_struct_new(VALUE klass, ...)
    pub fn rb_struct_new(klass: Value, ...) -> Value;
    // VALUE
    // rb_struct_s_members(VALUE klass)
    pub fn rb_struct_s_members(klass: Value) -> Value;
    // VALUE
    // rb_struct_size(VALUE s)
    pub fn rb_struct_size(object: Value) -> Value;
    // VALUE
    // rb_data_define(VALUE super, ...)
    //
    // An anonymous `Data` class (`Data.define`) under `super` (a `Data`
    // class, or 0 for `Data`). Member names are `const char *`, terminated
    // by a null pointer; raises `ArgumentError` for a duplicate.
    #[cfg(ruby_gte_3_3)]
    pub fn rb_data_define(superclass: Value, ...) -> Value;
}
