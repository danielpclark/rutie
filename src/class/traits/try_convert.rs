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
