use crate::rubysys::types::{c_int, c_long, Value};

extern "C" {
    pub static rb_cRange: Value;
}

extern "C" {
    // VALUE
    // rb_range_beg_len(VALUE range, long *begp, long *lenp, long len, int err)
    //
    // `Qfalse`: not a range; `Qnil`: out of range (with `err == 0`).
    pub fn rb_range_beg_len(
        range: Value,
        begin: *mut c_long,
        length: *mut c_long,
        total: c_long,
        err: c_int,
    ) -> Value;
    // VALUE
    // rb_range_new(VALUE beg, VALUE end, int exclude_end)
    pub fn rb_range_new(begin: Value, end: Value, exclude_end: c_int) -> Value;
    // int
    // rb_range_values(VALUE range, VALUE *begp, VALUE *endp, int *exclp)
    pub fn rb_range_values(
        range: Value,
        begin: *mut Value,
        end: *mut Value,
        exclude_end: *mut c_int,
    ) -> c_int;
}

// typedef struct { VALUE begin; VALUE end; VALUE step; int exclude_end; }
// rb_arithmetic_sequence_components_t
#[repr(C)]
pub struct ArithmeticSequenceComponents {
    pub begin: Value,
    pub end: Value,
    pub step: Value,
    pub exclude_end: c_int,
}

extern "C" {
    // int
    // rb_arithmetic_sequence_extract(VALUE obj, rb_arithmetic_sequence_components_t *component)
    pub fn rb_arithmetic_sequence_extract(
        object: Value,
        components: *mut ArithmeticSequenceComponents,
    ) -> c_int;
}
