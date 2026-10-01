use crate::{
    rubysys::{set, types::c_int},
    types::{InternalValue, Value},
};

pub fn new() -> Value {
    unsafe { set::rb_set_new() }
}

pub fn new_capa(capacity: usize) -> Value {
    unsafe { set::rb_set_new_capa(capacity) }
}

pub fn add(set: Value, element: Value) -> bool {
    unsafe { set::rb_set_add(set, element) }
}

pub fn lookup(set: Value, element: Value) -> bool {
    unsafe { set::rb_set_lookup(set, element) }
}

pub fn delete(set: Value, element: Value) -> bool {
    unsafe { set::rb_set_delete(set, element) }
}

pub fn clear(set: Value) -> Value {
    unsafe { set::rb_set_clear(set) }
}

pub fn size(set: Value) -> usize {
    unsafe { set::rb_set_size(set) }
}

crate::rutie_callback! {
    fn each_callback<F: FnMut(Value)>(element: Value, closure: Value) -> c_int {
        let closure = closure.value as *mut F;

        unsafe { (*closure)(element) };

        // ST_CONTINUE
        0
    }
}

pub fn each<F: FnMut(Value)>(set: Value, mut closure: F) {
    let closure = Value::from(&mut closure as *mut F as InternalValue);

    unsafe { set::rb_set_foreach(set, each_callback::<F>, closure) };
}
