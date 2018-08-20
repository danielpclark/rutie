use rubysys::rproc;

use binding::global::RubySpecialConsts;
use types::{InternalValue, Value};
use util;

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

pub fn f_binding(self_: Value) -> Value {
    unsafe { rproc::rb_f_binding(self_) }
}
