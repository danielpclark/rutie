use crate::rubysys::types::{c_int, Argc, Id, Value};

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
