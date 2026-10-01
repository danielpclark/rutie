use std::ptr;

use crate::{
    binding::{fixnum, global::RubySpecialConsts},
    rubysys::hash,
    types::{CallbackMutPtr, CallbackPtr, InternalValue, Value},
    AnyObject,
};

use crate::types::c_long;

pub fn new() -> Value {
    unsafe { hash::rb_hash_new() }
}

pub fn with_capacity(capacity: usize) -> Value {
    let capacity = capacity.min(c_long::MAX as usize) as c_long;

    unsafe { hash::rb_hash_new_capa(capacity) }
}

pub fn aref(hash: Value, key: Value) -> Value {
    unsafe { hash::rb_hash_aref(hash, key) }
}

pub fn aset(hash: Value, key: Value, value: Value) -> Value {
    unsafe { hash::rb_hash_aset(hash, key, value) }
}

pub fn clear(hash: Value) {
    let _ = unsafe { hash::rb_hash_clear(hash) };
}

pub fn delete(hash: Value, key: Value) -> Value {
    unsafe { hash::rb_hash_delete(hash, key) }
}

pub fn dup(hash: Value) -> Value {
    unsafe { hash::rb_hash_dup(hash) }
}

pub fn length(hash: Value) -> i64 {
    unsafe {
        let size = hash::rb_hash_size(hash);

        fixnum::num_to_i64(size)
    }
}

use crate::util::callback_call::hash_foreach_callback as each_callback;

pub fn each<F>(hash: Value, closure_callback: F)
where
    F: FnMut(AnyObject, AnyObject),
{
    let closure_ptr = &closure_callback as *const _ as CallbackMutPtr;

    unsafe {
        hash::rb_hash_foreach(
            hash,
            each_callback::<F, AnyObject, AnyObject> as CallbackPtr,
            closure_ptr,
        );
    }
}

pub fn freeze(hash: Value) -> Value {
    unsafe { hash::rb_hash_freeze(hash) }
}

// `None` when `key` is missing; the hash's default value or proc is not used.
pub fn lookup(hash: Value, key: Value) -> Option<Value> {
    let undef = Value::from(RubySpecialConsts::Undef as InternalValue);
    let result = unsafe { hash::rb_hash_lookup2(hash, key, undef) };

    if result.is_undef() {
        None
    } else {
        Some(result)
    }
}

pub fn fetch(hash: Value, key: Value) -> Value {
    unsafe { hash::rb_hash_fetch(hash, key) }
}

// Values from `other` overwrite (Ruby's `update`/`merge!`).
pub fn update(hash: Value, other: Value) -> Value {
    unsafe { hash::rb_hash_update_by(hash, other, ptr::null()) }
}

pub fn check_hash_type(object: Value) -> Value {
    unsafe { hash::rb_check_hash_type(object) }
}

// `keys_and_values` holds key, value pairs. Does not check whether `hash`
// is frozen.
pub fn bulk_insert(hash: Value, keys_and_values: &[Value]) {
    assert!(keys_and_values.len() % 2 == 0);

    unsafe {
        hash::rb_hash_bulk_insert(
            keys_and_values.len() as c_long,
            keys_and_values.as_ptr(),
            hash,
        )
    }
}

// `(Symbol-keyed entries, other entries)`, each `None` when empty.
pub fn extract_keywords(hash: Value) -> (Option<Value>, Option<Value>) {
    let mut rest = hash;
    let keywords = unsafe { crate::rubysys::vm::rb_extract_keywords(&mut rest) };
    let present = |value: Value| if value.value == 0 { None } else { Some(value) };

    (present(keywords), present(rest))
}
