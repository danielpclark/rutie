use crate::AnyObject;

pub use crate::rubysys::types::{
    c_char, c_int, c_long, c_void, size_t, st_retval, Argc, CallbackMutPtr, CallbackPtr,
    EncodingIndex, EncodingType, Id, InternalValue, RbDataType as DataType,
    RbDataTypeFunction as DataTypeFunction, SignedValue, Value, ValueType, VmPointer,
};

#[cfg(unix)]
pub use crate::rubysys::types::RawFd;

pub type Callback<I, O> = extern "C" fn(Argc, *const AnyObject, I) -> O;
