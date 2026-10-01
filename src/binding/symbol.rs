use crate::{
    rubysys::{encoding, symbol},
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

// Raises `EncodingError` for bytes that are invalid in `enc`.
pub fn intern_with_encoding(name: &[u8], enc: Value) -> Value {
    let (ptr, len) = (name.as_ptr() as *const c_char, name.len() as c_long);

    unsafe { id_to_sym(symbol::rb_intern3(ptr, len, encoding::rb_to_encoding(enc))) }
}

// The existing symbol, or `nil`; raises `EncodingError` for invalid bytes.
pub fn check_symbol_with_encoding(name: &[u8], enc: Value) -> Value {
    let (ptr, len) = (name.as_ptr() as *const c_char, name.len() as c_long);

    unsafe { symbol::rb_check_symbol_cstr(ptr, len, encoding::rb_to_encoding(enc)) }
}

pub fn is_symbol_name(name: &[u8], enc: Value) -> bool {
    let (ptr, len) = (name.as_ptr() as *const c_char, name.len() as c_long);

    util::c_int_to_bool(unsafe {
        symbol::rb_enc_symname2_p(ptr, len, encoding::rb_to_encoding(enc))
    })
}

// Raises `NameError` for names without a setter form (operators, ...).
pub fn attrset(symbol: Value) -> Value {
    unsafe { id_to_sym(symbol::rb_id_attrset(sym_to_id(symbol))) }
}

pub fn is_attrset_name(symbol: Value) -> bool {
    util::c_int_to_bool(unsafe { symbol::rb_is_attrset_id(sym_to_id(symbol)) })
}

pub fn is_global_name(symbol: Value) -> bool {
    util::c_int_to_bool(unsafe { symbol::rb_is_global_id(sym_to_id(symbol)) })
}

pub fn is_local_name(symbol: Value) -> bool {
    util::c_int_to_bool(unsafe { symbol::rb_is_local_id(sym_to_id(symbol)) })
}

pub fn is_junk_name(symbol: Value) -> bool {
    util::c_int_to_bool(unsafe { symbol::rb_is_junk_id(sym_to_id(symbol)) })
}

pub fn all_symbols() -> Value {
    unsafe { symbol::rb_sym_all_symbols() }
}

pub fn last_line() -> Value {
    unsafe { symbol::rb_lastline_get() }
}

pub fn set_last_line(value: Value) {
    unsafe { symbol::rb_lastline_set(value) }
}
