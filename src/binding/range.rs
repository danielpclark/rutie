use crate::{
    binding::global::RubySpecialConsts,
    rubysys::range,
    types::{c_int, c_long, InternalValue, Value},
    util,
};

pub fn new(begin: Value, end: Value, exclude_end: bool) -> Value {
    unsafe { range::rb_range_new(begin, end, util::bool_to_c_int(exclude_end)) }
}

// `(begin, end, exclude_end)`, or `None` if `range` is not range-like.
pub fn values(range: Value) -> Option<(Value, Value, bool)> {
    let nil = Value::from(RubySpecialConsts::Nil as InternalValue);
    let (mut begin, mut end, mut exclude_end) = (nil, nil, 0 as c_int);

    let is_range = unsafe { range::rb_range_values(range, &mut begin, &mut end, &mut exclude_end) };

    if util::c_int_to_bool(is_range) {
        Some((begin, end, util::c_int_to_bool(exclude_end)))
    } else {
        None
    }
}

// `Some((offset, length))` for the part of a sequence of `total` elements the
// range covers, or `None` when it starts out of range. Raises `TypeError`
// for non-integer bounds.
pub fn begin_length(range: Value, total: usize) -> Option<(usize, usize)> {
    let (mut begin, mut length): (c_long, c_long) = (0, 0);
    let result =
        unsafe { range::rb_range_beg_len(range, &mut begin, &mut length, total as c_long, 0) };

    if result.is_true() {
        Some((begin as usize, length as usize))
    } else {
        None
    }
}

// `(begin, end, step, exclude_end)` of a range or arithmetic sequence.
pub fn arithmetic_sequence(object: Value) -> Option<(Value, Value, Value, bool)> {
    let nil = Value::from(RubySpecialConsts::Nil as InternalValue);
    let mut components = range::ArithmeticSequenceComponents {
        begin: nil,
        end: nil,
        step: nil,
        exclude_end: 0,
    };

    let extracted = unsafe { range::rb_arithmetic_sequence_extract(object, &mut components) };

    if util::c_int_to_bool(extracted) {
        Some((
            components.begin,
            components.end,
            components.step,
            util::c_int_to_bool(components.exclude_end),
        ))
    } else {
        None
    }
}
