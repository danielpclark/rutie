#![allow(non_camel_case_types)]

// `IO::Buffer` (`ruby/io/buffer.h`), which Ruby marks experimental.
use crate::rubysys::types::{c_int, c_void, size_t, Value};

// `enum rb_io_buffer_flags`, which more than one of can be set at a time.
//
// The memory is owned by someone else, so the buffer can't be resized.
pub const RB_IO_BUFFER_EXTERNAL: rb_io_buffer_flags = 1;
// The memory was allocated by the buffer.
pub const RB_IO_BUFFER_INTERNAL: rb_io_buffer_flags = 2;
// The memory is mapped; a non-private mapping is also `EXTERNAL`.
pub const RB_IO_BUFFER_MAPPED: rb_io_buffer_flags = 4;
// A mapped buffer that is also shared.
#[cfg(ruby_gte_3_2)]
pub const RB_IO_BUFFER_SHARED: rb_io_buffer_flags = 8;
// The base address and size can't change (usually during a system call).
pub const RB_IO_BUFFER_LOCKED: rb_io_buffer_flags = 32;
// A private mapping, which doesn't change other processes or the file.
pub const RB_IO_BUFFER_PRIVATE: rb_io_buffer_flags = 64;
// The memory can't be modified.
pub const RB_IO_BUFFER_READONLY: rb_io_buffer_flags = 128;

// `enum rb_io_buffer_endian`.
pub const RB_IO_BUFFER_LITTLE_ENDIAN: c_int = 4;
pub const RB_IO_BUFFER_BIG_ENDIAN: c_int = 8;
#[cfg(target_endian = "little")]
pub const RB_IO_BUFFER_HOST_ENDIAN: c_int = RB_IO_BUFFER_LITTLE_ENDIAN;
#[cfg(target_endian = "big")]
pub const RB_IO_BUFFER_HOST_ENDIAN: c_int = RB_IO_BUFFER_BIG_ENDIAN;
pub const RB_IO_BUFFER_NETWORK_ENDIAN: c_int = RB_IO_BUFFER_BIG_ENDIAN;

// A C `enum`, which has the size of an `int` here.
pub type rb_io_buffer_flags = c_int;

// `rb_off_t` (`off_t` on 3.1): a 64-bit file offset. Ruby is built with
// large file support, and on Windows Ruby 3.2+ uses a 64-bit integer.
pub type rb_off_t = i64;

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // The `IO::Buffer` class.
    pub static rb_cIOBuffer: Value;
    // The operating system's page size.
    pub static RUBY_IO_BUFFER_PAGE_SIZE: size_t;
    // The default buffer size (`IO::Buffer::DEFAULT_SIZE`).
    pub static RUBY_IO_BUFFER_DEFAULT_SIZE: size_t;
}

#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_io_buffer_new(void *base, size_t size, enum rb_io_buffer_flags flags)
    //
    // With a null `base`, allocates `size` zeroed bytes (`INTERNAL`) or maps
    // them (`MAPPED`); otherwise wraps `base`, which must outlive the buffer.
    pub fn rb_io_buffer_new(base: *mut c_void, size: size_t, flags: rb_io_buffer_flags) -> Value;
    // VALUE
    // rb_io_buffer_map(VALUE io, size_t size, rb_off_t offset, enum rb_io_buffer_flags flags)
    pub fn rb_io_buffer_map(
        io: Value,
        size: size_t,
        offset: rb_off_t,
        flags: rb_io_buffer_flags,
    ) -> Value;
    // VALUE
    // rb_io_buffer_lock(VALUE self)
    //
    // Raises `IO::Buffer::LockedError` when already locked.
    pub fn rb_io_buffer_lock(buffer: Value) -> Value;
    // VALUE
    // rb_io_buffer_unlock(VALUE self)
    //
    // Raises `IO::Buffer::LockedError` when not locked.
    pub fn rb_io_buffer_unlock(buffer: Value) -> Value;
    // int
    // rb_io_buffer_try_unlock(VALUE self)
    //
    // Non-zero when the buffer was locked (and is now unlocked).
    pub fn rb_io_buffer_try_unlock(buffer: Value) -> c_int;
    // VALUE
    // rb_io_buffer_free(VALUE self)
    //
    // Raises `IO::Buffer::LockedError` when locked.
    pub fn rb_io_buffer_free(buffer: Value) -> Value;
    // int                                     (3.1, 3.2)
    // enum rb_io_buffer_flags                 (3.3)
    // rb_io_buffer_get_bytes(VALUE self, void **base, size_t *size)
    //
    // The flags, or `0` with a null `base` and a `0` size for a buffer with
    // no (valid) memory. Never raises for an `IO::Buffer`.
    pub fn rb_io_buffer_get_bytes(
        buffer: Value,
        base: *mut *mut c_void,
        size: *mut size_t,
    ) -> rb_io_buffer_flags;
    // void
    // rb_io_buffer_get_bytes_for_reading(VALUE self, const void **base, size_t *size)
    pub fn rb_io_buffer_get_bytes_for_reading(
        buffer: Value,
        base: *mut *const c_void,
        size: *mut size_t,
    );
    // void
    // rb_io_buffer_get_bytes_for_writing(VALUE self, void **base, size_t *size)
    //
    // Raises `IO::Buffer::AccessError` for a read-only buffer.
    pub fn rb_io_buffer_get_bytes_for_writing(
        buffer: Value,
        base: *mut *mut c_void,
        size: *mut size_t,
    );
    // VALUE
    // rb_io_buffer_transfer(VALUE self)
    //
    // A new buffer that takes over the memory, leaving `self` null.
    pub fn rb_io_buffer_transfer(buffer: Value) -> Value;
    // void
    // rb_io_buffer_resize(VALUE self, size_t size)
    pub fn rb_io_buffer_resize(buffer: Value, size: size_t);
    // void
    // rb_io_buffer_clear(VALUE self, uint8_t value, size_t offset, size_t length)
    //
    // `offset + length` is not checked for overflow.
    pub fn rb_io_buffer_clear(buffer: Value, value: u8, offset: size_t, length: size_t);
}

// Reading and writing return the byte count as an Integer, or a negative
// `errno` (they do not raise for a failed system call).
#[cfg(ruby_3_1)]
#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_io_buffer_read(VALUE self, VALUE io, size_t length)
    //
    // One `read(2)` of up to the buffer's size; `length` is only checked
    // against the size.
    pub fn rb_io_buffer_read(buffer: Value, io: Value, length: size_t) -> Value;
    // VALUE
    // rb_io_buffer_pread(VALUE self, VALUE io, size_t length, off_t offset)
    pub fn rb_io_buffer_pread(buffer: Value, io: Value, length: size_t, offset: rb_off_t) -> Value;
    // VALUE
    // rb_io_buffer_write(VALUE self, VALUE io, size_t length)
    pub fn rb_io_buffer_write(buffer: Value, io: Value, length: size_t) -> Value;
    // VALUE
    // rb_io_buffer_pwrite(VALUE self, VALUE io, size_t length, off_t offset)
    pub fn rb_io_buffer_pwrite(buffer: Value, io: Value, length: size_t, offset: rb_off_t)
        -> Value;
}

// `from` is the file offset and `offset` the offset in the buffer; `length`
// is the minimum to read or write.
#[cfg(ruby_gte_3_2)]
#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    // VALUE
    // rb_io_buffer_read(VALUE self, VALUE io, size_t length, size_t offset)
    pub fn rb_io_buffer_read(buffer: Value, io: Value, length: size_t, offset: size_t) -> Value;
    // VALUE
    // rb_io_buffer_pread(VALUE self, VALUE io, rb_off_t from, size_t length, size_t offset)
    pub fn rb_io_buffer_pread(
        buffer: Value,
        io: Value,
        from: rb_off_t,
        length: size_t,
        offset: size_t,
    ) -> Value;
    // VALUE
    // rb_io_buffer_write(VALUE self, VALUE io, size_t length, size_t offset)
    pub fn rb_io_buffer_write(buffer: Value, io: Value, length: size_t, offset: size_t) -> Value;
    // VALUE
    // rb_io_buffer_pwrite(VALUE self, VALUE io, rb_off_t from, size_t length, size_t offset)
    pub fn rb_io_buffer_pwrite(
        buffer: Value,
        io: Value,
        from: rb_off_t,
        length: size_t,
        offset: size_t,
    ) -> Value;
}
