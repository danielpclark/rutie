use std::convert::From;

use crate::{
    binding::{enumerator, vm},
    rubysys::exception::rb_eException,
    types::{Value, ValueType},
    AnyException, AnyObject, Array, Class, Fixnum, Object, VerifiedObject,
};

/// `Enumerator`
#[derive(Debug)]
#[repr(C)]
pub struct Enumerator {
    value: Value,
}

impl Enumerator {
    // External enumeration runs on a fiber, and Ruby refuses to resume a
    // fiber under a different `rb_protect` than the one it was created
    // under ("fiber called across stack rewinding barrier"). `protect_send`
    // opens a new `rb_protect` on every call, which only worked when every
    // call came from the same stack depth, so the enumeration methods go
    // through `vm::fiber_call` (`rb_rescue2`; `rb_protect` only on Rubies
    // whose fibers copy the machine stack, which need it).
    fn rescue_send(
        &self,
        method: &str,
        arguments: &[AnyObject],
    ) -> Result<AnyObject, AnyException> {
        let enumerator = self.value();
        let arguments = crate::util::arguments_to_values(arguments);

        vm::fiber_call(|| vm::call_method(enumerator, method, &arguments))
            .map(AnyObject::from)
            .map_err(AnyException::from)
    }

    /// Creates an enumerator over what `object.method(*arguments)` yields,
    /// like Ruby's `object.to_enum(method, *arguments)`
    /// (`rb_enumeratorize`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Enumerator, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let array: Array = (1..=3).map(|i| Fixnum::new(i).to_any_object()).collect();
    /// let mut pairs = Enumerator::new(&array, "each_slice", &[Fixnum::new(2).into()]);
    ///
    /// let first = pairs.next().unwrap().try_convert_to::<Array>().unwrap();
    ///
    /// assert_eq!(first.length(), 2);
    /// ```
    pub fn new<T: Object>(object: &T, method: &str, arguments: &[AnyObject]) -> Self {
        let arguments = crate::util::arguments_to_values(arguments);

        Enumerator::from(enumerator::enumeratorize(
            object.value(),
            method,
            &arguments,
        ))
    }

    /// Returns a Rust iterator over the enumerator's values, starting where
    /// the enumerator currently is. It ends at `StopIteration`; any other
    /// exception is yielded once as `Err` and ends the iteration.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Enumerator, VM};
    /// # VM::init();
    ///
    /// let enumerator = VM::eval("[1, 2, 3].each").unwrap().try_convert_to::<Enumerator>().unwrap();
    ///
    /// let sum: i64 = enumerator
    ///     .iter()
    ///     .map(|value| value.unwrap().try_convert_to::<Fixnum>().unwrap().to_i64())
    ///     .sum();
    ///
    /// assert_eq!(sum, 6);
    ///
    /// let failing = VM::eval("Enumerator.new { |y| y << 1; raise 'broken' }").unwrap()
    ///     .try_convert_to::<Enumerator>().unwrap();
    /// let results: Vec<_> = failing.iter().collect();
    ///
    /// assert_eq!(results.len(), 2);
    /// assert!(results[1].is_err());
    /// ```
    pub fn iter(&self) -> EnumeratorIterator {
        EnumeratorIterator {
            enumerator: Enumerator::from(self.value()),
            done: false,
        }
    }

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
        self.rescue_send("next", &[])
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
        self.rescue_send("next_values", &[])
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
        self.rescue_send("peek", &[])
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
        self.rescue_send("peek_values", &[])
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
        self.rescue_send("feed", &[object]).map(|_| ())
    }
}

impl From<Value> for Enumerator {
    fn from(value: Value) -> Self {
        Enumerator { value }
    }
}

impl Into<Value> for Enumerator {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Enumerator {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
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
        Class::enumerator().case_equals(object)
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

/// Rust iterator over an [`Enumerator`](struct.Enumerator.html); see
/// [`Enumerator::iter`](struct.Enumerator.html#method.iter).
#[derive(Debug)]
pub struct EnumeratorIterator {
    enumerator: Enumerator,
    done: bool,
}

impl Iterator for EnumeratorIterator {
    type Item = Result<AnyObject, AnyException>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        match self.enumerator.next() {
            Ok(value) => Some(Ok(value)),
            Err(error) => {
                self.done = true;

                if unsafe { stop_iteration_class() }.case_equals(&error) {
                    None
                } else {
                    Some(Err(error))
                }
            }
        }
    }
}

/// Iterates over the enumerator's values; see
/// [`Enumerator::iter`](struct.Enumerator.html#method.iter).
///
/// # Examples
///
/// ```
/// use rutie::{Enumerator, Object, VM};
/// # VM::init();
///
/// let enumerator = VM::eval("(1..4).each").unwrap().try_convert_to::<Enumerator>().unwrap();
///
/// assert_eq!(enumerator.into_iter().count(), 4);
/// ```
impl IntoIterator for Enumerator {
    type Item = Result<AnyObject, AnyException>;
    type IntoIter = EnumeratorIterator;

    fn into_iter(self) -> Self::IntoIter {
        EnumeratorIterator {
            enumerator: self,
            done: false,
        }
    }
}

unsafe fn stop_iteration_class() -> Class {
    Class::from(crate::rubysys::exception::rb_eStopIteration)
}

#[cfg(test)]
mod tests {
    use crate::{Array, Enumerator, Fixnum, Hash, Object, Symbol, VM};

    #[test]
    fn test_enumerator_new_and_iteration() {
        crate::on_ruby_thread(|| {
            let mut hash = Hash::new();
            hash.store(Symbol::new("a"), Fixnum::new(1));
            hash.store(Symbol::new("b"), Fixnum::new(2));

            // One loop, so every `next` comes from the same place, as Ruby 2.5
            // and 2.6 on arm64 macOS need (see `Fiber::new`).
            let pairs = Enumerator::new(&hash, "each_pair", &[]);
            let mut collected = Vec::new();
            for pair in pairs.iter() {
                collected.push(pair.unwrap());
            }
            assert_eq!(collected.len(), 2);
            assert_eq!(
                collected[1]
                    .try_convert_to::<Array>()
                    .unwrap()
                    .at(0)
                    .try_convert_to::<Symbol>(),
                Ok(Symbol::new("b"))
            );

            // The rest resumes one enumerator from several places.
            if cfg!(rutie_copy_stack_fibers) {
                return;
            }

            // Iteration continues from the enumerator's position.
            let mut numbers = VM::eval("[1, 2, 3].each")
                .unwrap()
                .try_convert_to::<Enumerator>()
                .unwrap();
            numbers.next().unwrap();
            assert_eq!(numbers.iter().count(), 2);
            numbers.rewind();
            assert_eq!(numbers.into_iter().count(), 3);

            // An infinite enumerator works lazily.
            let naturals = VM::eval("(1..Float::INFINITY).each")
                .unwrap()
                .try_convert_to::<Enumerator>()
                .unwrap();
            let first: Vec<i64> = naturals
                .iter()
                .take(3)
                .map(|value| value.unwrap().try_convert_to::<Fixnum>().unwrap().to_i64())
                .collect();
            assert_eq!(first, vec![1, 2, 3]);
        });
    }

    // Each call to `next` used to open its own `rb_protect`; calls from
    // different stack depths then failed with a `FiberError`.
    #[test]
    fn test_next_from_different_stack_depths() {
        crate::on_ruby_thread(|| {
            fn deeper(enumerator: &mut Enumerator, depth: usize) -> i64 {
                if depth == 0 {
                    enumerator
                        .next()
                        .unwrap()
                        .try_convert_to::<Fixnum>()
                        .unwrap()
                        .to_i64()
                } else {
                    let padding = [depth; 16];
                    deeper(enumerator, depth - 1) + (padding[0] - depth) as i64
                }
            }

            let mut enumerator = VM::eval("[1, 2, 3, 4].each")
                .unwrap()
                .try_convert_to::<Enumerator>()
                .unwrap();

            // Ruby 2.5 and 2.6 on arm64 macOS copy the fiber's stack and
            // resume it only from the `rb_protect` it started under (see
            // `Fiber::new`): another stack depth is a `FiberError`, not a crash.
            if cfg!(rutie_copy_stack_fibers) {
                assert_eq!(deeper(&mut enumerator, 0), 1);
                assert!(enumerator.next().is_err());

                return;
            }

            assert_eq!(deeper(&mut enumerator, 0), 1);
            assert_eq!(deeper(&mut enumerator, 5), 2);
            assert_eq!(
                enumerator.peek().unwrap().try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(3))
            );
            assert_eq!(deeper(&mut enumerator, 2), 3);
            assert_eq!(deeper(&mut enumerator, 9), 4);
            assert!(enumerator.next().is_err());
        });
    }

    #[test]
    fn test_try_compare() {
        crate::on_ruby_thread(|| {
            use std::cmp::Ordering;

            let a = crate::RString::new_utf8("a");
            let b = crate::RString::new_utf8("b");
            assert_eq!(a.try_compare(&b).unwrap(), Ordering::Less);
            assert_eq!(b.try_compare(&a).unwrap(), Ordering::Greater);
            assert!(a.try_compare(&Fixnum::new(1)).is_err());
        });
    }

    #[test]
    fn test_enumerator_values_and_feed() {
        crate::on_ruby_thread(|| {
            let mut pairs = VM::eval("{a: 1}.each")
                .unwrap()
                .try_convert_to::<Enumerator>()
                .unwrap();
            let peeked = pairs.peek_values().unwrap();
            assert_eq!(peeked.length(), 1);
            let values = pairs.next_values().unwrap();
            assert_eq!(values.at(0).try_convert_to::<Array>().unwrap().length(), 2);
            assert!(pairs.next_values().is_err());
            assert!(pairs.peek_values().is_err());

            // `feed` sets what the block's `yield` returns (as in `map`).
            let mut mapper = VM::eval("[1, 2].map")
                .unwrap()
                .try_convert_to::<Enumerator>()
                .unwrap();
            mapper.next().unwrap();
            mapper.feed(Symbol::new("first").to_any_object()).unwrap();
            // Feeding twice before `next` is an error.
            assert!(mapper.feed(Symbol::new("again").to_any_object()).is_err());
            mapper.next().unwrap();
            mapper.feed(Symbol::new("second").to_any_object()).unwrap();

            let error = mapper.next().unwrap_err();
            let result = unsafe { error.send("result", &[]) }
                .try_convert_to::<Array>()
                .unwrap();
            assert_eq!(
                result.at(0).try_convert_to::<Symbol>().unwrap().to_str(),
                "first"
            );
            assert_eq!(
                result.at(1).try_convert_to::<Symbol>().unwrap().to_str(),
                "second"
            );
        });
    }
}
