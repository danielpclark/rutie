use crate::rubysys::types::{c_char, c_int, Argc, CallbackPtr, Id, Value};

// VALUE (*)(VALUE klass)
pub type AllocFunction = rutie_callback!(type fn(klass: Value) -> Value);

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
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
    // VALUE
    // rb_class_subclasses(VALUE klass)
    //
    // The direct subclasses of `klass` (`Class#subclasses`), as an `Array`.
    pub fn rb_class_subclasses(klass: Value) -> Value;
    // VALUE
    // rb_class_attached_object(VALUE klass)
    //
    // The object the singleton class `klass` is attached to; raises
    // `TypeError` for any other class.
    pub fn rb_class_attached_object(klass: Value) -> Value;
    // VALUE
    // rb_refinement_new(void)
    //
    // An anonymous `Refinement`, not attached to any class.
    pub fn rb_refinement_new() -> Value;
    // VALUE
    // rb_cvar_find(VALUE klass, ID name, VALUE *front)
    //
    // `rb_cvar_get` that also stores the class (or the `T_ICLASS` of the
    // module) where the lookup found the variable in `*front`, which must
    // start as 0. Raises `NameError` when it is not defined.
    pub fn rb_cvar_find(klass: Value, name: Id, front: *mut Value) -> Value;
    // void
    // rb_deprecate_constant(VALUE mod, const char *name)
    //
    // Raises `NameError` when `mod` does not define the constant itself.
    pub fn rb_deprecate_constant(module: Value, name: *const c_char);
    // void
    // rb_obj_freeze_inline(VALUE obj)
    //
    // `RB_OBJ_FREEZE`: freezes `obj` (and its singleton class) without
    // calling `freeze`.
    pub fn rb_obj_freeze_inline(object: Value);
}

// `rb_alloc_func_t` as returned by `rb_get_alloc_func`: null when the class
// has no allocator.
pub type MaybeAllocFunction = Option<AllocFunction>;

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // void
    // rb_check_inheritable(VALUE super)
    //
    // Raises `TypeError` unless `super` can be subclassed.
    pub fn rb_check_inheritable(superclass: Value);
    // VALUE
    // rb_class_new(VALUE super)
    //
    // An anonymous subclass of `super`; `inherited` is not called.
    pub fn rb_class_new(superclass: Value) -> Value;
    // VALUE
    // rb_class_private_instance_methods(int argc, const VALUE *argv, VALUE mod)
    pub fn rb_class_private_instance_methods(
        argc: Argc,
        argv: *const Value,
        module: Value,
    ) -> Value;
    // VALUE
    // rb_class_protected_instance_methods(int argc, const VALUE *argv, VALUE mod)
    pub fn rb_class_protected_instance_methods(
        argc: Argc,
        argv: *const Value,
        module: Value,
    ) -> Value;
    // VALUE
    // rb_class_public_instance_methods(int argc, const VALUE *argv, VALUE mod)
    pub fn rb_class_public_instance_methods(argc: Argc, argv: *const Value, module: Value)
        -> Value;
    // VALUE
    // rb_define_class_id(ID id, VALUE super)
    //
    // An anonymous class (`id` is not used) with its own metaclass; no
    // constant is set and `inherited` is not called.
    pub fn rb_define_class_id(name: Id, superclass: Value) -> Value;
    // VALUE
    // rb_define_class_id_under(VALUE outer, ID id, VALUE super)
    pub fn rb_define_class_id_under(outer: Value, name: Id, superclass: Value) -> Value;
    // VALUE
    // rb_define_module_id(ID id)
    //
    // An anonymous module (`id` is not used).
    pub fn rb_define_module_id(name: Id) -> Value;
    // VALUE
    // rb_define_module_id_under(VALUE outer, ID id)
    pub fn rb_define_module_id_under(outer: Value, name: Id) -> Value;
    // void
    // rb_define_protected_method(VALUE klass, const char *mid, VALUE (*func)(ANYARGS), int arity)
    pub fn rb_define_protected_method(
        klass: Value,
        name: *const c_char,
        callback: CallbackPtr,
        argc: Argc,
    );
    // void
    // rb_define_global_function(const char *mid, VALUE (*func)(ANYARGS), int arity)
    //
    // A module function of `Kernel`.
    pub fn rb_define_global_function(name: *const c_char, callback: CallbackPtr, argc: Argc);
    // VALUE
    // rb_mod_included_modules(VALUE mod)
    pub fn rb_mod_included_modules(module: Value) -> Value;
    // VALUE
    // rb_mod_init_copy(VALUE clone, VALUE orig)
    //
    // `Module#initialize_copy`.
    pub fn rb_mod_init_copy(clone: Value, original: Value) -> Value;
    // VALUE
    // rb_module_new(void)
    pub fn rb_module_new() -> Value;
    // VALUE
    // rb_obj_singleton_methods(int argc, const VALUE *argv, VALUE obj)
    pub fn rb_obj_singleton_methods(argc: Argc, argv: *const Value, object: Value) -> Value;
    // void
    // rb_undef(VALUE mod, ID mid)
    //
    // The `undef` keyword: raises `NameError` for an undefined method.
    pub fn rb_undef(module: Value, name: Id);
    // VALUE
    // rb_class_get_superclass(VALUE klass)
    //
    // The raw superclass pointer: may be an include class (iclass) or `0`.
    pub fn rb_class_get_superclass(klass: Value) -> Value;
    // void
    // rb_copy_generic_ivar(VALUE clone, VALUE obj)
    pub fn rb_copy_generic_ivar(clone: Value, object: Value);
    // VALUE
    // rb_obj_setup(VALUE obj, VALUE klass, VALUE type)
    //
    // Fills the `RBasic` header of a newly allocated object.
    pub fn rb_obj_setup(object: Value, klass: Value, value_type: Value) -> Value;
    // void
    // rb_singleton_class_attached(VALUE klass, VALUE obj)
    pub fn rb_singleton_class_attached(klass: Value, object: Value);
    // VALUE
    // rb_singleton_class_clone(VALUE obj)
    pub fn rb_singleton_class_clone(object: Value) -> Value;
    // VALUE
    // rb_obj_hide(VALUE obj)
    //
    // Clears the object's class so Ruby code (`ObjectSpace`) cannot see it.
    pub fn rb_obj_hide(object: Value) -> Value;
    // VALUE
    // rb_obj_reveal(VALUE obj, VALUE klass)
    //
    // Undoes `rb_obj_hide`, setting the class to `klass`.
    pub fn rb_obj_reveal(object: Value, klass: Value) -> Value;
    // void
    // rb_freeze_singleton_class(VALUE klass)
    pub fn rb_freeze_singleton_class(klass: Value);
    // VALUE
    // rb_class_new_instance_kw(int argc, const VALUE *argv, VALUE klass, int kw_splat)
    pub fn rb_class_new_instance_kw(
        argc: Argc,
        argv: *const Value,
        klass: Value,
        kw_splat: c_int,
    ) -> Value;
    // VALUE
    // rb_class_new_instance_pass_kw(int argc, const VALUE *argv, VALUE klass)
    //
    // Passes keywords when the method running was called with keywords.
    pub fn rb_class_new_instance_pass_kw(argc: Argc, argv: *const Value, klass: Value) -> Value;
    // VALUE
    // rb_class_real(VALUE klass)
    //
    // Skips singleton classes and include classes.
    pub fn rb_class_real(klass: Value) -> Value;
    // VALUE
    // rb_obj_alloc(VALUE klass)
    pub fn rb_obj_alloc(klass: Value) -> Value;
    // void
    // rb_alias(VALUE klass, ID dst, ID src)
    pub fn rb_alias(klass: Value, new_name: Id, old_name: Id);
    // void
    // rb_attr(VALUE klass, ID name, int need_reader, int need_writer, int honour_visibility)
    pub fn rb_attr(klass: Value, name: Id, read: c_int, write: c_int, honour_visibility: c_int);
    // rb_alloc_func_t
    // rb_get_alloc_func(VALUE klass)
    pub fn rb_get_alloc_func(klass: Value) -> MaybeAllocFunction;
    // int
    // rb_method_basic_definition_p(VALUE klass, ID mid)
    //
    // Whether `mid` is still the built-in definition (not redefined).
    pub fn rb_method_basic_definition_p(klass: Value, name: Id) -> c_int;
    // VALUE
    // rb_mod_module_exec(int argc, const VALUE *argv, VALUE mod)
    //
    // Needs a Ruby block.
    pub fn rb_mod_module_exec(argc: Argc, argv: *const Value, module: Value) -> Value;
    // void
    // rb_remove_method(VALUE klass, const char *name)
    //
    // Raises `NameError` unless `klass` itself defines the method.
    pub fn rb_remove_method(klass: Value, name: *const c_char);
    // void
    // rb_remove_method_id(VALUE klass, ID mid)
    pub fn rb_remove_method_id(klass: Value, name: Id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        binding::symbol::internal_id,
        rubysys::{object, rb_cObject, variable},
        AnyObject, Class, Fixnum, Module, NilClass, Object, RString, Symbol, VM,
    };

    fn any(value: Value) -> AnyObject {
        AnyObject::from(value)
    }

    fn inspect(value: Value) -> String {
        any(value).inspect_object().to_string()
    }

    #[test]
    fn test_raw_class_and_variable_functions() {
        crate::on_ruby_thread(|| unsafe {
            let outer = Module::new("RutieRawOuter").value();

            let klass = rb_define_class_id_under(outer, internal_id("Raw"), rb_cObject);
            assert_eq!(inspect(klass), "RutieRawOuter::Raw");
            assert_eq!(
                inspect(variable::rb_class_path_cached(klass)),
                "\"RutieRawOuter::Raw\""
            );
            let module = rb_define_module_id_under(outer, internal_id("RawMod"));
            assert_eq!(inspect(module), "RutieRawOuter::RawMod");
            assert!(rb_define_module_id(internal_id("Ignored")).value != 0);

            let anonymous = rb_class_new(klass);
            assert!(variable::rb_class_path_cached(anonymous).is_nil());
            variable::rb_set_class_path(anonymous, outer, b"Named\0".as_ptr() as *const c_char);
            assert_eq!(inspect(anonymous), "RutieRawOuter::Named");
            assert_eq!(rb_class_get_superclass(anonymous), klass);

            let path = RString::new_utf8("RutieRawOuter::Raw").value();
            assert_eq!(variable::rb_path_to_class(path), klass);

            let name = b"@@raw\0".as_ptr() as *const c_char;
            variable::rb_define_class_variable(klass, name, Fixnum::new(1).value());
            variable::rb_cv_set(klass, name, Fixnum::new(2).value());
            assert_eq!(inspect(variable::rb_cv_get(klass, name)), "2");

            let object = rb_obj_alloc(klass);
            let ivar = b"@raw\0".as_ptr() as *const c_char;
            variable::rb_iv_set(object, ivar, Symbol::new("set").value());
            assert_eq!(inspect(variable::rb_iv_get(object, ivar)), ":set");
            assert_eq!(
                inspect(variable::rb_attr_get(object, internal_id("@raw"))),
                ":set"
            );
            assert!(variable::rb_attr_get(object, internal_id("@unset")).is_nil());

            Class::from(klass).const_set("FIRST", &Fixnum::new(1));
            Module::from(module).const_set("SECOND", &Fixnum::new(2));
            rb_include_module(klass, module);
            let own =
                variable::rb_const_list(variable::rb_mod_const_at(klass, std::ptr::null_mut()));
            assert_eq!(inspect(own), "[:FIRST]");
            let all =
                variable::rb_const_list(variable::rb_mod_const_of(klass, std::ptr::null_mut()));
            assert!(inspect(all).contains(":SECOND"));
            let removed = variable::rb_mod_remove_const(klass, Symbol::new("FIRST").value());
            assert_eq!(inspect(removed), "1");

            rb_attr(klass, internal_id("raw"), 1, 1, 0);
            rb_alias(klass, internal_id("raw_alias"), internal_id("raw"));
            assert_eq!(
                inspect(any(object).protect_send("raw_alias", &[]).unwrap().value()),
                ":set"
            );

            rb_undef(klass, internal_id("raw"));
            assert!(!any(object).respond_to("raw"));
            let undef_missing = crate::binding::vm::protect_value(|| {
                rb_undef(klass, internal_id("never_defined"));
                NilClass::new().value()
            });
            assert!(undef_missing.is_err());

            assert!(rb_get_alloc_func(klass).is_some());
            assert!(rb_get_alloc_func(Class::from_existing("Integer").value()).is_none());

            assert!(util_respond(object, "raw_alias", false));
            assert!(!util_respond(object, "initialize", false));
            assert!(util_respond(object, "initialize", true));

            let string = RString::new_utf8("hidden").value();
            rb_obj_hide(string);
            assert_eq!(
                rb_obj_class(rb_obj_reveal(
                    string,
                    rb_obj_class(RString::new_utf8("").value())
                )),
                rb_obj_class(RString::new_utf8("").value())
            );
            assert_eq!(inspect(string), "\"hidden\"");

            let float = object::rb_cstr_to_dbl(b"2.5e1junk\0".as_ptr() as *const c_char, 0);
            assert_eq!(float, 25.0);
        });
    }

    unsafe fn util_respond(object: Value, name: &str, private: bool) -> bool {
        object::rb_obj_respond_to(object, internal_id(name), private as c_int) != 0
    }

    #[test]
    fn test_obj_freeze_inline() {
        crate::on_ruby_thread(|| {
            let string = RString::new_utf8("thaw");

            assert!(!string.is_frozen());

            unsafe { super::rb_obj_freeze_inline(string.value()) };

            assert!(string.is_frozen());
        });
    }
}
