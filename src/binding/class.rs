use std::ffi::CStr;

use crate::{
    binding::symbol,
    helpers::scan_args::ScanArgsFormat,
    rubysys::{class, typed_data},
    typed_data::DataTypeWrapper,
    types::{c_int, c_void, Callback, CallbackPtr, Id, Value},
    util, Object,
};

pub fn define_class(name: &str, superclass: Value) -> Value {
    let name = util::str_to_cstring(name);

    unsafe { class::rb_define_class(name.as_ptr(), superclass) }
}

pub fn define_nested_class(outer: Value, name: &str, superclass: Value) -> Value {
    let name = util::str_to_cstring(name);

    unsafe { class::rb_define_class_under(outer, name.as_ptr(), superclass) }
}

pub fn const_get(klass: Value, name: &str) -> Value {
    unsafe { class::rb_const_get(klass, symbol::internal_id(name)) }
}

pub fn const_set(klass: Value, name: &str, value: Value) {
    let name = util::str_to_cstring(name);

    unsafe { class::rb_define_const(klass, name.as_ptr(), value) };
}

pub fn object_class(object: Value) -> Value {
    unsafe { class::rb_obj_class(object) }
}

pub fn superclass(klass: Value) -> Value {
    unsafe { class::rb_class_superclass(klass) }
}

pub fn singleton_class(object: Value) -> Value {
    unsafe { class::rb_singleton_class(object) }
}

pub fn ancestors(klass: Value) -> Value {
    unsafe { class::rb_mod_ancestors(klass) }
}

pub fn new_instance(klass: Value, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { class::rb_class_new_instance(argc, argv, klass) }
}

pub fn instance_variable_get(object: Value, name: &str) -> Value {
    unsafe { class::rb_ivar_get(object, symbol::internal_id(name)) }
}

pub fn instance_variable_set(object: Value, name: &str, value: Value) -> Value {
    unsafe { class::rb_ivar_set(object, symbol::internal_id(name), value) }
}

pub fn define_attribute(object: Value, name: &str, reader: bool, writer: bool) {
    let name = util::str_to_cstring(name);
    let reader = util::bool_to_c_int(reader);
    let writer = util::bool_to_c_int(writer);

    unsafe { class::rb_define_attr(object, name.as_ptr(), reader, writer) };
}

pub fn respond_to(object: Value, method: &str) -> bool {
    let result = unsafe { class::rb_respond_to(object, symbol::internal_id(method)) };

    util::c_int_to_bool(result)
}

pub fn define_method<I: Object, O: Object>(klass: Value, name: &str, callback: Callback<I, O>) {
    let name = util::str_to_cstring(name);

    unsafe {
        class::rb_define_method(klass, name.as_ptr(), callback as CallbackPtr, -1);
    }
}

pub fn define_private_method<I: Object, O: Object>(
    klass: Value,
    name: &str,
    callback: Callback<I, O>,
) {
    let name = util::str_to_cstring(name);

    unsafe {
        class::rb_define_private_method(klass, name.as_ptr(), callback as CallbackPtr, -1);
    }
}

pub fn define_singleton_method<I: Object, O: Object>(
    klass: Value,
    name: &str,
    callback: Callback<I, O>,
) {
    let name = util::str_to_cstring(name);

    unsafe {
        class::rb_define_singleton_method(klass, name.as_ptr(), callback as CallbackPtr, -1);
    }
}

pub fn wrap_data<T>(klass: Value, data: T, wrapper: &dyn DataTypeWrapper<T>) -> Value {
    let data = Box::into_raw(Box::new(data)) as *mut c_void;

    unsafe { typed_data::rb_data_typed_object_wrap(klass, data, wrapper.data_type()) }
}

pub fn get_data<T>(object: Value, wrapper: &dyn DataTypeWrapper<T>) -> &mut T {
    unsafe {
        let data = typed_data::rb_check_typeddata(object, wrapper.data_type());

        &mut *(data as *mut T)
    }
}

pub fn is_frozen(object: Value) -> Value {
    unsafe { class::rb_obj_frozen_p(object) }
}

pub fn freeze(object: Value) -> Value {
    unsafe { class::rb_obj_freeze(object) }
}

pub fn is_eql(object1: Value, object2: Value) -> Value {
    unsafe { class::rb_eql(object1, object2) }
}

pub fn equals(object1: Value, object2: Value) -> Value {
    unsafe { class::rb_equal(object1, object2) }
}

pub fn is_kind_of(object: Value, klass: Value) -> bool {
    unsafe { class::rb_obj_is_kind_of(object, klass) }.is_true()
}

pub fn define_alias(klass: Value, new_name: &str, old_name: &str) {
    let new_name = util::str_to_cstring(new_name);
    let old_name = util::str_to_cstring(old_name);

    unsafe { class::rb_define_alias(klass, new_name.as_ptr(), old_name.as_ptr()) };
}

pub fn undef_method(klass: Value, name: &str) {
    let name = util::str_to_cstring(name);

    unsafe { class::rb_undef_method(klass, name.as_ptr()) };
}

pub fn define_alloc_func(klass: Value, func: class::AllocFunction) {
    unsafe { class::rb_define_alloc_func(klass, func) };
}

pub fn undef_alloc_func(klass: Value) {
    unsafe { class::rb_undef_alloc_func(klass) };
}

// Looks up the keywords in `table` (required ones first) in `keyword_hash`
// (an empty hash when it is `nil`), writing them to `values`. Found keys are
// removed from `keyword_hash`; missing optional keywords are left `Qundef`.
// Raises `ArgumentError` for missing required or (unless `allow_extra`)
// unknown keywords, so the caller owns `table` and `values`.
pub fn get_kwargs(
    keyword_hash: Value,
    table: &[Id],
    required: usize,
    allow_extra: bool,
    values: &mut [Value],
) {
    assert!(required <= table.len() && values.len() >= table.len());

    let optional = (table.len() - required) as c_int;
    let optional = if allow_extra { -1 - optional } else { optional };

    unsafe {
        class::rb_get_kwargs(
            keyword_hash,
            table.as_ptr(),
            required as c_int,
            optional,
            values.as_mut_ptr(),
        )
    };
}

// The most `VALUE *` outputs any `rb_scan_args` format can have:
// 9 required + 9 optional + splat + 9 post + keywords + block.
pub const SCAN_ARGS_MAX_VARIABLES: usize = 30;

// `rb_scan_args` takes one `VALUE *` per variable in `format`. C ignores
// surplus variadic arguments, so all `SCAN_ARGS_MAX_VARIABLES` slots of
// `out` are always passed and `rb_scan_args` fills as many as it needs.
//
// Raises `ArgumentError` on an arity mismatch, so the caller owns `format`
// and `out`.
pub fn scan_args(
    arguments: &[Value],
    format: &CStr,
    out: &mut [Value; SCAN_ARGS_MAX_VARIABLES],
) -> c_int {
    // `rb_scan_args` aborts the process on a format it cannot parse.
    let valid = format
        .to_str()
        .map(|format| ScanArgsFormat::parse(format).is_ok())
        .unwrap_or(false);
    assert!(valid, "invalid scan args format");

    let (argc, argv) = util::process_arguments(arguments);
    let o = out.as_mut_ptr();

    unsafe {
        class::rb_scan_args(
            argc,
            argv,
            format.as_ptr(),
            o,
            o.add(1),
            o.add(2),
            o.add(3),
            o.add(4),
            o.add(5),
            o.add(6),
            o.add(7),
            o.add(8),
            o.add(9),
            o.add(10),
            o.add(11),
            o.add(12),
            o.add(13),
            o.add(14),
            o.add(15),
            o.add(16),
            o.add(17),
            o.add(18),
            o.add(19),
            o.add(20),
            o.add(21),
            o.add(22),
            o.add(23),
            o.add(24),
            o.add(25),
            o.add(26),
            o.add(27),
            o.add(28),
            o.add(29),
        )
    }
}
