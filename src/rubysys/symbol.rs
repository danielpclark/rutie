use rubysys::types::{c_char, Id, Value};

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
}
