use std::ptr;

use crate::{
    binding::fixnum,
    rubysys::io_buffer::{self, rb_io_buffer_flags, rb_off_t},
    types::{c_void, Value},
    util,
};

// `size` zeroed bytes allocated by Ruby; raises `IO::Buffer::AllocationError`
// when that fails.
pub fn new(size: usize) -> Value {
    unsafe { io_buffer::rb_io_buffer_new(ptr::null_mut(), size, io_buffer::RB_IO_BUFFER_INTERNAL) }
}

pub unsafe fn new_external(base: *mut c_void, size: usize, flags: rb_io_buffer_flags) -> Value {
    io_buffer::rb_io_buffer_new(base, size, flags)
}

pub unsafe fn map(io: Value, size: usize, offset: i64, flags: rb_io_buffer_flags) -> Value {
    io_buffer::rb_io_buffer_map(io, size, offset as rb_off_t, flags)
}

// Raises `IO::Buffer::LockedError` when already locked.
pub fn lock(buffer: Value) -> Value {
    unsafe { io_buffer::rb_io_buffer_lock(buffer) }
}

// Raises `IO::Buffer::LockedError` when not locked.
pub fn unlock(buffer: Value) -> Value {
    unsafe { io_buffer::rb_io_buffer_unlock(buffer) }
}

pub fn try_unlock(buffer: Value) -> bool {
    util::c_int_to_bool(unsafe { io_buffer::rb_io_buffer_try_unlock(buffer) })
}

// Raises `IO::Buffer::LockedError` when locked.
pub fn free(buffer: Value) -> Value {
    unsafe { io_buffer::rb_io_buffer_free(buffer) }
}

pub fn free_locked(buffer: Value) -> Value {
    unsafe { io_buffer::rb_io_buffer_free_locked(buffer) }
}

// The memory, its size and the flags; a null pointer, `0` and `0` for a
// buffer with no (valid) memory. Never raises.
pub fn get_bytes(buffer: Value) -> (*mut u8, usize, rb_io_buffer_flags) {
    let mut base = ptr::null_mut();
    let mut size = 0;
    let flags = unsafe { io_buffer::rb_io_buffer_get_bytes(buffer, &mut base, &mut size) };

    (base as *mut u8, size, flags)
}

// Raises for a buffer with no (valid) memory, except on Ruby 3.3, which
// returns a null pointer.
pub fn get_bytes_for_reading(buffer: Value) -> (*const u8, usize) {
    let mut base = ptr::null();
    let mut size = 0;

    unsafe { io_buffer::rb_io_buffer_get_bytes_for_reading(buffer, &mut base, &mut size) };

    (base as *const u8, size)
}

// Like `get_bytes_for_reading`, and raises `IO::Buffer::AccessError` for a
// read-only buffer.
pub fn get_bytes_for_writing(buffer: Value) -> (*mut u8, usize) {
    let mut base = ptr::null_mut();
    let mut size = 0;

    unsafe { io_buffer::rb_io_buffer_get_bytes_for_writing(buffer, &mut base, &mut size) };

    (base as *mut u8, size)
}

pub fn transfer(buffer: Value) -> Value {
    unsafe { io_buffer::rb_io_buffer_transfer(buffer) }
}

pub fn resize(buffer: Value, size: usize) {
    unsafe { io_buffer::rb_io_buffer_resize(buffer, size) }
}

// The caller checks that `offset + length` does not overflow.
pub fn clear(buffer: Value, value: u8, offset: usize, length: usize) {
    unsafe { io_buffer::rb_io_buffer_clear(buffer, value, offset, length) }
}

// The byte count as an Integer, or a negative `errno`. The caller checks that
// `offset` is within the buffer.
pub fn read(buffer: Value, io: Value, length: usize, offset: usize) -> Value {
    unsafe { io_buffer::rb_io_buffer_read(buffer, io, length, offset) }
}

pub fn pread(buffer: Value, io: Value, from: i64, length: usize, offset: usize) -> Value {
    unsafe { io_buffer::rb_io_buffer_pread(buffer, io, from, length, offset) }
}

pub fn write(buffer: Value, io: Value, length: usize, offset: usize) -> Value {
    unsafe { io_buffer::rb_io_buffer_write(buffer, io, length, offset) }
}

pub fn pwrite(buffer: Value, io: Value, from: i64, length: usize, offset: usize) -> Value {
    unsafe { io_buffer::rb_io_buffer_pwrite(buffer, io, from, length, offset) }
}
