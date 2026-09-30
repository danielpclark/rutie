use crate::{
    binding::{io, vm},
    AnyException, AnyObject, Object, RString,
};

/// Ruby's `Marshal`: serializes objects to bytes and back.
///
/// **Never load data from an untrusted source**: loading can instantiate
/// arbitrary classes and run their hooks, just like `Marshal.load` in Ruby.
pub struct Marshal;

impl Marshal {
    /// Serializes `object` (Ruby's `Marshal.dump`, `rb_marshal_dump`), or
    /// returns the `TypeError` for objects that cannot be dumped (procs,
    /// IO, singletons with methods, ...).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Marshal, Object, VM};
    /// # VM::init();
    ///
    /// let bytes = Marshal::dump(&Fixnum::new(42)).unwrap();
    /// let loaded = Marshal::load(&bytes).unwrap();
    ///
    /// assert_eq!(loaded.try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));
    ///
    /// assert!(Marshal::dump(&VM::eval("proc {}").unwrap()).is_err());
    /// ```
    pub fn dump<T: Object>(object: &T) -> Result<RString, AnyException> {
        let object = object.value();

        vm::protect_value(|| io::marshal_dump(object))
            .map(RString::from)
            .map_err(AnyException::from)
    }

    /// Deserializes an object dumped by [`Marshal::dump`](#method.dump) or
    /// Ruby's `Marshal.dump` (`rb_marshal_load`), or returns the error for
    /// malformed data. See the warning on [`Marshal`](struct.Marshal.html).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Marshal, Object, RString, VM};
    /// # VM::init();
    ///
    /// let bytes = VM::eval("Marshal.dump([1, 'two'])").unwrap().try_convert_to::<RString>().unwrap();
    /// let array = Marshal::load(&bytes).unwrap().try_convert_to::<Array>().unwrap();
    ///
    /// assert_eq!(array.length(), 2);
    /// assert!(Marshal::load(&RString::new_utf8("not marshal data")).is_err());
    /// ```
    pub fn load(data: &RString) -> Result<AnyObject, AnyException> {
        let data = data.value();

        vm::protect_value(|| io::marshal_load(data))
            .map(AnyObject::from)
            .map_err(AnyException::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Fixnum, Hash, Marshal, Object, RString, Symbol, VM};

    #[test]
    fn test_marshal_round_trip() {
        crate::on_ruby_thread(|| {
            let mut hash = Hash::new();
            hash.store(Symbol::new("key"), RString::new_utf8("value"));
            hash.store(Fixnum::new(1), crate::Integer::from(u128::max_value()));

            let bytes = Marshal::dump(&hash).unwrap();
            let loaded = Marshal::load(&bytes)
                .unwrap()
                .try_convert_to::<Hash>()
                .unwrap();

            assert!(loaded.equals(&hash));
            assert!(!loaded.is_equal(&hash));

            let from_ruby = VM::eval("Marshal.load(Marshal.dump(:sym))").unwrap();
            assert_eq!(from_ruby.try_convert_to::<Symbol>(), Ok(Symbol::new("sym")));

            let truncated = RString::new_utf8("\u{4}\u{8}[");
            assert!(Marshal::load(&truncated).is_err());
            assert!(Marshal::dump(&VM::eval("$stdout").unwrap()).is_err());
        });
    }
}
