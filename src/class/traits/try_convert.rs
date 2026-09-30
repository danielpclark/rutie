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

#[cfg(test)]
mod tests {
    use crate::{AnyObject, Array, Fixnum, Hash, NilClass, Object, TryConvert, VM};

    #[test]
    fn test_try_convert() {
        crate::on_ruby_thread(|| {
            let array: AnyObject = VM::eval("[1, 2]").unwrap();
            assert_eq!(Array::try_convert(array).unwrap().length(), 2);

            // Objects responding to `to_ary` convert too.
            let convertible = VM::eval("o = Object.new; def o.to_ary; [:x]; end; o").unwrap();
            assert_eq!(Array::try_convert(convertible).unwrap().length(), 1);

            let error: Result<Array, NilClass> = Array::try_convert(Fixnum::new(1).into());
            assert!(error.is_err());

            let hash = VM::eval("{a: 1}").unwrap();
            assert_eq!(Hash::try_convert(hash).unwrap().length(), 1);
            assert!(Hash::try_convert(Fixnum::new(1).to_any_object()).is_err());
        });
    }
}
