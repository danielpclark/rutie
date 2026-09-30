use crate::{
    binding::encoding,
    types::{EncodingIndex, Value, ValueType},
    AnyException, AnyObject, Class, Exception, NilClass, Object, RString, VerifiedObject,
};

#[derive(Debug)]
#[repr(C)]
pub struct Encoding {
    value: Value,
}

impl Encoding {
    /// Returns the `ASCII-8BIT` (binary) encoding (`rb_ascii8bit_encoding`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Encoding::ascii_8bit().name(), "ASCII-8BIT");
    /// ```
    pub fn ascii_8bit() -> Self {
        Encoding::from(encoding::ascii_8bit_encoding())
    }

    /// Returns the locale's encoding (`rb_locale_encoding`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, VM};
    /// # VM::init();
    ///
    /// assert!(!Encoding::locale().name().is_empty());
    /// ```
    pub fn locale() -> Self {
        Encoding::from(encoding::locale_encoding())
    }

    /// Returns the filesystem encoding (`rb_filesystem_encoding`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, VM};
    /// # VM::init();
    ///
    /// assert!(!Encoding::filesystem().name().is_empty());
    /// ```
    pub fn filesystem() -> Self {
        Encoding::from(encoding::filesystem_encoding())
    }

    /// Returns the encoding of `object` (a `String`, `Symbol`, `Regexp`,
    /// ...), or `None` for objects without one (`rb_enc_get`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, Fixnum, RString, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Encoding::of(&RString::new_utf8("x")).unwrap().name(), "UTF-8");
    /// assert!(Encoding::of(&Fixnum::new(1)).is_none());
    /// ```
    pub fn of<T: Object>(object: &T) -> Option<Self> {
        let found = encoding::encoding_of(object.value());

        if found.is_nil() {
            None
        } else {
            Some(Encoding::from(found))
        }
    }

    /// Returns Ruby's internal index for the encoding (`rb_to_encoding_index`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Encoding::utf8().index(), Encoding::utf8().index());
    /// assert_ne!(Encoding::utf8().index(), Encoding::us_ascii().index());
    /// ```
    pub fn index(&self) -> EncodingIndex {
        encoding::encoding_index(self.value())
    }

    /// Returns the character with code point `code` in this encoding as a
    /// string (Ruby's `Integer#chr(encoding)`, `rb_enc_uint_chr`), or the
    /// `RangeError` if the encoding cannot represent it.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Encoding::utf8().chr(0x1F980).unwrap().to_str(), "🦀");
    /// assert!(Encoding::us_ascii().chr(0xE9).is_err());
    /// ```
    pub fn chr(&self, code: u32) -> Result<RString, AnyException> {
        let encoding = self.value();

        crate::binding::vm::protect_value(|| encoding::chr(code, encoding))
            .map(RString::from)
            .map_err(AnyException::from)
    }

    /// Returns `true` if ASCII text is valid in this encoding (Ruby's
    /// `ascii_compatible?`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, VM};
    /// # VM::init();
    /// VM::init_loadpath(); // Needed for the encoding database
    /// VM::require("enc/encdb");
    ///
    /// assert!(Encoding::utf8().is_ascii_compatible());
    /// assert!(!Encoding::find("UTF-16LE").unwrap().is_ascii_compatible());
    /// ```
    pub fn is_ascii_compatible(&self) -> bool {
        unsafe { self.send("ascii_compatible?", &[]) }
            .value()
            .is_true()
    }

    /// Returns `true` for dummy encodings, which Ruby can name but not
    /// process (Ruby's `dummy?`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, VM};
    /// # VM::init();
    /// VM::init_loadpath(); // Needed for the encoding database
    /// VM::require("enc/encdb");
    ///
    /// assert!(!Encoding::utf8().is_dummy());
    /// assert!(Encoding::find("UTF-16").unwrap().is_dummy());
    /// ```
    pub fn is_dummy(&self) -> bool {
        unsafe { self.send("dummy?", &[]) }.value().is_true()
    }

    /// Creates a UTF-8 instance of `Encoding`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Encoding::utf8().name(), "UTF-8");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// Encoding::UTF_8
    /// ```
    pub fn utf8() -> Self {
        Self::from(encoding::utf8_encoding())
    }

    /// Creates a US-ASCII instance of `Encoding`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Encoding::us_ascii().name(), "US-ASCII");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// Encoding::US_ASCII
    /// ```
    pub fn us_ascii() -> Self {
        Self::from(encoding::usascii_encoding())
    }

    /// Creates a new instance of `Encoding` from the default external encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, Object, RString, VM};
    /// # VM::init();
    ///
    /// let from_ruby = VM::eval("Encoding.default_external.name").unwrap();
    ///
    /// assert_eq!(
    ///     Encoding::default_external().name(),
    ///     from_ruby.try_convert_to::<RString>().unwrap().to_str()
    /// );
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// Encoding.default_external
    /// ```
    pub fn default_external() -> Self {
        Self::from(encoding::default_external())
    }

    /// Creates an instance of `Ok(Encoding)` from the default internal encoding
    /// if there is one, otherwise it returns `Err(NilClass)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, VM};
    /// # VM::init();
    ///
    /// // Ruby has no default internal encoding unless one is set.
    /// assert!(Encoding::default_internal().is_err());
    ///
    /// VM::eval("Encoding.default_internal = 'UTF-8'").unwrap();
    /// assert_eq!(Encoding::default_internal().unwrap().name(), "UTF-8");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// Encoding.default_internal
    /// ```
    pub fn default_internal() -> Result<Self, NilClass> {
        let result = encoding::default_internal();

        if result.is_nil() {
            Err(NilClass::from(result))
        } else {
            Ok(Self::from(result))
        }
    }

    /// Returns encoding name.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, Encoding, VM};
    /// # VM::init();
    ///
    /// let enc = Encoding::utf8();
    ///
    /// assert_eq!(enc.name(), "UTF-8")
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// enc = Encoding::UTF_8
    ///
    /// enc.name == "UTF-8"
    /// ```
    pub fn name(&self) -> String {
        let name = unsafe { self.send("name", &[]) };

        RString::from(name.value()).to_string()
    }

    /// Find an `Ok(Encoding)` for given string name or return an `Err(AnyException)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{VM, Encoding};
    /// # VM::init();
    ///
    /// let encoding = Encoding::find("UTF-8");
    ///
    /// match encoding {
    ///     Ok(enc) => assert_eq!(enc.name(), "UTF-8"),
    ///     Err(_) => unreachable!()
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// encoding = Encoding.find("UTF-8")
    ///
    /// encoding.name == "UTF-8"
    /// ```
    ///
    /// The following is an example where a Ruby exception object of `ArgumentError` is returned.
    ///
    /// ```
    /// use rutie::{VM, Encoding, Exception};
    /// # VM::init();
    ///
    /// let encoding = Encoding::find("UTF8");
    ///
    /// match encoding {
    ///     Ok(_) => unreachable!(),
    ///     Err(e) => assert_eq!(e.message(), "unknown encoding name - UTF8")
    /// }
    /// ```
    pub fn find(s: &str) -> Result<Encoding, AnyException> {
        let idx = encoding::find_encoding_index(s);

        if idx < 0 {
            Err(AnyException::new(
                "ArgumentError",
                Some(&format!("unknown encoding name - {}", s)),
            ))
        } else {
            Ok(Encoding::from(encoding::from_encoding_index(idx)))
        }
    }

    /// Returns an instance of `Ok(Encoding)` if the objects are
    /// compatible encodings, otherwise it returns `Err(NilClass)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, VM, RString, NilClass};
    /// # VM::init();
    ///
    /// let utf8 = RString::new_utf8("asdf");
    /// let us_ascii= RString::new_usascii_unchecked("qwerty");
    ///
    /// let result = Encoding::is_compatible(&utf8, &us_ascii);
    ///
    /// assert!(result.is_ok());
    ///
    /// let result = Encoding::is_compatible(&utf8, &NilClass::new());
    ///
    /// assert!(result.is_err());
    /// ```
    pub fn is_compatible(obj1: &impl Object, obj2: &impl Object) -> Result<Self, NilClass> {
        let result = encoding::compatible_encoding(obj1.value(), obj2.value());

        if result.is_nil() {
            Err(NilClass::from(result))
        } else {
            Ok(Self::from(result))
        }
    }
}

impl Default for Encoding {
    fn default() -> Self {
        Encoding::default_external()
    }
}

impl From<Value> for Encoding {
    fn from(value: Value) -> Self {
        Encoding { value }
    }
}

impl Into<Value> for Encoding {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Encoding {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for Encoding {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Encoding {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        // `Encoding` instances are `T_DATA`, not classes.
        Class::encoding().case_equals(object)
    }

    fn error_message() -> &'static str {
        "Error converting to Encoding"
    }
}

impl PartialEq for Encoding {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Encoding, EncodingSupport, Fixnum, Object, RString, Symbol, VM};

    #[test]
    fn test_encoding_lookup_and_chr() {
        crate::on_ruby_thread(|| {
            // An embedded VM only knows the built-in encodings until the
            // encoding database is loaded.
            VM::init_loadpath();
            VM::require("enc/encdb");

            assert_eq!(Encoding::ascii_8bit().name(), "ASCII-8BIT");
            assert_eq!(
                Encoding::of(&Symbol::new("sym")).unwrap().name(),
                "US-ASCII"
            );
            assert!(Encoding::of(&Fixnum::new(1)).is_none());
            assert_eq!(
                Encoding::of(&RString::new_utf8("é")).unwrap().index(),
                Encoding::utf8().index()
            );

            let e = Encoding::utf8().chr(0xE9).unwrap();
            assert_eq!(e.to_str(), "é");
            assert_eq!(e.encoding().name(), "UTF-8");
            assert!(Encoding::utf8().chr(0xD800).is_err());

            let mut binary = RString::from_bytes(b"\xFF", &Encoding::ascii_8bit());
            assert!(binary
                .concat_bytes("é".as_bytes(), &Encoding::utf8())
                .is_err());

            assert!(Encoding::find("Shift_JIS").unwrap().is_ascii_compatible());
        });
    }

    #[test]
    fn test_encoding_try_convert() {
        crate::on_ruby_thread(|| {
            let encoding = VM::eval("''.encoding").unwrap();

            assert!(encoding.try_convert_to::<Encoding>().is_ok());
            assert!(RString::new_utf8("x").try_convert_to::<Encoding>().is_err());
        });
    }

    #[test]
    fn test_encoding_defaults_and_compatibility() {
        crate::on_ruby_thread(|| {
            assert_eq!(Encoding::us_ascii().name(), "US-ASCII");
            assert!(!Encoding::us_ascii().is_dummy());

            // The process-wide defaults match what Ruby reports.
            let external = VM::eval("Encoding.default_external.name").unwrap();
            assert_eq!(
                Encoding::default_external().name(),
                external.try_convert_to::<RString>().unwrap().to_str()
            );
            let internal = VM::eval("Encoding.default_internal").unwrap();
            assert_eq!(Encoding::default_internal().is_err(), internal.is_nil());

            // Without the encoding database loaded, `Encoding.find('locale')`
            // is unavailable, so compare with what Ruby's own objects report.
            let locale = Encoding::locale();
            assert!(!locale.name().is_empty());
            assert!(unsafe { locale.send("ascii_compatible?", &[]) }
                .value()
                .is_true());
            assert!(!Encoding::filesystem().name().is_empty());

            let utf8 = RString::new_utf8("é");
            let ascii = RString::new_usascii_unchecked("a");
            assert_eq!(
                Encoding::is_compatible(&utf8, &ascii).unwrap().name(),
                "UTF-8"
            );

            let binary = VM::eval(r#""\xFF".b"#).unwrap();
            assert!(Encoding::is_compatible(&utf8, &binary).is_err());
        });
    }
}
