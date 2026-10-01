// Event hooks and TracePoint (`ruby/internal/event.h` and the tracing part
// of `ruby/debug.h`).

use crate::rubysys::types::{c_int, c_void, Id, Value};

// typedef uint32_t rb_event_flag_t
pub type EventFlag = u32;

pub const RUBY_EVENT_NONE: EventFlag = 0x0000;
pub const RUBY_EVENT_LINE: EventFlag = 0x0001;
pub const RUBY_EVENT_CLASS: EventFlag = 0x0002;
pub const RUBY_EVENT_END: EventFlag = 0x0004;
pub const RUBY_EVENT_CALL: EventFlag = 0x0008;
pub const RUBY_EVENT_RETURN: EventFlag = 0x0010;
pub const RUBY_EVENT_C_CALL: EventFlag = 0x0020;
pub const RUBY_EVENT_C_RETURN: EventFlag = 0x0040;
pub const RUBY_EVENT_RAISE: EventFlag = 0x0080;
pub const RUBY_EVENT_ALL: EventFlag = 0x00ff;
pub const RUBY_EVENT_B_CALL: EventFlag = 0x0100;
pub const RUBY_EVENT_B_RETURN: EventFlag = 0x0200;
pub const RUBY_EVENT_THREAD_BEGIN: EventFlag = 0x0400;
pub const RUBY_EVENT_THREAD_END: EventFlag = 0x0800;
pub const RUBY_EVENT_FIBER_SWITCH: EventFlag = 0x1000;
pub const RUBY_EVENT_SCRIPT_COMPILED: EventFlag = 0x2000;
pub const RUBY_EVENT_RESCUE: EventFlag = 0x4000;
pub const RUBY_EVENT_TRACEPOINT_ALL: EventFlag = 0xffff;
pub const RUBY_EVENT_RESERVED_FOR_INTERNAL_USE: EventFlag = 0x03_0000;

// Internal events: their hooks must not call any Ruby API, not even to
// allocate an object.
pub const RUBY_INTERNAL_EVENT_SWITCH: EventFlag = 0x04_0000;
pub const RUBY_EVENT_SWITCH: EventFlag = 0x04_0000;
pub const RUBY_INTERNAL_EVENT_NEWOBJ: EventFlag = 0x10_0000;
pub const RUBY_INTERNAL_EVENT_FREEOBJ: EventFlag = 0x20_0000;
pub const RUBY_INTERNAL_EVENT_GC_START: EventFlag = 0x40_0000;
pub const RUBY_INTERNAL_EVENT_GC_END_MARK: EventFlag = 0x80_0000;
pub const RUBY_INTERNAL_EVENT_GC_END_SWEEP: EventFlag = 0x100_0000;
pub const RUBY_INTERNAL_EVENT_GC_ENTER: EventFlag = 0x200_0000;
pub const RUBY_INTERNAL_EVENT_GC_EXIT: EventFlag = 0x400_0000;
pub const RUBY_INTERNAL_EVENT_OBJSPACE_MASK: EventFlag = 0x7f0_0000;
pub const RUBY_INTERNAL_EVENT_MASK: EventFlag = 0xffff_0000;

// typedef enum { ... } rb_event_hook_flag_t
//
// `RUBY_EVENT_HOOK_FLAG_RAW_ARG` makes Ruby call the hook as an
// `rb_event_hook_raw_arg_func_t`: `(VALUE data, const rb_trace_arg_t *arg)`.
pub type EventHookFlag = c_int;

pub const RUBY_EVENT_HOOK_FLAG_SAFE: EventHookFlag = 0x01;
pub const RUBY_EVENT_HOOK_FLAG_DELETED: EventHookFlag = 0x02;
pub const RUBY_EVENT_HOOK_FLAG_RAW_ARG: EventHookFlag = 0x04;

// typedef void (*rb_event_hook_func_t)(rb_event_flag_t evflag, VALUE data, VALUE self, ID mid,
//                                      VALUE klass)
//
// `data` is what the hook was added with; `self`, `mid` and `klass` are the
// current receiver, method name and class (0 when there is none).
pub type EventHookFunction = rutie_callback!(type fn(
    event: EventFlag,
    data: Value,
    receiver: Value,
    method_id: Id,
    klass: Value,
));

// The callback of `rb_tracepoint_new`: the TracePoint and the `data` given.
pub type TracePointFunction = rutie_callback!(type fn(tpval: Value, data: *mut c_void));

// typedef struct rb_trace_arg_struct rb_trace_arg_t: the event being traced;
// only valid during the hook.
#[repr(C)]
pub struct TraceArg {
    _private: [u8; 0],
}

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // void
    // rb_add_event_hook(rb_event_hook_func_t func, rb_event_flag_t events, VALUE data)
    //
    // Calls `func` for each of `events` in every thread. `data` is marked
    // while the hook is registered.
    pub fn rb_add_event_hook(func: EventHookFunction, events: EventFlag, data: Value);
    // void
    // rb_add_event_hook2(rb_event_hook_func_t func, rb_event_flag_t events, VALUE data,
    //                    rb_event_hook_flag_t hook_flag)
    pub fn rb_add_event_hook2(
        func: EventHookFunction,
        events: EventFlag,
        data: Value,
        hook_flag: EventHookFlag,
    );
    // int
    // rb_remove_event_hook(rb_event_hook_func_t func)
    //
    // Removes every hook calling `func` (every hook when `func` is NULL) and
    // returns how many there were.
    pub fn rb_remove_event_hook(func: Option<EventHookFunction>) -> c_int;
    // int
    // rb_remove_event_hook_with_data(rb_event_hook_func_t func, VALUE data)
    //
    // Removes the hooks calling `func` with `data` (`Qundef` for any data).
    pub fn rb_remove_event_hook_with_data(func: Option<EventHookFunction>, data: Value) -> c_int;
    // void
    // rb_thread_add_event_hook(VALUE thval, rb_event_hook_func_t func,
    //                          rb_event_flag_t events, VALUE data)
    //
    // Like `rb_add_event_hook`, for one thread.
    pub fn rb_thread_add_event_hook(
        thread: Value,
        func: EventHookFunction,
        events: EventFlag,
        data: Value,
    );
    // void
    // rb_thread_add_event_hook2(VALUE thval, rb_event_hook_func_t func,
    //                           rb_event_flag_t events, VALUE data,
    //                           rb_event_hook_flag_t hook_flag)
    pub fn rb_thread_add_event_hook2(
        thread: Value,
        func: EventHookFunction,
        events: EventFlag,
        data: Value,
        hook_flag: EventHookFlag,
    );
    // int
    // rb_thread_remove_event_hook(VALUE thval, rb_event_hook_func_t func)
    pub fn rb_thread_remove_event_hook(thread: Value, func: Option<EventHookFunction>) -> c_int;
    // int
    // rb_thread_remove_event_hook_with_data(VALUE thval, rb_event_hook_func_t func,
    //                                       VALUE data)
    pub fn rb_thread_remove_event_hook_with_data(
        thread: Value,
        func: Option<EventHookFunction>,
        data: Value,
    ) -> c_int;

    // VALUE
    // rb_tracepoint_new(VALUE target_thread_not_supported_yet, rb_event_flag_t events,
    //                   void (*func)(VALUE, void *), void *data)
    //
    // A disabled TracePoint calling `func` for `events` in every thread (the
    // first argument is ignored; pass `Qnil`). Normal and internal events
    // cannot be mixed. `data` is not marked.
    pub fn rb_tracepoint_new(
        target_thread_not_supported_yet: Value,
        events: EventFlag,
        func: TracePointFunction,
        data: *mut c_void,
    ) -> Value;
    // VALUE
    // rb_tracepoint_disable(VALUE tpval)
    //
    // Returns `Qundef`.
    pub fn rb_tracepoint_disable(tpval: Value) -> Value;
    // VALUE
    // rb_tracepoint_enable(VALUE tpval)
    //
    // Returns `Qundef`; raises an `ArgumentError` for a TracePoint enabled
    // for a target.
    pub fn rb_tracepoint_enable(tpval: Value) -> Value;
    // VALUE
    // rb_tracepoint_enabled_p(VALUE tpval)
    pub fn rb_tracepoint_enabled_p(tpval: Value) -> Value;

    // rb_trace_arg_t *
    // rb_tracearg_from_tracepoint(VALUE tpval)
    //
    // The current event; raises a `RuntimeError` outside of a hook.
    pub fn rb_tracearg_from_tracepoint(tpval: Value) -> *mut TraceArg;
    // VALUE
    // rb_tracearg_binding(rb_trace_arg_t *trace_arg)
    //
    // `Qnil` when there is no Ruby frame.
    pub fn rb_tracearg_binding(trace_arg: *mut TraceArg) -> Value;
    // VALUE
    // rb_tracearg_callee_id(rb_trace_arg_t *trace_arg)
    //
    // The name the method was called by (a Symbol, or `Qnil`).
    pub fn rb_tracearg_callee_id(trace_arg: *mut TraceArg) -> Value;
    // VALUE
    // rb_tracearg_defined_class(rb_trace_arg_t *trace_arg)
    pub fn rb_tracearg_defined_class(trace_arg: *mut TraceArg) -> Value;
    // VALUE
    // rb_tracearg_eval_script(rb_trace_arg_t *trace_arg)
    //
    // The source of a `script_compiled` event, `Qnil` when loaded from a
    // file; raises a `RuntimeError` for other events.
    pub fn rb_tracearg_eval_script(trace_arg: *mut TraceArg) -> Value;
    // VALUE
    // rb_tracearg_event(rb_trace_arg_t *trace_arg)
    //
    // The event as a Symbol (`:line`, `:call`, ...).
    pub fn rb_tracearg_event(trace_arg: *mut TraceArg) -> Value;
    // rb_event_flag_t
    // rb_tracearg_event_flag(rb_trace_arg_t *trace_arg)
    pub fn rb_tracearg_event_flag(trace_arg: *mut TraceArg) -> EventFlag;
    // VALUE
    // rb_tracearg_instruction_sequence(rb_trace_arg_t *trace_arg)
    //
    // The `RubyVM::InstructionSequence` of a `script_compiled` event; raises
    // a `RuntimeError` for other events.
    pub fn rb_tracearg_instruction_sequence(trace_arg: *mut TraceArg) -> Value;
    // VALUE
    // rb_tracearg_lineno(rb_trace_arg_t *trace_arg)
    //
    // An Integer, 0 when there is no Ruby frame.
    pub fn rb_tracearg_lineno(trace_arg: *mut TraceArg) -> Value;
    // VALUE
    // rb_tracearg_method_id(rb_trace_arg_t *trace_arg)
    pub fn rb_tracearg_method_id(trace_arg: *mut TraceArg) -> Value;
    // VALUE
    // rb_tracearg_object(rb_trace_arg_t *trace_arg)
    //
    // The object of a `RUBY_INTERNAL_EVENT_NEWOBJ` or `_FREEOBJ` event;
    // raises a `RuntimeError` for other events.
    pub fn rb_tracearg_object(trace_arg: *mut TraceArg) -> Value;
    // VALUE
    // rb_tracearg_parameters(rb_trace_arg_t *trace_arg)
    //
    // The parameters (like `Method#parameters`) of a call or return event;
    // raises a `RuntimeError` for other events.
    pub fn rb_tracearg_parameters(trace_arg: *mut TraceArg) -> Value;
    // VALUE
    // rb_tracearg_path(rb_trace_arg_t *trace_arg)
    //
    // `Qnil` when there is no Ruby frame.
    pub fn rb_tracearg_path(trace_arg: *mut TraceArg) -> Value;
    // VALUE
    // rb_tracearg_raised_exception(rb_trace_arg_t *trace_arg)
    //
    // Raises a `RuntimeError` unless the event is `raise` or `rescue`.
    pub fn rb_tracearg_raised_exception(trace_arg: *mut TraceArg) -> Value;
    // VALUE
    // rb_tracearg_return_value(rb_trace_arg_t *trace_arg)
    //
    // Raises a `RuntimeError` unless the event is a return event.
    pub fn rb_tracearg_return_value(trace_arg: *mut TraceArg) -> Value;
    // VALUE
    // rb_tracearg_self(rb_trace_arg_t *trace_arg)
    pub fn rb_tracearg_self(trace_arg: *mut TraceArg) -> Value;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::{thread, vm};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CALLS: AtomicUsize = AtomicUsize::new(0);
    static RAW_CALLS: AtomicUsize = AtomicUsize::new(0);

    rutie_callback! {
        fn count_call(_event: EventFlag, _data: Value, _receiver: Value, _method_id: Id, _klass: Value) {
            CALLS.fetch_add(1, Ordering::SeqCst);
        }
    }

    // An `rb_event_hook_raw_arg_func_t`, for RUBY_EVENT_HOOK_FLAG_RAW_ARG.
    rutie_callback! {
        fn count_raw(_data: Value, trace_arg: *mut TraceArg) {
            assert_eq!(unsafe { rb_tracearg_event_flag(trace_arg) }, RUBY_EVENT_CALL);
            RAW_CALLS.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_event_hooks() {
        crate::on_ruby_thread(|| unsafe {
            vm::eval_string("def rutie_hooked = 1");
            let data = vm::eval_string("Object.new");

            rb_add_event_hook(count_call, RUBY_EVENT_CALL, data);
            vm::eval_string("rutie_hooked; rutie_hooked");
            assert_eq!(rb_remove_event_hook(Some(count_call)), 1);
            vm::eval_string("rutie_hooked");
            assert_eq!(CALLS.swap(0, Ordering::SeqCst), 2);

            let raw: rutie_callback!(type fn(Value, *mut TraceArg)) = count_raw;
            let raw: EventHookFunction = std::mem::transmute(raw);
            rb_add_event_hook2(
                raw,
                RUBY_EVENT_CALL,
                data,
                RUBY_EVENT_HOOK_FLAG_SAFE | RUBY_EVENT_HOOK_FLAG_RAW_ARG,
            );
            vm::eval_string("rutie_hooked");
            assert_eq!(rb_remove_event_hook_with_data(Some(raw), data), 1);
            assert_eq!(RAW_CALLS.swap(0, Ordering::SeqCst), 1);

            let current = thread::current();
            rb_thread_add_event_hook(current, count_call, RUBY_EVENT_CALL, data);
            rb_thread_add_event_hook2(
                current,
                count_call,
                RUBY_EVENT_RETURN,
                data,
                RUBY_EVENT_HOOK_FLAG_SAFE,
            );
            vm::eval_string("rutie_hooked");
            // A call and a return.
            assert_eq!(CALLS.swap(0, Ordering::SeqCst), 2);
            assert_eq!(
                rb_thread_remove_event_hook_with_data(current, Some(count_call), data),
                2
            );
            rb_thread_add_event_hook(current, count_call, RUBY_EVENT_CALL, data);
            assert_eq!(rb_thread_remove_event_hook(current, Some(count_call)), 1);
            vm::eval_string("rutie_hooked");
            assert_eq!(CALLS.load(Ordering::SeqCst), 0);
        });
    }
}
