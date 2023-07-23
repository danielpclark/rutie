use std::convert::From;

use crate::{types::Value, AnyException, AnyObject, Array, Class, Object, VerifiedObject};

/// `Enumerator`
#[derive(Debug)]
#[repr(C)]
pub struct Enumerator {
    value: Value,
}

impl Enumerator {
    /// Advances the iterator and returns the next value.
    ///
    /// Returns [`Err`] when iteration is finished.
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
    /// assert_eq!(Ok(Fixnum::new(2).to_any_object()), iter.next());
    /// assert_eq!(Ok(Fixnum::new(1).to_any_object()), iter.next());
    ///
    /// // ... and then Err once it's over.
    /// assert!(iter.next().is_err(), "not error!");
    ///
    /// // More calls will always return Err.
    /// assert!(iter.next().is_err(), "not error!");
    /// assert!(iter.next().is_err(), "not error!");
    /// ```
    pub fn next(&mut self) -> Result<AnyObject, AnyException> {
        self.protect_send("next", &[])
    }

    /// Advances the iterator and returns the next values.
    ///
    /// Returns [`Err`] when iteration is finished.
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
    /// assert_eq!(Ok(result1), iter.next_values());
    /// let mut result2 = Array::with_capacity(1);
    /// result2.push(Fixnum::new(2));
    /// assert_eq!(Ok(result2), iter.next_values());
    ///
    /// // ... and then Err once it's over.
    /// assert!(iter.next_values().is_err(), "not error!");
    ///
    /// // More calls will always retirn Err.
    /// assert!(iter.next_values().is_err(), "not error!");
    /// assert!(iter.next_values().is_err(), "not error!");
    /// ```
    pub fn next_values(&mut self) -> Result<Array, AnyException> {
        self.protect_send("next_values", &[])
            .map(|v| Array::from(v.value()))
    }

    /// Peeks into the iterator and returns the next value.
    ///
    /// Returns [`Err`] when iteration is finished.
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
    /// assert_eq!(Ok(Fixnum::new(2).to_any_object()), iter.peek());
    /// assert_eq!(Ok(Fixnum::new(2).to_any_object()), iter.peek());
    /// ```
    pub fn peek(&self) -> Result<AnyObject, AnyException> {
        self.protect_send("peek", &[])
    }

    /// Peeks into the iterator and returns the next values.
    ///
    /// Returns [`Err`] when iteration is finished.
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
    /// assert_eq!(Ok(result1.dup()), iter.peek_values());
    /// assert_eq!(Ok(result1), iter.peek_values());
    /// ```
    pub fn peek_values(&self) -> Result<Array, AnyException> {
        self.protect_send("peek_values", &[])
            .map(|v| Array::from(v.value()))
    }

    /// Rewind the iteration back to the beginning.
    ///
    /// Returns [`Err`] when iteration is finished.
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
    /// assert_eq!(Ok(Fixnum::new(2).to_any_object()), iter.next());
    /// assert_eq!(Ok(Fixnum::new(1).to_any_object()), iter.next());
    /// assert!(iter.next().is_err(), "not error!");
    ///
    /// iter.rewind();
    ///
    /// // A call to next() returns the next value...
    /// assert_eq!(Ok(Fixnum::new(2).to_any_object()), iter.next());
    /// assert_eq!(Ok(Fixnum::new(1).to_any_object()), iter.next());
    /// assert!(iter.next().is_err(), "not error!");
    /// ```
    pub fn rewind(&mut self) -> &mut Self {
        unsafe { self.send("rewind", &[]) };
        self
    }

    /// Feed a return value back in to internal yield inside enumerator.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM, VerifiedObject, Enumerator, Class};
    /// # VM::init();
    ///
    /// let mut e_iter = VM::eval("[1,2,3].map").unwrap().
    ///   try_convert_to::<Enumerator>().unwrap();
    ///
    /// assert_eq!(Ok(Fixnum::new(1).to_any_object()), e_iter.next());
    /// e_iter.feed(Fixnum::new(999).to_any_object());
    /// assert_eq!(Ok(Fixnum::new(2).to_any_object()), e_iter.next());
    /// e_iter.feed(Fixnum::new(888).to_any_object());
    /// assert_eq!(Ok(Fixnum::new(3).to_any_object()), e_iter.next());
    /// e_iter.feed(Fixnum::new(777).to_any_object());
    ///
    /// match e_iter.next() {
    ///     Ok(_) => unreachable!(),
    ///     Err(e) => {
    ///         let mut expected = Array::with_capacity(3);
    ///         expected.push(Fixnum::new(999).to_any_object());
    ///         expected.push(Fixnum::new(888).to_any_object());
    ///         expected.push(Fixnum::new(777).to_any_object());
    ///
    ///         assert!(Class::from_existing("StopIteration").case_equals(&e));
    ///         assert_eq!(expected.to_any_object(), unsafe { e.send("result", &[]) });
    ///     },
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// e = [1,2,3].map
    /// p e.next           #=> 1
    /// e.feed 999
    /// p e.next           #=> 2
    /// e.feed 888
    /// p e.next           #=> 3
    /// e.feed 777
    /// begin
    ///   e.next
    /// rescue StopIteration
    ///   p $!.result      #=> [999, 888, 777]
    /// end
    /// ```
    pub fn feed(&mut self, object: AnyObject) -> Result<(), AnyException> {
        self.protect_send("feed", &[object]).map(|_| ())
    }
}

impl From<Value> for Enumerator {
    fn from(value: Value) -> Self {
        Enumerator { value }
    }
}

impl From<Enumerator> for Value {
    fn from(val: Enumerator) -> Self {
        val.value
    }
}

impl From<Enumerator> for AnyObject {
    fn from(val: Enumerator) -> Self {
        AnyObject::from(val.value)
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

impl PartialEq for Enumerator {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}
