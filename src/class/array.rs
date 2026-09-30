use std::{
    cmp::Ordering,
    convert::From,
    default::Default,
    iter::{FromIterator, IntoIterator, Iterator},
};

use crate::{
    binding::array,
    types::{Value, ValueType},
    AnyException, AnyObject, Enumerator, Fixnum, NilClass, Object, RString, TryConvert,
    VerifiedObject,
};

/// `Array`
#[derive(Debug)]
#[repr(C)]
pub struct Array {
    value: Value,
}

impl Array {
    /// Converts `object` to a `Array` the way Ruby's `Array(object)`
    /// (`Kernel#Array`, `rb_Array`) does, calling `to_ary` or `to_a`, wrapping anything else in a one-element array (`nil` becomes `[]`). Returns the exception when
    /// it cannot be converted.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, NilClass, Object, VM};
    /// # VM::init();
    ///
    /// let wrapped = Array::convert(&Fixnum::new(1)).unwrap();
    /// assert_eq!(wrapped.length(), 1);
    ///
    /// let empty = Array::convert(&NilClass::new()).unwrap();
    /// assert_eq!(empty.length(), 0);
    ///
    /// let range = Array::convert(&VM::eval("1..3").unwrap()).unwrap();
    /// assert_eq!(range.length(), 3);
    /// ```
    pub fn convert<T: Object>(object: &T) -> Result<Self, AnyException> {
        let object = object.value();

        crate::binding::vm::protect_value(|| crate::binding::object::to_array(object))
            .map(Self::from)
            .map_err(AnyException::from)
    }

    /// Creates a new instance of empty `Array`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, VM};
    /// # VM::init();
    ///
    /// Array::new();
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// []
    /// ```
    pub fn new() -> Self {
        Self::from(array::new())
    }

    /// Creates a new instance of empty `Array` with reserved space for `capacity` elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, VM};
    /// # VM::init();
    ///
    /// let mut array = Array::with_capacity(2);
    ///
    /// assert_eq!(array.length(), 0);
    ///
    /// array.push(Fixnum::new(1));
    /// array.push(Fixnum::new(2));
    ///
    /// assert_eq!(array.length(), 2);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// []
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self::from(array::with_capacity(capacity))
    }

    /// Retrieves the length of the array.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, VM};
    /// # VM::init();
    ///
    /// let mut array = Array::new().push(Fixnum::new(1));
    ///
    /// assert_eq!(array.length(), 1);
    ///
    /// array.push(Fixnum::new(2));
    ///
    /// assert_eq!(array.length(), 2);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// array = [1]
    /// array.length == 1
    ///
    /// array << 2
    /// array.length == 2
    /// ```
    pub fn length(&self) -> usize {
        array::len(self.value()) as usize
    }

    /// Retrieves an `AnyObject` from the element at `index` position.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let array = Array::new().push(Fixnum::new(1));
    ///
    /// assert_eq!(array.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// array = [1]
    ///
    /// array[0] == 1
    /// ```
    pub fn at(&self, index: i64) -> AnyObject {
        let result = array::entry(self.value(), index);

        AnyObject::from(result)
    }

    /// Joins all elements of `Array` to Ruby `String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, RString, VM};
    /// # VM::init();
    ///
    /// let array = Array::new()
    ///     .push(RString::new_utf8("Hello"))
    ///     .push(RString::new_utf8("World!"));
    ///
    /// let joined_string = array.join(RString::new_utf8(", "));
    ///
    /// assert_eq!(joined_string.to_str(), "Hello, World!");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// array = ['Hello', 'World!']
    ///
    /// joined_string = array.join(', ')
    ///
    /// joined_string == 'Hello, World!'
    /// ```
    pub fn join(&self, separator: RString) -> RString {
        let result = array::join(self.value(), separator.value());

        RString::from(result)
    }

    /// Pushes an object to `Array`.
    ///
    /// Ruby raises `FrozenError` if the array is frozen; check
    /// [`is_frozen`](trait.Object.html#method.is_frozen) first or call it inside
    /// [`VM::protect`](struct.VM.html#method.protect) when that is possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let mut array = Array::new();
    ///
    /// array.push(Fixnum::new(1));
    ///
    /// assert_eq!(array.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// array = []
    /// array << 1
    ///
    /// array[0] == 1
    /// ```
    pub fn push<T: Object>(&mut self, item: T) -> Self {
        let result = array::push(self.value(), item.value());

        Array::from(result)
    }

    /// Stores an object at `index` position.
    ///
    /// Ruby raises `FrozenError` if the array is frozen; check
    /// [`is_frozen`](trait.Object.html#method.is_frozen) first or call it inside
    /// [`VM::protect`](struct.VM.html#method.protect) when that is possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let mut array = Array::new().push(Fixnum::new(1));
    ///
    /// array.store(0, Fixnum::new(2));
    ///
    /// assert_eq!(array.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// array = [1]
    /// array[0] = 2
    ///
    /// array[0] == 2
    /// ```
    pub fn store<T: Object>(&mut self, index: i64, item: T) {
        array::store(self.value(), index, item.value());
    }

    /// Removes and returns the last element of the array.
    ///
    /// Ruby raises `FrozenError` if the array is frozen; check
    /// [`is_frozen`](trait.Object.html#method.is_frozen) first or call it inside
    /// [`VM::protect`](struct.VM.html#method.protect) when that is possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let mut array = Array::new().push(Fixnum::new(1));
    ///
    /// assert_eq!(array.pop().try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// array = [1]
    ///
    /// array.pop == 1
    /// ```
    pub fn pop(&mut self) -> AnyObject {
        let result = array::pop(self.value());

        AnyObject::from(result)
    }

    /// Inserts `item` at the beginning of the array.
    ///
    /// Ruby raises `FrozenError` if the array is frozen; check
    /// [`is_frozen`](trait.Object.html#method.is_frozen) first or call it inside
    /// [`VM::protect`](struct.VM.html#method.protect) when that is possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let mut array = Array::new().push(Fixnum::new(1));
    ///
    /// array.unshift(Fixnum::new(2));
    ///
    /// assert_eq!(array.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// array = [1]
    /// array.unshift(2)
    ///
    /// array[0] == 2
    /// ```
    pub fn unshift<T: Object>(&mut self, item: T) -> Array {
        let result = array::unshift(self.value(), item.value());

        Array::from(result)
    }

    /// Removes the first item of the array and returns it.
    ///
    /// Ruby raises `FrozenError` if the array is frozen; check
    /// [`is_frozen`](trait.Object.html#method.is_frozen) first or call it inside
    /// [`VM::protect`](struct.VM.html#method.protect) when that is possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let mut array = Array::new().push(Fixnum::new(1)).push(Fixnum::new(2));
    ///
    /// let item = array.shift();
    ///
    /// assert_eq!(item.try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// assert_eq!(array.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// array = [1, 2]
    ///
    /// item = array.shift
    ///
    /// item == 1
    /// array[0] == 2
    /// ```
    pub fn shift(&mut self) -> AnyObject {
        let result = array::shift(self.value());

        AnyObject::from(result)
    }

    /// Creates a copy of the array.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let array = Array::new().push(Fixnum::new(1));
    /// let copy = array.dup();
    ///
    /// assert_eq!(array.at(0), copy.at(0));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// array = [1]
    /// copy = array.dup
    ///
    /// array[0] == copy[0]
    /// ```
    pub fn dup(&self) -> Array {
        let result = array::dup(self.value());

        Array::from(result)
    }

    /// Creates a string representation of the array.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, VM};
    /// # VM::init();
    ///
    /// let array = Array::new().push(Fixnum::new(1)).push(Fixnum::new(2));
    ///
    /// assert_eq!(array.to_s().to_str(), "[1, 2]");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// array = [1, 2]
    ///
    /// array.to_s == "[1, 2]"
    /// ```
    pub fn to_s(&self) -> RString {
        let result = array::to_s(self.value());

        RString::from(result)
    }

    /// Returns a new array containing array's elements in reverse order.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let mut array = Array::new().push(Fixnum::new(1)).push(Fixnum::new(2));
    ///
    /// let reversed_array = array.reverse();
    ///
    /// assert_eq!(reversed_array.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// assert_eq!(reversed_array.at(1).try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// array = [1, 2]
    ///
    /// reversed_array = array.reverse
    ///
    /// reversed_array[0] == 2
    /// reversed_array[1] == 1
    /// ```
    pub fn reverse(&self) -> Array {
        self.dup().reverse_bang()
    }

    /// Reverses `self` in place.
    ///
    /// Ruby raises `FrozenError` if the array is frozen; check
    /// [`is_frozen`](trait.Object.html#method.is_frozen) first or call it inside
    /// [`VM::protect`](struct.VM.html#method.protect) when that is possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let mut array = Array::new().push(Fixnum::new(1)).push(Fixnum::new(2));
    ///
    /// array.reverse_bang();
    ///
    /// assert_eq!(array.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// assert_eq!(array.at(1).try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// array = [1, 2]
    ///
    /// array.reverse!
    ///
    /// array[0] == 2
    /// array[1] == 1
    /// ```
    pub fn reverse_bang(&mut self) -> Array {
        let result = array::reverse_bang(self.value());

        Array::from(result)
    }

    /// Appends the elements of `other` array to `self`.
    ///
    /// Ruby raises `FrozenError` if the array is frozen; check
    /// [`is_frozen`](trait.Object.html#method.is_frozen) first or call it inside
    /// [`VM::protect`](struct.VM.html#method.protect) when that is possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let mut array = Array::new().push(Fixnum::new(1));
    /// let other = Array::new().push(Fixnum::new(2));
    ///
    /// array.concat(&other);
    ///
    /// assert_eq!(array.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// assert_eq!(array.at(1).try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// array = [1]
    /// other = [2]
    ///
    /// array.concat(other)
    ///
    /// array[0] == 1
    /// array[1] == 2
    /// ```
    pub fn concat(&mut self, other: &Array) -> Array {
        let result = array::concat(self.value(), other.value());

        Array::from(result)
    }

    /// Returns a new array created by sorting `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let array = Array::new().push(Fixnum::new(2)).push(Fixnum::new(1));
    ///
    /// let sorted_array = array.sort();
    ///
    /// assert_eq!(sorted_array.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// assert_eq!(sorted_array.at(1).try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// array = [2, 1]
    ///
    /// sorted_array = array.sort
    ///
    /// sorted_array[0] == 1
    /// sorted_array[1] == 2
    /// ```
    pub fn sort(&self) -> Array {
        let result = array::sort(self.value());
        Array::from(result)
    }

    /// Sorts the array in place.
    ///
    /// Ruby raises `FrozenError` if the array is frozen; check
    /// [`is_frozen`](trait.Object.html#method.is_frozen) first or call it inside
    /// [`VM::protect`](struct.VM.html#method.protect) when that is possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let mut array = Array::new().push(Fixnum::new(2)).push(Fixnum::new(1));
    ///
    /// array.sort_bang();
    ///
    /// assert_eq!(array.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// assert_eq!(array.at(1).try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// array = [2, 1]
    ///
    /// array.sort!
    ///
    /// array[0] == 1
    /// array[1] == 2
    /// ```
    pub fn sort_bang(&mut self) -> Array {
        let result = array::sort_bang(self.value());
        Array::from(result)
    }

    /// Returns an `Enumerator` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM, VerifiedObject, Enumerator};
    /// # VM::init();
    ///
    /// let enumerator = Array::new().push(Fixnum::new(2)).push(Fixnum::new(1)).to_enum();
    ///
    /// assert!(Enumerator::is_correct_type(&enumerator), "incorrect type!");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// enumerator = [2, 1].to_enum
    ///
    /// Enumerator === enumerator
    /// ```
    pub fn to_enum(&self) -> Enumerator {
        unsafe { self.send("to_enum", &[]) }
            .try_convert_to::<Enumerator>()
            .unwrap()
    }

    /// Removes every element equal (`==`) to `item` and returns `item`, or
    /// `nil` if there was none (Ruby's `delete`, `rb_ary_delete`).
    ///
    /// Ruby raises `FrozenError` if the array is frozen; check
    /// [`is_frozen`](trait.Object.html#method.is_frozen) first or call it inside
    /// [`VM::protect`](struct.VM.html#method.protect) when that is possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let mut array: Array = [1, 2, 1, 3].iter().map(|&i| Fixnum::new(i).to_any_object()).collect();
    ///
    /// let removed = array.delete(Fixnum::new(1));
    ///
    /// assert_eq!(removed.try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// assert_eq!(array.length(), 2);
    /// assert!(array.delete(Fixnum::new(9)).is_nil());
    /// ```
    pub fn delete<T: Object>(&mut self, item: T) -> AnyObject {
        AnyObject::from(array::delete(self.value(), item.value()))
    }

    /// Removes and returns the element at `index` (from the end when
    /// negative), or `nil` when out of range (Ruby's `delete_at`,
    /// `rb_ary_delete_at`).
    ///
    /// Ruby raises `FrozenError` if the array is frozen; check
    /// [`is_frozen`](trait.Object.html#method.is_frozen) first or call it inside
    /// [`VM::protect`](struct.VM.html#method.protect) when that is possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let mut array: Array = (1..=3).map(|i| Fixnum::new(i).to_any_object()).collect();
    ///
    /// assert_eq!(array.delete_at(-1).try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
    /// assert!(array.delete_at(10).is_nil());
    /// assert_eq!(array.length(), 2);
    /// ```
    pub fn delete_at(&mut self, index: i64) -> AnyObject {
        AnyObject::from(array::delete_at(self.value(), index))
    }

    /// Returns `true` if an element is equal (`==`) to `item` (Ruby's
    /// `include?`, `rb_ary_includes`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, RString, VM};
    /// # VM::init();
    ///
    /// let array = Array::new().push(Fixnum::new(1)).push(RString::new_utf8("two"));
    ///
    /// assert!(array.includes(&RString::new_utf8("two")));
    /// assert!(!array.includes(&Fixnum::new(2)));
    /// ```
    pub fn includes<T: Object>(&self, item: &T) -> bool {
        array::includes(self.value(), item.value())
    }

    /// Removes every element (Ruby's `clear`, `rb_ary_clear`).
    ///
    /// Ruby raises `FrozenError` if the array is frozen; check
    /// [`is_frozen`](trait.Object.html#method.is_frozen) first or call it inside
    /// [`VM::protect`](struct.VM.html#method.protect) when that is possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, VM};
    /// # VM::init();
    ///
    /// let mut array = Array::new().push(Fixnum::new(1));
    /// array.clear();
    ///
    /// assert_eq!(array.length(), 0);
    /// ```
    pub fn clear(&mut self) {
        array::clear(self.value())
    }

    /// Returns up to `len` elements starting at `start` as a new array, or
    /// `None` if `start` is past the end (Ruby's `ary[start, len]`,
    /// `rb_ary_subseq`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let array: Array = (1..=5).map(|i| Fixnum::new(i).to_any_object()).collect();
    ///
    /// let middle = array.slice(1, 3).unwrap();
    /// assert_eq!(middle.length(), 3);
    /// assert_eq!(middle.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    ///
    /// assert_eq!(array.slice(4, 10).unwrap().length(), 1);
    /// assert_eq!(array.slice(5, 1).unwrap().length(), 0);
    /// assert!(array.slice(6, 1).is_none());
    /// ```
    pub fn slice(&self, start: usize, len: usize) -> Option<Array> {
        let result = array::subseq(self.value(), start, len);

        if result.is_nil() {
            None
        } else {
            Some(Array::from(result))
        }
    }

    /// Returns a new array with the elements of this one followed by
    /// `other`'s (Ruby's `+`, `rb_ary_plus`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, VM};
    /// # VM::init();
    ///
    /// let a = Array::new().push(Fixnum::new(1));
    /// let b = Array::new().push(Fixnum::new(2));
    ///
    /// assert_eq!(a.plus(&b).length(), 2);
    /// assert_eq!(a.length(), 1);
    /// ```
    pub fn plus(&self, other: &Array) -> Array {
        Array::from(array::plus(self.value(), other.value()))
    }

    /// Compares two arrays element by element like Ruby's `<=>`
    /// (`rb_ary_cmp`); `None` when elements cannot be compared.
    ///
    /// Raises whatever an element's `<=>` raises.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, RString, VM};
    /// use std::cmp::Ordering;
    /// # VM::init();
    ///
    /// let a = Array::new().push(Fixnum::new(1)).push(Fixnum::new(2));
    /// let b = Array::new().push(Fixnum::new(1)).push(Fixnum::new(3));
    ///
    /// assert_eq!(a.compare(&b), Some(Ordering::Less));
    /// assert_eq!(a.compare(&a), Some(Ordering::Equal));
    ///
    /// let text = Array::new().push(RString::new_utf8("x"));
    ///
    /// assert_eq!(a.compare(&text), None);
    /// ```
    pub fn compare(&self, other: &Array) -> Option<Ordering> {
        let result = array::compare(self.value(), other.value());

        if result.is_nil() {
            None
        } else {
            Some(Fixnum::from(result).to_i64().cmp(&0))
        }
    }

    /// Replaces the contents of this array with `other`'s (Ruby's
    /// `replace`, `rb_ary_replace`).
    ///
    /// Ruby raises `FrozenError` if the array is frozen; check
    /// [`is_frozen`](trait.Object.html#method.is_frozen) first or call it inside
    /// [`VM::protect`](struct.VM.html#method.protect) when that is possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, VM};
    /// # VM::init();
    ///
    /// let mut array = Array::new().push(Fixnum::new(1));
    /// array.replace(&Array::new().push(Fixnum::new(2)).push(Fixnum::new(3)));
    ///
    /// assert_eq!(array.length(), 2);
    /// ```
    pub fn replace(&mut self, other: &Array) {
        array::replace(self.value(), other.value());
    }

    /// Truncates the array to `len` elements, or extends it with `nil`s
    /// (`rb_ary_resize`).
    ///
    /// Ruby raises `FrozenError` if the array is frozen; check
    /// [`is_frozen`](trait.Object.html#method.is_frozen) first or call it inside
    /// [`VM::protect`](struct.VM.html#method.protect) when that is possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let mut array = Array::new().push(Fixnum::new(1)).push(Fixnum::new(2));
    ///
    /// array.resize(4);
    /// assert_eq!(array.length(), 4);
    /// assert!(array.at(3).is_nil());
    ///
    /// array.resize(1);
    /// assert_eq!(array.length(), 1);
    /// ```
    pub fn resize(&mut self, len: usize) {
        array::resize(self.value(), len);
    }

    /// Rotates the elements in place so the one at `count` comes first
    /// (backwards when negative) and returns the array (Ruby's `rotate!`,
    /// `rb_ary_rotate`).
    ///
    /// Ruby raises `FrozenError` if the array is frozen; check
    /// [`is_frozen`](trait.Object.html#method.is_frozen) first or call it inside
    /// [`VM::protect`](struct.VM.html#method.protect) when that is possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let mut array: Array = (1..=4).map(|i| Fixnum::new(i).to_any_object()).collect();
    ///
    /// array.rotate_bang(1);
    /// assert_eq!(array.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    ///
    /// array.rotate_bang(-1);
    /// assert_eq!(array.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// ```
    pub fn rotate_bang(&mut self, count: i64) -> Array {
        array::rotate(self.value(), count);

        Array::from(self.value())
    }

    /// Returns the first element that is an array whose first element is
    /// equal to `key` (Ruby's `assoc`, `rb_ary_assoc`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let pairs = VM::eval("[[:a, 1], [:b, 2]]").unwrap().try_convert_to::<Array>().unwrap();
    ///
    /// let pair = pairs.assoc(&Symbol::new("b")).unwrap();
    ///
    /// assert_eq!(pair.at(1).try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// assert!(pairs.assoc(&Symbol::new("c")).is_none());
    /// ```
    pub fn assoc<T: Object>(&self, key: &T) -> Option<Array> {
        let result = array::assoc(self.value(), key.value());

        if result.is_nil() {
            None
        } else {
            Some(Array::from(result))
        }
    }

    /// Returns the first element that is an array whose second element is
    /// equal to `value` (Ruby's `rassoc`, `rb_ary_rassoc`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let pairs = VM::eval("[[:a, 1], [:b, 2]]").unwrap().try_convert_to::<Array>().unwrap();
    ///
    /// let pair = pairs.rassoc(&Fixnum::new(1)).unwrap();
    ///
    /// assert_eq!(pair.at(0).try_convert_to::<Symbol>(), Ok(Symbol::new("a")));
    /// assert!(pairs.rassoc(&Fixnum::new(3)).is_none());
    /// ```
    pub fn rassoc<T: Object>(&self, value: &T) -> Option<Array> {
        let result = array::rassoc(self.value(), value.value());

        if result.is_nil() {
            None
        } else {
            Some(Array::from(result))
        }
    }
}

/// Implicit or `nil` conversion, like Ruby's `Array.try_convert`
/// (`rb_check_array_type`): arrays and objects with `to_ary` convert,
/// anything else is `Err(nil)`.
///
/// # Examples
///
/// ```
/// use rutie::{Array, Fixnum, NilClass, Object, TryConvert, VM};
/// # VM::init();
///
/// let array = Array::new().push(Fixnum::new(1));
///
/// assert_eq!(Array::try_convert(array.to_any_object()).unwrap().length(), 1);
/// assert_eq!(Array::try_convert(Fixnum::new(1).to_any_object()), Err(NilClass::new()));
/// ```
impl TryConvert<AnyObject> for Array {
    type Nil = NilClass;

    fn try_convert(obj: AnyObject) -> Result<Self, NilClass> {
        let result = array::check_array_type(obj.value());

        if result.is_nil() {
            Err(NilClass::from(result))
        } else {
            Ok(Self::from(result))
        }
    }
}

impl Default for Array {
    fn default() -> Self {
        Array::new()
    }
}

impl From<Value> for Array {
    fn from(value: Value) -> Self {
        Array { value }
    }
}

impl Into<Value> for Array {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Array {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for Array {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Array {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.value().ty() == ValueType::Array
    }

    fn error_message() -> &'static str {
        "Error converting to Array"
    }
}

impl PartialEq for Array {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

pub struct ArrayIterator {
    array: Array,
    current_index: i64,
}

impl ArrayIterator {
    fn new(array: Array) -> ArrayIterator {
        ArrayIterator {
            array,
            current_index: 0,
        }
    }
}

impl Iterator for ArrayIterator {
    type Item = AnyObject;

    fn next(&mut self) -> Option<AnyObject> {
        let item = if (self.current_index as usize) < self.len() {
            Some(self.array.at(self.current_index))
        } else {
            None
        };

        self.current_index += 1;

        item
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let total = self.len() as usize;
        (total, Some(total))
    }
}

impl ExactSizeIterator for ArrayIterator {
    fn len(&self) -> usize {
        self.array.length() as usize
    }
}

/// Allows Arrays to be iterable in Rust.
///
/// # Examples
///
/// ```
/// use rutie::{Array, Fixnum, Object, VM};
/// # VM::init();
///
/// let mut array = Array::new()
///     .push(Fixnum::new(1))
///     .push(Fixnum::new(2))
///     .push(Fixnum::new(3));
///
/// let mut sum: i64 = 0;
///
/// for item in array.into_iter() {
///     sum += item.try_convert_to::<Fixnum>().unwrap().to_i64();
/// }
///
/// assert_eq!(sum, 6);
/// ```
impl IntoIterator for Array {
    type Item = AnyObject;
    type IntoIter = ArrayIterator;

    fn into_iter(self) -> Self::IntoIter {
        ArrayIterator::new(self)
    }
}

/// Converts an iterator into `Array`.
///
/// # Examples
///
/// ```
/// use rutie::{Array, Fixnum, Object, VM};
/// # VM::init();
///
/// let array: Array = (1..6)
///     .map(|num| num * 2)
///     .map(|num| Fixnum::new(num).to_any_object())
///     .collect();
///
/// assert_eq!(array.length(), 5);
///
/// for i in 0..5 {
///     let expected_number = (i + 1) * 2;
///
///     assert_eq!(array.at(i).try_convert_to::<Fixnum>().unwrap().to_i64(), expected_number);
/// }
/// ```
impl FromIterator<AnyObject> for Array {
    fn from_iter<I: IntoIterator<Item = AnyObject>>(iter: I) -> Self {
        let mut array = Array::new();

        for i in iter {
            array.push(i);
        }

        array
    }
}

#[cfg(test)]
mod tests {
    use crate::{Array, Fixnum, NilClass, Object, RString, Symbol, TryConvert, VM};
    use std::cmp::Ordering;

    fn fixnums(values: &[i64]) -> Array {
        values
            .iter()
            .map(|&value| Fixnum::new(value).to_any_object())
            .collect()
    }

    #[test]
    fn test_array_editing() {
        crate::on_ruby_thread(|| {
            let mut array = fixnums(&[1, 2, 3, 2]);

            array.delete(Fixnum::new(2));
            assert_eq!(array, fixnums(&[1, 3]));
            assert!(array.includes(&Fixnum::new(3)));

            array.resize(3);
            assert!(array.at(2).is_nil());
            array.delete_at(2);
            assert_eq!(array, fixnums(&[1, 3]));

            array.replace(&fixnums(&[4, 5, 6]));
            array.rotate_bang(2);
            assert_eq!(array, fixnums(&[6, 4, 5]));

            assert_eq!(array.plus(&fixnums(&[7])).length(), 4);
            assert_eq!(array.slice(1, 5).unwrap(), fixnums(&[4, 5]));

            array.clear();
            assert_eq!(array.length(), 0);

            let mut frozen = fixnums(&[1]).freeze();
            assert!(VM::protect(|| {
                frozen.clear();
                NilClass::new().into()
            })
            .is_err());
            VM::error_pop().unwrap();
        });
    }

    #[test]
    fn test_array_comparison_and_lookup() {
        crate::on_ruby_thread(|| {
            assert_eq!(
                fixnums(&[1, 2]).compare(&fixnums(&[1, 2, 3])),
                Some(Ordering::Less)
            );
            assert_eq!(
                fixnums(&[2]).compare(&fixnums(&[1, 9])),
                Some(Ordering::Greater)
            );

            let pairs = VM::eval("[[1, :one], [2, :two], 3]")
                .unwrap()
                .try_convert_to::<Array>()
                .unwrap();
            assert_eq!(
                pairs
                    .assoc(&Fixnum::new(2))
                    .unwrap()
                    .at(1)
                    .try_convert_to::<Symbol>(),
                Ok(Symbol::new("two"))
            );
            assert_eq!(
                pairs
                    .rassoc(&Symbol::new("one"))
                    .unwrap()
                    .at(0)
                    .try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(1))
            );

            let convertible = VM::eval("o = Object.new; def o.to_ary; [1, 2]; end; o").unwrap();
            assert_eq!(Array::try_convert(convertible).unwrap(), fixnums(&[1, 2]));
            assert!(Array::try_convert(RString::new_utf8("x").to_any_object()).is_err());
        });
    }

    #[test]
    fn test_array_stack_and_order_operations() {
        crate::on_ruby_thread(|| {
            let fixnums = |array: &Array| -> Vec<i64> {
                (0..array.length() as i64)
                    .map(|i| array.at(i).try_convert_to::<Fixnum>().unwrap().to_i64())
                    .collect()
            };

            let mut array: Array = [3, 1, 2]
                .iter()
                .map(|&n| Fixnum::new(n).to_any_object())
                .collect();

            array.unshift(Fixnum::new(0));
            assert_eq!(fixnums(&array), vec![0, 3, 1, 2]);
            assert_eq!(array.shift().try_convert_to::<Fixnum>(), Ok(Fixnum::new(0)));
            assert_eq!(array.pop().try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
            assert_eq!(fixnums(&array), vec![3, 1]);

            let mut empty = Array::new();
            assert!(empty.pop().is_nil());
            assert!(empty.shift().is_nil());

            let mut array: Array = [3, 1, 2]
                .iter()
                .map(|&n| Fixnum::new(n).to_any_object())
                .collect();
            assert_eq!(fixnums(&array.sort()), vec![1, 2, 3]);
            assert_eq!(fixnums(&array), vec![3, 1, 2]);
            assert_eq!(fixnums(&array.reverse()), vec![2, 1, 3]);
            assert_eq!(fixnums(&array), vec![3, 1, 2]);

            array.sort_bang();
            assert_eq!(fixnums(&array), vec![1, 2, 3]);
            array.reverse_bang();
            assert_eq!(fixnums(&array), vec![3, 2, 1]);

            let mut enumerator = array.to_enum();
            assert_eq!(
                enumerator.next().unwrap().try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(3))
            );

            // Mutating a frozen array raises.
            let result = VM::protect(|| {
                let mut frozen = Array::new().freeze();
                frozen.push(NilClass::new());
                NilClass::new().into()
            });
            assert!(result.is_err());
            VM::clear_error_info();
        });
    }
}
