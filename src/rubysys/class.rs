use rubysys::types::{Argc, c_char, c_int, CallbackPtr, Id, Value};

extern "C" {
    // VALUE
    // rb_class_new_instance(int argc, const VALUE *argv, VALUE klass)
    pub fn rb_class_new_instance(argc: Argc, argv: *const Value, klass: Value) -> Value;
    // VALUE
    // rb_class_superclass(VALUE klass)
    pub fn rb_class_superclass(klass: Value) -> Value;
    // VALUE
    // rb_const_get(VALUE obj, ID id)
    pub fn rb_const_get(klass: Value, name: Id) -> Value;
    // void
    // rb_define_attr(VALUE klass, const char *name, int read, int write)
    pub fn rb_define_attr(klass: Value, name: *const c_char, read: c_int, write: c_int);
    // VALUE
    // rb_define_class(const char *name, VALUE super)
    pub fn rb_define_class(name: *const c_char, superclass: Value) -> Value;
    // VALUE
    // rb_define_class_under(VALUE outer, const char *name, VALUE super)
    pub fn rb_define_class_under(outer: Value, name: *const c_char, superclass: Value) -> Value;
    // void
    // rb_define_const(VALUE klass, const char *name, VALUE val)
    pub fn rb_define_const(klass: Value, name: *const c_char, value: Value);
    // void
    // rb_define_method(VALUE klass, const char *name, VALUE (*func)(ANYARGS), int argc)
    pub fn rb_define_method(klass: Value, name: *const c_char, callback: CallbackPtr, argc: Argc);
    // VALUE
    // rb_define_module(const char *name)
    pub fn rb_define_module(name: *const c_char) -> Value;
    // void
    // rb_define_module_function(VALUE module, const char *name, VALUE (*func)(ANYARGS), int argc)
    pub fn rb_define_module_function(klass: Value, name: *const c_char, callback: CallbackPtr, argc: Argc);
    // VALUE
    // rb_define_module_under(VALUE outer, const char *name)
    pub fn rb_define_module_under(outer: Value, name: *const c_char) -> Value;
    // void
    // rb_define_private_method(VALUE klass, const char *name, VALUE (*func)(ANYARGS), int argc)
    pub fn rb_define_private_method(klass: Value, name: *const c_char, callback: CallbackPtr, argc: Argc);
    // void
    // rb_define_singleton_method(VALUE obj, const char *name, VALUE (*func)(ANYARGS), int argc)
    pub fn rb_define_singleton_method(klass: Value,
                                      name: *const c_char,
                                      callback: CallbackPtr,
                                      argc: Argc);
    // void
    // rb_extend_object(VALUE object, VALUE module)
    pub fn rb_extend_object(object: Value, module: Value);
    // void
    // rb_include_module(VALUE klass, VALUE module)
    pub fn rb_include_module(klass: Value, module: Value);
    // VALUE
    // rb_ivar_get(VALUE obj, ID id)
    pub fn rb_ivar_get(object: Value, name: Id) -> Value;
    // VALUE
    // rb_ivar_set(VALUE obj, ID id, VALUE val)
    pub fn rb_ivar_set(object: Value, name: Id, value: Value) -> Value;
    // VALUE
    // rb_mod_ancestors(VALUE mod)
    pub fn rb_mod_ancestors(module: Value) -> Value;
    // VALUE
    // rb_obj_class(VALUE obj)
    pub fn rb_obj_class(object: Value) -> Value;
    // VALUE
    // rb_obj_freeze(VALUE obj)
    pub fn rb_obj_freeze(object: Value) -> Value;
    // VALUE
    // rb_obj_frozen_p(VALUE obj)
    pub fn rb_obj_frozen_p(object: Value) -> Value;
    // void
    // rb_prepend_module(VALUE klass, VALUE module)
    pub fn rb_prepend_module(klass: Value, module: Value);
    // int
    // rb_respond_to(VALUE obj, ID id)
    pub fn rb_respond_to(object: Value, id: Id) -> c_int;
    // VALUE
    // rb_singleton_class(VALUE obj)
    pub fn rb_singleton_class(object: Value) -> Value;
    // int
    // rb_scan_args(int argc, const VALUE *argv, const char *fmt, ...)
    pub fn rb_scan_args(argc: Argc, argv: *const Value, fmt: *const c_char, ...) -> c_int;
}
