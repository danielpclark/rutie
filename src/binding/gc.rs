use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    binding::{fixnum, hash, vm},
    rubysys::gc,
    types::{InternalValue, Value},
    util,
};

pub fn adjust_memory_usage(diff: isize) {
    unsafe { gc::rb_gc_adjust_memory_usage(diff) };
}

pub fn count() -> usize {
    unsafe { gc::rb_gc_count() }
}

pub fn disable() -> Value {
    unsafe { gc::rb_gc_disable() }
}

pub fn enable() -> Value {
    unsafe { gc::rb_gc_enable() }
}

pub fn force_recycle(obj: Value) {
    unsafe { gc::rb_gc_force_recycle(obj) }
}

pub fn mark(value: Value) {
    unsafe { gc::rb_gc_mark(value) };
}

pub fn mark_movable(value: Value) {
    unsafe { gc::rb_gc_mark_movable(value) };
}

pub fn location(value: Value) -> Value {
    unsafe { gc::rb_gc_location(value) }
}

pub fn mark_maybe(value: Value) {
    unsafe { gc::rb_gc_mark_maybe(value) };
}

// `rb_gc_register_address` roots a `VALUE` *variable* by its address, which
// must stay valid until it is unregistered. `register` only gets a copy of
// the object (registering the copy's address left the GC reading a dead
// stack slot), so registered objects are kept in an identity hash instead,
// with a count per object, and the hash itself is a permanent GC root.
fn registry() -> Value {
    static REGISTRY: AtomicUsize = AtomicUsize::new(0);

    // Only called with the GVL held, so there is no race to create it.
    let existing = REGISTRY.load(Ordering::Acquire);

    if existing != 0 {
        return Value::from(existing as InternalValue);
    }

    let hash = hash::new();
    vm::call_method(hash, "compare_by_identity", &[]);
    unsafe { gc::rb_gc_register_mark_object(hash) };
    REGISTRY.store(hash.value as usize, Ordering::Release);

    hash
}

pub fn register(obj: Value) {
    let registry = registry();
    let count = hash::lookup(registry, obj).map_or(0, fixnum::num_to_i64);

    hash::aset(registry, obj, fixnum::i64_to_num(count + 1));
}

pub fn register_mark(obj: Value) {
    unsafe { gc::rb_gc_register_mark_object(obj) }
}

pub fn start() {
    unsafe { gc::rb_gc_start() };
}

pub fn stat(key: Value) -> usize {
    unsafe { gc::rb_gc_stat(key) }
}

pub fn unregister(obj: Value) {
    let registry = registry();

    match hash::lookup(registry, obj).map(fixnum::num_to_i64) {
        Some(count) if count > 1 => {
            hash::aset(registry, obj, fixnum::i64_to_num(count - 1));
        }
        Some(_) => {
            hash::delete(registry, obj);
        }
        None => {}
    }
}

pub fn registered_count(obj: Value) -> i64 {
    hash::lookup(registry(), obj).map_or(0, fixnum::num_to_i64)
}

pub unsafe fn is_marked(obj: Value) -> bool {
    let int = gc::rb_objspace_marked_object_p(obj);

    util::c_int_to_bool(int)
}

pub fn define_finalizer(object: Value, block: Value) -> Value {
    unsafe { gc::rb_define_finalizer(object, block) }
}

pub fn undefine_finalizer(object: Value) -> Value {
    unsafe { gc::rb_undefine_finalizer(object) }
}

pub fn latest_gc_info(hash: Value) -> Value {
    unsafe { gc::rb_gc_latest_gc_info(hash) }
}

pub fn writebarrier(parent: Value, child: Value) {
    unsafe { gc::rb_gc_writebarrier(parent, child) }
}

pub fn writebarrier_unprotect(object: Value) {
    unsafe { gc::rb_gc_writebarrier_unprotect(object) }
}
