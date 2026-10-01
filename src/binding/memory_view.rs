use std::{ffi::CStr, ptr};

use crate::{
    rubysys::memory_view::{self, rb_memory_view_t},
    rubysys::types::ssize_t,
    types::Value,
};

pub fn is_available(object: Value) -> bool {
    unsafe { memory_view::rb_memory_view_available_p(object) }
}

// Fills `view` in place (an exporter may point into it), or returns `false`
// without touching it. A filled view must be released, from where it is.
pub unsafe fn get(object: Value, view: *mut rb_memory_view_t, flags: i32) -> bool {
    memory_view::rb_memory_view_get(object, view, flags)
}

pub unsafe fn release(view: *mut rb_memory_view_t) -> bool {
    memory_view::rb_memory_view_release(view)
}

// `rb_memory_view_is_contiguous`, an inline function in the headers. Ruby
// reads `shape` and `strides` without checking for null.
pub unsafe fn is_contiguous(view: *const rb_memory_view_t) -> bool {
    memory_view::rb_memory_view_is_row_major_contiguous(view)
        || memory_view::rb_memory_view_is_column_major_contiguous(view)
}

// The caller checks the indices against the view's shape. Raises when the
// view's format can't be parsed.
pub unsafe fn get_item(view: *mut rb_memory_view_t, indices: &[ssize_t]) -> Value {
    memory_view::rb_memory_view_get_item(view, indices.as_ptr())
}

pub fn item_size_from_format(format: &CStr) -> Option<usize> {
    let mut error = ptr::null();
    let size =
        unsafe { memory_view::rb_memory_view_item_size_from_format(format.as_ptr(), &mut error) };

    if size < 0 {
        None
    } else {
        Some(size as usize)
    }
}

pub fn contiguous_strides(item_size: ssize_t, shape: &[ssize_t], row_major: bool) -> Vec<ssize_t> {
    let mut strides = vec![0; shape.len()];

    unsafe {
        memory_view::rb_memory_view_fill_contiguous_strides(
            shape.len() as ssize_t,
            item_size,
            shape.as_ptr(),
            row_major,
            strides.as_mut_ptr(),
        )
    };

    strides
}
