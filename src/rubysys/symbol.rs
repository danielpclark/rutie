use rubysys::types::{c_char, c_long, Id, Value};

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
}
