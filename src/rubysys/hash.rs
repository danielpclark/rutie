use crate::rubysys::types::{c_long, CallbackMutPtr, CallbackPtr, Value};

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
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
    // rb_hash_new_capa(long capa)
    pub fn rb_hash_new_capa(capa: c_long) -> Value;
    // VALUE
    // rb_hash_size(VALUE hash)
    pub fn rb_hash_size(hash: Value) -> Value;
    // VALUE
    // rb_hash_freeze(VALUE hash)
    pub fn rb_hash_freeze(hash: Value) -> Value;
    // VALUE
    // rb_check_hash_type(VALUE hash)
    pub fn rb_check_hash_type(object: Value) -> Value;
    // VALUE
    // rb_hash_delete_if(VALUE hash)
    //
    // Needs a Ruby block.
    pub fn rb_hash_delete_if(hash: Value) -> Value;
    // VALUE
    // rb_hash_fetch(VALUE hash, VALUE key)
    //
    // Raises `KeyError` when `key` is missing.
    pub fn rb_hash_fetch(hash: Value, key: Value) -> Value;
    // VALUE
    // rb_hash_lookup(VALUE hash, VALUE key)
    pub fn rb_hash_lookup(hash: Value, key: Value) -> Value;
    // VALUE
    // rb_hash_lookup2(VALUE hash, VALUE key, VALUE def)
    //
    // Returns `def` when `key` is missing; the hash's default is not used.
    pub fn rb_hash_lookup2(hash: Value, key: Value, default: Value) -> Value;
    // VALUE
    // rb_hash_set_ifnone(VALUE hash, VALUE ifnone)
    //
    // Does not check frozen state nor clear a default proc; prefer `default=`.
    pub fn rb_hash_set_ifnone(hash: Value, ifnone: Value) -> Value;
    // VALUE
    // rb_hash_update_by(VALUE hash1, VALUE hash2, rb_hash_update_func *func)
    //
    // With a null `func`, values from `hash2` overwrite.
    pub fn rb_hash_update_by(hash: Value, other: Value, func: CallbackPtr) -> Value;
}
