use crate::{rubysys::gc, types::Value, util};

pub fn adjust_memory_usage(diff: isize) {
    unsafe { gc::rb_gc_adjust_memory_usage(diff) };
}

pub fn count() -> usize {
    unsafe { gc::rb_gc_count() as usize }
}

pub fn disable() -> Value {
    unsafe { gc::rb_gc_disable().into() }
}

pub fn enable() -> Value {
    unsafe { gc::rb_gc_enable().into() }
}

pub fn force_recycle(obj: Value) {
    unsafe { gc::rb_gc_force_recycle(obj.into()) }
}

pub fn mark(value: Value) {
    unsafe { gc::rb_gc_mark(value.into()) };
}

pub fn mark_maybe(value: Value) {
    unsafe { gc::rb_gc_mark_maybe(value.into()) };
}

pub fn register(obj: Value) {
    let addr = &obj as *const _ as *mut _;

    unsafe { gc::rb_gc_register_address(addr) }
}

pub fn register_mark(obj: Value) {
    unsafe { gc::rb_gc_register_mark_object(obj.into()) }
}

pub fn start() {
    unsafe { gc::rb_gc_start() };
}

pub fn stat(key: Value) -> usize {
    unsafe { gc::rb_gc_stat(key.into()) as usize }
}

pub fn unregister(obj: Value) {
    let addr = &obj as *const _ as *mut _;

    unsafe { gc::rb_gc_unregister_address(addr) }
}

pub unsafe fn is_marked(obj: Value) -> bool {
    let int = gc::rb_objspace_marked_object_p(obj);

    util::c_int_to_bool(int)
}
