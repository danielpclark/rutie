#![allow(non_camel_case_types)]

// MemoryView (`ruby/memory_view.h`): a C-level protocol for objects to
// export raw memory to other C code, like Python's buffer protocol.
use crate::rubysys::types::{c_char, c_int, c_void, size_t, ssize_t, RbDataType, Value};

// `enum ruby_memory_view_flags`, what the consumer asks `rb_memory_view_get`
// for.
pub const RUBY_MEMORY_VIEW_SIMPLE: c_int = 0;
pub const RUBY_MEMORY_VIEW_WRITABLE: c_int = 1 << 0;
pub const RUBY_MEMORY_VIEW_FORMAT: c_int = 1 << 1;
pub const RUBY_MEMORY_VIEW_MULTI_DIMENSIONAL: c_int = 1 << 2;
pub const RUBY_MEMORY_VIEW_STRIDES: c_int = (1 << 3) | RUBY_MEMORY_VIEW_MULTI_DIMENSIONAL;
pub const RUBY_MEMORY_VIEW_ROW_MAJOR: c_int = (1 << 4) | RUBY_MEMORY_VIEW_STRIDES;
pub const RUBY_MEMORY_VIEW_COLUMN_MAJOR: c_int = (1 << 5) | RUBY_MEMORY_VIEW_STRIDES;
pub const RUBY_MEMORY_VIEW_ANY_CONTIGUOUS: c_int =
    RUBY_MEMORY_VIEW_ROW_MAJOR | RUBY_MEMORY_VIEW_COLUMN_MAJOR;
pub const RUBY_MEMORY_VIEW_INDIRECT: c_int = (1 << 6) | RUBY_MEMORY_VIEW_STRIDES;

// `rb_memory_view_item_component_t`, one component of an item's format.
//
// Ruby 3.1 and 3.2 declare the two flags as bit-fields:
//
//     char format;
//     unsigned native_size_p: 1;
//     unsigned little_endian_p: 1;
//
// which GCC and Clang put in the byte after `format`, and the Microsoft
// layout (MSVC, and MinGW's default `-mms-bitfields`) in a new `unsigned` at
// offset 4. Read them with `native_size_p()` and `little_endian_p()`, which
// also work on 3.3+, where they are `bool` fields.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct rb_memory_view_item_component_t {
    pub format: c_char,
    #[cfg(not(windows))]
    pub bitfields: u8,
    #[cfg(windows)]
    pub bitfields: u32,
    pub offset: size_t,
    pub size: size_t,
    pub repeat: size_t,
}

impl rb_memory_view_item_component_t {
    // Bit positions in `bitfields`: allocated from the low bit, except by
    // GCC on big-endian targets, which allocates from the high bit of the
    // `unsigned` (whose second byte is `bitfields`).
    #[cfg(any(windows, target_endian = "little"))]
    const NATIVE_SIZE_BIT: u32 = 0;
    #[cfg(any(windows, target_endian = "little"))]
    const LITTLE_ENDIAN_BIT: u32 = 1;
    #[cfg(all(not(windows), target_endian = "big"))]
    const NATIVE_SIZE_BIT: u32 = 7;
    #[cfg(all(not(windows), target_endian = "big"))]
    const LITTLE_ENDIAN_BIT: u32 = 6;

    pub fn native_size_p(&self) -> bool {
        (self.bitfields as u32 >> Self::NATIVE_SIZE_BIT) & 1 == 1
    }

    pub fn little_endian_p(&self) -> bool {
        (self.bitfields as u32 >> Self::LITTLE_ENDIAN_BIT) & 1 == 1
    }
}

// The anonymous `item_desc` struct of `rb_memory_view_t`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct rb_memory_view_item_desc {
    // Allocated by `rb_memory_view_prepare_item_desc` and
    // `rb_memory_view_get_item`, freed by `rb_memory_view_release`.
    pub components: *const rb_memory_view_item_component_t,
    pub length: size_t,
}

// `rb_memory_view_t`, the same in Ruby 3.1, 3.2 and 3.3 (3.0 had no
// `_memory_view_entry`).
#[repr(C)]
#[derive(Debug)]
pub struct rb_memory_view_t {
    // The object that exported the memory.
    pub obj: Value,
    // The exported memory.
    pub data: *mut c_void,
    // The number of bytes in `data`.
    pub byte_size: ssize_t,
    pub readonly: bool,
    // The format of an item as pack-template specifiers, or null for
    // unsigned bytes.
    pub format: *const c_char,
    // The number of bytes in each item.
    pub item_size: ssize_t,
    pub item_desc: rb_memory_view_item_desc,
    // The number of dimensions.
    pub ndim: ssize_t,
    // `ndim` element counts, or null when `ndim` is 1.
    pub shape: *const ssize_t,
    // `ndim` byte strides, or null.
    pub strides: *const ssize_t,
    // `ndim` offsets for a nested (indirect) array, or null for a flat one.
    pub sub_offsets: *const ssize_t,
    // The exporter's own data.
    pub private_data: *mut c_void,
    // Set by `rb_memory_view_get`; internal. Ruby 3.1+.
    #[cfg(ruby_gte_3_1)]
    pub _memory_view_entry: *const rb_memory_view_entry_t,
}

// typedef bool (* rb_memory_view_get_func_t)(VALUE obj, rb_memory_view_t *view, int flags)
pub type rb_memory_view_get_func_t =
    rutie_callback!(type fn(obj: Value, view: *mut rb_memory_view_t, flags: c_int) -> bool);
// typedef bool (* rb_memory_view_release_func_t)(VALUE obj, rb_memory_view_t *view)
pub type rb_memory_view_release_func_t =
    rutie_callback!(type fn(obj: Value, view: *mut rb_memory_view_t) -> bool);
// typedef bool (* rb_memory_view_available_p_func_t)(VALUE obj)
pub type rb_memory_view_available_p_func_t = rutie_callback!(type fn(obj: Value) -> bool);

// `rb_memory_view_entry_t`, an exporter's callbacks, registered for a class
// with `rb_memory_view_register`. Ruby calls `get_func` and
// `available_p_func` without checking for null.
#[repr(C)]
pub struct rb_memory_view_entry_t {
    pub get_func: Option<rb_memory_view_get_func_t>,
    pub release_func: Option<rb_memory_view_release_func_t>,
    pub available_p_func: Option<rb_memory_view_available_p_func_t>,
}

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // The objects with exported views and their counts, marked by the GC
    // (for testing).
    pub static rb_memory_view_exported_object_registry: Value;
    pub static rb_memory_view_exported_object_registry_data_type: RbDataType;

    // bool
    // rb_memory_view_register(VALUE klass, const rb_memory_view_entry_t *entry)
    //
    // Ruby keeps `entry`, so it must be `'static`.
    pub fn rb_memory_view_register(klass: Value, entry: *const rb_memory_view_entry_t) -> bool;
    // bool
    // rb_memory_view_is_row_major_contiguous(const rb_memory_view_t *view)
    pub fn rb_memory_view_is_row_major_contiguous(view: *const rb_memory_view_t) -> bool;
    // bool
    // rb_memory_view_is_column_major_contiguous(const rb_memory_view_t *view)
    pub fn rb_memory_view_is_column_major_contiguous(view: *const rb_memory_view_t) -> bool;
    // void
    // rb_memory_view_fill_contiguous_strides(const ssize_t ndim, const ssize_t item_size, const ssize_t *const shape, const bool row_major_p, ssize_t *const strides)
    pub fn rb_memory_view_fill_contiguous_strides(
        ndim: ssize_t,
        item_size: ssize_t,
        shape: *const ssize_t,
        row_major_p: bool,
        strides: *mut ssize_t,
    );
    // bool
    // rb_memory_view_init_as_byte_array(rb_memory_view_t *view, VALUE obj, void *data, const ssize_t len, const bool readonly)
    //
    // Fills `view` as a one-dimensional array of `len` unsigned bytes.
    pub fn rb_memory_view_init_as_byte_array(
        view: *mut rb_memory_view_t,
        obj: Value,
        data: *mut c_void,
        len: ssize_t,
        readonly: bool,
    ) -> bool;
    // ssize_t
    // rb_memory_view_parse_item_format(const char *format, rb_memory_view_item_component_t **members, size_t *n_members, const char **err)
    //
    // The item size, or `-1` with `*err` at the failing character. `*members`
    // (when `members` is not null) is allocated with `ruby_xmalloc`; free it
    // with `ruby_xfree`.
    pub fn rb_memory_view_parse_item_format(
        format: *const c_char,
        members: *mut *mut rb_memory_view_item_component_t,
        n_members: *mut size_t,
        err: *mut *const c_char,
    ) -> ssize_t;
    // ssize_t
    // rb_memory_view_item_size_from_format(const char *format, const char **err)
    //
    // The item size, or `-1` with `*err` at the failing character.
    pub fn rb_memory_view_item_size_from_format(
        format: *const c_char,
        err: *mut *const c_char,
    ) -> ssize_t;
    // void *
    // rb_memory_view_get_item_pointer(rb_memory_view_t *view, const ssize_t *indices)
    //
    // `indices` has `view->ndim` entries.
    pub fn rb_memory_view_get_item_pointer(
        view: *mut rb_memory_view_t,
        indices: *const ssize_t,
    ) -> *mut c_void;
    // VALUE
    // rb_memory_view_extract_item_members(const void *ptr, const rb_memory_view_item_component_t *members, const size_t n_members)
    pub fn rb_memory_view_extract_item_members(
        ptr: *const c_void,
        members: *const rb_memory_view_item_component_t,
        n_members: size_t,
    ) -> Value;
    // void
    // rb_memory_view_prepare_item_desc(rb_memory_view_t *view)
    pub fn rb_memory_view_prepare_item_desc(view: *mut rb_memory_view_t);
    // VALUE
    // rb_memory_view_get_item(rb_memory_view_t *view, const ssize_t *indices)
    //
    // The item at `indices` as a Ruby value (an Array for several members).
    pub fn rb_memory_view_get_item(view: *mut rb_memory_view_t, indices: *const ssize_t) -> Value;
    // bool
    // rb_memory_view_available_p(VALUE obj)
    pub fn rb_memory_view_available_p(obj: Value) -> bool;
    // bool
    // rb_memory_view_get(VALUE obj, rb_memory_view_t* memory_view, int flags)
    //
    // Fills `memory_view` and keeps `obj` alive until the view is released,
    // or returns `false` without touching it.
    pub fn rb_memory_view_get(obj: Value, memory_view: *mut rb_memory_view_t, flags: c_int)
        -> bool;
    // bool
    // rb_memory_view_release(rb_memory_view_t* memory_view)
    pub fn rb_memory_view_release(memory_view: *mut rb_memory_view_t) -> bool;
}
