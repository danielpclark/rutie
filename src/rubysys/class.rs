use crate::rubysys::types::{c_char, c_int, Argc, CallbackPtr, Id, Value};

// VALUE (*)(VALUE klass)
pub type AllocFunction = extern "C" fn(klass: Value) -> Value;

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
    pub fn rb_define_module_function(
        klass: Value,
        name: *const c_char,
        callback: CallbackPtr,
        argc: Argc,
    );
    // VALUE
    // rb_define_module_under(VALUE outer, const char *name)
    pub fn rb_define_module_under(outer: Value, name: *const c_char) -> Value;
    // void
    // rb_define_private_method(VALUE klass, const char *name, VALUE (*func)(ANYARGS), int argc)
    pub fn rb_define_private_method(
        klass: Value,
        name: *const c_char,
        callback: CallbackPtr,
        argc: Argc,
    );
    // void
    // rb_define_singleton_method(VALUE obj, const char *name, VALUE (*func)(ANYARGS), int argc)
    pub fn rb_define_singleton_method(
        klass: Value,
        name: *const c_char,
        callback: CallbackPtr,
        argc: Argc,
    );
    // int
    // rb_eql(VALUE obj1, VALUE obj2)
    //
    // Ruby 2 returns `Qtrue`/`Qfalse` through the `int`, Ruby 3 `TRUE`/`FALSE`;
    // only "nonzero" is meaningful.
    pub fn rb_eql(obj1: Value, obj2: Value) -> c_int;
    // VALUE
    // rb_equal(VALUE obj1, VALUE obj2)
    pub fn rb_equal(obj1: Value, obj2: Value) -> Value;
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
    // int
    // rb_scan_args_kw(int kw_flag, int argc, const VALUE *argv, const char *fmt, ...)
    //
    // `kw_flag`: `RB_SCAN_ARGS_PASS_CALLED_KEYWORDS` (0), `RB_SCAN_ARGS_KEYWORDS`
    // (1) or `RB_SCAN_ARGS_LAST_HASH_KEYWORDS` (3).
    pub fn rb_scan_args_kw(
        kw_flag: c_int,
        argc: Argc,
        argv: *const Value,
        fmt: *const c_char,
        ...
    ) -> c_int;
    // void
    // rb_define_alias(VALUE klass, const char *name1, const char *name2)
    pub fn rb_define_alias(klass: Value, new_name: *const c_char, old_name: *const c_char);
    // void
    // rb_define_alloc_func(VALUE klass, rb_alloc_func_t func)
    pub fn rb_define_alloc_func(klass: Value, func: AllocFunction);
    // void
    // rb_define_method_id(VALUE klass, ID mid, VALUE (*func)(ANYARGS), int argc)
    pub fn rb_define_method_id(klass: Value, name: Id, callback: CallbackPtr, argc: Argc);
    // int
    // rb_get_kwargs(VALUE keyword_hash, const ID *table, int required, int optional, VALUE *values)
    pub fn rb_get_kwargs(
        keyword_hash: Value,
        table: *const Id,
        required: c_int,
        optional: c_int,
        values: *mut Value,
    ) -> c_int;
    // VALUE
    // rb_obj_is_kind_of(VALUE obj, VALUE c)
    pub fn rb_obj_is_kind_of(object: Value, klass: Value) -> Value;
    // void
    // rb_undef_alloc_func(VALUE klass)
    pub fn rb_undef_alloc_func(klass: Value);
    // void
    // rb_undef_method(VALUE klass, const char *name)
    pub fn rb_undef_method(klass: Value, name: *const c_char);
    // VALUE
    // rb_class_inherited_p(VALUE mod, VALUE arg)
    //
    // `Qtrue` if `mod <= arg`, `Qfalse` if `arg < mod`, `Qnil` if unrelated.
    pub fn rb_class_inherited_p(module: Value, other: Value) -> Value;
    // VALUE
    // rb_class_instance_methods(int argc, const VALUE *argv, VALUE mod)
    pub fn rb_class_instance_methods(argc: Argc, argv: *const Value, module: Value) -> Value;
    // VALUE
    // rb_class_name(VALUE klass)
    pub fn rb_class_name(klass: Value) -> Value;
    // VALUE
    // rb_class_path(VALUE klass)
    pub fn rb_class_path(klass: Value) -> Value;
    // int
    // rb_const_defined(VALUE klass, ID id)
    pub fn rb_const_defined(klass: Value, name: Id) -> c_int;
    // int
    // rb_const_defined_at(VALUE klass, ID id)
    pub fn rb_const_defined_at(klass: Value, name: Id) -> c_int;
    // VALUE
    // rb_const_remove(VALUE mod, ID id)
    pub fn rb_const_remove(module: Value, name: Id) -> Value;
    // void
    // rb_const_set(VALUE klass, ID id, VALUE val)
    pub fn rb_const_set(klass: Value, name: Id, value: Value);
    // VALUE
    // rb_cvar_defined(VALUE klass, ID id)
    pub fn rb_cvar_defined(klass: Value, name: Id) -> Value;
    // VALUE
    // rb_cvar_get(VALUE klass, ID id)
    pub fn rb_cvar_get(klass: Value, name: Id) -> Value;
    // void
    // rb_cvar_set(VALUE klass, ID id, VALUE val)
    pub fn rb_cvar_set(klass: Value, name: Id, value: Value);
    // void
    // rb_define_global_const(const char *name, VALUE val)
    pub fn rb_define_global_const(name: *const c_char, value: Value);
    // int
    // rb_method_boundp(VALUE klass, ID id, int ex)
    //
    // `ex` bits: 0x01 excludes private (and, with 0x02, protected) methods.
    pub fn rb_method_boundp(klass: Value, name: Id, ex: c_int) -> c_int;
    // VALUE
    // rb_mod_include_p(VALUE mod, VALUE mod2)
    pub fn rb_mod_include_p(module: Value, other: Value) -> Value;
    // VALUE
    // rb_mod_module_eval(int argc, const VALUE *argv, VALUE mod)
    pub fn rb_mod_module_eval(argc: Argc, argv: *const Value, module: Value) -> Value;
    // VALUE
    // rb_mod_name(VALUE mod)
    pub fn rb_mod_name(module: Value) -> Value;
    // VALUE
    // rb_path2class(const char *path)
    pub fn rb_path2class(path: *const c_char) -> Value;
}
