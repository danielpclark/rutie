use rubysys::types::{Argc, Value};

extern "C" {
    pub fn rb_proc_call_with_block(rproc: Value,
                                   argc: Argc,
                                   argv: *const Value,
                                   pass_procval: Value)
                                   -> Value;
}
