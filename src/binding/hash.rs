use crate::{
    binding::fixnum,
    rubysys::hash,
    types::{CallbackMutPtr, CallbackPtr, Value},
    AnyObject,
};

pub fn new() -> Value {
    unsafe { hash::rb_hash_new() }
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
