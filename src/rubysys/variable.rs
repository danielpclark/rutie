use crate::rubysys::types::{c_char, Id, Value};

// Ruby 2.5 and 2.6 call a getter as `getter(id, data, gvar)` and a setter as
// `setter(value, id, data, gvar)`; Ruby 2.7 drops the trailing `gvar`. Rust
// callbacks take the common leading arguments, which is valid for both.

// VALUE getter(ID id, VALUE *data)
pub type GlobalGetter = extern "C" fn(id: Id, data: *mut Value) -> Value;
// void setter(VALUE value, ID id, VALUE *data)
pub type GlobalSetter = extern "C" fn(value: Value, id: Id, data: *mut Value);

extern "C" {
    // void
    // rb_define_hooked_variable(const char *name, VALUE *var,
    //                           VALUE (*getter)(ANYARGS), void (*setter)(ANYARGS))
    //
    // `*var` is marked with `rb_gc_mark_maybe`, so `var` must point to a
    // readable `VALUE`-sized word for as long as the variable exists.
    pub fn rb_define_hooked_variable(
        name: *const c_char,
        var: *mut Value,
        getter: Option<GlobalGetter>,
        setter: Option<GlobalSetter>,
    );
    // void
    // rb_define_readonly_variable(const char *name, const VALUE *var)
    pub fn rb_define_readonly_variable(name: *const c_char, var: *const Value);
    // void
    // rb_define_variable(const char *name, VALUE *var)
    pub fn rb_define_variable(name: *const c_char, var: *mut Value);
    // void
    // rb_define_virtual_variable(const char *name,
    //                            VALUE (*getter)(ANYARGS), void (*setter)(ANYARGS))
    pub fn rb_define_virtual_variable(
        name: *const c_char,
        getter: Option<GlobalGetter>,
        setter: Option<GlobalSetter>,
    );
    // VALUE
    // rb_gv_get(const char *name)
    pub fn rb_gv_get(name: *const c_char) -> Value;
    // VALUE
    // rb_gv_set(const char *name, VALUE val)
    pub fn rb_gv_set(name: *const c_char, value: Value) -> Value;
}
