use types::Value;

use {Object, VerifiedObject};

/// Representation of any Ruby object while its type is unknown
///
/// As Ruby is a dynamically typed language, at some points Ruru does not know the exact Ruby type
/// of the object, for example:
///
/// - Retrieving an object from array;
///
/// - Retrieving an object from hash;
///
/// - Receiving arguments to a method;
///
/// - Initializing a new instance of a non-built-in class.
///
/// In these cases you should cast `AnyObject` to the required type.
///
/// # Examples
///
/// ### Retrieving an object from `Array`
///
/// ```
/// use rutie::{Array, Fixnum, Object, VM};
/// # VM::init();
///
/// let array = Array::new().push(Fixnum::new(1));
/// let value = array.at(0).try_convert_to::<Fixnum>(); // `Array::at()` returns `AnyObject`
///
/// assert_eq!(value, Ok(Fixnum::new(1)));
/// ```
///
/// ### Retrieving an object from `Hash`
///
/// ```
/// use rutie::{Fixnum, Hash, Object, Symbol, VM};
/// # VM::init();
///
/// let mut hash = Hash::new();
///
/// hash.store(Symbol::new("key"), Fixnum::new(1));
///
/// // `Hash::at()` returns `AnyObject`
/// let value = hash.at(&Symbol::new("key")).try_convert_to::<Fixnum>();
///
/// assert_eq!(value, Ok(Fixnum::new(1)));
/// ```
///
/// You can find more examples in `Class`, `Object` and `VerifiedObject` documentation.
#[derive(Clone, Debug, PartialEq)]
pub struct AnyObject {
    value: Value,
}

impl From<Value> for AnyObject {
    fn from(value: Value) -> Self {
        AnyObject { value: value }
    }
}

impl Object for AnyObject {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

// Any object can be safely converted to `AnyObject` :)
impl VerifiedObject for AnyObject {
    fn is_correct_type<T: Object>(_: &T) -> bool {
        true
    }

    fn error_message() -> &'static str {
        unreachable!()
    }
}
