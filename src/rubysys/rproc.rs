use rubysys::types::{Argc, Value};

extern "C" {
    // VALUE
    // rb_proc_call_with_block(VALUE self, int argc, const VALUE *argv, VALUE passed_procval)
    pub fn rb_proc_call_with_block(rproc: Value,
                                   argc: Argc,
                                   argv: *const Value,
                                   pass_procval: Value)
                                   -> Value;
}
