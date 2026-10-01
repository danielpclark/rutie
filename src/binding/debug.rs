use std::convert::TryFrom;

use crate::{
    binding::vm,
    rubysys::debug::{self, DebugInspector},
    types::{c_int, c_long, c_void, CallbackMutPtr, Value},
};

// Fills a buffer with the `start + limit` innermost frames with `fill`, and
// returns the frames from the `start`th, with their line numbers. Ruby 3.1 to
// 3.3 ignore the `start` argument of `rb_profile_frames`, so Rutie always
// asks from the top and skips frames itself.
fn collect_frames<F>(start: usize, limit: usize, fill: F) -> Vec<(Value, c_int)>
where
    F: FnOnce(c_int, *mut Value, *mut c_int) -> c_int,
{
    let total = c_int::try_from(start.saturating_add(limit)).unwrap_or(c_int::MAX);
    let mut frames = vec![Value::from(0); total as usize];
    let mut lines: Vec<c_int> = vec![0; total as usize];

    let filled = fill(total, frames.as_mut_ptr(), lines.as_mut_ptr()).max(0) as usize;

    frames
        .into_iter()
        .zip(lines)
        .take(filled)
        .skip(start)
        .collect()
}

pub fn profile_frames(start: usize, limit: usize) -> Vec<(Value, c_int)> {
    collect_frames(start, limit, |total, frames, lines| unsafe {
        debug::rb_profile_frames(0, total, frames, lines)
    })
}

// The `profile_frame_*` functions take a frame from `profile_frames`.
pub fn profile_frame_path(frame: Value) -> Value {
    unsafe { debug::rb_profile_frame_path(frame) }
}

pub fn profile_frame_absolute_path(frame: Value) -> Value {
    unsafe { debug::rb_profile_frame_absolute_path(frame) }
}

pub fn profile_frame_label(frame: Value) -> Value {
    unsafe { debug::rb_profile_frame_label(frame) }
}

pub fn profile_frame_base_label(frame: Value) -> Value {
    unsafe { debug::rb_profile_frame_base_label(frame) }
}

pub fn profile_frame_full_label(frame: Value) -> Value {
    unsafe { debug::rb_profile_frame_full_label(frame) }
}

pub fn profile_frame_first_lineno(frame: Value) -> Value {
    unsafe { debug::rb_profile_frame_first_lineno(frame) }
}

pub fn profile_frame_classpath(frame: Value) -> Value {
    unsafe { debug::rb_profile_frame_classpath(frame) }
}

pub fn profile_frame_is_singleton_method(frame: Value) -> bool {
    unsafe { debug::rb_profile_frame_singleton_method_p(frame) }.is_true()
}

pub fn profile_frame_method_name(frame: Value) -> Value {
    unsafe { debug::rb_profile_frame_method_name(frame) }
}

pub fn profile_frame_qualified_method_name(frame: Value) -> Value {
    unsafe { debug::rb_profile_frame_qualified_method_name(frame) }
}

// `data` points to an `Option<F>` on the caller's stack, taken exactly once.
rutie_callback! {
    fn debug_inspector_callback<F>(dc: *const DebugInspector, data: CallbackMutPtr) -> Value
    where
        F: FnOnce(*const DebugInspector) -> Value,
    {
        match unsafe { (*(data as *mut Option<F>)).take() } {
            Some(func) => vm::call_catching_panic(move || func(dc)),
            None => Value::from(0),
        }
    }
}

// Calls `func` with a debug context that is only valid during the call.
pub fn debug_inspector_open<F>(func: F) -> Value
where
    F: FnOnce(*const DebugInspector) -> Value,
{
    let mut func = Some(func);

    unsafe {
        debug::rb_debug_inspector_open(
            debug_inspector_callback::<F>,
            &mut func as *mut Option<F> as *mut c_void,
        )
    }
}

// The `debug_inspector_*` functions take the context `debug_inspector_open`
// passes, and an `index` within its backtrace locations.
pub unsafe fn debug_inspector_backtrace_locations(dc: *const DebugInspector) -> Value {
    debug::rb_debug_inspector_backtrace_locations(dc)
}

pub unsafe fn debug_inspector_frame_self(dc: *const DebugInspector, index: usize) -> Value {
    debug::rb_debug_inspector_frame_self_get(dc, index as c_long)
}

pub unsafe fn debug_inspector_frame_class(dc: *const DebugInspector, index: usize) -> Value {
    debug::rb_debug_inspector_frame_class_get(dc, index as c_long)
}

pub unsafe fn debug_inspector_frame_binding(dc: *const DebugInspector, index: usize) -> Value {
    debug::rb_debug_inspector_frame_binding_get(dc, index as c_long)
}

pub unsafe fn debug_inspector_frame_iseq(dc: *const DebugInspector, index: usize) -> Value {
    debug::rb_debug_inspector_frame_iseq_get(dc, index as c_long)
}

#[cfg(ruby_gte_3_2)]
pub unsafe fn debug_inspector_frame_depth(dc: *const DebugInspector, index: usize) -> Value {
    debug::rb_debug_inspector_frame_depth(dc, index as c_long)
}

#[cfg(ruby_gte_3_2)]
pub fn debug_inspector_current_depth() -> Value {
    unsafe { debug::rb_debug_inspector_current_depth() }
}
