use crate::{
    binding::{string, symbol},
    rubysys::object,
    types::Value,
    util,
};

pub fn dup(object: Value) -> Value {
    unsafe { object::rb_obj_dup(object) }
}

pub fn clone(object: Value) -> Value {
    unsafe { object::rb_obj_clone(object) }
}

pub fn id(object: Value) -> Value {
    unsafe { object::rb_obj_id(object) }
}

pub fn inspect(object: Value) -> Value {
    unsafe { object::rb_inspect(object) }
}

pub fn as_string(object: Value) -> Value {
    unsafe { object::rb_obj_as_string(object) }
}

pub fn is_instance_of(object: Value, klass: Value) -> bool {
    unsafe { object::rb_obj_is_instance_of(object, klass) }.is_true()
}

pub fn method(object: Value, name: &str) -> Value {
    let name = symbol::id_to_sym(symbol::internal_id(name));

    unsafe { object::rb_obj_method(object, name) }
}

// `None` when `receiver` does not respond to `method`.
pub fn check_funcall(receiver: Value, method: &str, arguments: &[Value]) -> Option<Value> {
    let (argc, argv) = util::process_arguments(arguments);
    let result =
        unsafe { object::rb_check_funcall(receiver, symbol::internal_id(method), argc, argv) };

    if result.is_undef() {
        None
    } else {
        Some(result)
    }
}

pub fn instance_variables(object: Value) -> Value {
    unsafe { object::rb_obj_instance_variables(object) }
}

pub fn is_instance_variable_defined(object: Value, name: &str) -> bool {
    unsafe { object::rb_ivar_defined(object, symbol::internal_id(name)) }.is_true()
}

pub fn remove_instance_variable(object: Value, name: &str) -> Value {
    let name = symbol::id_to_sym(symbol::internal_id(name));

    unsafe { object::rb_obj_remove_instance_variable(object, name) }
}

pub fn hash(object: Value) -> Value {
    unsafe { object::rb_hash(object) }
}

pub fn instance_eval(object: Value, code: &str) -> Value {
    let arguments = [string::new_utf8(code)];
    let (argc, argv) = util::process_arguments(&arguments);

    unsafe { object::rb_obj_instance_eval(argc, argv, object) }
}

pub fn to_string(object: Value) -> Value {
    unsafe { object::rb_String(object) }
}

pub fn to_array(object: Value) -> Value {
    unsafe { object::rb_Array(object) }
}

pub fn to_integer(object: Value) -> Value {
    unsafe { object::rb_Integer(object) }
}

pub fn to_float(object: Value) -> Value {
    unsafe { object::rb_Float(object) }
}

pub fn to_hash(object: Value) -> Value {
    unsafe { object::rb_Hash(object) }
}

pub fn format(format: Value, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { object::rb_str_format(argc, argv, format) }
}

pub fn p(object: Value) {
    unsafe { object::rb_p(object) }
}
