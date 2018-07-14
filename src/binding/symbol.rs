use rubysys::symbol;

use types::{c_char, c_long, Id, Value};
use util;

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

