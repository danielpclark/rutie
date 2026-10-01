use std::{cell::RefCell, ptr};

use crate::{
    binding::{global::RubySpecialConsts, symbol, vm},
    rubysys::{
        class,
        tracepoint::{self, EventFlag, TraceArg},
        typed_data::{self, RbDataType, RbDataTypeFunction},
    },
    types::{c_char, c_void, InternalValue, Value},
};

fn nil() -> Value {
    Value::from(RubySpecialConsts::Nil as InternalValue)
}

// The Rust closure of a TracePoint made by `new`, owned by a hidden
// typed-data object stored in the TracePoint (so it lives as long as the
// TracePoint, and an enabled TracePoint is always reachable). The `RefCell`
// skips a nested call: one made by another Ruby thread while the closure
// waits in Ruby code with the GVL released.
struct TraceClosure(RefCell<Box<dyn FnMut(*mut TraceArg) + Send>>);

extern "C" fn free_trace_closure(data: *mut c_void) {
    unsafe { drop(Box::from_raw(data as *mut TraceClosure)) };
}

lazy_static! {
    static ref TRACE_CLOSURE_TYPE: RbDataType = RbDataType {
        wrap_struct_name: b"Rutie/TraceClosure\0".as_ptr() as *const c_char,
        function: RbDataTypeFunction {
            dmark: None,
            dfree: Some(free_trace_closure),
            dsize: None,
            reserved: [ptr::null_mut(); 2],
        },
        parent: ptr::null(),
        data: ptr::null_mut(),
        flags: Value::from(0),
    };
}

rutie_callback! {
    fn tracepoint_callback(tpval: Value, data: *mut c_void) {
        let closure = unsafe { &*(data as *const TraceClosure) };

        let mut func = match closure.0.try_borrow_mut() {
            Ok(func) => func,
            Err(_) => return,
        };

        // Whatever the closure raises (a panic too) carries on once the
        // closure is released.
        let state = vm::protect_state(|| {
            let trace_arg = unsafe { tracepoint::rb_tracearg_from_tracepoint(tpval) };
            func(trace_arg);

            nil()
        });

        drop(func);

        if state != 0 {
            vm::jump_tag(state);
        }
    }
}

pub fn new<F>(events: EventFlag, func: F) -> Value
where
    F: FnMut(*mut TraceArg) + Send + 'static,
{
    let closure = Box::into_raw(Box::new(TraceClosure(RefCell::new(Box::new(func)))));
    let owner = unsafe {
        typed_data::rb_data_typed_object_wrap(
            Value::from(0),
            closure as *mut c_void,
            &*TRACE_CLOSURE_TYPE,
        )
    };

    let tracepoint = unsafe {
        tracepoint::rb_tracepoint_new(nil(), events, tracepoint_callback, closure as *mut c_void)
    };

    // An ID without `@` makes an instance variable Ruby code cannot see.
    unsafe { class::rb_ivar_set(tracepoint, symbol::internal_id("__rutie_closure__"), owner) };

    tracepoint
}

pub fn enable(tracepoint: Value) {
    unsafe { tracepoint::rb_tracepoint_enable(tracepoint) };
}

pub fn disable(tracepoint: Value) {
    unsafe { tracepoint::rb_tracepoint_disable(tracepoint) };
}

pub fn is_enabled(tracepoint: Value) -> bool {
    unsafe { tracepoint::rb_tracepoint_enabled_p(tracepoint) }.is_true()
}

pub fn event(arg: *mut TraceArg) -> Value {
    unsafe { tracepoint::rb_tracearg_event(arg) }
}

pub fn event_flag(arg: *mut TraceArg) -> EventFlag {
    unsafe { tracepoint::rb_tracearg_event_flag(arg) }
}

pub fn lineno(arg: *mut TraceArg) -> Value {
    unsafe { tracepoint::rb_tracearg_lineno(arg) }
}

pub fn path(arg: *mut TraceArg) -> Value {
    unsafe { tracepoint::rb_tracearg_path(arg) }
}

pub fn method_id(arg: *mut TraceArg) -> Value {
    unsafe { tracepoint::rb_tracearg_method_id(arg) }
}

pub fn callee_id(arg: *mut TraceArg) -> Value {
    unsafe { tracepoint::rb_tracearg_callee_id(arg) }
}

pub fn defined_class(arg: *mut TraceArg) -> Value {
    unsafe { tracepoint::rb_tracearg_defined_class(arg) }
}

pub fn binding(arg: *mut TraceArg) -> Value {
    unsafe { tracepoint::rb_tracearg_binding(arg) }
}

pub fn receiver(arg: *mut TraceArg) -> Value {
    unsafe { tracepoint::rb_tracearg_self(arg) }
}

// The rest raise a `RuntimeError` for an event they do not apply to.

pub fn return_value(arg: *mut TraceArg) -> Value {
    unsafe { tracepoint::rb_tracearg_return_value(arg) }
}

pub fn raised_exception(arg: *mut TraceArg) -> Value {
    unsafe { tracepoint::rb_tracearg_raised_exception(arg) }
}

pub fn parameters(arg: *mut TraceArg) -> Value {
    unsafe { tracepoint::rb_tracearg_parameters(arg) }
}

pub fn eval_script(arg: *mut TraceArg) -> Value {
    unsafe { tracepoint::rb_tracearg_eval_script(arg) }
}

pub fn instruction_sequence(arg: *mut TraceArg) -> Value {
    unsafe { tracepoint::rb_tracearg_instruction_sequence(arg) }
}

pub fn object(arg: *mut TraceArg) -> Value {
    unsafe { tracepoint::rb_tracearg_object(arg) }
}
