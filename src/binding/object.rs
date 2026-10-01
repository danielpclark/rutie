use std::{ffi::CStr, hint::black_box};

use libc::uintptr_t;

use crate::{
    binding::{string, symbol},
    rubysys::{array, cstr, object, variable},
    types::{c_int, st_retval, Id, InternalValue, Value, ValueType},
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

pub fn any_to_s(object: Value) -> Value {
    unsafe { object::rb_any_to_s(object) }
}

// Raises `TypeError` when `object` cannot be converted, so the caller owns
// `class_name` and `method`.
pub fn convert_type(
    object: Value,
    value_type: ValueType,
    class_name: &CStr,
    method: &CStr,
) -> Value {
    unsafe {
        object::rb_convert_type(
            object,
            value_type as c_int,
            class_name.as_ptr(),
            method.as_ptr(),
        )
    }
}

// `nil` when `object` does not respond to `method`; raises `TypeError` when
// `method` returns something else, so the caller owns the C strings.
pub fn check_convert_type(
    object: Value,
    value_type: ValueType,
    class_name: &CStr,
    method: &CStr,
) -> Value {
    unsafe {
        object::rb_check_convert_type(
            object,
            value_type as c_int,
            class_name.as_ptr(),
            method.as_ptr(),
        )
    }
}

pub fn to_int(object: Value) -> Value {
    unsafe { object::rb_to_int(object) }
}

pub fn check_to_int(object: Value) -> Value {
    unsafe { object::rb_check_to_int(object) }
}

// The caller owns `method`, since the conversion can raise.
pub fn check_to_integer(object: Value, method: &CStr) -> Value {
    unsafe { object::rb_check_to_integer(object, method.as_ptr()) }
}

pub fn check_to_float(object: Value) -> Value {
    unsafe { object::rb_check_to_float(object) }
}

// `None` when `receiver` does not respond to `method`. The last argument
// must be a `Hash`; it is passed as keywords.
pub fn check_funcall_with_keywords(
    receiver: Value,
    method: &str,
    arguments: &[Value],
) -> Option<Value> {
    let (argc, argv) = util::process_arguments(arguments);
    let result = unsafe {
        object::rb_check_funcall_kw(receiver, symbol::internal_id(method), argc, argv, 1)
    };

    if result.is_undef() {
        None
    } else {
        Some(result)
    }
}

pub fn instance_variable_count(object: Value) -> usize {
    unsafe { variable::rb_ivar_count(object) as usize }
}

// Pushes each name (as a `Symbol`) and value onto the `Array` `arg`.
extern "C" fn collect_instance_variable(name: Id, value: Value, arg: uintptr_t) -> c_int {
    let pairs = Value::from(arg as InternalValue);

    unsafe {
        array::rb_ary_push(pairs, symbol::id_to_sym(name));
        array::rb_ary_push(pairs, value);
    }

    st_retval::Continue as c_int
}

// The instance variables of `object`, as a flat `Array` of names and values.
pub fn instance_variable_pairs(object: Value) -> Value {
    // Kept in this frame's stack (see `RB_GC_GUARD`) while Ruby fills it.
    let pairs = unsafe { array::rb_ary_new() };
    black_box(&pairs);

    unsafe {
        variable::rb_ivar_foreach(object, collect_instance_variable, pairs.value as uintptr_t)
    };

    pairs
}

// Copied at once: the C string belongs to the class's name String.
pub fn class_name(object: Value) -> String {
    unsafe { util::cstr_to_string(cstr::rb_obj_classname(object)) }
}
