use rubysys::util as rubysys_util;

use types::{Id, Value};
use util;

pub fn get_constant(name: &str, parent_object: Value) -> Value {
    let constant_id = internal_id(name);

    unsafe { rubysys_util::rb_const_get(parent_object, constant_id) }
}

pub fn internal_id(string: &str) -> Id {
    let str = util::str_to_cstring(string);

    unsafe { rubysys_util::rb_intern(str.as_ptr()) }
}

pub fn call_method(receiver: Value, method: &str, arguments: Option<Vec<Value>>) -> Value {
    let (argc, argv) = util::process_arguments(&arguments);
    let method_id = internal_id(method);

    // TODO: Update the signature of `rb_funcallv` in ruby-sys to receive an `Option`
    unsafe { rubysys_util::rb_funcallv(receiver, method_id, argc, argv) }
}

pub fn call_public_method(receiver: Value, method: &str, arguments: Option<Vec<Value>>) -> Value {
    let (argc, argv) = util::process_arguments(&arguments);
    let method_id = internal_id(method);

    // TODO: Update the signature of `rb_funcallv_public` in ruby-sys to receive an `Option`
    unsafe { rubysys_util::rb_funcallv_public(receiver, method_id, argc, argv) }
}
