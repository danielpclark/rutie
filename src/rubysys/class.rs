use rubysys::types::{Argc, c_char, c_int, CallbackPtr, Id, Value};

extern "C" {
    pub fn rb_class_new_instance(argc: Argc, argv: *const Value, klass: Value) -> Value;
    pub fn rb_class_superclass(klass: Value) -> Value;
    pub fn rb_const_get(klass: Value, name: Id) -> Value;
    pub fn rb_define_attr(klass: Value, name: *const c_char, read: c_int, write: c_int);
    pub fn rb_define_class(name: *const c_char, superclass: Value) -> Value;
    pub fn rb_define_class_under(outer: Value, name: *const c_char, superclass: Value) -> Value;
    pub fn rb_define_const(klass: Value, name: *const c_char, value: Value);
    pub fn rb_define_method(klass: Value, name: *const c_char, callback: CallbackPtr, argc: Argc);
    pub fn rb_define_module(name: *const c_char) -> Value;
    pub fn rb_define_module_function(klass: Value, name: *const c_char, callback: CallbackPtr, argc: Argc);
    pub fn rb_define_module_under(outer: Value, name: *const c_char) -> Value;
    pub fn rb_define_private_method(klass: Value, name: *const c_char, callback: CallbackPtr, argc: Argc);
    pub fn rb_extend_object(object: Value, module: Value);
    pub fn rb_include_module(klass: Value, module: Value);
    pub fn rb_ivar_get(object: Value, name: Id) -> Value;
    pub fn rb_ivar_set(object: Value, name: Id, value: Value) -> Value;
    pub fn rb_mod_ancestors(module: Value) -> Value;
    pub fn rb_obj_class(object: Value) -> Value;
    pub fn rb_obj_freeze(object: Value) -> Value;
    pub fn rb_obj_frozen_p(object: Value) -> Value;
    pub fn rb_prepend_module(klass: Value, module: Value);
    pub fn rb_respond_to(object: Value, id: Id) -> c_int;
    pub fn rb_singleton_class(object: Value) -> Value;

    pub fn rb_define_singleton_method(klass: Value,
                                      name: *const c_char,
                                      callback: CallbackPtr,
                                      argc: Argc);
}
