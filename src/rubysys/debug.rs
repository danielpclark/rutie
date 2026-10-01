// Frame profiling, the debug inspector and postponed jobs (`ruby/debug.h`).

use crate::rubysys::types::{c_int, c_long, c_uint, c_void, Value};

// typedef struct rb_debug_inspector_struct rb_debug_inspector_t;
#[repr(C)]
pub struct DebugInspector {
    _private: [u8; 0],
}

// typedef VALUE (*rb_debug_inspector_func_t)(const rb_debug_inspector_t *dc, void *data);
pub type DebugInspectorFunction =
    rutie_callback!(type fn(dc: *const DebugInspector, data: *mut c_void) -> Value);

// typedef void (*rb_postponed_job_func_t)(void *arg);
pub type PostponedJobFunction = rutie_callback!(type fn(arg: *mut c_void));

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // Fills `buff` with up to `limit` frames of the current thread's stack,
    // from the `start`th (0 is the top), and `lines` (which may be NULL) with
    // their line numbers. Returns the number of frames filled.
    //
    // The frames are opaque (an iseq or method entry): only the
    // `rb_profile_frame_*` functions may be called on them. A block is
    // reported as its method. Ruby 3.1 to 3.3 ignore `start`.
    //
    // int
    // rb_profile_frames(int start, int limit, VALUE *buff, int *lines)
    pub fn rb_profile_frames(
        start: c_int,
        limit: c_int,
        buff: *mut Value,
        lines: *mut c_int,
    ) -> c_int;
    // VALUE
    // rb_profile_frame_path(VALUE frame)
    pub fn rb_profile_frame_path(frame: Value) -> Value;
    // VALUE
    // rb_profile_frame_absolute_path(VALUE frame)
    pub fn rb_profile_frame_absolute_path(frame: Value) -> Value;
    // VALUE
    // rb_profile_frame_label(VALUE frame)
    pub fn rb_profile_frame_label(frame: Value) -> Value;
    // VALUE
    // rb_profile_frame_base_label(VALUE frame)
    pub fn rb_profile_frame_base_label(frame: Value) -> Value;
    // VALUE
    // rb_profile_frame_full_label(VALUE frame)
    pub fn rb_profile_frame_full_label(frame: Value) -> Value;
    // VALUE
    // rb_profile_frame_first_lineno(VALUE frame)
    pub fn rb_profile_frame_first_lineno(frame: Value) -> Value;
    // VALUE
    // rb_profile_frame_classpath(VALUE frame)
    pub fn rb_profile_frame_classpath(frame: Value) -> Value;
    // VALUE
    // rb_profile_frame_singleton_method_p(VALUE frame)
    pub fn rb_profile_frame_singleton_method_p(frame: Value) -> Value;
    // VALUE
    // rb_profile_frame_method_name(VALUE frame)
    pub fn rb_profile_frame_method_name(frame: Value) -> Value;
    // VALUE
    // rb_profile_frame_qualified_method_name(VALUE frame)
    pub fn rb_profile_frame_qualified_method_name(frame: Value) -> Value;

    // Calls `func` with a debug context for the current stack, which is only
    // valid during the call, and returns what it returns.
    //
    // VALUE
    // rb_debug_inspector_open(rb_debug_inspector_func_t func, void *data)
    pub fn rb_debug_inspector_open(func: DebugInspectorFunction, data: *mut c_void) -> Value;
    // An `Array` of `Thread::Backtrace::Location`, one per frame.
    //
    // VALUE
    // rb_debug_inspector_backtrace_locations(const rb_debug_inspector_t *dc)
    pub fn rb_debug_inspector_backtrace_locations(dc: *const DebugInspector) -> Value;
    // The `rb_debug_inspector_frame_*` functions raise an `ArgumentError` for
    // an `index` out of the range of the backtrace locations.
    //
    // VALUE
    // rb_debug_inspector_frame_self_get(const rb_debug_inspector_t *dc, long index)
    pub fn rb_debug_inspector_frame_self_get(dc: *const DebugInspector, index: c_long) -> Value;
    // VALUE
    // rb_debug_inspector_frame_class_get(const rb_debug_inspector_t *dc, long index)
    pub fn rb_debug_inspector_frame_class_get(dc: *const DebugInspector, index: c_long) -> Value;
    // VALUE
    // rb_debug_inspector_frame_binding_get(const rb_debug_inspector_t *dc, long index)
    pub fn rb_debug_inspector_frame_binding_get(dc: *const DebugInspector, index: c_long) -> Value;
    // `Qnil` for a frame not in Ruby.
    //
    // VALUE
    // rb_debug_inspector_frame_iseq_get(const rb_debug_inspector_t *dc, long index)
    pub fn rb_debug_inspector_frame_iseq_get(dc: *const DebugInspector, index: c_long) -> Value;
    // The stack depth of the frame (an `Integer`), counting the special
    // frames the inspector skips.
    //
    // VALUE
    // rb_debug_inspector_frame_depth(const rb_debug_inspector_t *dc, long index)
    #[cfg(ruby_gte_3_2)]
    pub fn rb_debug_inspector_frame_depth(dc: *const DebugInspector, index: c_long) -> Value;
    // The stack depth of the current frame (an `Integer`).
    //
    // VALUE
    // rb_debug_inspector_current_depth(void)
    #[cfg(ruby_gte_3_2)]
    pub fn rb_debug_inspector_current_depth() -> Value;

    // Deprecated in Ruby 3.3 for `rb_postponed_job_preregister` and
    // `rb_postponed_job_trigger`. Returns 0 when the job could not be
    // registered.
    //
    // int
    // rb_postponed_job_register(unsigned int flags, rb_postponed_job_func_t func, void *data)
    pub fn rb_postponed_job_register(
        flags: c_uint,
        func: PostponedJobFunction,
        data: *mut c_void,
    ) -> c_int;
    // Like `rb_postponed_job_register`, but does nothing when `func` is
    // already registered. Deprecated in Ruby 3.3.
    //
    // int
    // rb_postponed_job_register_one(unsigned int flags, rb_postponed_job_func_t func,
    //                               void *data)
    pub fn rb_postponed_job_register_one(
        flags: c_uint,
        func: PostponedJobFunction,
        data: *mut c_void,
    ) -> c_int;
}
