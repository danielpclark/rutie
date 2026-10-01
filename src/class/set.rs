use std::convert::From;

use crate::{binding::set, types::Value, AnyObject, Class, Object, VerifiedObject};

/// `Set`, which is part of Ruby's core from Ruby 4.0 and has a C API there.
#[derive(Debug)]
#[repr(C)]
pub struct Set {
    value: Value,
}

impl Set {
    /// Creates an empty `Set` (`rb_set_new`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Set, VM};
    /// # VM::init();
    ///
    /// let set = Set::new();
    ///
    /// assert_eq!(set.length(), 0);
    /// assert!(set.is_empty());
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// set = Set.new
    ///
    /// set.size == 0
    /// ```
    pub fn new() -> Self {
        Self::from(set::new())
    }

    /// Creates an empty `Set` with room for `capacity` elements before it
    /// grows (`rb_set_new_capa`). The capacity is only a hint.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Set, VM};
    /// # VM::init();
    ///
    /// let mut set = Set::with_capacity(100);
    /// assert_eq!(set.length(), 0);
    ///
    /// set.add(Fixnum::new(1));
    /// assert_eq!(set.length(), 1);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self::from(set::new_capa(capacity))
    }

    /// Adds `element`, returning `true` if it was not in the set already
    /// (`rb_set_add`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, Set, VM};
    /// # VM::init();
    ///
    /// let mut set = Set::new();
    ///
    /// assert!(set.add(RString::new_utf8("a")));
    /// assert!(!set.add(RString::new_utf8("a")));
    /// assert_eq!(set.length(), 1);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// set = Set.new
    ///
    /// set.add?("a") # => set
    /// set.add?("a") # => nil
    /// ```
    pub fn add<T: Object>(&mut self, element: T) -> bool {
        set::add(self.value(), element.value())
    }

    /// Returns `true` if `element` is in the set (`rb_set_lookup`, like
    /// `Set#include?`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Set, VM};
    /// # VM::init();
    ///
    /// let mut set = Set::new();
    /// set.add(Fixnum::new(1));
    ///
    /// assert!(set.contains(&Fixnum::new(1)));
    /// assert!(!set.contains(&Fixnum::new(2)));
    /// ```
    pub fn contains<T: Object>(&self, element: &T) -> bool {
        set::lookup(self.value(), element.value())
    }

    /// Removes `element`, returning `true` if it was in the set
    /// (`rb_set_delete`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Set, VM};
    /// # VM::init();
    ///
    /// let mut set = Set::new();
    /// set.add(Fixnum::new(1));
    ///
    /// assert!(set.delete(&Fixnum::new(1)));
    /// assert!(!set.delete(&Fixnum::new(1)));
    /// assert!(set.is_empty());
    /// ```
    pub fn delete<T: Object>(&mut self, element: &T) -> bool {
        set::delete(self.value(), element.value())
    }

    /// Removes every element (`rb_set_clear`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Set, VM};
    /// # VM::init();
    ///
    /// let mut set = Set::new();
    /// set.add(Fixnum::new(1));
    /// set.add(Fixnum::new(2));
    ///
    /// set.clear();
    ///
    /// assert!(set.is_empty());
    /// ```
    pub fn clear(&mut self) {
        set::clear(self.value());
    }

    /// Returns the number of elements (`rb_set_size`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Set, VM};
    /// # VM::init();
    ///
    /// let set = VM::eval("Set[1, 2, 3]").unwrap().try_convert_to::<Set>().unwrap();
    ///
    /// assert_eq!(set.length(), 3);
    /// ```
    pub fn length(&self) -> usize {
        set::size(self.value())
    }

    /// Returns `true` if the set has no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Set, VM};
    /// # VM::init();
    ///
    /// let mut set = Set::new();
    /// assert!(set.is_empty());
    ///
    /// set.add(Fixnum::new(1));
    /// assert!(!set.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.length() == 0
    }

    /// Calls `func` with each element, in insertion order (`rb_set_foreach`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Set, VM};
    /// # VM::init();
    ///
    /// let set = VM::eval("Set[1, 2, 3]").unwrap().try_convert_to::<Set>().unwrap();
    /// let mut sum = 0;
    ///
    /// set.each(|element| {
    ///     sum += element.try_convert_to::<Fixnum>().unwrap().to_i64();
    /// });
    ///
    /// assert_eq!(sum, 6);
    /// ```
    pub fn each<F>(&self, mut func: F)
    where
        F: FnMut(AnyObject),
    {
        set::each(self.value(), |element| func(AnyObject::from(element)));
    }

    /// Returns the elements as a `Vec`, in insertion order.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Set, VM};
    /// # VM::init();
    ///
    /// let set = VM::eval("Set[3, 1, 2]").unwrap().try_convert_to::<Set>().unwrap();
    /// let elements: Vec<i64> = set
    ///     .to_vec()
    ///     .iter()
    ///     .map(|element| element.try_convert_to::<Fixnum>().unwrap().to_i64())
    ///     .collect();
    ///
    /// assert_eq!(elements, vec![3, 1, 2]);
    /// ```
    pub fn to_vec(&self) -> Vec<AnyObject> {
        let mut elements = Vec::with_capacity(self.length());

        self.each(|element| elements.push(element));

        elements
    }
}

impl Default for Set {
    fn default() -> Self {
        Set::new()
    }
}

impl From<Value> for Set {
    fn from(value: Value) -> Self {
        Set { value }
    }
}

impl From<Set> for Value {
    fn from(set: Set) -> Self {
        set.value
    }
}

impl From<Set> for AnyObject {
    fn from(set: Set) -> Self {
        AnyObject::from(set.value)
    }
}

impl Object for Set {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Set {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.is_kind_of(&Class::set())
    }

    fn error_message() -> &'static str {
        "Error converting to Set"
    }
}

impl PartialEq for Set {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Fixnum, Object, RString, Set, VM};

    #[test]
    fn test_set_api() {
        crate::on_ruby_thread(|| {
            let mut set = Set::with_capacity(4);
            assert!(set.is_empty());

            assert!(set.add(Fixnum::new(1)));
            assert!(set.add(RString::new_utf8("two")));
            assert!(!set.add(Fixnum::new(1)));
            assert_eq!(set.length(), 2);
            assert!(set.contains(&RString::new_utf8("two")));

            // The same set Ruby sees.
            let ruby_size = unsafe { set.send("size", &[]) };
            assert_eq!(ruby_size.try_convert_to::<Fixnum>().unwrap().to_i64(), 2);

            let mut seen = Vec::new();
            set.each(|element| seen.push(element.class().name().unwrap().to_string()));
            assert_eq!(seen, vec!["Integer".to_string(), "String".to_string()]);

            assert!(set.delete(&Fixnum::new(1)));
            assert_eq!(set.to_vec().len(), 1);

            set.clear();
            assert!(set.is_empty());

            // Only sets (and subclasses) convert.
            let subclass = VM::eval("Class.new(Set).new([1])").unwrap();
            assert_eq!(subclass.try_convert_to::<Set>().unwrap().length(), 1);
            assert!(VM::eval("[1]").unwrap().try_convert_to::<Set>().is_err());
        });
    }
}
