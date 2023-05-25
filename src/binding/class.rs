use libc::c_int;
use libc::c_void;
use typed_data::RbDataType;

use crate::util::bool_to_value;
use crate::util::c_int_to_bool;
use crate::Boolean;
use crate::{
    binding::symbol,
    rubysys::{class, typed_data},
    typed_data::DataTypeWrapper,
    types::{c_long, Callback, CallbackPtr, Value},
    util, Object,
};

pub fn define_class(name: &str, superclass: Value) -> Value {
    let name = util::str_to_cstring(name);

    unsafe { class::rb_define_class(name.as_ptr(), superclass.into()).into() }
}

pub fn define_nested_class(outer: Value, name: &str, superclass: Value) -> Value {
    let name = util::str_to_cstring(name);

    unsafe { class::rb_define_class_under(outer.into(), name.as_ptr(), superclass.into()).into() }
}

pub fn const_get(klass: Value, name: &str) -> Value {
    unsafe { class::rb_const_get(klass.into(), symbol::internal_id(name)).into() }
}

pub fn const_set(klass: Value, name: &str, value: Value) {
    let name = util::str_to_cstring(name);

    unsafe { class::rb_define_const(klass.into(), name.as_ptr(), value.into()) };
}

pub fn object_class(object: Value) -> Value {
    unsafe { class::rb_obj_class(object.into()).into() }
}

pub fn superclass(klass: Value) -> Value {
    unsafe { class::rb_class_superclass(klass.into()).into() }
}

pub fn singleton_class(object: Value) -> Value {
    unsafe { class::rb_singleton_class(object.into()).into() }
}

pub fn ancestors(klass: Value) -> Value {
    unsafe { class::rb_mod_ancestors(klass.into()).into() }
}

pub fn new_instance(klass: Value, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { class::rb_class_new_instance(argc, argv, klass).into() }
}

pub fn instance_variable_get(object: Value, name: &str) -> Value {
    unsafe { class::rb_ivar_get(object.into(), symbol::internal_id(name)).into() }
}

pub fn instance_variable_set(object: Value, name: &str, value: Value) -> Value {
    unsafe { class::rb_ivar_set(object.into(), symbol::internal_id(name), value.into()).into() }
}

pub fn define_attribute(object: Value, name: &str, reader: bool, writer: bool) {
    let name = util::str_to_cstring(name);
    let reader = util::bool_to_c_int(reader);
    let writer = util::bool_to_c_int(writer);

    unsafe { class::rb_define_attr(object.into(), name.as_ptr(), reader, writer) };
}

pub fn respond_to(object: Value, method: &str) -> bool {
    let result = unsafe { class::rb_respond_to(object.into(), symbol::internal_id(method)) };

    c_int_to_bool(result)
}

pub fn define_method<I: Object, O: Object>(klass: Value, name: &str, callback: Callback<I, O>) {
    let name = util::str_to_cstring(name);

    unsafe {
        let callback = std::mem::transmute::<Callback<I, O>, _>(callback);
        class::rb_define_method(klass.into(), name.as_ptr(), callback, -1)
    }
}

pub fn define_private_method<I: Object, O: Object>(
    klass: Value,
    name: &str,
    callback: Callback<I, O>,
) {
    let name = util::str_to_cstring(name);

    unsafe {
        let callback = std::mem::transmute::<Callback<I, O>, _>(callback);
        class::rb_define_private_method(klass.into(), name.as_ptr(), callback, -1);
    }
}

pub fn define_singleton_method<I: Object, O: Object>(
    klass: Value,
    name: &str,
    callback: Callback<I, O>,
) {
    let name = util::str_to_cstring(name);

    unsafe {
        let callback = std::mem::transmute::<Callback<I, O>, _>(callback);
        class::rb_define_singleton_method(klass.into(), name.as_ptr(), callback, -1);
    }
}

pub fn wrap_data<T>(klass: Value, data: T, wrapper: &dyn DataTypeWrapper<T>) -> Value {
    let data = Box::into_raw(Box::new(data)) as *mut c_void;

    unsafe { typed_data::rb_data_typed_object_wrap(klass, data, wrapper.data_type()) }
}

pub fn get_data<T>(object: Value, wrapper: &dyn DataTypeWrapper<T>) -> &mut T {
    unsafe {
        let data = typed_data::rb_check_typeddata(object, wrapper.data_type());

        &mut *(data as *mut T)
    }
}

pub fn is_frozen(object: Value) -> Value {
    unsafe { class::rb_obj_frozen_p(object.into()).into() }
}

pub fn freeze(object: Value) -> Value {
    unsafe { class::rb_obj_freeze(object.into()).into() }
}

pub fn is_eql(object1: Value, object2: Value) -> Value {
    let result = unsafe { class::rb_eql(object1.into(), object2.into()) };
    // In 3.1 and earlier we get Qtrue/ Qfalse
    if cfg!(ruby_lte_3_1) {
        result.into()
    } else {
        // After 3.1 we get TRUE/true
        bool_to_value(c_int_to_bool(result))
    }
}

pub fn equals(object1: Value, object2: Value) -> Value {
    unsafe { class::rb_equal(object1.into(), object2.into()).into() }
}
