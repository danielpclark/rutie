use rubysys::types::{c_char, Id, Value};

extern "C" {
    pub fn rb_id2sym(id: Id) -> Value;
    pub fn rb_id2name(id: Id) -> *const c_char;
    pub fn rb_sym2id(id: Value) -> Id;
}
