use rubysys::types::Value;

extern "C" {
    pub fn rb_gc_mark(value: Value);
}
