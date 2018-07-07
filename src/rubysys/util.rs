use rubysys::types::{Argc, c_char, c_int, c_long, Id, Value};

// TODO: Remove this module and move methods in modules where they belong.
//       This will be a minor version bump.

extern "C" {
    // VALUE
    // rb_const_get(VALUE obj, ID id)
    pub fn rb_const_get(klass: Value, id: Id) -> Value; // in rubysys::class
    // VALUE
    // rb_funcallv(VALUE recv, ID mid, int argc, const VALUE *argv)
    pub fn rb_funcallv(receiver: Value, method: Id, argc: Argc, argv: *const Value) -> Value; // should be in rubysys::vm
    // VALUE
    // rb_funcallv_public(VALUE recv, ID mid, int argc, const VALUE *argv)
    pub fn rb_funcallv_public(receiver: Value, method: Id, argc: Argc, argv: *const Value) -> Value; // should be in rubysys::vm
    // VALUE
    // rb_block_call(VALUE obj, ID mid, int argc, const VALUE * argv,
    //               VALUE (*bl_proc) (ANYARGS), VALUE data2)
    pub fn rb_block_call(obj: Value, method_id: Id, argc: Argc, argv: *const Value,
                         block: extern fn(Value, Value, Argc, *const Value) -> Value,
                         outer_scope: Value) -> Value; // should be in rubysys::vm
    // int
    // rb_scan_args(int argc, const VALUE *argv, const char *fmt, ...)
    pub fn rb_scan_args(argc: Argc, argv: *const Value, fmt: *const c_char, ...) -> c_int; // should be in rubysys::class
    // VALUE
    // rb_ary_new_from_values(long n, const VALUE *elts)
    pub fn rb_ary_new_from_values(n: c_long, args: *const Value) -> Value; // in rubysys::array
    // ID
    // rb_intern(const char *name)
    pub fn rb_intern(name: *const c_char) -> Id; // should be in rubysys::symbol
    // ID
    // rb_intern2(const char *name, long len)
    pub fn rb_intern2(name: *const c_char, len: c_long) -> Id; // should be in rubysys::symbol
    // const char *
    // rb_id2name(ID id)
    pub fn rb_id2name(method_id: Id) -> *const c_char; // in rubysys::symbol
}
