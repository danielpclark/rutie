use rubysys::gc;

use types::{ Value, CallbackPtr };

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

pub fn mark_locations(start: Value, end: Value) {
    let start = &start as *const _ as CallbackPtr;
    let end = &end as *const _ as CallbackPtr;

    unsafe { gc::rb_gc_mark_locations(start, end) }
}

pub fn start() {
    unsafe { gc::rb_gc_start() };
}
