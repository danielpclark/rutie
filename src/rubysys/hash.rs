use rubysys::types::{CallbackPtr, CallbackMutPtr, Value};

extern "C" {
    // VALUE
    // rb_hash_aref(VALUE hash, VALUE key)
    pub fn rb_hash_aref(hash: Value, key: Value) -> Value;
    // VALUE
    // rb_hash_aset(VALUE hash, VALUE key, VALUE val)
    pub fn rb_hash_aset(hash: Value, key: Value, value: Value) -> Value;
    // VALUE
    // rb_hash_clear(VALUE hash)
    pub fn rb_hash_clear(hash: Value) -> Value;
    // VALUE
    // rb_hash_delete(VALUE hash, VALUE key)
    pub fn rb_hash_delete(hash: Value, key: Value) -> Value;
    // VALUE
    // rb_hash_dup(VALUE hash)
    pub fn rb_hash_dup(hash: Value) -> Value;
    // void
    // rb_hash_foreach(VALUE hash, int (*func)(ANYARGS), VALUE farg)
    pub fn rb_hash_foreach(hash: Value, callback: CallbackPtr, pass: CallbackMutPtr);
    // VALUE
    // rb_hash_new(void)
    pub fn rb_hash_new() -> Value;
    // VALUE
    // rb_hash_size(VALUE hash)
    pub fn rb_hash_size(hash: Value) -> Value;
}
