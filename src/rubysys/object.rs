use crate::rubysys::types::{c_char, c_double, c_int, Argc, Id, Value};

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_Array(VALUE val)
    pub fn rb_Array(object: Value) -> Value;
    // VALUE
    // rb_check_funcall(VALUE recv, ID mid, int argc, const VALUE *argv)
    //
    // Returns `Qundef` when `recv` does not respond to `mid`.
    pub fn rb_check_funcall(receiver: Value, method: Id, argc: Argc, argv: *const Value) -> Value;
    // VALUE
    // rb_Float(VALUE val)
    pub fn rb_Float(object: Value) -> Value;
    // VALUE
    // rb_hash(VALUE obj)
    pub fn rb_hash(object: Value) -> Value;
    // VALUE
    // rb_Hash(VALUE val)
    pub fn rb_Hash(object: Value) -> Value;
    // VALUE
    // rb_inspect(VALUE obj)
    pub fn rb_inspect(object: Value) -> Value;
    // VALUE
    // rb_Integer(VALUE val)
    pub fn rb_Integer(object: Value) -> Value;
    // VALUE
    // rb_ivar_defined(VALUE obj, ID id)
    pub fn rb_ivar_defined(object: Value, name: Id) -> Value;
    // VALUE
    // rb_obj_as_string(VALUE obj)
    pub fn rb_obj_as_string(object: Value) -> Value;
    // VALUE
    // rb_obj_clone(VALUE obj)
    pub fn rb_obj_clone(object: Value) -> Value;
    // VALUE
    // rb_obj_dup(VALUE obj)
    pub fn rb_obj_dup(object: Value) -> Value;
    // VALUE
    // rb_obj_id(VALUE obj)
    pub fn rb_obj_id(object: Value) -> Value;
    // VALUE
    // rb_obj_instance_eval(int argc, const VALUE *argv, VALUE self)
    pub fn rb_obj_instance_eval(argc: Argc, argv: *const Value, object: Value) -> Value;
    // VALUE
    // rb_obj_instance_variables(VALUE obj)
    pub fn rb_obj_instance_variables(object: Value) -> Value;
    // VALUE
    // rb_obj_is_instance_of(VALUE obj, VALUE c)
    pub fn rb_obj_is_instance_of(object: Value, klass: Value) -> Value;
    // VALUE
    // rb_obj_method(VALUE obj, VALUE vid)
    pub fn rb_obj_method(object: Value, name: Value) -> Value;
    // VALUE
    // rb_obj_remove_instance_variable(VALUE obj, VALUE name)
    pub fn rb_obj_remove_instance_variable(object: Value, name: Value) -> Value;
    // void
    // rb_p(VALUE obj)
    pub fn rb_p(object: Value);
    // VALUE
    // rb_str_format(int argc, const VALUE *argv, VALUE fmt)
    pub fn rb_str_format(argc: c_int, argv: *const Value, format: Value) -> Value;
    // VALUE
    // rb_String(VALUE val)
    pub fn rb_String(object: Value) -> Value;
}

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_any_to_s(VALUE obj)
    //
    // The default `#<ClassName:0x...>` form of `Kernel#to_s`.
    pub fn rb_any_to_s(object: Value) -> Value;
    // VALUE
    // rb_check_convert_type(VALUE val, int type, const char *name, const char *mid)
    //
    // `nil` when `val` does not respond to `mid`; raises `TypeError` when
    // `mid` returns something not of `type`.
    pub fn rb_check_convert_type(
        object: Value,
        value_type: c_int,
        class_name: *const c_char,
        method: *const c_char,
    ) -> Value;
    // VALUE
    // rb_check_to_float(VALUE val)
    //
    // `nil` unless `val` is a `Numeric` that converts with `to_f`.
    pub fn rb_check_to_float(object: Value) -> Value;
    // VALUE
    // rb_check_to_int(VALUE val)
    //
    // `nil` unless `to_int` returns an `Integer`.
    pub fn rb_check_to_int(object: Value) -> Value;
    // VALUE
    // rb_check_to_integer(VALUE val, const char *mid)
    //
    // `nil` unless `mid` returns an `Integer`.
    pub fn rb_check_to_integer(object: Value, method: *const c_char) -> Value;
    // VALUE
    // rb_convert_type(VALUE val, int type, const char *name, const char *mid)
    //
    // Raises `TypeError` when `val` cannot be converted.
    pub fn rb_convert_type(
        object: Value,
        value_type: c_int,
        class_name: *const c_char,
        method: *const c_char,
    ) -> Value;
    // double
    // rb_cstr_to_dbl(const char *str, int mode)
    //
    // `0` parses as much of `str` as it can. A nonzero `mode` (badcheck)
    // should raise `ArgumentError` for malformed input, but Ruby 4.0 crashes
    // there instead (it builds the message with a null encoding), so only
    // pass `0`, or use `rb_str_to_dbl`.
    pub fn rb_cstr_to_dbl(string: *const c_char, badcheck: c_int) -> c_double;
    // VALUE
    // rb_obj_init_copy(VALUE src, VALUE dst)
    //
    // `Kernel#initialize_copy`: raises unless `dst` is unfrozen and of the
    // same class as `src`.
    pub fn rb_obj_init_copy(object: Value, original: Value) -> Value;
    // VALUE
    // rb_to_int(VALUE val)
    //
    // Raises `TypeError` unless `to_int` returns an `Integer`.
    pub fn rb_to_int(object: Value) -> Value;
    // int
    // rb_obj_respond_to(VALUE obj, ID mid, int private_p)
    pub fn rb_obj_respond_to(object: Value, method: Id, include_private: c_int) -> c_int;
    // VALUE
    // rb_obj_instance_exec(int argc, const VALUE *argv, VALUE recv)
    //
    // Needs a Ruby block.
    pub fn rb_obj_instance_exec(argc: Argc, argv: *const Value, object: Value) -> Value;
    // VALUE
    // rb_check_funcall_kw(VALUE recv, ID mid, int argc, const VALUE *argv, int kw_splat)
    //
    // Returns `Qundef` when `recv` does not respond to `mid`.
    pub fn rb_check_funcall_kw(
        receiver: Value,
        method: Id,
        argc: Argc,
        argv: *const Value,
        kw_splat: c_int,
    ) -> Value;
    // VALUE
    // rb_apply(VALUE recv, ID mid, VALUE args)
    //
    // Calls `mid` with the elements of the `Array` `args`.
    pub fn rb_apply(receiver: Value, method: Id, arguments: Value) -> Value;
}
