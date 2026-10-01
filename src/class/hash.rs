use std::{convert::From, default::Default};

use crate::{
    binding::{exception, hash, vm},
    types::{Value, ValueType},
    AnyException, AnyObject, Array, NilClass, Object, TryConvert, VerifiedObject,
};

/// `Hash`
#[derive(Debug)]
#[repr(C)]
pub struct Hash {
    value: Value,
}

impl Hash {
    /// Converts `object` to a `Hash` the way Ruby's `Hash(object)`
    /// (`Kernel#Hash`, `rb_Hash`) does, calling `to_hash`; `nil` and `[]` become an empty hash. Returns the exception when
    /// it cannot be converted.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Hash, NilClass, Object, VM};
    /// # VM::init();
    ///
    /// let empty = Hash::convert(&NilClass::new()).unwrap();
    /// assert_eq!(empty.length(), 0);
    ///
    /// let hash = VM::eval("{ a: 1 }").unwrap();
    /// assert_eq!(Hash::convert(&hash).unwrap().length(), 1);
    ///
    /// // Only `to_hash` is used, so an array of pairs is not converted.
    /// assert!(Hash::convert(&VM::eval("[[:a, 1]]").unwrap()).is_err());
    ///
    /// assert!(Hash::convert(&Fixnum::new(1)).is_err());
    /// ```
    pub fn convert<T: Object>(object: &T) -> Result<Self, AnyException> {
        let object = object.value();

        crate::binding::vm::protect_value(|| crate::binding::object::to_hash(object))
            .map(Self::from)
            .map_err(AnyException::from)
    }

    /// Creates a new instance of empty `Hash`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Hash, VM};
    /// # VM::init();
    ///
    /// let hash = Hash::new();
    ///
    /// assert_eq!(hash.length(), 0);
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

    /// Creates an empty `Hash` with room for `capacity` entries before it
    /// grows (`rb_hash_new_capa`).
    ///
    /// The capacity is a hint; the `Hash` is empty either way.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Hash, Object, VM};
    /// # VM::init();
    ///
    /// let mut hash = Hash::with_capacity(100);
    /// assert_eq!(hash.length(), 0);
    ///
    /// for i in 0..100 {
    ///     hash.store(Fixnum::new(i), Fixnum::new(i * i));
    /// }
    ///
    /// assert_eq!(hash.length(), 100);
    /// assert_eq!(hash.at(&Fixnum::new(9)), Fixnum::new(81).into());
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// Hash.new(capacity: 100)
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self::from(hash::with_capacity(capacity))
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
    /// Ruby raises `FrozenError` if the hash is frozen; check
    /// [`is_frozen`](trait.Object.html#method.is_frozen) first or call it inside
    /// [`VM::protect`](struct.VM.html#method.protect) when that is possible.
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
    /// Ruby raises `FrozenError` if the hash is frozen; check
    /// [`is_frozen`](trait.Object.html#method.is_frozen) first or call it inside
    /// [`VM::protect`](struct.VM.html#method.protect) when that is possible.
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
    /// Ruby raises `FrozenError` if the hash is frozen; check
    /// [`is_frozen`](trait.Object.html#method.is_frozen) first or call it inside
    /// [`VM::protect`](struct.VM.html#method.protect) when that is possible.
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

    /// Returns the value stored for `key`, or `None` if there is no such
    /// key. Unlike [`at`](#method.at), the hash's default is not used
    /// (`rb_hash_lookup2`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Hash, NilClass, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let mut hash = VM::eval("Hash.new(0)").unwrap().try_convert_to::<Hash>().unwrap();
    /// hash.store(Symbol::new("present"), NilClass::new());
    ///
    /// assert!(hash.lookup(&Symbol::new("present")).unwrap().is_nil());
    /// assert!(hash.lookup(&Symbol::new("missing")).is_none());
    ///
    /// // `at` would return the default instead:
    /// assert_eq!(hash.at(&Symbol::new("missing")).try_convert_to::<Fixnum>(), Ok(Fixnum::new(0)));
    /// ```
    pub fn lookup<T: Object>(&self, key: &T) -> Option<AnyObject> {
        hash::lookup(self.value(), key.value()).map(AnyObject::from)
    }

    /// Returns `true` if the hash has `key` (Ruby's `key?`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Hash, Symbol, VM};
    /// # VM::init();
    ///
    /// let mut hash = Hash::new();
    /// hash.store(Symbol::new("a"), Fixnum::new(1));
    ///
    /// assert!(hash.has_key(&Symbol::new("a")));
    /// assert!(!hash.has_key(&Symbol::new("b")));
    /// ```
    pub fn has_key<T: Object>(&self, key: &T) -> bool {
        hash::lookup(self.value(), key.value()).is_some()
    }

    /// Returns the value for `key`, or the `KeyError` if there is no such
    /// key (Ruby's `fetch`, `rb_hash_fetch`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Fixnum, Hash, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let mut hash = Hash::new();
    /// hash.store(Symbol::new("a"), Fixnum::new(1));
    ///
    /// assert_eq!(hash.fetch(&Symbol::new("a")).unwrap().try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    ///
    /// let error = hash.fetch(&Symbol::new("b")).unwrap_err();
    ///
    /// assert!(Class::from_existing("KeyError").case_equals(&error));
    /// ```
    pub fn fetch<T: Object>(&self, key: &T) -> Result<AnyObject, AnyException> {
        let (hash, key) = (self.value(), key.value());

        vm::protect_value(|| hash::fetch(hash, key))
            .map(AnyObject::from)
            .map_err(AnyException::from)
    }

    /// Returns the keys in insertion order (Ruby's `keys`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Hash, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let mut hash = Hash::new();
    /// hash.store(Symbol::new("a"), Fixnum::new(1));
    /// hash.store(Symbol::new("b"), Fixnum::new(2));
    ///
    /// let keys = hash.keys();
    ///
    /// assert_eq!(keys.length(), 2);
    /// assert_eq!(keys.at(1).try_convert_to::<Symbol>(), Ok(Symbol::new("b")));
    /// ```
    pub fn keys(&self) -> Array {
        let mut keys = Array::with_capacity(self.length());

        self.each(|key, _| {
            keys.push(key);
        });

        keys
    }

    /// Returns the values in insertion order (Ruby's `values`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Hash, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let mut hash = Hash::new();
    /// hash.store(Symbol::new("a"), Fixnum::new(1));
    /// hash.store(Symbol::new("b"), Fixnum::new(2));
    ///
    /// let values = hash.values();
    ///
    /// assert_eq!(values.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// ```
    pub fn values(&self) -> Array {
        let mut values = Array::with_capacity(self.length());

        self.each(|_, value| {
            values.push(value);
        });

        values
    }

    /// Adds every pair of `other` to this hash, replacing existing keys'
    /// values (Ruby's `update`/`merge!`, `rb_hash_update_by`).
    ///
    /// Ruby raises `FrozenError` if the hash is frozen; check
    /// [`is_frozen`](trait.Object.html#method.is_frozen) first or call it inside
    /// [`VM::protect`](struct.VM.html#method.protect) when that is possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Hash, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let mut hash = Hash::new();
    /// hash.store(Symbol::new("a"), Fixnum::new(1));
    ///
    /// let mut other = Hash::new();
    /// other.store(Symbol::new("a"), Fixnum::new(10));
    /// other.store(Symbol::new("b"), Fixnum::new(2));
    ///
    /// hash.update(&other);
    ///
    /// assert_eq!(hash.length(), 2);
    /// assert_eq!(hash.at(&Symbol::new("a")).try_convert_to::<Fixnum>(), Ok(Fixnum::new(10)));
    /// ```
    pub fn update(&mut self, other: &Hash) {
        hash::update(self.value(), other.value());
    }

    /// Stores all key, value `pairs` in the hash in one step
    /// (`rb_hash_bulk_insert`), or returns the `FrozenError` if the hash is
    /// frozen. Like [`store`](#method.store), an unfrozen `String` key is
    /// stored as a frozen copy.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{AnyObject, Exception, Fixnum, Hash, Object, RString, Symbol, VM};
    /// # VM::init();
    ///
    /// let mut hash = Hash::new();
    ///
    /// hash.bulk_insert(&[
    ///     (Symbol::new("a").into(), Fixnum::new(1).into()),
    ///     (RString::new_utf8("b").into(), Fixnum::new(2).into()),
    /// ])
    /// .unwrap();
    ///
    /// assert_eq!(hash.length(), 2);
    /// assert_eq!(hash.at(&RString::new_utf8("b")).try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    ///
    /// hash.freeze();
    ///
    /// let error = hash.bulk_insert(&[(Symbol::new("c").into(), Fixnum::new(3).into())]).unwrap_err();
    ///
    /// assert!(error.message().starts_with("can't modify frozen Hash"));
    /// ```
    pub fn bulk_insert(&mut self, pairs: &[(AnyObject, AnyObject)]) -> Result<(), AnyException> {
        let hash = self.value();
        let keys_and_values: Vec<Value> = pairs
            .iter()
            .flat_map(|(key, value)| [key.value(), value.value()])
            .collect();

        vm::protect_value(|| {
            exception::check_frozen(hash);
            hash::bulk_insert(hash, &keys_and_values);

            NilClass::new().value()
        })
        .map(|_| ())
        .map_err(AnyException::from)
    }

    /// Splits the hash into a new hash of its `Symbol`-keyed entries (the
    /// ones that can be keyword arguments) and a new hash of the others
    /// (`rb_extract_keywords`). The hash itself is unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Hash, Object, RString, Symbol, VM};
    /// # VM::init();
    ///
    /// let options = VM::eval("{ verbose: true, 'path' => '/tmp', level: 2 }")
    ///     .unwrap()
    ///     .try_convert_to::<Hash>()
    ///     .unwrap();
    ///
    /// let (keywords, rest) = options.split_keywords();
    ///
    /// assert_eq!(keywords.length(), 2);
    /// assert_eq!(keywords.at(&Symbol::new("level")).try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// assert_eq!(rest.length(), 1);
    /// assert!(!rest.at(&RString::new_utf8("path")).is_nil());
    /// assert_eq!(options.length(), 3);
    /// ```
    pub fn split_keywords(&self) -> (Hash, Hash) {
        let (keywords, rest) = hash::extract_keywords(self.value());

        (
            keywords.map(Hash::from).unwrap_or_default(),
            rest.map(Hash::from).unwrap_or_default(),
        )
    }

    /// Sets the value returned for missing keys (Ruby's `default=`),
    /// replacing any default proc.
    ///
    /// Ruby raises `FrozenError` if the hash is frozen.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Hash, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let mut hash = Hash::new();
    /// hash.set_default(Fixnum::new(0));
    ///
    /// assert_eq!(hash.at(&Symbol::new("missing")).try_convert_to::<Fixnum>(), Ok(Fixnum::new(0)));
    /// ```
    pub fn set_default<T: Object>(&mut self, value: T) {
        // `rb_hash_set_ifnone` skips the frozen check and keeps a default
        // proc flag, so go through `default=`.
        unsafe { self.send("default=", &[value.to_any_object()]) };
    }

    /// Returns an iterator over `(key, value)` pairs in insertion order.
    ///
    /// The keys are taken when the iterator is created; values are looked
    /// up as it advances, and keys removed in the meantime are skipped.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Hash, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let mut hash = Hash::new();
    /// hash.store(Symbol::new("a"), Fixnum::new(1));
    /// hash.store(Symbol::new("b"), Fixnum::new(2));
    ///
    /// let sum: i64 = hash
    ///     .iter()
    ///     .map(|(_key, value)| value.try_convert_to::<Fixnum>().unwrap().to_i64())
    ///     .sum();
    ///
    /// assert_eq!(sum, 3);
    /// ```
    pub fn iter(&self) -> HashIterator {
        HashIterator {
            hash: Hash::from(self.value()),
            keys: self.keys(),
            index: 0,
        }
    }
}

/// Iterator over the `(key, value)` pairs of a [`Hash`](struct.Hash.html);
/// see [`Hash::iter`](struct.Hash.html#method.iter).
///
/// Keep it on the stack (as `for` loops and iterator chains do), where
/// Ruby's garbage collector can see the hash and key array it holds.
#[derive(Debug)]
pub struct HashIterator {
    hash: Hash,
    keys: Array,
    index: usize,
}

impl Iterator for HashIterator {
    type Item = (AnyObject, AnyObject);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.keys.length() {
            let key = self.keys.at(self.index as i64);
            self.index += 1;

            if let Some(value) = self.hash.lookup(&key) {
                return Some((key, value));
            }
        }

        None
    }
}

/// Implicit or `nil` conversion, like Ruby's `Hash.try_convert`
/// (`rb_check_hash_type`): hashes and objects with `to_hash` convert,
/// anything else is `Err(nil)`.
///
/// # Examples
///
/// ```
/// use rutie::{Array, Hash, NilClass, Object, TryConvert, VM};
/// # VM::init();
///
/// assert!(Hash::try_convert(Hash::new().to_any_object()).is_ok());
/// assert_eq!(Hash::try_convert(Array::new().to_any_object()), Err(NilClass::new()));
/// ```
impl TryConvert<AnyObject> for Hash {
    type Nil = NilClass;

    fn try_convert(obj: AnyObject) -> Result<Self, NilClass> {
        let result = hash::check_hash_type(obj.value());

        if result.is_nil() {
            Err(NilClass::from(result))
        } else {
            Ok(Self::from(result))
        }
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

impl Into<Value> for Hash {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Hash {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
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
    use super::super::super::{
        AnyObject, Fixnum, Hash, NilClass, Object, RString, Symbol, TryConvert, VM,
    };

    #[test]
    fn test_hash_each() {
        crate::on_ruby_thread(|| {
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
        });
    }

    #[test]
    fn test_hash_lookup_keys_and_update() {
        crate::on_ruby_thread(|| {
            let mut hash = VM::eval("Hash.new { |h, k| h[k] = :from_proc }")
                .unwrap()
                .try_convert_to::<Hash>()
                .unwrap();

            hash.store(Symbol::new("a"), Fixnum::new(1));

            // `lookup` and `has_key` never call the default proc.
            assert!(hash.lookup(&Symbol::new("zzz")).is_none());
            assert!(!hash.has_key(&Symbol::new("zzz")));
            assert_eq!(hash.length(), 1);
            assert!(hash.fetch(&Symbol::new("zzz")).is_err());

            let mut other = Hash::new();
            other.store(Symbol::new("b"), Fixnum::new(2));
            hash.update(&other);

            assert_eq!(hash.keys().length(), 2);
            assert_eq!(
                hash.values().at(1).try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(2))
            );

            // Replacing the default proc with a value.
            hash.set_default(Fixnum::new(0));
            assert_eq!(
                hash.at(&Symbol::new("missing")).try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(0))
            );
            assert_eq!(hash.length(), 2);

            let pairs: Vec<_> = hash.iter().collect();
            assert_eq!(pairs.len(), 2);
            assert_eq!(pairs[0].0.try_convert_to::<Symbol>(), Ok(Symbol::new("a")));

            // Keys removed while iterating are skipped.
            let mut iterator = hash.iter();
            hash.delete(Symbol::new("b"));
            assert_eq!(iterator.by_ref().count(), 1);

            let frozen = Hash::new().freeze();
            let result = VM::protect(|| {
                let mut frozen = Hash::from(frozen.value());
                frozen.set_default(Fixnum::new(1));
                NilClass::new().into()
            });
            assert!(result.is_err());
            VM::error_pop().unwrap();

            let convertible = VM::eval("o = Object.new; def o.to_hash; { x: 1 }; end; o").unwrap();
            assert_eq!(Hash::try_convert(convertible).unwrap().length(), 1);
        });
    }

    #[test]
    fn test_with_capacity() {
        crate::on_ruby_thread(|| {
            let mut hash = Hash::with_capacity(64);
            assert_eq!(hash.length(), 0);

            for i in 0..200 {
                hash.store(Fixnum::new(i), Fixnum::new(-i));
            }

            assert_eq!(hash.length(), 200);
            assert_eq!(hash.at(&Fixnum::new(150)), Fixnum::new(-150).into());
            assert_eq!(Hash::with_capacity(0).length(), 0);
            // Clamped to Ruby's `long`, not an overflow.
            assert_eq!(Hash::with_capacity(usize::MAX >> 40).length(), 0);
        });
    }

    #[test]
    fn test_bulk_insert_and_split_keywords() {
        crate::on_ruby_thread(|| {
            let mut hash = Hash::new();
            hash.bulk_insert(&[]).unwrap();
            assert_eq!(hash.length(), 0);

            // Large enough to switch from the array table to an st table.
            let pairs: Vec<(AnyObject, AnyObject)> = (0..40)
                .map(|i| (Fixnum::new(i).into(), Fixnum::new(i * i).into()))
                .collect();
            hash.bulk_insert(&pairs).unwrap();
            assert_eq!(hash.length(), 40);
            assert_eq!(
                hash.at(&Fixnum::new(7)).try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(49))
            );

            // String keys are stored as frozen copies; later pairs win.
            let mut key = RString::new_utf8("key");
            hash.bulk_insert(&[
                (key.to_any_object(), Fixnum::new(1).into()),
                (key.to_any_object(), Fixnum::new(2).into()),
            ])
            .unwrap();
            key.concat("-changed");
            assert_eq!(
                hash.at(&RString::new_utf8("key"))
                    .try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(2))
            );
            let stored = VM::eval("->(h) { h.keys.last }")
                .unwrap()
                .try_convert_to::<crate::Proc>()
                .unwrap()
                .call(&[hash.to_any_object()]);
            assert!(stored.is_frozen());

            let (keywords, rest) = hash.split_keywords();
            assert_eq!((keywords.length(), rest.length()), (0, 41));

            let only_symbols = VM::eval("{ a: 1, b: 2 }")
                .unwrap()
                .try_convert_to::<Hash>()
                .unwrap();
            let (keywords, rest) = only_symbols.split_keywords();
            assert_eq!((keywords.length(), rest.length()), (2, 0));

            let (keywords, rest) = Hash::new().split_keywords();
            assert_eq!((keywords.length(), rest.length()), (0, 0));
        });
    }
}
