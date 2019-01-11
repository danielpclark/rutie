use rubysys::gc;

use types::Value;

pub fn mark(value: Value) {
    unsafe { gc::rb_gc_mark(value) };
}

pub fn adjust_memory_usage(diff: isize) {
    unsafe { gc::rb_gc_adjust_memory_usage(diff) };
}
