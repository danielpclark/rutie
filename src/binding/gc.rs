use crate::{
    rubysys::gc,
    types::{CallbackPtr, Value},
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

pub fn mark_maybe(value: Value) {
    unsafe { gc::rb_gc_mark_maybe(value) };
}

pub fn register(obj: Value) {
    let addr = &obj as *const _ as CallbackPtr;

    unsafe { gc::rb_gc_register_address(addr) }
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
    let addr = &obj as *const _ as CallbackPtr;

    unsafe { gc::rb_gc_unregister_address(addr) }
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
