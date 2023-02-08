use crate::{
    binding::fixnum,
    rubysys::hash,
    types::{CallbackMutPtr, CallbackPtr, Value},
    AnyObject,
};

pub fn new() -> Value {
    unsafe { hash::rb_hash_new().into() }
}

pub fn aref(hash: Value, key: Value) -> Value {
    unsafe { hash::rb_hash_aref(hash.into(), key.into()).into() }
}

pub fn aset(hash: Value, key: Value, value: Value) -> Value {
    unsafe { hash::rb_hash_aset(hash.into(), key.into(), value.into()).into() }
}

pub fn clear(hash: Value) {
    let _ = unsafe { hash::rb_hash_clear(hash.into()) };
}

pub fn delete(hash: Value, key: Value) -> Value {
    unsafe { hash::rb_hash_delete(hash.into(), key.into()).into() }
}

pub fn dup(hash: Value) -> Value {
    unsafe { hash::rb_hash_dup(hash.into()).into() }
}

pub fn length(hash: Value) -> i64 {
    unsafe {
        let size = hash::rb_hash_size(hash.into()).into();

        fixnum::num_to_i64(size)
    }
}

use crate::util::callback_call::hash_foreach_callback as each_callback;

// pub fn each<F>(hash: Value, closure_callback: F)
// where
//     F: FnMut(AnyObject, AnyObject),
// {
//     let closure_ptr = &closure_callback as *const _ as CallbackMutPtr;
//
//     unsafe {
//         hash::rb_hash_foreach(
//             hash.into(),
//             each_callback::<F, AnyObject, AnyObject> as CallbackPtr,
//             closure_ptr,
//         );
//     }
// }
