use rubysys::{class, typed_data};

use binding::symbol;
use typed_data::DataTypeWrapper;
use types::{c_void, Callback, CallbackPtr, Value};
use util;

use Object;

pub fn define_class(name: &str, superclass: Value) -> Value {
    let name = util::str_to_cstring(name);

    unsafe { class::rb_define_class(name.as_ptr(), superclass) }
}

pub fn define_nested_class(outer: Value, name: &str, superclass: Value) -> Value {
    let name = util::str_to_cstring(name);

    unsafe { class::rb_define_class_under(outer, name.as_ptr(), superclass) }
}

pub fn const_get(klass: Value, name: &str) -> Value {
    unsafe { class::rb_const_get(klass, symbol::internal_id(name)) }
}

pub fn const_set(klass: Value, name: &str, value: Value) {
    let name = util::str_to_cstring(name);

    unsafe { class::rb_define_const(klass, name.as_ptr(), value) };
}

pub fn object_class(object: Value) -> Value {
    unsafe { class::rb_obj_class(object) }
}

pub fn superclass(klass: Value) -> Value {
    unsafe { class::rb_class_superclass(klass) }
}

pub fn singleton_class(object: Value) -> Value {
    unsafe { class::rb_singleton_class(object) }
}

pub fn ancestors(klass: Value) -> Value {
    unsafe { class::rb_mod_ancestors(klass) }
}

pub fn new_instance(klass: Value, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { class::rb_class_new_instance(argc, argv, klass) }
}

pub fn instance_variable_get(object: Value, name: &str) -> Value {
    unsafe { class::rb_ivar_get(object, symbol::internal_id(name)) }
}

pub fn instance_variable_set(object: Value, name: &str, value: Value) -> Value {
    unsafe { class::rb_ivar_set(object, symbol::internal_id(name), value) }
}

pub fn define_attribute(object: Value, name: &str, reader: bool, writer: bool) {
    let name = util::str_to_cstring(name);
    let reader = util::bool_to_c_int(reader);
    let writer = util::bool_to_c_int(writer);

    unsafe { class::rb_define_attr(object, name.as_ptr(), reader, writer) };
}

pub fn respond_to(object: Value, method: &str) -> bool {
    let result = unsafe { class::rb_respond_to(object, symbol::internal_id(method)) };

    util::c_int_to_bool(result)
}

pub fn define_method<I: Object, O: Object>(klass: Value, name: &str, callback: Callback<I, O>) {
    let name = util::str_to_cstring(name);

    unsafe {
        class::rb_define_method(klass, name.as_ptr(), callback as CallbackPtr, -1);
    }
}

pub fn define_private_method<I: Object, O: Object>(klass: Value, name: &str, callback: Callback<I, O>) {
    let name = util::str_to_cstring(name);

    unsafe {
        class::rb_define_private_method(klass, name.as_ptr(), callback as CallbackPtr, -1);
    }
}

pub fn define_singleton_method<I: Object, O: Object>(
    klass: Value,
    name: &str,
    callback: Callback<I, O>,
) {
    let name = util::str_to_cstring(name);

    unsafe {
        class::rb_define_singleton_method(klass, name.as_ptr(), callback as CallbackPtr, -1);
    }
}

pub fn wrap_data<T>(klass: Value, data: T, wrapper: &DataTypeWrapper<T>) -> Value {
    let data = Box::into_raw(Box::new(data)) as *mut c_void;

    unsafe { typed_data::rb_data_typed_object_wrap(klass, data, wrapper.data_type()) }
}

pub fn get_data<T>(object: Value, wrapper: &DataTypeWrapper<T>) -> &mut T {
    unsafe {
        let data = typed_data::rb_check_typeddata(object, wrapper.data_type());

        &mut *(data as *mut T)
    }
}

pub fn is_frozen(object: Value) -> Value {
    unsafe { class::rb_obj_frozen_p(object) }
}

pub fn freeze(object: Value) -> Value {
    unsafe { class::rb_obj_freeze(object) }
}
