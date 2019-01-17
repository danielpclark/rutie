use rubysys::gc;

use types::Value;

pub fn adjust_memory_usage(diff: isize) {
    unsafe { gc::rb_gc_adjust_memory_usage(diff) };
}

pub fn disable() -> Value {
    unsafe { gc::rb_gc_disable() }
}

pub fn enable() -> Value {
    unsafe { gc::rb_gc_enable() }
}

pub fn mark(value: Value) {
    unsafe { gc::rb_gc_mark(value) };
}

pub fn start() {
    unsafe { gc::rb_gc_start() };
}
