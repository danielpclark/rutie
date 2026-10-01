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
    /// process (Ruby's `dummy?`, `rb_enc_dummy_p`).
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
        encoding::is_dummy(self.value())
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

    /// Registers a dummy encoding (one Ruby can name but not process, like
    /// `UTF-7`) called `name`, and returns it (`rb_define_dummy_encoding`).
    /// Returns the `ArgumentError` if an encoding has that name.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, VM};
    /// # VM::init();
    ///
    /// let dummy = Encoding::define_dummy("X-Rutie-Dummy").unwrap();
    ///
    /// assert_eq!(dummy.name(), "X-Rutie-Dummy");
    /// assert!(dummy.is_dummy());
    /// assert_eq!(Encoding::find("X-Rutie-Dummy").unwrap(), dummy);
    /// assert!(Encoding::define_dummy("UTF-8").is_err());
    /// ```
    pub fn define_dummy(name: &str) -> Result<Encoding, AnyException> {
        crate::binding::vm::protect_value(|| encoding::define_dummy(name))
            .map(Encoding::from)
            .map_err(AnyException::from)
    }

    /// Registers `alias` as another name for the encoding `original`
    /// (`rb_enc_alias`). Returns the `ArgumentError` if `alias` is taken or
    /// `original` is not an encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, VM};
    /// # VM::init();
    ///
    /// Encoding::add_alias("X-RUTIE-UTF8", "UTF-8").unwrap();
    ///
    /// assert_eq!(Encoding::find("X-RUTIE-UTF8").unwrap(), Encoding::utf8());
    /// assert!(Encoding::add_alias("X-RUTIE-UTF8", "UTF-8").is_err());
    /// assert!(Encoding::add_alias("X-RUTIE-NONE", "NO-SUCH-ENCODING").is_err());
    /// ```
    pub fn add_alias(alias: &str, original: &str) -> Result<(), AnyException> {
        let mut status = 0;

        crate::binding::vm::protect_value(|| {
            status = encoding::alias(alias, original);

            NilClass::new().value()
        })
        .map_err(AnyException::from)?;

        if status < 0 {
            let message = format!("unknown encoding name - {}", original);

            Err(AnyException::new("ArgumentError", Some(&message)))
        } else {
            Ok(())
        }
    }

    /// Returns `true` if `object` can carry an encoding: a `String`,
    /// `Symbol` or `Regexp`, or an object Ruby gave the encoding flag
    /// (`rb_enc_capable`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, Fixnum, RString, Symbol, VM};
    /// # VM::init();
    ///
    /// assert!(Encoding::is_capable(&RString::new_utf8("x")));
    /// assert!(Encoding::is_capable(&Symbol::new("x")));
    /// assert!(!Encoding::is_capable(&Fixnum::new(1)));
    /// ```
    pub fn is_capable<T: Object>(object: &T) -> bool {
        encoding::is_capable(object.value())
    }

    /// Returns `true` for the Unicode encodings, such as UTF-8 and UTF-16
    /// (`rb_enc_unicode_p`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, VM};
    /// # VM::init();
    ///
    /// assert!(Encoding::utf8().is_unicode());
    /// assert!(!Encoding::us_ascii().is_unicode());
    /// ```
    pub fn is_unicode(&self) -> bool {
        encoding::is_unicode(self.value())
    }

    /// Returns the name of the locale's character map, such as `"UTF-8"` or
    /// `"ANSI_X3.4-1968"` (Ruby's `Encoding.locale_charmap`,
    /// `rb_locale_charmap`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, Object, RString, VM};
    /// # VM::init();
    ///
    /// let charmap = VM::eval("Encoding.locale_charmap").unwrap();
    ///
    /// assert_eq!(Encoding::locale_charmap(), charmap.try_convert_to::<RString>().unwrap());
    /// ```
    pub fn locale_charmap() -> RString {
        RString::from(encoding::locale_charmap())
    }

    /// Returns how many characters `bytes` hold in this encoding, counting
    /// each invalid byte as one (`rb_enc_strlen`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Encoding::utf8().count_chars("añb".as_bytes()), 3);
    /// assert_eq!(Encoding::ascii_8bit().count_chars("añb".as_bytes()), 4);
    /// ```
    pub fn count_chars(&self, bytes: &[u8]) -> usize {
        encoding::strlen(bytes, self.value())
    }

    /// Returns the byte offset of the first occurrence of `needle` in
    /// `haystack`, or `None` (`rb_memsearch`). The encoding picks the search
    /// algorithm; the bytes are compared exactly.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, VM};
    /// # VM::init();
    ///
    /// let utf8 = Encoding::utf8();
    ///
    /// assert_eq!(utf8.search("ñb".as_bytes(), "añbañb".as_bytes()), Some(1));
    /// assert_eq!(utf8.search(b"zz", b"abc"), None);
    /// assert_eq!(utf8.search(b"", b"abc"), Some(0));
    /// ```
    pub fn search(&self, needle: &[u8], haystack: &[u8]) -> Option<usize> {
        encoding::memsearch(needle, haystack, self.value())
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
    use crate::{Encoding, EncodingSupport, Exception, Fixnum, Object, RString, Symbol, VM};

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

    #[test]
    fn test_encoding_registration_and_queries() {
        crate::on_ruby_thread(|| {
            let dummy = Encoding::define_dummy("X-RUTIE-TEST-DUMMY").unwrap();
            assert!(dummy.is_dummy());
            assert!(!dummy.is_unicode());
            let from_ruby = VM::eval("Encoding.find('X-RUTIE-TEST-DUMMY')").unwrap();
            assert_eq!(from_ruby.try_convert_to::<Encoding>().unwrap(), dummy);
            assert!(Encoding::define_dummy("X-RUTIE-TEST-DUMMY").is_err());

            Encoding::add_alias("X-RUTIE-TEST-ALIAS", "X-RUTIE-TEST-DUMMY").unwrap();
            assert_eq!(Encoding::find("X-RUTIE-TEST-ALIAS").unwrap(), dummy);
            assert!(Encoding::add_alias("X-RUTIE-TEST-ALIAS", "UTF-8").is_err());
            let unknown = Encoding::add_alias("X-RUTIE-TEST-OTHER", "NO-SUCH").unwrap_err();
            assert!(crate::Class::from_existing("ArgumentError").case_equals(&unknown));

            assert!(Encoding::is_capable(&VM::eval("/re/").unwrap()));
            assert!(!Encoding::is_capable(&VM::eval("Object.new").unwrap()));
            assert!(!Encoding::is_capable(&crate::NilClass::new()));

            assert!(!Encoding::ascii_8bit().is_unicode());
            assert!(!Encoding::utf8().is_dummy());
            assert!(!Encoding::locale_charmap().to_str().is_empty());

            let utf8 = Encoding::utf8();
            assert_eq!(utf8.count_chars(b""), 0);
            // Each invalid byte is a character.
            assert_eq!(utf8.count_chars(b"a\xFF\xFEb"), 4);
            assert_eq!(utf8.count_chars("🦀".as_bytes()), 1);
            // So is each byte of a truncated character.
            assert_eq!(utf8.count_chars(&"🦀".as_bytes()[..2]), 2);

            assert_eq!(utf8.search(b"abc", b"abc"), Some(0));
            assert_eq!(utf8.search(b"abcd", b"abc"), None);
            assert_eq!(utf8.search(b"c", b"abc"), Some(2));
            let long_needle = "ü".repeat(10);
            let haystack = format!("{}{}", "x".repeat(50), long_needle);
            assert_eq!(
                utf8.search(long_needle.as_bytes(), haystack.as_bytes()),
                Some(50)
            );
            // A needle ending in a lead byte.
            assert_eq!(
                utf8.search(b"abcdefghij\xF0", b"0123456789abcdefghij\xF0"),
                Some(10)
            );
            assert_eq!(
                Encoding::ascii_8bit().search(b"123456789", b"0123456789"),
                Some(1)
            );
        });
    }
}
