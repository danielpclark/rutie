use crate::{
    binding::symbol,
    rubysys::enumerator,
    types::{c_int, Value},
    util,
};

pub fn enumeratorize(object: Value, method: &str, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);
    let method = symbol::id_to_sym(symbol::internal_id(method));

    unsafe { enumerator::rb_enumeratorize(object, method, argc, argv) }
}

// -1, 0 or 1 for a `<=>` result; raises `ArgumentError` when it is `nil`.
pub fn cmpint(result: Value, a: Value, b: Value) -> c_int {
    unsafe { enumerator::rb_cmpint(result, a, b) }
}

pub fn values_pack(arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { enumerator::rb_enum_values_pack(argc, argv) }
}
