use super::types::{c_int, size_t, ssize_t, CallbackPtr, Value};

extern "C" {
    // void
    // rb_gc_adjust_memory_usage(ssize_t diff)
    pub fn rb_gc_adjust_memory_usage(diff: ssize_t);
    // size_t
    // rb_gc_count(void)
    pub fn rb_gc_count() -> size_t;
    // VALUE
    // rb_gc_disable(void)
    pub fn rb_gc_disable() -> Value;
    // VALUE
    // rb_gc_enable(void)
    pub fn rb_gc_enable() -> Value;
    // void
    // rb_gc_force_recycle(VALUE obj)
    pub fn rb_gc_force_recycle(obj: Value);
    // void
    // rb_gc_mark(VALUE ptr)
    pub fn rb_gc_mark(value: Value);
    // void
    // rb_gc_mark_maybe(VALUE obj)
    pub fn rb_gc_mark_maybe(obj: Value);
    // void
    // rb_gc_register_address(VALUE *addr)
    pub fn rb_gc_register_address(addr: CallbackPtr);
    // void
    // rb_gc_register_mark_object(VALUE obj)
    pub fn rb_gc_register_mark_object(obj: Value);
    // VALUE
    // rb_gc_start(void)
    pub fn rb_gc_start() -> Value;
    // size_t
    // rb_gc_stat(VALUE key)
    pub fn rb_gc_stat(key: Value) -> size_t;
    // void
    // rb_gc_unregister_address(VALUE *addr)
    pub fn rb_gc_unregister_address(addr: CallbackPtr);
    // int
    // rb_objspace_marked_object_p(VALUE obj)
    pub fn rb_objspace_marked_object_p(obj: Value) -> c_int;
}
