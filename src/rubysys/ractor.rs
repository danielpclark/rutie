use crate::rubysys::types::{c_void, Value};

// void (*)(void *ptr)
pub type RactorLocalStorageFunction = rutie_callback!(type fn(ptr: *mut c_void));

// struct rb_ractor_local_storage_type
//
// How Ruby marks and frees the pointers kept under a key made by
// `rb_ractor_local_storage_ptr_newkey`, each of which may be null.
#[repr(C)]
pub struct RbRactorLocalStorageType {
    pub mark: Option<RactorLocalStorageFunction>,
    pub free: Option<RactorLocalStorageFunction>,
}

unsafe impl Send for RbRactorLocalStorageType {}
unsafe impl Sync for RbRactorLocalStorageType {}

// struct rb_ractor_local_key_struct, opaque.
#[repr(C)]
pub struct RbRactorLocalKeyStruct {
    _private: [u8; 0],
}

// typedef struct rb_ractor_local_key_struct *rb_ractor_local_key_t;
//
// A key is shared by every Ractor, and each Ractor keeps its own value for
// it. Keys are never freed.
pub type RbRactorLocalKey = *mut RbRactorLocalKeyStruct;

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // RUBY_EXTERN const struct rb_ractor_local_storage_type rb_ractor_local_storage_type_free;
    //
    // `RB_RACTOR_LOCAL_STORAGE_TYPE_FREE`: no `mark`, and `ruby_xfree` as `free`.
    pub static rb_ractor_local_storage_type_free: RbRactorLocalStorageType;

    // VALUE
    // rb_ractor_stdin(void)
    pub fn rb_ractor_stdin() -> Value;
    // VALUE
    // rb_ractor_stdout(void)
    pub fn rb_ractor_stdout() -> Value;
    // VALUE
    // rb_ractor_stderr(void)
    pub fn rb_ractor_stderr() -> Value;
    // void
    // rb_ractor_stdin_set(VALUE io)
    //
    // Sets the current Ractor's `$stdin` without checking `io`.
    pub fn rb_ractor_stdin_set(io: Value);
    // void
    // rb_ractor_stdout_set(VALUE io)
    pub fn rb_ractor_stdout_set(io: Value);
    // void
    // rb_ractor_stderr_set(VALUE io)
    pub fn rb_ractor_stderr_set(io: Value);

    // rb_ractor_local_key_t
    // rb_ractor_local_storage_value_newkey(void)
    pub fn rb_ractor_local_storage_value_newkey() -> RbRactorLocalKey;
    // VALUE
    // rb_ractor_local_storage_value(rb_ractor_local_key_t key)
    //
    // The current Ractor's value for `key`, or `Qnil` when it has none.
    pub fn rb_ractor_local_storage_value(key: RbRactorLocalKey) -> Value;
    // bool
    // rb_ractor_local_storage_value_lookup(rb_ractor_local_key_t key, VALUE *val)
    //
    // Stores the current Ractor's value for `key` in `*val` and returns
    // `true`, or returns `false` when it has none.
    pub fn rb_ractor_local_storage_value_lookup(key: RbRactorLocalKey, val: *mut Value) -> bool;
    // void
    // rb_ractor_local_storage_value_set(rb_ractor_local_key_t key, VALUE val)
    //
    // The value is marked as long as the Ractor lives.
    pub fn rb_ractor_local_storage_value_set(key: RbRactorLocalKey, val: Value);

    // rb_ractor_local_key_t
    // rb_ractor_local_storage_ptr_newkey(const struct rb_ractor_local_storage_type *type)
    //
    // Ruby keeps `type`, which must live forever.
    pub fn rb_ractor_local_storage_ptr_newkey(
        storage_type: *const RbRactorLocalStorageType,
    ) -> RbRactorLocalKey;
    // void *
    // rb_ractor_local_storage_ptr(rb_ractor_local_key_t key)
    //
    // The current Ractor's pointer for `key`, or null when it has none.
    pub fn rb_ractor_local_storage_ptr(key: RbRactorLocalKey) -> *mut c_void;
    // void
    // rb_ractor_local_storage_ptr_set(rb_ractor_local_key_t key, void *ptr)
    //
    // The type's `free` runs on the pointer when the Ractor is freed;
    // replacing the pointer does not free the old one.
    pub fn rb_ractor_local_storage_ptr_set(key: RbRactorLocalKey, ptr: *mut c_void);

    // VALUE
    // rb_ractor_make_shareable(VALUE obj)
    //
    // Deep-freezes `obj` and marks it shareable (`Ractor.make_shareable`);
    // raises `Ractor::Error` for an object that cannot be shared.
    pub fn rb_ractor_make_shareable(object: Value) -> Value;
    // VALUE
    // rb_ractor_make_shareable_copy(VALUE obj)
    //
    // `rb_ractor_make_shareable` on a deep copy of `obj`.
    pub fn rb_ractor_make_shareable_copy(object: Value) -> Value;
    // bool
    // rb_ractor_shareable_p_continue(VALUE obj)
    //
    // The slow path of the inline `rb_ractor_shareable_p` (see
    // `binding::ractor::is_shareable`), for heap objects without the
    // `FL_SHAREABLE` flag.
    pub fn rb_ractor_shareable_p_continue(object: Value) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    static NO_CALLBACKS: RbRactorLocalStorageType = RbRactorLocalStorageType {
        mark: None,
        free: None,
    };

    #[test]
    fn test_ractor_local_storage_ptr() {
        crate::on_ruby_thread(|| unsafe {
            // `RB_RACTOR_LOCAL_STORAGE_TYPE_FREE`: no mark function, a free one.
            assert!(rb_ractor_local_storage_type_free.mark.is_none());
            assert!(rb_ractor_local_storage_type_free.free.is_some());

            static mut DATA: u64 = 42;

            let key = rb_ractor_local_storage_ptr_newkey(&NO_CALLBACKS);
            let data = std::ptr::addr_of_mut!(DATA) as *mut c_void;

            assert!(rb_ractor_local_storage_ptr(key).is_null());

            rb_ractor_local_storage_ptr_set(key, data);

            assert_eq!(rb_ractor_local_storage_ptr(key), data);
            assert_eq!(*(rb_ractor_local_storage_ptr(key) as *const u64), 42);
        });
    }
}
