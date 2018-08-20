use rubysys::types::{Argc, Value, c_int, c_void, CallbackPtr};
use rubysys::constant::UNLIMITED_ARGUMENTS;
use {AnyException, Exception};

use std::mem;

extern "C" {
    // VALUE
    // rb_proc_call_with_block(VALUE self, int argc, const VALUE *argv, VALUE passed_procval)
    pub fn rb_proc_call_with_block(rproc: Value,
                                   argc: Argc,
                                   argv: *const Value,
                                   pass_procval: Value)
                                   -> Value;
    // VALUE
    // rb_binding_new(void)
    pub fn rb_binding_new() -> Value;
}

pub fn check_arity(argc: c_int, min: c_int, max: c_int) -> Result<c_int, AnyException> {
    if argc < min || (max != UNLIMITED_ARGUMENTS as c_int && argc > max) {
        let err_mess = if min == max {
            format!("wrong number of arguments (given {}, expected {})", argc, min)
        } else if max == UNLIMITED_ARGUMENTS as c_int {
            format!("wrong number of arguments (given {}, expected {}+)", argc, min)
        } else {
            format!("wrong number of arguments (given {}, expected {}..{})", argc, min, max)
        };

        return Err(AnyException::new("ArgumentError", Some(&err_mess)));
    }

    Ok(argc)
}
