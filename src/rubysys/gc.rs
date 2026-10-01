use crate::rubysys::{
    st::StTable,
    types::{c_int, c_long, c_void, size_t, ssize_t, CallbackPtr, Value},
};

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

// `ruby/internal/gc.h`: the rest of the GC API, mostly for C-level code
// (marking objects held in `st_table`s or C arrays).
#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // int
    // rb_during_gc(void)
    pub fn rb_during_gc() -> c_int;
    // void
    // rb_gc(void)
    //
    // A full, immediate GC (what `GC.start` does with its default options).
    pub fn rb_gc();
    // void
    // rb_gc_copy_finalizer(VALUE dst, VALUE src)
    //
    // Gives `dst` the finalizers of `src`.
    pub fn rb_gc_copy_finalizer(dst: Value, src: Value);
    // void
    // rb_gc_mark_locations(const VALUE *start, const VALUE *end)
    //
    // `rb_gc_mark_maybe` for each `VALUE` from `start` up to `end`
    // (excluded); only during marking.
    pub fn rb_gc_mark_locations(start: *const Value, end: *const Value);
    // void
    // rb_gc_update_tbl_refs(st_table *ptr)
    //
    // Updates the values of a table marked with `rb_mark_tbl_no_pin` after
    // compaction.
    pub fn rb_gc_update_tbl_refs(table: *mut StTable);
    // void
    // rb_global_variable(VALUE *)
    //
    // The same as `rb_gc_register_address`.
    pub fn rb_global_variable(address: *mut Value);
    // void
    // rb_mark_hash(struct st_table *tbl)
    //
    // Marks the keys and values of a table of `VALUE`s; only during marking.
    pub fn rb_mark_hash(table: *mut StTable);
    // void
    // rb_mark_set(struct st_table *tbl)
    //
    // Marks the keys of a table.
    pub fn rb_mark_set(table: *mut StTable);
    // void
    // rb_mark_tbl(struct st_table *tbl)
    //
    // Marks (and pins) the values of a table.
    pub fn rb_mark_tbl(table: *mut StTable);
    // void
    // rb_mark_tbl_no_pin(struct st_table *tbl)
    //
    // Marks the values of a table, letting compaction move them (see
    // `rb_gc_update_tbl_refs`).
    pub fn rb_mark_tbl_no_pin(table: *mut StTable);
    // void
    // rb_memerror(void)
    //
    // Raises a `NoMemoryError`.
    pub fn rb_memerror() -> !;
}

// `ruby/internal/xmalloc.h` and `ruby/internal/memory.h`: memory allocated
// by Ruby, which counts towards the GC's malloc limit (and may start a GC).
// The allocators raise a `NoMemoryError` instead of returning NULL; memory
// from them must be freed with `ruby_xfree`.
#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // void *
    // ruby_xcalloc(size_t nelems, size_t elemsiz)
    pub fn ruby_xcalloc(nelems: size_t, elemsiz: size_t) -> *mut c_void;
    // void
    // ruby_xfree(void *ptr)
    pub fn ruby_xfree(ptr: *mut c_void);
    // void *
    // ruby_xmalloc(size_t size)
    pub fn ruby_xmalloc(size: size_t) -> *mut c_void;
    // void *
    // ruby_xmalloc2(size_t nelems, size_t elemsiz)
    //
    // `nelems * elemsiz` bytes, raising an `ArgumentError` on overflow.
    pub fn ruby_xmalloc2(nelems: size_t, elemsiz: size_t) -> *mut c_void;
    // void *
    // ruby_xrealloc(void *ptr, size_t newsiz)
    pub fn ruby_xrealloc(ptr: *mut c_void, newsiz: size_t) -> *mut c_void;
    // void *
    // ruby_xrealloc2(void *ptr, size_t newelems, size_t newsiz)
    pub fn ruby_xrealloc2(ptr: *mut c_void, newelems: size_t, newsiz: size_t) -> *mut c_void;

    // void *
    // rb_alloc_tmp_buffer(volatile VALUE *store, long len)
    //
    // A temporary buffer of `len` bytes (`ALLOCV`) owned by a hidden object
    // stored in `*store`, which must stay on the stack (or be marked) while
    // the buffer is used. The GC marks any `VALUE`s in the buffer. Free it
    // early with `rb_free_tmp_buffer`.
    pub fn rb_alloc_tmp_buffer(store: *mut Value, len: c_long) -> *mut c_void;
    // void *
    // rb_alloc_tmp_buffer_with_count(volatile VALUE *store, size_t len, size_t count)
    //
    // Like `rb_alloc_tmp_buffer`; `count` is how many `VALUE`-sized words of
    // the buffer the GC scans (`rb_alloc_tmp_buffer` passes all of them).
    pub fn rb_alloc_tmp_buffer_with_count(
        store: *mut Value,
        len: size_t,
        count: size_t,
    ) -> *mut c_void;
    // void
    // rb_free_tmp_buffer(volatile VALUE *store)
    //
    // Frees the buffer of `*store` and sets it to `Qfalse`.
    pub fn rb_free_tmp_buffer(store: *mut Value);
    // void
    // ruby_malloc_size_overflow(size_t x, size_t y)
    //
    // Raises the `ArgumentError` for `x * y` overflowing.
    pub fn ruby_malloc_size_overflow(x: size_t, y: size_t) -> !;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        rubysys::{
            st,
            typed_data::{self, RbDataType, RbDataTypeFunction},
        },
        types::c_char,
    };
    use std::ptr;

    // A C-level object holding `VALUE`s in an `st_table` and an array.
    struct Holder {
        table: *mut StTable,
        values: [Value; 2],
    }

    extern "C" fn mark_holder(data: *mut c_void) {
        let holder = unsafe { &mut *(data as *mut Holder) };

        unsafe {
            rb_mark_hash(holder.table);
            rb_mark_set(holder.table);
            rb_mark_tbl(holder.table);
            rb_mark_tbl_no_pin(holder.table);
            rb_gc_mark_locations(holder.values.as_ptr(), holder.values.as_ptr().add(2));
            assert_eq!(rb_during_gc(), 1);
        }
    }

    static HOLDER_TYPE: RbDataType = RbDataType {
        wrap_struct_name: b"RutieGcTestHolder\0".as_ptr() as *const c_char,
        function: RbDataTypeFunction {
            dmark: Some(mark_holder),
            dfree: None,
            dsize: None,
            reserved: [ptr::null_mut(); 2],
        },
        parent: ptr::null(),
        data: ptr::null_mut(),
        flags: Value { value: 0 },
    };

    #[test]
    fn test_marking_and_allocation() {
        crate::on_ruby_thread(|| unsafe {
            assert_eq!(rb_during_gc(), 0);

            let key = crate::binding::vm::eval_string("'key' * 2");
            let value = crate::binding::vm::eval_string("'value' * 2");
            let table = st::rb_st_init_numtable_with_size(1);
            st::rb_st_add_direct(table, key.value as _, value.value as _);

            // Leaked: the holder lives as long as the test process.
            let holder = Box::leak(Box::new(Holder {
                table,
                values: [key, value],
            }));
            let object = typed_data::rb_data_typed_object_wrap(
                Value::from(0),
                holder as *mut Holder as *mut c_void,
                &HOLDER_TYPE,
            );
            rb_gc_register_mark_object(object);

            rb_gc();
            rb_gc_update_tbl_refs(table);
            assert_eq!(crate::binding::string::value_to_string(value), "valuevalue");

            let global = Box::leak(Box::new(crate::binding::vm::eval_string("[1]")));
            rb_global_variable(global);
            rb_gc();

            let memory = ruby_xmalloc(16) as *mut u8;
            *memory.add(15) = 1;
            let memory = ruby_xrealloc(memory as *mut c_void, 32) as *mut u8;
            assert_eq!(*memory.add(15), 1);
            let memory = ruby_xrealloc2(memory as *mut c_void, 8, 8) as *mut u8;
            assert_eq!(*memory.add(15), 1);
            ruby_xfree(memory as *mut c_void);

            let zeroed = ruby_xcalloc(4, 8) as *mut u64;
            assert_eq!(*zeroed.add(3), 0);
            ruby_xfree(zeroed as *mut c_void);
            ruby_xfree(ruby_xmalloc2(3, 5));

            // Size overflows raise an `ArgumentError`.
            let overflow = crate::binding::vm::protect_value(|| {
                ruby_malloc_size_overflow(usize::MAX, 2);
            });
            assert!(overflow.is_err());
            let error = crate::binding::vm::protect_value(|| {
                ruby_xmalloc2(usize::MAX, 2);
                Value::from(0)
            });
            assert!(error.is_err());

            let mut store = Value::from(0);
            let buffer = rb_alloc_tmp_buffer(&mut store, 64) as *mut u8;
            *buffer.add(63) = 7;
            assert!(store.value != 0);
            rb_free_tmp_buffer(&mut store);
            assert!(store.is_false());
            let buffer = rb_alloc_tmp_buffer_with_count(&mut store, 16, 2) as *mut Value;
            *buffer = key;
            rb_gc();
            rb_free_tmp_buffer(&mut store);
        });
    }
}
