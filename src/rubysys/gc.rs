use rubysys::types::Value;

extern "C" {
    // void
    // rb_gc_mark(VALUE ptr)
    pub fn rb_gc_mark(value: Value);
}
