use std::{ptr, slice};

use crate::{
    binding::{global::RubySpecialConsts, symbol, vm},
    rubysys::{
        rproc,
        typed_data::{self, RbDataType, RbDataTypeFunction},
    },
    types::{c_char, c_int, c_void, InternalValue, Value},
    util,
};

pub fn call(rproc: Value, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe {
        rproc::rb_proc_call_with_block(
            rproc,
            argc,
            argv,
            Value::from(RubySpecialConsts::Nil as InternalValue),
        )
    }
}

pub fn binding_new() -> Value {
    unsafe { rproc::rb_binding_new() }
}

// The Rust closure behind a `Proc` made by `new`. It is owned by a hidden
// typed-data object that the proc's block keeps alive, so it is dropped when
// the proc is garbage collected.
struct ProcClosure(Box<dyn FnMut(&[Value]) -> Value>);

extern "C" fn free_proc_closure(data: *mut c_void) {
    unsafe { drop(Box::from_raw(data as *mut ProcClosure)) };
}

lazy_static! {
    static ref PROC_CLOSURE_TYPE: RbDataType = RbDataType {
        wrap_struct_name: b"Rutie/ProcClosure\0".as_ptr() as *const c_char,
        function: RbDataTypeFunction {
            dmark: None,
            dfree: Some(free_proc_closure),
            dsize: None,
            reserved: [ptr::null_mut(); 2],
        },
        parent: ptr::null(),
        data: ptr::null_mut(),
        flags: Value::from(0),
    };
}

rutie_callback! {
    pub(crate) fn proc_callback(
        _yielded: Value,
        data: Value,
        argc: c_int,
        argv: *const Value,
        _block_arg: Value,
    ) -> Value {
        let closure = unsafe {
            &mut *(typed_data::rb_check_typeddata(data, &*PROC_CLOSURE_TYPE) as *mut ProcClosure)
        };
        let arguments: &[Value] = if argc > 0 && !argv.is_null() {
            unsafe { slice::from_raw_parts(argv, argc as usize) }
        } else {
            &[]
        };

        vm::call_catching_panic(move || (closure.0)(arguments))
    }
}

// A hidden typed-data object owning `func`, to pass as the data argument of
// `proc_callback` (for `rb_proc_new` and `rb_fiber_new`).
pub(crate) fn closure_data<F>(func: F) -> Value
where
    F: FnMut(&[Value]) -> Value + 'static,
{
    let closure = Box::into_raw(Box::new(ProcClosure(Box::new(func)))) as *mut c_void;

    // `klass` 0 makes a hidden object that Ruby code cannot reach.
    unsafe { typed_data::rb_data_typed_object_wrap(Value::from(0), closure, &*PROC_CLOSURE_TYPE) }
}

pub fn new<F>(func: F) -> Value
where
    F: FnMut(&[Value]) -> Value + 'static,
{
    let data = closure_data(func);

    unsafe { rproc::rb_proc_new(proc_callback, data) }
}

pub fn arity(rproc: Value) -> i32 {
    unsafe { rproc::rb_proc_arity(rproc) as i32 }
}

pub fn is_lambda(rproc: Value) -> bool {
    unsafe { rproc::rb_proc_lambda_p(rproc) }.is_true()
}

pub fn method_call(method: Value, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { rproc::rb_method_call(argc, argv, method) }
}

pub fn object_method_arity(object: Value, name: &str) -> i32 {
    unsafe { rproc::rb_obj_method_arity(object, symbol::internal_id(name)) as i32 }
}

pub fn module_method_arity(module: Value, name: &str) -> i32 {
    unsafe { rproc::rb_mod_method_arity(module, symbol::internal_id(name)) as i32 }
}

// With `keywords`, the last argument must be a `Hash`; it is passed as
// keywords. A `nil` block passes no block.
pub fn call_with_block(rproc: Value, arguments: &[Value], block: Value, keywords: bool) -> Value {
    let (argc, argv) = util::process_arguments(arguments);
    let kw_splat = util::bool_to_c_int(keywords);

    unsafe { rproc::rb_proc_call_with_block_kw(rproc, argc, argv, block, kw_splat) }
}

pub fn method_call_with_block(
    method: Value,
    arguments: &[Value],
    block: Value,
    keywords: bool,
) -> Value {
    let (argc, argv) = util::process_arguments(arguments);
    let kw_splat = util::bool_to_c_int(keywords);

    unsafe { rproc::rb_method_call_with_block_kw(argc, argv, method, block, kw_splat) }
}

// The last argument must be a `Hash`; it is passed as keywords.
pub fn call_with_keywords(rproc: Value, arguments: &[Value]) -> Value {
    let arguments = unsafe {
        crate::rubysys::array::rb_ary_new_from_values(
            arguments.len() as crate::types::c_long,
            arguments.as_ptr(),
        )
    };

    unsafe { rproc::rb_proc_call_kw(rproc, arguments, 1) }
}

// The last argument must be a `Hash`; it is passed as keywords.
pub fn method_call_with_keywords(method: Value, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe { rproc::rb_method_call_kw(argc, argv, method, 1) }
}
