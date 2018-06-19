use rubysys::rproc;

use binding::global::RubySpecialConsts;
use types::{InternalValue, Value};
use util;

pub fn call(rproc: Value, arguments: Option<Vec<Value>>) -> Value {
    let (argc, argv) = util::process_arguments(&arguments);

    unsafe {
        rproc::rb_proc_call_with_block(
            rproc,
            argc,
            argv,
            Value::from(RubySpecialConsts::Nil as InternalValue),
        )
    }
}
