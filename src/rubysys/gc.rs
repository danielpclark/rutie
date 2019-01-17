use rubysys::types::{size_t, ssize_t, Value, CallbackPtr};

extern "C" {
    // void
    // rb_gc_adjust_memory_usage(ssize_t diff)
    pub fn rb_gc_adjust_memory_usage(diff: ssize_t);
    // // void
    // // rb_gc_call_finalizer_at_exit(void)
    // pub fn rb_gc_call_finalizer_at_exit();
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
    // // VALUE
    // // rb_gc_latest_gc_info(VALUE key)
    // pub fn rb_gc_latest_gc_info(key: Value) -> Value;
    // void
    // rb_gc_mark(VALUE ptr)
    pub fn rb_gc_mark(value: Value);
    // void
    // rb_gc_mark_locations(const VALUE *start, const VALUE *end)
    pub fn rb_gc_mark_locations(start: CallbackPtr, end: CallbackPtr);
    // void
    // rb_gc_mark_maybe(VALUE obj)
    pub fn rb_gc_mark_maybe(obj: Value);
    // // void
    // // rb_gc_register_address(VALUE *addr)
    // pub fn rb_gc_register_address(CallbackPtr);
    // // void
    // // rb_gc_register_mark_object(VALUE obj)
    // pub fn rb_gc_register_mark_object(obj: Value);
    // VALUE
    // rb_gc_start(void)
    pub fn rb_gc_start() -> Value;
    // // size_t
    // // rb_gc_stat(VALUE key)
    // pub fn rb_gc_stat(key: Value) -> size_t;
    // // void
    // // rb_gc_unregister_address(VALUE *addr)
    // pub fn rb_gc_unregister_address(CallbackPtr);
    // // void
    // // rb_gc_writebarrier(VALUE a, VALUE b)
    // pub fn rb_gc_writebarrier(a: Value, b: Value);
    // // void
    // // rb_gc_writebarrier_unprotect(VALUE obj)
    // pub fn rb_gc_writebarrier_unprotect(obj: Value);
}
