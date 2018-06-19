use rubysys::types::{CallbackPtr, CallbackMutPtr, Value};

extern "C" {
    pub fn rb_hash_aref(hash: Value, key: Value) -> Value;
    pub fn rb_hash_aset(hash: Value, key: Value, value: Value) -> Value;
    pub fn rb_hash_clear(hash: Value) -> Value;
    pub fn rb_hash_delete(hash: Value, key: Value) -> Value;
    pub fn rb_hash_dup(hash: Value) -> Value;
    pub fn rb_hash_foreach(hash: Value, callback: CallbackPtr, pass: CallbackMutPtr);
    pub fn rb_hash_new() -> Value;
    pub fn rb_hash_size(hash: Value) -> Value;
}
