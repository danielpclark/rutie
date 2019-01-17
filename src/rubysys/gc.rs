use rubysys::types::{ssize_t, Value};

extern "C" {
    // VALUE
    // rb_gc_disable(void)
    pub fn rb_gc_disable() -> Value;
    // VALUE
    // rb_gc_enable(void)
    pub fn rb_gc_enable() -> Value;
    // void
    // rb_gc_mark(VALUE ptr)
    pub fn rb_gc_mark(value: Value);
    // void
    // rb_gc_adjust_memory_usage(ssize_t diff)
    pub fn rb_gc_adjust_memory_usage(diff: ssize_t);
}
