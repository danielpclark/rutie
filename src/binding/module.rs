use rubysys::class;

use binding::global::rb_cObject;
use binding::class as binding_class;
use types::{Value, Callback, CallbackPtr};
use util;

use Object;

pub fn define_module(name: &str) -> Value {
    let name = util::str_to_cstring(name);

    unsafe { class::rb_define_module(name.as_ptr()) }
}

pub fn define_nested_module(outer: Value, name: &str) -> Value {
    let name = util::str_to_cstring(name);

    unsafe { class::rb_define_module_under(outer, name.as_ptr()) }
}

pub fn define_module_function<I: Object, O: Object>(klass: Value, name: &str, callback: Callback<I, O>) {
    let name = util::str_to_cstring(name);

    unsafe {
        class::rb_define_module_function(klass, name.as_ptr(), callback as CallbackPtr, -1);
    }
}

pub fn include_module(klass: Value, module: &str) {
    let object_module = unsafe { rb_cObject };

    let module_value = binding_class::const_get(object_module, module);

    unsafe { class::rb_include_module(klass, module_value) };
}

pub fn prepend_module(klass: Value, module: &str) {
    let object_module = unsafe { rb_cObject };

    let module_value = binding_class::const_get(object_module, module);

    unsafe { class::rb_prepend_module(klass, module_value) };
}
