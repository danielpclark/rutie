use rubysys::types::{Argc, Value, c_int, c_void, CallbackPtr};
use rubysys::constant::UNLIMITED_ARGUMENTS;
use {AnyException, Exception};

extern "C" {
    // VALUE
    // rb_proc_call_with_block(VALUE self, int argc, const VALUE *argv, VALUE passed_procval)
    pub fn rb_proc_call_with_block(rproc: Value,
                                   argc: Argc,
                                   argv: *const Value,
                                   pass_procval: Value)
                                   -> Value;
    // VALUE
    // rb_obj_is_proc(VALUE proc)
    pub fn rb_obj_is_proc(proc: Value) -> Value;
    // VALUE
    // rb_proc_lambda_p(VALUE procval)
    pub fn rb_proc_lambda_p(proc: Value) -> Value;
    // #define RB_BLOCK_CALL_FUNC_ARGLIST(yielded_arg, callback_arg) \
    // VALUE yielded_arg, VALUE callback_arg, int argc, const VALUE *argv, VALUE blockarg
    // typedef VALUE rb_block_call_func(RB_BLOCK_CALL_FUNC_ARGLIST(yielded_arg, callback_arg));
    pub fn rb_block_call_func(yielded_arg: Value, callback_arg: Value, argc: c_int, argv: *const Value, blockarg: Value) -> Value;
    // MJIT_FUNC_EXPORTED VALUE
    // rb_func_proc_new(rb_block_call_func_t func, VALUE val)
    pub fn rb_func_proc_new(func: CallbackPtr, block_code: Value) -> Value;
    // VALUE
    // rb_func_lambda_new(rb_block_call_func_t func, VALUE val, int min_argc, int max_argc)
    pub fn rb_func_lambda_new(func: CallbackPtr, block_code: Value, min_argc: c_int, max_argc: c_int) -> Value;
    // static VALUE
    // rb_f_binding(VALUE self)
    pub fn rb_f_binding(self_: Value) -> Value;
    // VALUE
    // rb_binding_new(void)
    pub fn rb_binding_new() -> Value;
}

#[link_name = "vm_ifunc_argc"]
#[repr(C)]
struct InternalFunctionArgc {
    // TODO: change when `SIZEOF_INT * 2 > SIZEOF_VALUE`
    min: c_int,
    max: c_int,
}

#[link_name = "vm_ifunc"]
#[repr(C)]
struct InternalFunction {
    flags: Value,
    reserved: Value,
    func: CallbackPtr,
    data: *const c_void,
    argc: InternalFunctionArgc
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
