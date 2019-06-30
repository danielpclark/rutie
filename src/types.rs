use AnyObject;

pub use rubysys::types::{c_char, c_int, c_long, c_void, size_t, Argc, CallbackMutPtr, CallbackPtr,
                          Id, InternalValue, RbDataType as DataType, EncodingIndex, EncodingType,
                          RbDataTypeFunction as DataTypeFunction, SignedValue, Value, ValueType,
                          VmPointer};

#[cfg(unix)]
pub use rubysys::types::RawFd;

pub type Callback<I, O> = extern "C" fn(Argc, *const AnyObject, I) -> O;
