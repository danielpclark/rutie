use std::{convert::From, default::Default};

use crate::{
    binding::hash,
    types::{Value, ValueType},
    AnyObject, Object, VerifiedObject,
};

/// `Hash`
#[derive(Debug)]
#[repr(C)]
pub struct Hash {
    value: Value,
}

impl Hash {
    /// Creates a new instance of empty `Hash`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Hash, VM};
    /// # VM::init();
    ///
    /// Hash::new();
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// {}
    /// ```
    pub fn new() -> Self {
        Self::from(hash::new())
    }

    /// Retrieves an `AnyObject` from element stored at `key` key.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Hash, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let mut hash = Hash::new();
    ///
    /// hash.store(Symbol::new("key"), Fixnum::new(1));
    ///
    /// assert_eq!(hash.at(&Symbol::new("key")).try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// hash = {}
    /// hash[:key] = 1
    ///
    /// hash[:key] == 1
    /// ```
    pub fn at<T: Object>(&self, key: &T) -> AnyObject {
        let result = hash::aref(self.value(), key.value());

        AnyObject::from(result)
    }

    /// Associates the `value` with the `key`.
    ///
    /// Both `key` and `value` must be types which implement `Object` trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Hash, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let mut hash = Hash::new();
    ///
    /// hash.store(Symbol::new("key"), Fixnum::new(1));
    ///
    /// assert_eq!(hash.at(&Symbol::new("key")).try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// hash = {}
    /// hash[:key] = 1
    ///
    /// hash[:key] == 1
    /// ```
    pub fn store<K: Object, V: Object>(&mut self, key: K, value: V) -> AnyObject {
        let result = hash::aset(self.value(), key.value(), value.value());

        AnyObject::from(result)
    }

    /// Retrieves the length of the hash.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Hash, Fixnum, Symbol, VM};
    /// # VM::init();
    ///
    /// let mut hash = Hash::new();
    ///
    /// hash.store(Symbol::new("key1"), Fixnum::new(1));
    /// assert_eq!(hash.length(), 1);
    ///
    /// hash.store(Symbol::new("key2"), Fixnum::new(2));
    /// assert_eq!(hash.length(), 2);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// hash = {}
    ///
    /// hash[:key1] = 1
    /// hash.length == 1
    ///
    /// hash[:key2] = 2
    /// hash.length == 2
    /// ```
    pub fn length(&self) -> usize {
        hash::length(self.value()) as usize
    }

    /// Removes all key-value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Hash, Fixnum, Symbol, VM};
    /// # VM::init();
    ///
    /// let mut hash = Hash::new();
    ///
    /// hash.store(Symbol::new("key1"), Fixnum::new(1));
    /// hash.store(Symbol::new("key2"), Fixnum::new(2));
    /// assert_eq!(hash.length(), 2);
    ///
    /// hash.clear();
    /// assert_eq!(hash.length(), 0);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// hash = {}
    ///
    /// hash[:key1] = 1
    /// hash[:key2] = 2
    /// hash.length == 2
    ///
    /// hash.clear
    ///
    /// hash.length == 0
    /// ```
    pub fn clear(&self) {
        hash::clear(self.value())
    }

    /// Deletes the key-value pair and returns the value from hash whose key is equal to key. If
    /// the key is not found, it returns nil.
    ///
    /// `key` must be a type which implements the `Object` trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Hash, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let mut hash = Hash::new();
    ///
    /// hash.store(Symbol::new("key1"), Fixnum::new(1));
    /// hash.store(Symbol::new("key2"), Fixnum::new(2));
    /// assert_eq!(hash.length(), 2);
    ///
    /// let deleted = hash.delete(Symbol::new("key2"));
    /// assert_eq!(hash.length(), 1);
    /// assert_eq!(deleted.try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// hash = {}
    ///
    /// hash[:key1] = 1
    /// hash[:key2] = 2
    /// hash.length == 2
    ///
    /// deleted = hash.delete(:key2)
    ///
    /// hash.length == 1
    /// deleted == 2
    /// ```
    pub fn delete<K: Object>(&mut self, key: K) -> AnyObject {
        let result = hash::delete(self.value(), key.value());

        AnyObject::from(result)
    }

    /// Runs a closure for each `key` and `value` pair.
    ///
    /// Key and value have `AnyObject` type.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Hash, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let mut hash = Hash::new();
    ///
    /// hash.store(Symbol::new("first_key"), Fixnum::new(1));
    /// hash.store(Symbol::new("second_key"), Fixnum::new(2));
    ///
    /// let mut doubled_values: Vec<i64> = Vec::new();
    ///
    /// hash.each(|_key, value| {
    ///     if let Ok(value) = value.try_convert_to::<Fixnum>() {
    ///         doubled_values.push(value.to_i64() * 2);
    ///     }
    /// });
    ///
    /// assert_eq!(doubled_values, vec![2, 4]);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// hash = {
    ///   first_key: 1,
    ///   second_key: 2
    /// }
    ///
    /// doubled_values = []
    ///
    /// hash.each do |_key, value|
    ///   doubled_values << [value * 2]
    /// end
    ///
    /// doubled_values == [2, 4]
    /// ```
    pub fn each<F>(&self, closure: F)
    where
        F: FnMut(AnyObject, AnyObject),
    {
        hash::each(self.value(), closure);
    }
}

impl Clone for Hash {
    fn clone(&self) -> Hash {
        Hash {
            value: hash::dup(self.value()),
        }
    }
}

impl Default for Hash {
    fn default() -> Self {
        Hash::new()
    }
}

impl From<Value> for Hash {
    fn from(value: Value) -> Self {
        Hash { value }
    }
}

impl From<Hash> for Value {
    fn from(val: Hash) -> Self {
        val.value
    }
}

impl From<Hash> for AnyObject {
    fn from(val: Hash) -> Self {
        AnyObject::from(val.value)
    }
}

impl Object for Hash {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Hash {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.value().ty() == ValueType::Hash
    }

    fn error_message() -> &'static str {
        "Error converting to Hash"
    }
}

impl PartialEq for Hash {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::{Fixnum, Hash, Object, Symbol};
    use rb_sys_test_helpers::ruby_test;

    #[ruby_test]
    fn test_hash_each() {
        let mut hash = Hash::new();

        let len: i64 = 200;

        for i in 0..len {
            hash.store(Symbol::new(&format!("key_{}", i)), Fixnum::new(i));
        }

        assert_eq!(hash.length(), len as usize);

        let mut counter: i64 = 0;

        hash.each(|k, v| {
            assert_eq!(
                k.try_convert_to::<Symbol>().map(|s| s.to_string()),
                Ok(format!("key_{}", counter))
            );
            assert_eq!(
                v.try_convert_to::<Fixnum>().map(|f| f.to_i64()),
                Ok(counter)
            );

            counter += 1;
        });

        assert_eq!(counter, len);
    }
}
