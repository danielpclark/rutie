use crate::rubysys::types::{c_int, Argc, Value};

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
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

// VALUE (*)(VALUE recv, VALUE args, VALUE eobj), the
// `rb_enumerator_size_func` computing an enumerator's `size`.
pub type EnumeratorSizeFunction =
    rutie_callback!(type fn(receiver: Value, arguments: Value, enumerator: Value) -> Value);

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_enumeratorize_with_size(VALUE recv, VALUE meth, int argc, const VALUE *argv,
    //                            rb_enumerator_size_func *func)
    pub fn rb_enumeratorize_with_size(
        object: Value,
        method: Value,
        argc: Argc,
        argv: *const Value,
        size: Option<EnumeratorSizeFunction>,
    ) -> Value;
    // VALUE
    // rb_enumeratorize_with_size_kw(VALUE recv, VALUE meth, int argc, const VALUE *argv,
    //                               rb_enumerator_size_func *func, int kw_splat)
    pub fn rb_enumeratorize_with_size_kw(
        object: Value,
        method: Value,
        argc: Argc,
        argv: *const Value,
        size: Option<EnumeratorSizeFunction>,
        kw_splat: c_int,
    ) -> Value;
}
