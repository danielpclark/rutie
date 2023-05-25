use std::mem;

use crate::{
    binding::{class as binding_class, global::rb_cObject},
    rubysys::class,
    types::{Callback, Value},
    util, Object,
};

pub fn define_module(name: &str) -> Value {
    let name = util::str_to_cstring(name);

    unsafe { class::rb_define_module(name.as_ptr()).into() }
}

pub fn define_nested_module(outer: Value, name: &str) -> Value {
    let name = util::str_to_cstring(name);

    unsafe { class::rb_define_module_under(outer.into(), name.as_ptr()).into() }
}

pub fn define_module_function<I: Object, O: Object>(
    klass: Value,
    name: &str,
    callback: Callback<I, O>,
) {
    let name = util::str_to_cstring(name);

    unsafe {
        let callback = mem::transmute::<Callback<I, O>, _>(callback);
        class::rb_define_module_function(klass.into(), name.as_ptr(), callback, -1);
    }
}

pub fn include_module(klass: Value, module: &str) {
    let object_module = unsafe { rb_cObject };

    let module_value = binding_class::const_get(object_module.into(), module);

    unsafe { class::rb_include_module(klass.into(), module_value.into()) };
}

pub fn prepend_module(klass: Value, module: &str) {
    let object_module = unsafe { rb_cObject };

    let module_value = binding_class::const_get(object_module.into(), module.into()).into();

    unsafe { class::rb_prepend_module(klass.into(), module_value) };
}
