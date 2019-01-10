use rubysys::types::{ssize_t, Value};

extern "C" {
    // void
    // rb_gc_mark(VALUE ptr)
    pub fn rb_gc_mark(value: Value);

    // void
    // rb_gc_adjust_memory_usage(ssize_t diff)
    pub fn rb_gc_adjust_memory_usage(diff: ssize_t);
}
