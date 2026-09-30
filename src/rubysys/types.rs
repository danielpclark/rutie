use crate::rubysys::libc::{intptr_t, uintptr_t};

pub use crate::rubysys::{
    libc::{c_char, c_double, c_int, c_long, c_uint, c_void, size_t, ssize_t},
    typed_data::{RbDataType, RbDataTypeFunction},
    value::{Value, ValueType},
};

#[cfg(unix)]
pub use std::os::unix::io::RawFd;

// On Windows Ruby takes descriptors of the C runtime it is built with (what
// `IO#fileno` returns), not a `HANDLE` or `SOCKET`.
#[cfg(windows)]
pub type RawFd = c_int;

pub type Id = uintptr_t;
pub type InternalValue = uintptr_t;
pub type SignedValue = intptr_t;

pub type EncodingIndex = c_int;
pub type EncodingType = CallbackPtr;

pub type VmPointer = CallbackPtr;

pub type Argc = c_int;
pub type CallbackPtr = *const c_void;
pub type CallbackMutPtr = *mut c_void;
pub type BlockCallFunction = rutie_callback!(type fn(
    yielded_arg: Value,
    callback_arg: Value,
    argc: c_int,
    argv: *const Value,
    block_arg: Value,
) -> Value);

#[repr(C)]
pub struct RBasic {
    pub flags: InternalValue,
    pub klass: InternalValue,
}

#[repr(C)]
pub enum st_retval {
    Continue,
    Stop,
    Delete,
    Check,
    Replace,
}
