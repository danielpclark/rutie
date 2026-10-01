use crate::{
    rubysys::{
        constant::UNLIMITED_ARGUMENTS,
        types::{c_int, Argc, BlockCallFunction, Id, Value},
    },
    AnyException, Exception,
};

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_proc_call_with_block(VALUE self, int argc, const VALUE *argv, VALUE passed_procval)
    pub fn rb_proc_call_with_block(
        rproc: Value,
        argc: Argc,
        argv: *const Value,
        pass_procval: Value,
    ) -> Value;
    // VALUE
    // rb_binding_new(void)
    pub fn rb_binding_new() -> Value;
    pub fn rb_obj_is_proc(obj: Value) -> Value;
    pub fn rb_obj_is_method(obj: Value) -> Value;
    // VALUE
    // rb_block_lambda(void)
    pub fn rb_block_lambda() -> Value;
    // VALUE
    // rb_method_call(int argc, const VALUE *argv, VALUE method)
    pub fn rb_method_call(argc: Argc, argv: *const Value, method: Value) -> Value;
    // VALUE
    // rb_method_call_with_block(int argc, const VALUE *argv, VALUE method, VALUE passed_procval)
    pub fn rb_method_call_with_block(
        argc: Argc,
        argv: *const Value,
        method: Value,
        pass_procval: Value,
    ) -> Value;
    // int
    // rb_mod_method_arity(VALUE mod, ID id)
    //
    // Returns 0 for an undefined method.
    pub fn rb_mod_method_arity(module: Value, name: Id) -> c_int;
    // int
    // rb_obj_method_arity(VALUE obj, ID id)
    //
    // Returns 0 for an undefined method.
    pub fn rb_obj_method_arity(object: Value, name: Id) -> c_int;
    // int
    // rb_proc_arity(VALUE self)
    pub fn rb_proc_arity(rproc: Value) -> c_int;
    // VALUE
    // rb_proc_call(VALUE self, VALUE args)
    pub fn rb_proc_call(rproc: Value, arguments: Value) -> Value;
    // VALUE
    // rb_proc_lambda_p(VALUE self)
    pub fn rb_proc_lambda_p(rproc: Value) -> Value;
    // VALUE
    // rb_proc_new(VALUE (*func)(ANYARGS), VALUE val)
    //
    // `func` is called like a block function with `val` as its second
    // argument; `val` is kept alive (and marked) by the proc.
    pub fn rb_proc_new(func: BlockCallFunction, val: Value) -> Value;
}

pub fn check_arity(argc: c_int, min: c_int, max: c_int) -> Result<c_int, AnyException> {
    if argc < min || (max != UNLIMITED_ARGUMENTS as c_int && argc > max) {
        let err_mess = if min == max {
            format!(
                "wrong number of arguments (given {}, expected {})",
                argc, min
            )
        } else if max == UNLIMITED_ARGUMENTS as c_int {
            format!(
                "wrong number of arguments (given {}, expected {}+)",
                argc, min
            )
        } else {
            format!(
                "wrong number of arguments (given {}, expected {}..{})",
                argc, min, max
            )
        };

        return Err(AnyException::new("ArgumentError", Some(&err_mess)));
    }

    Ok(argc)
}

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_method_call_kw(int argc, const VALUE *argv, VALUE recv, int kw_splat)
    pub fn rb_method_call_kw(
        argc: Argc,
        argv: *const Value,
        method: Value,
        kw_splat: c_int,
    ) -> Value;
    // VALUE
    // rb_method_call_with_block_kw(int argc, const VALUE *argv, VALUE recv, VALUE proc, int kw_splat)
    pub fn rb_method_call_with_block_kw(
        argc: Argc,
        argv: *const Value,
        method: Value,
        pass_procval: Value,
        kw_splat: c_int,
    ) -> Value;
    // VALUE
    // rb_proc_call_kw(VALUE recv, VALUE args, int kw_splat)
    pub fn rb_proc_call_kw(rproc: Value, arguments: Value, kw_splat: c_int) -> Value;
    // VALUE
    // rb_proc_call_with_block_kw(VALUE recv, int argc, const VALUE *argv, VALUE proc, int kw_splat)
    pub fn rb_proc_call_with_block_kw(
        rproc: Value,
        argc: Argc,
        argv: *const Value,
        pass_procval: Value,
        kw_splat: c_int,
    ) -> Value;
}
