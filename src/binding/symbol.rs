use crate::{
    rubysys::symbol,
    types::{c_char, c_long, Id, Value},
    util,
};

pub fn value_to_str<'a>(value: Value) -> &'a str {
    let ptr = sym_to_ptr(value);

    unsafe { util::cstr_to_str(ptr) }
}

pub fn value_to_string(value: Value) -> String {
    let ptr = sym_to_ptr(value);

    unsafe { util::cstr_to_string(ptr) }
}

pub fn id_to_sym(id: Id) -> Value {
    unsafe { symbol::rb_id2sym(id) }
}

fn sym_to_ptr(value: Value) -> *const c_char {
    let id = sym_to_id(value);

    id_to_name(id)
}

fn sym_to_id(sym: Value) -> Id {
    unsafe { symbol::rb_sym2id(sym) }
}

fn id_to_name(id: Id) -> *const c_char {
    unsafe { symbol::rb_id2name(id) }
}

pub fn internal_id(string: &str) -> Id {
    let str = string.as_ptr() as *const c_char;
    let len = string.len() as c_long;

    unsafe { symbol::rb_intern2(str, len) }
}

pub fn id_to_string(id: Id) -> String {
    unsafe { util::cstr_to_string(id_to_name(id)) }
}

pub fn sym_to_str(symbol: Value) -> Value {
    unsafe { symbol::rb_sym2str(symbol) }
}

// The symbol for the String `name`, or `None` (creating nothing) if no such
// symbol exists yet.
pub fn check_symbol(name: Value) -> Option<Value> {
    let mut name = name;
    let id = unsafe { symbol::rb_check_id(&mut name) };

    if id == 0 {
        None
    } else {
        Some(id_to_sym(id))
    }
}

pub fn to_symbol(name: Value) -> Value {
    unsafe { symbol::rb_to_symbol(name) }
}

pub fn is_const_name(symbol: Value) -> bool {
    util::c_int_to_bool(unsafe { symbol::rb_is_const_id(sym_to_id(symbol)) })
}

pub fn is_instance_variable_name(symbol: Value) -> bool {
    util::c_int_to_bool(unsafe { symbol::rb_is_instance_id(sym_to_id(symbol)) })
}

pub fn is_class_variable_name(symbol: Value) -> bool {
    util::c_int_to_bool(unsafe { symbol::rb_is_class_id(sym_to_id(symbol)) })
}
