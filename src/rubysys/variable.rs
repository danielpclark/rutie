use crate::rubysys::{
    libc::uintptr_t,
    types::{c_char, c_int, c_void, size_t, Argc, Id, Value},
};

// `rb_gvar_getter_t` and `rb_gvar_setter_t`.
// VALUE getter(ID id, VALUE *data)
pub type GlobalGetter = rutie_callback!(type fn(id: Id, data: *mut Value) -> Value);
// void setter(VALUE value, ID id, VALUE *data)
pub type GlobalSetter = rutie_callback!(type fn(value: Value, id: Id, data: *mut Value));

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // void
    // rb_define_hooked_variable(const char *name, VALUE *var,
    //                           VALUE (*getter)(ANYARGS), void (*setter)(ANYARGS))
    //
    // `*var` is marked with `rb_gc_mark_maybe`, so `var` must point to a
    // readable `VALUE`-sized word for as long as the variable exists.
    pub fn rb_define_hooked_variable(
        name: *const c_char,
        var: *mut Value,
        getter: Option<GlobalGetter>,
        setter: Option<GlobalSetter>,
    );
    // void
    // rb_define_readonly_variable(const char *name, const VALUE *var)
    pub fn rb_define_readonly_variable(name: *const c_char, var: *const Value);
    // void
    // rb_define_variable(const char *name, VALUE *var)
    pub fn rb_define_variable(name: *const c_char, var: *mut Value);
    // void
    // rb_define_virtual_variable(const char *name,
    //                            VALUE (*getter)(ANYARGS), void (*setter)(ANYARGS))
    pub fn rb_define_virtual_variable(
        name: *const c_char,
        getter: Option<GlobalGetter>,
        setter: Option<GlobalSetter>,
    );
    // VALUE
    // rb_gv_get(const char *name)
    pub fn rb_gv_get(name: *const c_char) -> Value;
    // VALUE
    // rb_gv_set(const char *name, VALUE val)
    pub fn rb_gv_set(name: *const c_char, value: Value) -> Value;
}

// int (*func)(ID name, VALUE val, st_data_t arg), for `rb_ivar_foreach`;
// returns an `st_retval` (`ST_CONTINUE`, `ST_STOP`).
pub type IvarForeachCallback = extern "C" fn(name: Id, value: Value, arg: uintptr_t) -> c_int;

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // void
    // rb_alias_variable(ID dst, ID src)
    //
    // Global variables: `alias $dst $src`.
    pub fn rb_alias_variable(new_name: Id, old_name: Id);
    // VALUE
    // rb_attr_get(VALUE obj, ID name)
    //
    // `rb_ivar_get` returning `nil` for a missing variable; `name` may be an
    // internal ID without `@`.
    pub fn rb_attr_get(object: Value, name: Id) -> Value;
    // VALUE
    // rb_autoload_load(VALUE space, ID name)
    //
    // `Qtrue` if the autoload was run, `Qfalse` if there is none (or it was
    // already loaded).
    pub fn rb_autoload_load(module: Value, name: Id) -> Value;
    // VALUE
    // rb_autoload_p(VALUE space, ID name)
    //
    // The path registered with `autoload`, or `nil`.
    pub fn rb_autoload_p(module: Value, name: Id) -> Value;
    // VALUE
    // rb_class_path_cached(VALUE mod)
    //
    // Same as `rb_mod_name`.
    pub fn rb_class_path_cached(module: Value) -> Value;
    // int
    // rb_const_defined_from(VALUE space, ID name)
    //
    // Like `rb_const_defined`, but leaves out `Object`'s constants unless
    // `space` is `Object` (the lookup of `space::name`).
    pub fn rb_const_defined_from(module: Value, name: Id) -> c_int;
    // VALUE
    // rb_const_get_at(VALUE space, ID name)
    //
    // Raises `NameError` unless `space` itself defines `name`.
    pub fn rb_const_get_at(module: Value, name: Id) -> Value;
    // VALUE
    // rb_const_get_from(VALUE space, ID name)
    //
    // The lookup of `space::name`: ancestors, but not `Object` (unless
    // `space` is `Object`); raises `NameError` when missing.
    pub fn rb_const_get_from(module: Value, name: Id) -> Value;
    // VALUE
    // rb_const_list(void*)
    //
    // Turns (and frees) the table from `rb_mod_const_at`/`rb_mod_const_of`
    // into an `Array` of `Symbol`s.
    pub fn rb_const_list(table: *mut c_void) -> Value;
    // VALUE
    // rb_cv_get(VALUE klass, const char *name)
    //
    // Raises `NameError` when the class variable is not defined.
    pub fn rb_cv_get(klass: Value, name: *const c_char) -> Value;
    // void
    // rb_cv_set(VALUE klass, const char *name, VALUE val)
    pub fn rb_cv_set(klass: Value, name: *const c_char, value: Value);
    // void
    // rb_define_class_variable(VALUE, const char*, VALUE)
    pub fn rb_define_class_variable(klass: Value, name: *const c_char, value: Value);
    // VALUE
    // rb_f_global_variables(void)
    pub fn rb_f_global_variables() -> Value;
    // VALUE
    // rb_f_trace_var(int argc, const VALUE *argv)
    //
    // `trace_var(name, command)`; without a command, the current block.
    pub fn rb_f_trace_var(argc: Argc, argv: *const Value) -> Value;
    // VALUE
    // rb_f_untrace_var(int argc, const VALUE *argv)
    pub fn rb_f_untrace_var(argc: Argc, argv: *const Value) -> Value;
    // void
    // rb_free_generic_ivar(VALUE obj)
    //
    // Frees the instance variables of a non-`T_OBJECT`; for `dfree` and
    // other code that frees `obj` itself.
    pub fn rb_free_generic_ivar(object: Value);
    // st_index_t
    // rb_ivar_count(VALUE obj)
    pub fn rb_ivar_count(object: Value) -> size_t;
    // void
    // rb_ivar_foreach(VALUE obj, int (*func)(ID name, VALUE val, st_data_t arg), st_data_t arg)
    //
    // `func` must not add or remove instance variables of `obj`.
    pub fn rb_ivar_foreach(object: Value, func: IvarForeachCallback, arg: uintptr_t);
    // VALUE
    // rb_mod_class_variables(int argc, const VALUE *argv, VALUE recv)
    pub fn rb_mod_class_variables(argc: Argc, argv: *const Value, module: Value) -> Value;
    // void *
    // rb_mod_const_at(VALUE, void*)
    //
    // Adds the constants of the module to the `st_table` (a new one when
    // null) and returns the table.
    pub fn rb_mod_const_at(module: Value, table: *mut c_void) -> *mut c_void;
    // void *
    // rb_mod_const_of(VALUE, void*)
    //
    // `rb_mod_const_at` for the module and its ancestors.
    pub fn rb_mod_const_of(module: Value, table: *mut c_void) -> *mut c_void;
    // VALUE
    // rb_mod_constants(int argc, const VALUE *argv, VALUE recv)
    pub fn rb_mod_constants(argc: Argc, argv: *const Value, module: Value) -> Value;
    // VALUE
    // rb_mod_remove_const(VALUE space, VALUE name)
    pub fn rb_mod_remove_const(module: Value, name: Value) -> Value;
    // VALUE
    // rb_mod_remove_cvar(VALUE mod, VALUE name)
    pub fn rb_mod_remove_cvar(module: Value, name: Value) -> Value;
    // VALUE
    // rb_path_to_class(VALUE path)
    pub fn rb_path_to_class(path: Value) -> Value;
    // void
    // rb_set_class_path(VALUE klass, VALUE space, const char *name)
    pub fn rb_set_class_path(klass: Value, outer: Value, name: *const c_char);
    // void
    // rb_set_class_path_string(VALUE klass, VALUE space, VALUE name)
    pub fn rb_set_class_path_string(klass: Value, outer: Value, name: Value);
    // VALUE
    // rb_iv_get(VALUE obj, const char *name)
    pub fn rb_iv_get(object: Value, name: *const c_char) -> Value;
    // VALUE
    // rb_iv_set(VALUE obj, const char *name, VALUE val)
    pub fn rb_iv_set(object: Value, name: *const c_char, value: Value) -> Value;
}
