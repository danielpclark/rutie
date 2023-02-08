pub use libc::{c_char, c_double, c_int, c_long, c_void, size_t, ssize_t};
use libc::{intptr_t, uintptr_t};

pub use super::{
    typed_data::{RbDataType, RbDataTypeFunction},
    value::{Value, ValueType},
};

#[cfg(unix)]
pub use std::os::unix::io::RawFd;

pub type Id = rb_sys::ID;
pub type InternalValue = rb_sys::VALUE;
pub type SignedValue = intptr_t;

pub type EncodingIndex = c_int;
pub type EncodingType = CallbackPtr;

pub type VmPointer = CallbackPtr;

pub type Argc = c_int;
pub type CallbackPtr = *const c_void;
pub type CallbackMutPtr = *mut c_void;
pub type BlockCallFunction = extern "C" fn(
    yielded_arg: Value,
    callback_arg: Value,
    argc: c_int,
    argv: *const Value,
    block_arg: Value,
) -> Value;

pub use rb_sys::RBasic;

#[repr(C)]
pub enum st_retval {
    Continue = rb_sys::st_retval::ST_CONTINUE as isize,
    Stop = rb_sys::st_retval::ST_STOP as isize,
    Delete = rb_sys::st_retval::ST_DELETE as isize,
    Check = rb_sys::st_retval::ST_CHECK as isize,
    Replace = rb_sys::st_retval::ST_REPLACE as isize,
}
