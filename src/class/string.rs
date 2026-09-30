use std::{
    cmp::Ordering,
    convert::From,
    panic::{self, AssertUnwindSafe},
};

use crate::{
    binding::{class::is_frozen, encoding, string, vm},
    rubysys::encoding::{ENC_CODERANGE_7BIT, ENC_CODERANGE_VALID},
    types::{Value, ValueType},
    AnyException, AnyObject, Array, Boolean, CodepointIterator, Encoding, EncodingSupport,
    Exception, Hash, Integer, NilClass, Object, TryConvert, VerifiedObject,
};

/// `String`
#[derive(Debug)]
#[repr(C)]
pub struct RString {
    value: Value,
}

impl RString {
    /// Converts `object` to a `String` the way Ruby's `String(object)`
    /// (`Kernel#String`, `rb_String`) does, calling `to_str` or `to_s`. Returns the exception when
    /// it cannot be converted.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, RString, Symbol, VM};
    /// # VM::init();
    ///
    /// let number = RString::convert(&Fixnum::new(42)).unwrap();
    /// assert_eq!(number.to_str(), "42");
    ///
    /// let symbol = RString::convert(&Symbol::new("name")).unwrap();
    /// assert_eq!(symbol.to_str(), "name");
    ///
    /// // `BasicObject` has no `to_s`:
    /// let basic = VM::eval("BasicObject.new").unwrap();
    /// assert!(RString::convert(&basic).is_err());
    /// ```
    pub fn convert<T: Object>(object: &T) -> Result<Self, AnyException> {
        let object = object.value();

        crate::binding::vm::protect_value(|| crate::binding::object::to_string(object))
            .map(Self::from)
            .map_err(AnyException::from)
    }

    /// Creates a new instance of Ruby `String` containing given `string`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new("Hello, World!");
    ///
    /// assert_eq!(string.to_str(), "Hello, World!");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// str = 'Hello, World!'
    ///
    /// str == 'Hello, World!'
    /// ```
    #[deprecated(
        since = "0.3.2",
        note = "please use `new_usascii_unchecked` or `new_utf8` instead"
    )]
    pub fn new(string: &str) -> Self {
        Self::new_usascii_unchecked(string)
    }

    /// Creates a new instance of Ruby `String`, with UTF8 encoding, containing
    /// given `string`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("Hello, World!");
    ///
    /// assert_eq!(string.to_string(), "Hello, World!".to_string());
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// str = 'Hello, World!'
    ///
    /// str == 'Hello, World!'
    /// ```
    pub fn new_utf8(string: &str) -> Self {
        Self::from(string::new_utf8(string))
    }

    /// Creates a new instance of Ruby `String` containing given `string`.
    ///
    /// Despite the name, the string is tagged `ASCII-8BIT` (binary), as
    /// `rb_str_new` does; use [`new_utf8`](#method.new_utf8) for text, or
    /// [`from_bytes`](#method.from_bytes) to choose the encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{EncodingSupport, RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_usascii_unchecked("Hello, World!");
    ///
    /// assert_eq!(string.to_str(), "Hello, World!");
    /// assert_eq!(string.encoding().name(), "ASCII-8BIT");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// str = 'Hello, World!'
    ///
    /// str == 'Hello, World!'
    /// ```
    pub fn new_usascii_unchecked(string: &str) -> Self {
        Self::from(string::new(string))
    }

    /// Creates a new instance of Ruby `String` from given byte
    /// sequence with given `Encoding`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, Encoding, EncodingSupport, VM};
    /// # VM::init();
    ///
    /// let bytes = [197, 130, 97, 197, 130];
    /// let enc = Encoding::find("UTF-8").unwrap();
    ///
    /// let string = RString::from_bytes(&bytes, &enc);
    ///
    /// assert_eq!(string.to_str(), "łał");
    ///
    /// # VM::init_loadpath();
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// let result = string.encode(Encoding::find("UTF-16").unwrap(), None);
    ///
    /// assert_eq!(result.to_bytes_unchecked(), [254, 255, 1, 66, 0, 97, 1, 66])
    /// ```
    pub fn from_bytes(bytes: &[u8], enc: &Encoding) -> Self {
        Self::from(string::new_from_bytes(bytes, enc.value()))
    }

    /// Retrieves underlying Rust `String` from Ruby `String` object.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("Hello, World!");
    ///
    /// assert_eq!(string.to_string(), "Hello, World!".to_string());
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// str = 'Hello, World!'
    ///
    /// str == 'Hello, World!'
    /// ```
    pub fn to_string(&self) -> String {
        string::value_to_string(self.value())
    }

    /// Retrieves underlying Rust `String` from Ruby `String` object.
    ///
    /// Unlike `to_string()` it does not perform any checks for internal null-bytes.
    ///
    /// This function may be used to safely get binary data from Ruby.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("Hello,\0World!");
    ///
    /// assert_eq!(string.to_string_unchecked(), "Hello,\0World!".to_string());
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// str = 'Hello,\0World!'
    ///
    /// str == 'Hello,\0World!'
    /// ```
    pub fn to_string_unchecked(&self) -> String {
        string::value_to_string_unchecked(self.value())
    }

    /// Retrieves `Vec<u8>` from Ruby `String` object.
    ///
    /// Unlike `to_string()` it does not perform any checks for internal null-bytes.
    ///
    /// This function may be used to safely get binary data from Ruby.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("Hello,\0World!");
    ///
    /// assert_eq!(string.to_vec_u8_unchecked(), (b"Hello,\0World!").to_vec());
    /// ```
    pub fn to_vec_u8_unchecked(&self) -> Vec<u8> {
        self.to_bytes_unchecked().to_vec()
    }

    /// Retrieves underlying `&str` from Ruby `String` object.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("Hello, World!");
    ///
    /// assert_eq!(string.to_str(), "Hello, World!");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// str = 'Hello, World!'
    ///
    /// str == 'Hello, World!'
    /// ```
    pub fn to_str(&self) -> &str {
        let value = self.value();

        string::value_to_str(value)
    }

    /// Retrieves underlying `&str` from Ruby `String` object.
    ///
    /// Unlike `to_str()` it does not perform any checks for internal null-bytes.
    ///
    /// This function may be used to safely get binary data from Ruby.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("Hello,\0World!");
    ///
    /// assert_eq!(string.to_str_unchecked(), "Hello,\0World!");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// str = 'Hello,\0World!'
    ///
    /// str == 'Hello,\0World!'
    /// ```
    pub fn to_str_unchecked(&self) -> &str {
        let value = self.value();

        string::value_to_str_unchecked(value)
    }

    /// Retrieves underlying `&[u8]` from Ruby `String` object.
    ///
    /// Unlike `to_str()` it does not perform any checks for internal null-bytes.
    ///
    /// This function may be used to safely get binary data from Ruby.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("Hello,\0World!");
    ///
    /// assert_eq!(string.to_bytes_unchecked(), b"Hello,\0World!");
    /// ```
    pub fn to_bytes_unchecked(&self) -> &[u8] {
        let value = self.value();

        string::value_to_bytes_unchecked(value)
    }

    /// Returns an array of each characters codepoints.  This is useful as
    /// a strings encoding determines where the codepoints are.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, Array, Fixnum, Encoding, EncodingSupport, VM};
    /// # VM::init();
    /// # VM::init_loadpath(); // Needed for alternate encodings
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// let string = RString::from_bytes(b"foo\x93_a", &Encoding::find("cp932").unwrap());
    ///
    /// let codepoints: Array = [102, 111, 111, 37727, 97].
    ///   into_iter().map(|cp| Fixnum::new(cp as i64).to_any_object()).collect();
    ///
    /// assert!(string.codepoints().equals(&codepoints), "not equal!");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// str = "foo\x93_a".force_encoding("cp932")
    ///
    /// str.codepoints == [102, 111, 111, 37727, 97]
    /// ```
    pub fn codepoints(&self) -> Array {
        CodepointIterator::new(self)
            .into_iter()
            .map(|n| Integer::new(n as i64).to_any_object())
            .collect()
    }

    /// Returns the length of the string in bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("Hello, World!");
    /// let utf8_string = RString::new_utf8("⓯");
    ///
    /// assert_eq!(string.bytesize(), 13);
    /// assert_eq!(utf8_string.bytesize(), 3);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// string = 'Hello, World!'
    /// utf8_string = '⓯'
    ///
    /// string.bytesize == 13
    /// utf8_string.bytesize == 3
    /// ```
    pub fn bytesize(&self) -> i64 {
        string::bytesize(self.value())
    }

    /// Returns the number of characters in the string
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("Hello, World!");
    /// let utf8_string = RString::new_utf8("⓯");
    ///
    /// assert_eq!(string.count_chars(), 13);
    /// assert_eq!(utf8_string.count_chars(), 1);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// string = 'Hello, World!'
    /// utf8_string = '⓯'
    ///
    /// string.length == 13
    /// utf8_string.length == 1
    /// ```
    pub fn count_chars(&self) -> i64 {
        string::count_chars(self.value())
    }

    /// Appends a given string slice onto the end of this String.
    ///
    /// Ruby raises `FrozenError` if the string is frozen; check
    /// [`is_frozen`](trait.Object.html#method.is_frozen) first or call it inside
    /// [`VM::protect`](struct.VM.html#method.protect) when that is possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let mut string = RString::new_utf8("Hello, ");
    /// string.concat("World!");
    ///
    /// assert_eq!(string.to_string(), "Hello, World!".to_string());
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// str = 'Hello, '
    /// str << 'World!'
    ///
    /// str == 'Hello, World!'
    /// ```
    pub fn concat(&mut self, string: &str) {
        string::concat(self.value(), string.as_bytes());
    }

    /// Creates an empty UTF-8 string with room for `capacity` bytes, to fill
    /// without reallocating (`rb_str_buf_new`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let mut buffer = RString::with_capacity(64);
    ///
    /// assert!(buffer.capacity() >= 64);
    /// assert_eq!(buffer.bytesize(), 0);
    ///
    /// buffer.concat("héllo");
    ///
    /// assert_eq!(buffer.to_str(), "héllo");
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self::from(string::with_capacity(capacity))
    }

    /// Returns how many bytes the string can hold without reallocating
    /// (`rb_str_capacity`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("abc");
    ///
    /// assert!(string.capacity() >= 3);
    /// ```
    pub fn capacity(&self) -> usize {
        string::capacity(self.value())
    }

    /// Compares two strings byte by byte like Ruby's `<=>` (`rb_str_cmp`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// use std::cmp::Ordering;
    /// # VM::init();
    ///
    /// let apple = RString::new_utf8("apple");
    /// let banana = RString::new_utf8("banana");
    ///
    /// assert_eq!(apple.compare(&banana), Ordering::Less);
    /// assert_eq!(apple.compare(&apple), Ordering::Equal);
    /// assert!(banana > apple);
    /// ```
    pub fn compare(&self, other: &RString) -> Ordering {
        string::compare(self.value(), other.value()).cmp(&0)
    }

    /// Returns a copy shortened to at most `max_chars` characters, ending
    /// in `"..."` when it was cut (`rb_str_ellipsize`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let long = RString::new_utf8("a rather long sentence");
    ///
    /// assert_eq!(long.ellipsize(9).to_str(), "a rath...");
    /// assert_eq!(long.ellipsize(100).to_str(), "a rather long sentence");
    /// ```
    pub fn ellipsize(&self, max_chars: usize) -> RString {
        RString::from(string::ellipsize(self.value(), max_chars))
    }

    /// Returns a new string joining this one and `other` (Ruby's `+`,
    /// `rb_str_plus`).
    ///
    /// Raises `Encoding::CompatibilityError` for incompatible encodings.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let joined = RString::new_utf8("Hello, ").plus(&RString::new_utf8("World"));
    ///
    /// assert_eq!(joined.to_str(), "Hello, World");
    /// ```
    pub fn plus(&self, other: &RString) -> RString {
        RString::from(string::plus(self.value(), other.value()))
    }

    /// Replaces the contents (and encoding) of this string with `other`'s
    /// (Ruby's `replace`, `rb_str_replace`).
    ///
    /// Ruby raises `FrozenError` if the string is frozen.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let mut string = RString::new_utf8("old");
    /// string.replace(&RString::new_utf8("new"));
    ///
    /// assert_eq!(string.to_str(), "new");
    /// ```
    pub fn replace(&mut self, other: &RString) {
        string::replace(self.value(), other.value());
    }

    /// Shortens the string to `byte_len` bytes; does nothing if it is not
    /// longer than that (`rb_str_resize`).
    ///
    /// Cutting inside a multibyte character leaves an invalid string. Ruby
    /// raises `FrozenError` if the string is frozen.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let mut string = RString::new_utf8("truncated");
    ///
    /// string.truncate(5);
    /// assert_eq!(string.to_str(), "trunc");
    ///
    /// string.truncate(50);
    /// assert_eq!(string.to_str(), "trunc");
    /// ```
    pub fn truncate(&mut self, byte_len: usize) {
        string::truncate(self.value(), byte_len);
    }

    /// Returns a copy with invalid byte sequences replaced by `replacement`,
    /// or by U+FFFD (or `?` for non-Unicode encodings) when it is `None`
    /// (Ruby's `scrub`, `rb_str_scrub`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, EncodingSupport, RString, VM};
    /// # VM::init();
    ///
    /// let broken = RString::from_bytes(b"ab\xFFc", &Encoding::utf8());
    ///
    /// assert!(!broken.is_valid_encoding());
    /// assert_eq!(broken.scrub(None).to_str(), "ab\u{FFFD}c");
    /// assert_eq!(broken.scrub(Some("?")).to_str(), "ab?c");
    /// ```
    pub fn scrub(&self, replacement: Option<&str>) -> RString {
        let replacement = replacement
            .map(|replacement| string::new_utf8(replacement))
            .unwrap_or_else(|| NilClass::new().value());
        let result = string::scrub(self.value(), replacement);

        if result.is_nil() {
            RString::from(string::dup(self.value()))
        } else {
            RString::from(result)
        }
    }

    /// Splits the string on `separator` (Ruby's `split` with a string;
    /// `" "` splits on runs of whitespace), `rb_str_split`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let parts = RString::new_utf8("a,b,,c").split(",");
    ///
    /// assert_eq!(parts.length(), 4);
    ///
    /// let words = RString::new_utf8("  one  two ").split(" ");
    ///
    /// assert_eq!(words.length(), 2);
    /// ```
    pub fn split(&self, separator: &str) -> Array {
        Array::from(string::split(self.value(), separator))
    }

    /// Returns `len` bytes starting at byte `start` as a new string with
    /// the same encoding, or `None` if that range is not within the string
    /// (Ruby's `byteslice`, `rb_str_subseq`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("héllo");
    ///
    /// assert_eq!(string.byte_slice(3, 3).unwrap().to_str(), "llo");
    /// assert!(string.byte_slice(4, 10).is_none());
    /// ```
    pub fn byte_slice(&self, start: usize, len: usize) -> Option<RString> {
        string::byte_slice(self.value(), start, len).map(RString::from)
    }

    /// Returns up to `len` characters starting at character `start` (from
    /// the end when negative), or `None` if `start` is out of range or `len`
    /// is negative (Ruby's `str[start, len]`, `rb_str_substr`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("héllo");
    ///
    /// assert_eq!(string.substr(1, 3).unwrap().to_str(), "éll");
    /// assert_eq!(string.substr(-2, 5).unwrap().to_str(), "lo");
    /// assert!(string.substr(10, 1).is_none());
    /// ```
    pub fn substr(&self, start: i64, len: i64) -> Option<RString> {
        let result = string::substr(self.value(), start, len);

        if result.is_nil() {
            None
        } else {
            Some(RString::from(result))
        }
    }

    /// Returns the string repeated `count` times (Ruby's `*`, `rb_str_times`).
    ///
    /// Raises `ArgumentError` if the result would be too big.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// assert_eq!(RString::new_utf8("ab").times(3).to_str(), "ababab");
    /// assert_eq!(RString::new_utf8("ab").times(0).to_str(), "");
    /// ```
    pub fn times(&self, count: usize) -> RString {
        let count = Integer::from(count as u64);

        RString::from(string::times(self.value(), count.value()))
    }

    /// Parses a leading integer in `base` like Ruby's `to_i`, ignoring
    /// anything after it and returning `0` when there is none
    /// (`rb_str_to_inum`).
    ///
    /// # Panics
    ///
    /// If `base` is not between 2 and 36.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// assert_eq!(RString::new_utf8("42abc").to_i(10).to_i64(), 42);
    /// assert_eq!(RString::new_utf8("ff").to_i(16).to_i64(), 255);
    /// assert_eq!(RString::new_utf8("junk").to_i(10).to_i64(), 0);
    /// ```
    pub fn to_i(&self, base: u32) -> Integer {
        assert!((2..=36).contains(&base), "invalid radix {}", base);

        Integer::from(string::to_integer(self.value(), base, false))
    }

    /// Parses the whole string as an integer in `base` like Ruby's
    /// `Integer(string, base)` (`rb_str_to_inum` with checking), returning
    /// the `ArgumentError` for anything that is not a valid integer.
    ///
    /// A `base` of `0` accepts the `0b`, `0o`, `0` and `0x` prefixes.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// assert_eq!(RString::new_utf8(" 1_000 ").parse_integer(10).unwrap().to_i64(), 1000);
    /// assert_eq!(RString::new_utf8("0x1f").parse_integer(0).unwrap().to_i64(), 31);
    /// assert!(RString::new_utf8("42abc").parse_integer(10).is_err());
    /// assert!(RString::new_utf8("1").parse_integer(99).is_err());
    /// ```
    pub fn parse_integer(&self, base: u32) -> Result<Integer, AnyException> {
        if base == 1 || base > 36 {
            let message = format!("invalid radix {}", base);

            return Err(AnyException::new("ArgumentError", Some(&message)));
        }

        let string = self.value();

        vm::protect_value(|| string::to_integer(string, base, true))
            .map(Integer::from)
            .map_err(AnyException::from)
    }

    /// Parses a leading floating point number like Ruby's `to_f`, returning
    /// `0.0` when there is none (`rb_str_to_dbl`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// assert_eq!(RString::new_utf8("2.5kg").to_f(), 2.5);
    /// assert_eq!(RString::new_utf8("none").to_f(), 0.0);
    /// ```
    pub fn to_f(&self) -> f64 {
        string::to_f64(self.value(), false)
    }

    /// Parses the whole string as a floating point number like Ruby's
    /// `Float(string)` (`rb_str_to_dbl` with checking), returning the
    /// `ArgumentError` for anything else.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// assert_eq!(RString::new_utf8("1.5e3").parse_float().unwrap(), 1500.0);
    /// assert!(RString::new_utf8("2.5kg").parse_float().is_err());
    /// ```
    pub fn parse_float(&self) -> Result<f64, AnyException> {
        let string = self.value();
        let mut result = 0.0;

        vm::protect_value(|| {
            result = string::to_f64(string, true);

            NilClass::new().value()
        })
        .map(|_| result)
        .map_err(AnyException::from)
    }

    /// Returns what the string's bytes are in its encoding (computed once
    /// and cached by Ruby, `rb_enc_str_coderange`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{CodeRange, Encoding, RString, VM};
    /// # VM::init();
    ///
    /// assert_eq!(RString::new_utf8("plain").coderange(), CodeRange::SevenBit);
    /// assert_eq!(RString::new_utf8("héllo").coderange(), CodeRange::Valid);
    ///
    /// let broken = RString::from_bytes(b"\xFF", &Encoding::utf8());
    ///
    /// assert_eq!(broken.coderange(), CodeRange::Broken);
    /// ```
    pub fn coderange(&self) -> CodeRange {
        let coderange = string::coderange(self.value()) as isize;

        if coderange == ENC_CODERANGE_7BIT {
            CodeRange::SevenBit
        } else if coderange == ENC_CODERANGE_VALID {
            CodeRange::Valid
        } else {
            CodeRange::Broken
        }
    }

    /// Appends `bytes`, which are in encoding `enc`, converting them the
    /// way Ruby's `<<` does (`rb_enc_str_buf_cat`). Returns the
    /// `Encoding::CompatibilityError` when the encodings cannot be combined.
    ///
    /// Ruby raises `FrozenError` inside the `Err` if the string is frozen.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, RString, VM};
    /// # VM::init();
    ///
    /// let mut string = RString::new_utf8("abc ");
    ///
    /// string.concat_bytes(b"def", &Encoding::us_ascii()).unwrap();
    /// assert_eq!(string.to_str(), "abc def");
    ///
    /// let mut ascii = RString::from_bytes(b"x", &Encoding::us_ascii());
    /// ascii.concat_bytes("é".as_bytes(), &Encoding::utf8()).unwrap();
    /// assert_eq!(ascii.to_str(), "xé");
    /// ```
    pub fn concat_bytes(&mut self, bytes: &[u8], enc: &Encoding) -> Result<(), AnyException> {
        let (string, enc) = (self.value(), enc.value());

        vm::protect_value(|| encoding::concat_bytes(string, bytes, enc))
            .map(|_| ())
            .map_err(AnyException::from)
    }

    /// Calls `f` with the string's bytes while the string is locked
    /// against modification (`rb_str_locktmp`), so the slice stays valid
    /// even if `f` calls into Ruby.
    ///
    /// Ruby code that tries to modify the string meanwhile raises
    /// `RuntimeError`. The lock is released when `f` returns, panics, or a
    /// Ruby exception propagates out of it.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("locked");
    ///
    /// let (count, modify_failed) = string.with_locked_bytes(|bytes| {
    ///     let modify = string.protect_send("<<", &[RString::new_utf8("!").into()]);
    ///
    ///     (bytes.len(), modify.is_err())
    /// });
    ///
    /// assert_eq!(count, 6);
    /// assert!(modify_failed);
    ///
    /// // Unlocked again.
    /// assert!(string.protect_send("<<", &[RString::new_utf8("!").into()]).is_ok());
    /// ```
    pub fn with_locked_bytes<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        let value = self.value();
        let bytes = string::value_to_bytes_unchecked(value);

        // Already locked by an outer call: that lock covers this one.
        if string::is_lockedtmp(value) {
            return f(bytes);
        }

        string::locktmp(value);

        let mut outcome = None;

        vm::ensure(
            || {
                outcome = Some(panic::catch_unwind(AssertUnwindSafe(|| f(bytes))));

                NilClass::new().value()
            },
            || {
                string::unlocktmp(value);
            },
        );

        match outcome.expect("ensure body did not run") {
            Ok(result) => result,
            Err(payload) => panic::resume_unwind(payload),
        }
    }
}

/// What a string's bytes are in its encoding; see
/// [`RString::coderange`](struct.RString.html#method.coderange).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeRange {
    /// Only ASCII characters (every byte below 128).
    SevenBit,
    /// Valid, with at least one non-ASCII character.
    Valid,
    /// Contains invalid byte sequences.
    Broken,
}

impl EncodingSupport for RString {
    /// Get the strings `Encoding`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM, EncodingSupport};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("Hello");
    /// string.encoding();
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// string = "Hello"
    /// string.encoding()
    /// ```
    fn encoding(&self) -> Encoding {
        Encoding::from(encoding::from_encoding_index(encoding::enc_get_index(
            self.value(),
        )))
    }

    /// Changes the encoding to encoding and returns `Result<Self, AnyException>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM, EncodingSupport, Encoding};
    /// # VM::init();
    ///
    /// let mut string = RString::new_utf8("Hello");
    /// string.force_encoding(Encoding::us_ascii());
    ///
    /// assert_eq!(string.encoding().name(), "US-ASCII");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// string = "Hello"
    /// string.force_encoding(Encoding::US_ASCII)
    ///
    /// string.encoding.name == "US-ASCII"
    /// ```
    ///
    /// ```
    /// use rutie::{RString, VM, EncodingSupport, Encoding, Object, Exception};
    /// # VM::init();
    ///
    /// let mut string = RString::new_utf8("Hello");
    /// string.force_encoding(Encoding::utf8());
    /// string.freeze();
    /// let result = string.force_encoding(Encoding::us_ascii());
    ///
    /// match result {
    ///     Ok(_) => assert_eq!("This is a bad path.", "You shouldn't get this message."),
    ///     Err(happy_path) => assert_eq!(happy_path.message(), "can\'t modify frozen String"),
    /// }
    /// ```
    fn force_encoding(&mut self, enc: Encoding) -> Result<Self, AnyException> {
        if string::is_lockedtmp(self.value()) {
            return Err(AnyException::new(
                "RuntimeError",
                Some("can't modify string; temporarily locked"),
            ));
        }

        if self.is_frozen() {
            return Err(AnyException::new(
                "FrozenError",
                Some("can't modify frozen String"),
            ));
        }

        self.value = encoding::force_encoding(self.value(), enc.value());
        encoding::coderange_clear(self.value);

        Ok(Self::from(self.value()))
    }

    /// Transcodes to encoding and returns `Self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM, EncodingSupport, Encoding};
    /// # VM::init();
    ///
    /// let mut string = RString::new_utf8("Hello");
    /// let result = string.encode(Encoding::us_ascii(), None);
    ///
    /// assert_eq!(result.encoding().name(), "US-ASCII");
    /// ```
    ///
    /// Options are Ruby's `String#encode` options:
    ///
    /// ```
    /// use rutie::{Encoding, EncodingSupport, Hash, RString, Symbol, VM};
    /// # VM::init();
    /// VM::init_loadpath();
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// let mut options = Hash::new();
    /// options.store(Symbol::new("undef"), Symbol::new("replace"));
    /// options.store(Symbol::new("replace"), RString::new_utf8("*"));
    ///
    /// let result = RString::new_utf8("héllo").encode(Encoding::us_ascii(), Some(options));
    ///
    /// assert_eq!(result.to_str(), "h*llo");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// string = "Hello"
    /// result = string.encode(Encoding::US_ASCII)
    ///
    /// result.encoding.name == "US-ASCII"
    /// ```
    fn encode(&self, enc: Encoding, opts: Option<Hash>) -> Self {
        let nil = NilClass::new().value();

        let value = match opts {
            Some(options) => {
                let mut ecopts = nil;
                let ecflags = encoding::econv_prepare_opts(options.value(), &mut ecopts);

                encoding::encode(self.value(), enc.value(), ecflags, ecopts)
            }
            None => encoding::encode(self.value(), enc.value(), 0, nil),
        };

        Self::from(value)
    }

    /// Transcodes to encoding and returns `Self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM, EncodingSupport, Encoding, Object};
    /// # VM::init();
    ///
    /// let mut string = RString::new_utf8("Hello");
    ///
    /// assert!(string.is_valid_encoding(), "not valid encoding!");
    ///
    /// # VM::init_loadpath();
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// let result = VM::eval("'Hello'.force_encoding('UTF-32')").unwrap().
    ///   try_convert_to::<RString>().unwrap();
    ///
    /// assert!(!result.is_valid_encoding(), "is valid encoding!");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// string = "Hello"
    ///
    /// string.valid_encoding? == true
    ///
    /// result = string.encode(Encoding::UTF_32)
    ///
    /// result.valid_encoding? == false
    /// ```
    fn is_valid_encoding(&self) -> bool {
        let result = unsafe { self.send("valid_encoding?", &[]) };
        result.try_convert_to::<Boolean>().unwrap().to_bool()
    }

    /// Reveals if the given object has a compatible encoding with this String.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM, EncodingSupport};
    /// # VM::init();
    ///
    /// let string1 = RString::new_utf8("Hello");
    /// let string2 = RString::new_usascii_unchecked("Hello");
    ///
    /// assert!(string1.compatible_with(&string2));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// str1 = 'Hello'.force_encoding("UTF-8")
    /// str2 = 'Hello'.force_encoding("US-ASCII")
    ///
    /// str1 + str2 == "HelloHello"
    /// ```
    fn compatible_with(&self, other: &impl Object) -> bool {
        encoding::is_compatible_encoding(self.value(), other.value())
    }

    /// Returns `AnyObject` of the compatible encoding between the two objects
    /// or nil if incompatible.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM, EncodingSupport};
    /// # VM::init();
    ///
    /// let string1 = RString::new_utf8("Hello");
    /// let string2 = RString::new_usascii_unchecked("Hello");
    ///
    /// RString::compatible_encoding(&string1, &string2);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// str1 = 'Hello'.force_encoding("UTF-8")
    /// str2 = 'Hello'.force_encoding("US-ASCII")
    ///
    /// begin
    ///   (str1 + str2).encoding
    /// rescue
    ///   nil
    /// end
    /// ```
    fn compatible_encoding(obj1: &impl Object, obj2: &impl Object) -> AnyObject {
        encoding::compatible_encoding(obj1.value(), obj2.value()).into()
    }
}

impl From<Value> for RString {
    fn from(value: Value) -> Self {
        RString { value }
    }
}

impl From<String> for RString {
    fn from(string: String) -> Self {
        Self::new_utf8(string.as_str())
    }
}

impl From<&'static str> for RString {
    fn from(string: &'static str) -> Self {
        Self::new_utf8(string)
    }
}

impl Into<Value> for RString {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for RString {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

/// Implicit or `nil` conversion
///
/// # Examples
///
/// ```
/// use rutie::{RString, Fixnum, VM, TryConvert, NilClass, Object};
/// # VM::init();
///
/// let four = Fixnum::new(4);
/// let result = RString::try_convert(four.to_any_object());
///
/// assert_eq!(result, Err(NilClass::new()));
///
/// let five = RString::new_utf8("5");
/// let result2 = RString::try_convert(five.to_any_object());
///
/// if let Ok(r) = result2 {
///   assert_eq!(r.to_str(), "5")
/// } else {
///   unreachable!()
/// }
///
/// ```
///
/// Ruby:
///
/// ```ruby
/// four = 4
/// result = String.try_convert(four)
///
/// result == nil
///
/// five = "5"
/// result = String.try_convert(five)
///
/// result == "5"
/// ```
impl TryConvert<AnyObject> for RString {
    type Nil = NilClass;

    fn try_convert(obj: AnyObject) -> Result<Self, NilClass> {
        let result = string::method_to_str(obj.value());

        if result.is_nil() {
            Err(NilClass::from(result))
        } else {
            Ok(Self::from(result))
        }
    }
}

impl Object for RString {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for RString {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.value().ty() == ValueType::RString
    }

    fn error_message() -> &'static str {
        "Error converting to String"
    }
}

impl PartialOrd for RString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.compare(other))
    }
}

impl PartialEq for RString {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{CodeRange, Encoding, EncodingSupport, Object, RString, VM};
    use std::{
        cmp::Ordering,
        panic::{self, AssertUnwindSafe},
    };

    #[test]
    fn test_string_slicing_and_building() {
        crate::on_ruby_thread(|| {
            let string = RString::new_utf8("añb");

            // "ñ" is two bytes: byte 1..3.
            assert_eq!(string.byte_slice(1, 2).unwrap().to_str(), "ñ");
            assert!(string.byte_slice(usize::MAX, 2).is_none());
            assert!(string.byte_slice(0, usize::MAX).is_none());
            assert_eq!(string.substr(1, 1).unwrap().to_str(), "ñ");
            assert!(string.substr(0, -1).is_none());

            let mut buffer = RString::with_capacity(8);
            buffer.concat("x");
            assert_eq!(buffer.encoding().name(), "UTF-8");
            assert_eq!(buffer.times(3).to_str(), "xxx");
            assert_eq!(buffer.plus(&string).to_str(), "xañb");

            let mut copy = RString::new_utf8("abcdef");
            copy.truncate(3);
            assert_eq!(copy.to_str(), "abc");
            copy.replace(&RString::new_utf8("zz"));
            assert_eq!(copy.to_str(), "zz");

            assert_eq!(copy.compare(&RString::new_utf8("zz")), Ordering::Equal);
            assert!(RString::new_utf8("a") < RString::new_utf8("b"));

            let frozen = RString::new_utf8("frozen").freeze();
            let result = VM::protect(|| {
                let mut frozen = frozen.dup().freeze();
                frozen.truncate(1);
                frozen.into()
            });
            assert!(result.is_err());
            VM::error_pop().unwrap();
        });
    }

    #[test]
    fn test_string_parsing_and_encoding() {
        crate::on_ruby_thread(|| {
            assert_eq!(RString::new_utf8("-12x").to_i(10).to_i64(), -12);
            assert_eq!(
                RString::new_utf8("0b11").parse_integer(0).unwrap().to_i64(),
                3
            );
            assert!(RString::new_utf8("").parse_integer(10).is_err());
            assert_eq!(RString::new_utf8(" 3.25 ").parse_float().unwrap(), 3.25);
            assert!(RString::new_utf8("x").parse_float().is_err());
            assert_eq!(RString::new_utf8(".5").to_f(), 0.5);

            let huge = RString::new_utf8("123456789012345678901234567890")
                .parse_integer(10)
                .unwrap();
            assert_eq!(huge.as_string().to_str(), "123456789012345678901234567890");

            let broken = RString::from_bytes(b"a\xE2\x82", &Encoding::utf8());
            assert_eq!(broken.coderange(), CodeRange::Broken);
            let fixed = broken.scrub(Some("*"));
            assert_eq!(fixed.to_str(), "a*");
            assert_eq!(fixed.coderange(), CodeRange::SevenBit);

            // A valid string is copied, not returned as is.
            let valid = RString::new_utf8("ok");
            assert!(!valid.scrub(None).is_equal(&valid));

            assert_eq!(RString::new_utf8("a b").split(" ").length(), 2);
            assert_eq!(
                RString::new_utf8("0123456789").ellipsize(5).to_str(),
                "01..."
            );
        });
    }

    #[test]
    fn test_with_locked_bytes() {
        crate::on_ruby_thread(|| {
            let string = RString::new_utf8("data");

            let nested = string.with_locked_bytes(|outer| {
                string.with_locked_bytes(|inner| outer.len() + inner.len())
            });
            assert_eq!(nested, 8);

            // A Ruby exception out of the closure still unlocks.
            let result = VM::protect(|| {
                string.with_locked_bytes(|_| unsafe { VM::eval_str("raise 'inside'") })
            });
            assert!(result.is_err());
            VM::error_pop().unwrap();
            assert!(string
                .protect_send("<<", &[RString::new_utf8("!").into()])
                .is_ok());

            // So does a panic, which keeps unwinding as a panic.
            let panicked = panic::catch_unwind(AssertUnwindSafe(|| {
                string.with_locked_bytes(|_| panic!("inside"));
            }));
            assert!(panicked.is_err());
            assert!(string
                .protect_send("<<", &[RString::new_utf8("!").into()])
                .is_ok());
            assert_eq!(string.to_str(), "data!!");
        });
    }

    #[test]
    fn test_string_sizes_and_unchecked_access() {
        crate::on_ruby_thread(|| {
            let string = RString::new_utf8("héllo");
            assert_eq!(string.bytesize(), 6);
            assert_eq!(string.count_chars(), 5);
            assert_eq!(string.codepoints().length(), 5);

            let with_room = RString::with_capacity(64);
            assert!(with_room.capacity() >= 64);

            assert_eq!(string.to_str_unchecked(), "héllo");
            assert_eq!(string.to_string_unchecked(), "héllo");
            assert_eq!(string.to_bytes_unchecked(), "héllo".as_bytes());
            assert_eq!(string.to_vec_u8_unchecked(), "héllo".as_bytes().to_vec());

            let ascii = RString::new_usascii_unchecked("plain");
            assert_eq!(ascii.encoding().name(), "ASCII-8BIT");
            assert_eq!(ascii.to_str(), "plain");

            let binary = VM::eval(r#""\xFF\xFE".b"#)
                .unwrap()
                .try_convert_to::<RString>()
                .unwrap();
            assert_eq!(binary.bytesize(), 2);
            assert_eq!(binary.to_bytes_unchecked(), &[0xFF, 0xFE]);
        });
    }
}
