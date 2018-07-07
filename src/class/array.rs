use std::convert::From;
use std::default::Default;
use std::iter::{FromIterator, IntoIterator, Iterator};

use binding::array;
use types::{Value, ValueType};

use {AnyObject, Object, RString, VerifiedObject};

/// `Array`
#[derive(Debug, PartialEq)]
pub struct Array {
    value: Value,
}

impl Array {
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
    ///     .push(RString::new("Hello"))
    ///     .push(RString::new("World!"));
    ///
    /// let joined_string = array.join(RString::new(", "));
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

    /// Inserts `item` at the beggining of the array.
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
}

impl Default for Array {
    fn default() -> Self {
        Array::new()
    }
}

impl From<Value> for Array {
    fn from(value: Value) -> Self {
        Array { value: value }
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

pub struct ArrayIterator {
    array: Array,
    current_index: i64,
}

impl ArrayIterator {
    fn new(array: Array) -> ArrayIterator {
        ArrayIterator {
            array: array,
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
