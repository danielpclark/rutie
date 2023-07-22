use crate::{
    binding::global::RubySpecialConsts,
    rubysys::rproc,
    types::{InternalValue, Value},
    util,
};

pub fn call(rproc: Value, arguments: &[Value]) -> Value {
    let (argc, argv) = util::process_arguments(arguments);

    unsafe {
        rproc::rb_proc_call_with_block(
            rproc,
            argc,
            argv as *const _,
            Value::from(RubySpecialConsts::Nil as InternalValue),
        )
    }
}

pub fn binding_new() -> Value {
    unsafe { rproc::rb_binding_new() }
}
