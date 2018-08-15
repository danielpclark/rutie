use std::convert::From;
use std::default::Default;

use types::{Value, ValueType};

use {
    AnyObject,
    AnyException,
    Array,
    Class,
    Fixnum,
    Object,
    VerifiedObject,
};

/// `Enumerator`
#[derive(Debug, PartialEq)]
pub struct Enumerator {
    value: Value,
}

impl Enumerator {
    /// Advances the iterator and returns the next values.
    ///
    /// Returns [`None`] when iteration is finished.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM, VerifiedObject, Enumerator};
    /// # VM::init();
    ///
    /// let mut array = Array::with_capacity(2);
    ///
    /// array.push(Fixnum::new(1));
    /// array.push(Fixnum::new(2));
    ///
    /// let mut iter = array.to_enum();
    ///
    /// // A call to next_values() returns the next values...
    /// let mut result1 = Array::with_capacity(1);
    /// result1.push(Fixnum::new(1));
    /// assert_eq!(Some(result1), iter.next_values());
    /// let mut result2 = Array::with_capacity(1);
    /// result2.push(Fixnum::new(2));
    /// assert_eq!(Some(result2), iter.next_values());
    ///
    /// // ... and then None once it's over.
    /// assert_eq!(None, iter.next_values());
    ///
    /// // More calls will always retirn None.
    /// assert_eq!(None, iter.next_values());
    /// assert_eq!(None, iter.next_values());
    /// ```
    pub fn next_values(&mut self) -> Option<Array> {
        self.protect_send("next_values", None).
            map(|v| Array::from(v.value()) ).ok()
    }

    /// Peeks into the iterator and returns the next value.
    ///
    /// Returns [`None`] when iteration is finished.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM, VerifiedObject, Enumerator};
    /// # VM::init();
    ///
    /// let mut iter = Array::new().push(Fixnum::new(2)).push(Fixnum::new(1)).to_enum();
    ///
    /// // A call to peek() returns the next value without progressing the iteration
    /// assert_eq!(Some(Fixnum::new(2).to_any_object()), iter.peek());
    /// assert_eq!(Some(Fixnum::new(2).to_any_object()), iter.peek());
    /// ```
    pub fn peek(&self) -> Option<AnyObject> {
        self.protect_send("peek", None).ok()
    }

    /// Peeks into the iterator and returns the next values.
    ///
    /// Returns [`None`] when iteration is finished.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM, VerifiedObject, Enumerator};
    /// # VM::init();
    ///
    /// let mut array = Array::with_capacity(2);
    ///
    /// array.push(Fixnum::new(1));
    /// array.push(Fixnum::new(2));
    ///
    /// let mut iter = array.to_enum();
    ///
    /// // A call to peek_values() returns the next values without progressing the iteration
    /// let mut result1 = Array::with_capacity(1);
    /// result1.push(Fixnum::new(1));
    /// assert_eq!(Some(result1.dup()), iter.peek_values());
    /// assert_eq!(Some(result1), iter.peek_values());
    /// ```
    pub fn peek_values(&self) -> Option<Array> {
        self.protect_send("peek_values", None).
            map(|v| Array::from(v.value()) ).ok()
    }

    /// Rewind the iteration back to the beginning.
    ///
    /// Returns [`None`] when iteration is finished.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM, VerifiedObject, Enumerator};
    /// # VM::init();
    ///
    /// let mut iter = Array::new().push(Fixnum::new(2)).push(Fixnum::new(1)).to_enum();
    ///
    /// // A call to next() returns the next value...
    /// assert_eq!(Some(Fixnum::new(2).to_any_object()), iter.next());
    /// assert_eq!(Some(Fixnum::new(1).to_any_object()), iter.next());
    /// assert_eq!(None, iter.next());
    ///
    /// iter.rewind();
    ///
    /// // A call to next() returns the next value...
    /// assert_eq!(Some(Fixnum::new(2).to_any_object()), iter.next());
    /// assert_eq!(Some(Fixnum::new(1).to_any_object()), iter.next());
    /// assert_eq!(None, iter.next());
    /// ```
    pub fn rewind(&mut self) -> &mut Self {
        self.send("rewind", None);
        self
    }

    pub fn feed(&mut self, object: AnyObject) -> Result<(), AnyException> {
        self.protect_send("feed", Some(&[object])).map(|_|())
    }
}

impl From<Value> for Enumerator {
    fn from(value: Value) -> Self {
        Enumerator { value: value }
    }
}

impl Object for Enumerator {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Enumerator {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        Class::from_existing("Enumerator").case_equals(object)
    }

    fn error_message() -> &'static str {
        "Error converting to Enumerator"
    }
}

impl Iterator for Enumerator {
    /// The return type for the iterator.
    type Item = AnyObject;

    /// Advances the iterator and returns the next value.
    ///
    /// Returns [`None`] when iteration is finished.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM, VerifiedObject, Enumerator};
    /// # VM::init();
    ///
    /// let mut iter = Array::new().push(Fixnum::new(2)).push(Fixnum::new(1)).to_enum();
    ///
    /// // A call to next() returns the next value...
    /// assert_eq!(Some(Fixnum::new(2).to_any_object()), iter.next());
    /// assert_eq!(Some(Fixnum::new(1).to_any_object()), iter.next());
    ///
    /// // ... and then None once it's over.
    /// assert_eq!(None, iter.next());
    ///
    /// // More calls will always retirn None.
    /// assert_eq!(None, iter.next());
    /// assert_eq!(None, iter.next());
    /// ```
    fn next(&mut self) -> Option<AnyObject> {
        self.protect_send("next", None).ok()
    }
}
