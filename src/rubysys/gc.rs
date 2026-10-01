use crate::rubysys::types::{c_int, c_void, size_t, ssize_t, CallbackPtr, Value};

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
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
    // rb_gc_mark(VALUE ptr)
    pub fn rb_gc_mark(value: Value);
    // void
    // rb_gc_mark_movable(VALUE ptr)
    pub fn rb_gc_mark_movable(value: Value);
    // VALUE
    // rb_gc_location(VALUE value)
    pub fn rb_gc_location(value: Value) -> Value;
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
    // VALUE
    // rb_define_finalizer(VALUE obj, VALUE block)
    pub fn rb_define_finalizer(object: Value, block: Value) -> Value;
    // VALUE
    // rb_gc_latest_gc_info(VALUE key)
    //
    // `key` is a Hash to fill or a Symbol to look up.
    pub fn rb_gc_latest_gc_info(key: Value) -> Value;
    // void
    // rb_gc_writebarrier(VALUE a, VALUE b)
    pub fn rb_gc_writebarrier(parent: Value, child: Value);
    // void
    // rb_gc_writebarrier_unprotect(VALUE obj)
    pub fn rb_gc_writebarrier_unprotect(object: Value);
    // VALUE
    // rb_memory_id(VALUE obj)
    pub fn rb_memory_id(object: Value) -> Value;
    // VALUE
    // rb_undefine_finalizer(VALUE obj)
    pub fn rb_undefine_finalizer(object: Value) -> Value;
}

// Ruby's allocator (`ruby/internal/xmalloc.h`): memory counted by the GC,
// which runs a collection and retries before raising `NoMemoryError`. Ruby
// frees what it allocates for C code with `ruby_xfree`, and memory given to
// Ruby to free must come from these.
#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // void *
    // ruby_xmalloc(size_t size)
    pub fn ruby_xmalloc(size: size_t) -> *mut c_void;
    // void *
    // ruby_xmalloc2(size_t nelems, size_t elemsiz)
    //
    // Raises `ArgumentError` when `nelems * elemsiz` overflows.
    pub fn ruby_xmalloc2(nelems: size_t, elemsiz: size_t) -> *mut c_void;
    // void *
    // ruby_xcalloc(size_t nelems, size_t elemsiz)
    //
    // Zeroed.
    pub fn ruby_xcalloc(nelems: size_t, elemsiz: size_t) -> *mut c_void;
    // void *
    // ruby_xrealloc(void *ptr, size_t newsiz)
    pub fn ruby_xrealloc(ptr: *mut c_void, newsiz: size_t) -> *mut c_void;
    // void *
    // ruby_xrealloc2(void *ptr, size_t newelems, size_t newsiz)
    pub fn ruby_xrealloc2(ptr: *mut c_void, newelems: size_t, newsiz: size_t) -> *mut c_void;
    // void
    // ruby_xfree(void *ptr)
    pub fn ruby_xfree(ptr: *mut c_void);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xmalloc_family() {
        crate::on_ruby_thread(|| unsafe {
            let bytes = ruby_xmalloc(4) as *mut u8;
            bytes.copy_from(b"ruby".as_ptr(), 4);

            let bytes = ruby_xrealloc(bytes as *mut c_void, 8) as *mut u8;
            assert_eq!(std::slice::from_raw_parts(bytes, 4), b"ruby");
            ruby_xfree(bytes as *mut c_void);

            let words = ruby_xcalloc(3, 8) as *mut u64;
            assert_eq!(std::slice::from_raw_parts(words, 3), &[0, 0, 0]);

            let words = ruby_xrealloc2(words as *mut c_void, 6, 8) as *mut u64;
            assert_eq!(std::slice::from_raw_parts(words, 3), &[0, 0, 0]);
            ruby_xfree(words as *mut c_void);

            let pairs = ruby_xmalloc2(2, 16) as *mut [u64; 2];
            pairs.write([1, 2]);
            assert_eq!(*pairs, [1, 2]);
            ruby_xfree(pairs as *mut c_void);
        });
    }
}
