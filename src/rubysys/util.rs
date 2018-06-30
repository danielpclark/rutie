use rubysys::types::{Argc, c_char, c_int, c_long, Id, Value};

extern "C" {
    pub fn rb_const_get(klass: Value, id: Id) -> Value;
    pub fn rb_funcallv(receiver: Value, method: Id, argc: Argc, argv: *const Value) -> Value;
    pub fn rb_funcallv_public(receiver: Value, method: Id, argc: Argc, argv: *const Value) -> Value;
    pub fn rb_block_call(obj: Value, method_id: Id, argc: Argc, argv: *const Value, block: extern fn(Value, Value, Argc, *const Value) -> Value, outer_scope: Value) -> Value;
    pub fn rb_scan_args(argc: Argc, argv: *const Value, fmt: *const c_char, ...) -> c_int;
    pub fn rb_ary_new_from_values(n: c_long, args: *const Value) -> Value;
    pub fn rb_intern(name: *const c_char) -> Id;
    pub fn rb_intern2(name: *const c_char, len: c_long) -> Id;
    pub fn rb_id2name(method_id: Id) -> *const c_char;
}
