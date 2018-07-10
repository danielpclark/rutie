use AnyObject;

pub use rubysys::types::{c_char, c_int, c_long, c_void, size_t, Argc, CallbackMutPtr,
                          CallbackPtr, Id, InternalValue, RbDataType as DataType,
                          RbDataTypeFunction as DataTypeFunction, SignedValue, Value, ValueType};

#[cfg(unix)]
pub use rubysys::types::RawFd;

pub type Callback<I, O> = extern "C" fn(Argc, *const AnyObject, I) -> O;

/// Implicit conversion or `nil`.
///
/// This is meant for “implicit conversions” much like Ruby's:
///
///  * `Array.try_convert`
///  * `Hash.try_convert`
///  * `String.try_convert`
///  * `Regexp.try_convert`
///  * `IO.try_convert`
///
/// This is NOT Rust object to Rust object casting for Ruby objects like `try_convert_to<T>` is.
pub trait TryConvert<T>: Sized {
    /// The type returned in the event of a conversion error.
    type Nil;

    /// Performs the conversion.
    fn try_convert(value: T) -> Result<Self, Self::Nil>;
}
