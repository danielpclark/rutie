use crate::{
    rubysys::{
        constant::FL_SHAREABLE,
        ractor::{self, RbRactorLocalKey},
        types::RBasic,
    },
    types::{InternalValue, Value},
};

pub fn stdin() -> Value {
    unsafe { ractor::rb_ractor_stdin() }
}

pub fn stdout() -> Value {
    unsafe { ractor::rb_ractor_stdout() }
}

pub fn stderr() -> Value {
    unsafe { ractor::rb_ractor_stderr() }
}

// Ruby does not check the object; the caller passes an `IO`.
pub fn set_stdin(io: Value) {
    unsafe { ractor::rb_ractor_stdin_set(io) }
}

pub fn set_stdout(io: Value) {
    unsafe { ractor::rb_ractor_stdout_set(io) }
}

pub fn set_stderr(io: Value) {
    unsafe { ractor::rb_ractor_stderr_set(io) }
}

// Raises `Ractor::Error` for an object that cannot be made shareable.
pub fn make_shareable(object: Value) -> Value {
    unsafe { ractor::rb_ractor_make_shareable(object) }
}

pub fn make_shareable_copy(object: Value) -> Value {
    unsafe { ractor::rb_ractor_make_shareable_copy(object) }
}

// `rb_ractor_shareable_p`, an inline function in `ruby/ractor.h`: special
// constants and objects flagged `FL_SHAREABLE` are shareable; anything else
// is checked (and flagged when it is) by `rb_ractor_shareable_p_continue`.
pub fn is_shareable(object: Value) -> bool {
    if object.is_special_const() {
        return true;
    }

    let flags = unsafe { (*(object.value as *const RBasic)).flags };

    flags & (FL_SHAREABLE as InternalValue) != 0
        || unsafe { ractor::rb_ractor_shareable_p_continue(object) }
}

pub fn local_storage_value_newkey() -> RbRactorLocalKey {
    unsafe { ractor::rb_ractor_local_storage_value_newkey() }
}

pub fn local_storage_value_lookup(key: RbRactorLocalKey) -> Option<Value> {
    let mut value = Value::from(0);

    if unsafe { ractor::rb_ractor_local_storage_value_lookup(key, &mut value) } {
        Some(value)
    } else {
        None
    }
}

pub fn local_storage_value_set(key: RbRactorLocalKey, value: Value) {
    unsafe { ractor::rb_ractor_local_storage_value_set(key, value) }
}
