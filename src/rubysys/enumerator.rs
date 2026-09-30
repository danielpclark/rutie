use crate::rubysys::types::{c_int, Argc, Value};

extern "C" {
    // int
    // rb_cmpint(VALUE val, VALUE a, VALUE b)
    //
    // Converts a `<=>` result to -1/0/1; raises (`rb_cmperr`) for `nil`.
    pub fn rb_cmpint(result: Value, a: Value, b: Value) -> c_int;
    // void
    // rb_cmperr(VALUE x, VALUE y)
    pub fn rb_cmperr(x: Value, y: Value) -> !;
    // VALUE
    // rb_enum_values_pack(int argc, const VALUE *argv)
    pub fn rb_enum_values_pack(argc: Argc, argv: *const Value) -> Value;
    // VALUE
    // rb_enumeratorize(VALUE obj, VALUE meth, int argc, const VALUE *argv)
    pub fn rb_enumeratorize(object: Value, method: Value, argc: Argc, argv: *const Value) -> Value;
}
