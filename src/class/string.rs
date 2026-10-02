use std::{
    cmp::Ordering,
    convert::From,
    ffi::CStr,
    panic::{self, AssertUnwindSafe},
};

use crate::{
    binding::{class::is_frozen, encoding, string, vm},
    rubysys::encoding::{ENC_CODERANGE_7BIT, ENC_CODERANGE_VALID},
    types::{Value, ValueType},
    AnyException, AnyObject, Array, Boolean, CodepointIterator, Encoding, EncodingSupport,
    Exception, Hash, Integer, NilClass, Object, TryConvert, VerifiedObject,
};

// Only path separators (`/`, and `\` on Windows).
fn is_separators(bytes: &[u8]) -> bool {
    !bytes.is_empty()
        && bytes
            .iter()
            .all(|&byte| byte == b'/' || (cfg!(windows) && byte == b'\\'))
}

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

    /// Creates a string from `bytes` read from outside Ruby (a file, the
    /// environment, ...), in the default external encoding, transcoded to
    /// the default internal encoding when one is set
    /// (`rb_external_str_new`). When the external encoding is US-ASCII,
    /// non-ASCII bytes make a binary (`ASCII-8BIT`) string.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, EncodingSupport, RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_external(b"input");
    ///
    /// assert_eq!(string.to_str(), "input");
    /// assert_eq!(string.encoding(), Encoding::default_external());
    /// ```
    pub fn new_external(bytes: &[u8]) -> Self {
        Self::from(string::new_external(bytes))
    }

    /// Creates a string from `bytes` read from outside Ruby in encoding
    /// `enc`, transcoded to the default internal encoding when one is set
    /// (`rb_external_str_new_with_enc`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, EncodingSupport, RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_external_with_encoding("é".as_bytes(), &Encoding::utf8());
    ///
    /// assert_eq!(string.to_str(), "é");
    /// assert_eq!(string.encoding(), Encoding::utf8());
    /// ```
    pub fn new_external_with_encoding(bytes: &[u8], enc: &Encoding) -> Self {
        Self::from(string::new_external_with_encoding(bytes, enc.value()))
    }

    /// Creates a string from `bytes` in the locale's encoding
    /// (`rb_locale_str_new`), handled as
    /// [`new_external`](#method.new_external) does.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, EncodingSupport, RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_locale(b"text");
    ///
    /// assert_eq!(string.to_str(), "text");
    /// assert_eq!(string.encoding(), Encoding::locale());
    /// ```
    pub fn new_locale(bytes: &[u8]) -> Self {
        Self::from(string::new_locale(bytes))
    }

    /// Creates a string from `bytes` in the filesystem encoding, as for a
    /// file name (`rb_filesystem_str_new`), handled as
    /// [`new_external`](#method.new_external) does.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, EncodingSupport, RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_filesystem(b"notes.txt");
    ///
    /// assert_eq!(string.to_str(), "notes.txt");
    /// assert_eq!(string.encoding(), Encoding::filesystem());
    /// ```
    pub fn new_filesystem(bytes: &[u8]) -> Self {
        Self::from(string::new_filesystem(bytes))
    }

    /// Creates a UTF-8 string that uses the bytes of `string` in place
    /// instead of copying them (`rb_utf8_str_new_static`). Ruby copies them
    /// only if the string is modified.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, EncodingSupport, RString, VM};
    /// use std::ffi::CStr;
    /// # VM::init();
    ///
    /// let text = CStr::from_bytes_with_nul(b"static text\0").unwrap();
    /// let mut string = RString::new_static(text);
    ///
    /// assert_eq!(string.to_str(), "static text");
    /// assert_eq!(string.encoding(), Encoding::utf8());
    ///
    /// string.concat("!");
    /// assert_eq!(string.to_str(), "static text!");
    /// ```
    pub fn new_static(string: &'static CStr) -> Self {
        Self::from(string::new_static_utf8(string))
    }

    /// Creates a string in encoding `enc` that uses the bytes of `string` in
    /// place instead of copying them (`rb_enc_str_new_static`). Strings in
    /// ASCII-incompatible encodings, such as UTF-16, are copied, since Ruby
    /// would read past the NUL looking for their wider terminator.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, EncodingSupport, RString, VM};
    /// use std::ffi::CStr;
    /// # VM::init();
    ///
    /// let bytes = CStr::from_bytes_with_nul(b"\xFF\xFE\0").unwrap();
    /// let string = RString::new_static_with_encoding(bytes, &Encoding::ascii_8bit());
    ///
    /// assert_eq!(string.to_bytes_unchecked(), b"\xFF\xFE");
    /// assert_eq!(string.encoding(), Encoding::ascii_8bit());
    /// ```
    pub fn new_static_with_encoding(string: &'static CStr, enc: &Encoding) -> Self {
        Self::from(string::new_static(string, enc.value()))
    }

    /// Returns the frozen, deduplicated UTF-8 string with the contents
    /// `string`, the one Ruby uses for equal string literals
    /// (`rb_enc_interned_str`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// let first = RString::interned("shared");
    /// let second = RString::interned("shared");
    ///
    /// assert!(first.is_frozen());
    /// assert!(first.equals(&second));
    /// assert_eq!(first.value().value, second.value().value);
    /// ```
    pub fn interned(string: &str) -> Self {
        Self::interned_bytes(string.as_bytes(), &Encoding::utf8())
    }

    /// Returns the frozen, deduplicated string with the contents `bytes` in
    /// encoding `enc` (`rb_enc_interned_str`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, EncodingSupport, Object, RString, VM};
    /// # VM::init();
    ///
    /// let interned = RString::interned_bytes(b"abc", &Encoding::us_ascii());
    ///
    /// assert!(interned.is_frozen());
    /// assert_eq!(interned.encoding(), Encoding::us_ascii());
    /// ```
    pub fn interned_bytes(bytes: &[u8], enc: &Encoding) -> Self {
        Self::from(string::interned(bytes, enc.value()))
    }

    /// Returns the frozen, deduplicated copy of the string (Ruby's `-str`,
    /// `rb_str_to_interned_str`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("dedup me");
    /// let interned = string.to_interned();
    ///
    /// assert!(interned.is_frozen());
    /// assert!(!string.is_frozen());
    /// assert_eq!(interned.value().value, RString::interned("dedup me").value().value);
    /// ```
    pub fn to_interned(&self) -> RString {
        RString::from(string::to_interned(self.value()))
    }

    /// Appends `other`, choosing the result's encoding the way Ruby's `<<`
    /// does (`rb_str_append`). Returns the error (such as
    /// `Encoding::CompatibilityError` or `FrozenError`) if it cannot.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, RString, VM};
    /// # VM::init();
    ///
    /// let mut string = RString::new_utf8("crab: ");
    /// string.append(&RString::new_utf8("🦀")).unwrap();
    ///
    /// assert_eq!(string.to_str(), "crab: 🦀");
    ///
    /// let mut binary = RString::from_bytes(b"\xFF", &Encoding::ascii_8bit());
    /// assert!(binary.append(&RString::new_utf8("é")).is_err());
    /// ```
    pub fn append(&mut self, other: &RString) -> Result<(), AnyException> {
        let (string, other) = (self.value(), other.value());

        vm::protect_value(|| string::append(string, other))
            .map(|_| ())
            .map_err(AnyException::from)
    }

    /// Appends the character with code point `code` in the string's
    /// encoding (Ruby's `<<` with an Integer, `rb_str_concat`). Returns the
    /// `RangeError` if the encoding cannot represent it.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let mut string = RString::new_utf8("caf");
    /// string.concat_codepoint(0xE9).unwrap();
    ///
    /// assert_eq!(string.to_str(), "café");
    /// assert!(string.concat_codepoint(0xD800).is_err());
    /// ```
    pub fn concat_codepoint(&mut self, code: u32) -> Result<(), AnyException> {
        let (string, code) = (self.value(), Integer::from(code).value());

        vm::protect_value(|| string::concat_object(string, code))
            .map(|_| ())
            .map_err(AnyException::from)
    }

    /// Returns `true` if the two strings' encodings allow comparing their
    /// bytes, such as two ASCII-only strings in ASCII-compatible encodings
    /// (`rb_str_comparable`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, RString, VM};
    /// # VM::init();
    ///
    /// let utf8 = RString::new_utf8("é");
    /// let ascii = RString::new_utf8("abc");
    /// let binary = RString::from_bytes(b"\xFF", &Encoding::ascii_8bit());
    ///
    /// assert!(utf8.is_comparable(&ascii));
    /// assert!(!utf8.is_comparable(&binary));
    /// ```
    pub fn is_comparable(&self, other: &RString) -> bool {
        string::is_comparable(self.value(), other.value())
    }

    /// Returns `true` if the strings have the same bytes in comparable
    /// encodings, as Ruby's `eql?` and Hash keys compare them
    /// (`rb_str_hash_cmp`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, RString, VM};
    /// # VM::init();
    ///
    /// let utf8 = RString::new_utf8("é");
    /// let binary = RString::from_bytes("é".as_bytes(), &Encoding::ascii_8bit());
    ///
    /// assert!(utf8.is_eql(&RString::new_utf8("é")));
    /// assert!(!utf8.is_eql(&binary));
    /// ```
    pub fn is_eql(&self, other: &RString) -> bool {
        string::is_eql(self.value(), other.value())
    }

    /// Makes room for at least `additional` more bytes without changing the
    /// contents (`rb_str_modify_expand`). Returns the error if the string
    /// cannot be modified.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let mut string = RString::new_utf8("abc");
    /// string.reserve(1000).unwrap();
    ///
    /// assert!(string.capacity() >= 1003);
    /// assert_eq!(string.to_str(), "abc");
    /// ```
    pub fn reserve(&mut self, additional: usize) -> Result<(), AnyException> {
        let string = self.value();

        vm::protect_value(|| {
            string::modify_expand(string, additional);

            string
        })
        .map(|_| ())
        .map_err(AnyException::from)
    }

    /// Removes the first `count` bytes, or every byte if there are fewer
    /// (`rb_str_drop_bytes`). Returns the error if the string cannot be
    /// modified.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// let mut string = RString::new_utf8("prefix:value");
    /// string.drop_bytes(7).unwrap();
    ///
    /// assert_eq!(string.to_str(), "value");
    ///
    /// string.drop_bytes(100).unwrap();
    /// assert_eq!(string.to_str(), "");
    ///
    /// let mut frozen = RString::new_utf8("frozen").freeze();
    /// assert!(frozen.drop_bytes(1).is_err());
    /// ```
    pub fn drop_bytes(&mut self, count: usize) -> Result<(), AnyException> {
        let string = self.value();

        vm::protect_value(|| string::drop_bytes(string, count))
            .map(|_| ())
            .map_err(AnyException::from)
    }

    /// Replaces `len` characters starting at character `start` (counting
    /// from the end when negative) with `other` (Ruby's
    /// `str[start, len] = other`, `rb_str_update`). Returns the
    /// `IndexError` when `start` is outside the string.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let mut string = RString::new_utf8("héllo world");
    /// string.splice(0, 5, &RString::new_utf8("goodbye")).unwrap();
    ///
    /// assert_eq!(string.to_str(), "goodbye world");
    ///
    /// string.splice(-5, 5, &RString::new_utf8("crab")).unwrap();
    /// assert_eq!(string.to_str(), "goodbye crab");
    ///
    /// assert!(string.splice(100, 1, &RString::new_utf8("x")).is_err());
    /// ```
    pub fn splice(&mut self, start: i64, len: usize, other: &RString) -> Result<(), AnyException> {
        let (string, other) = (self.value(), other.value());

        vm::protect_value(|| {
            string::update(string, start, len, other);

            string
        })
        .map(|_| ())
        .map_err(AnyException::from)
    }

    /// Returns the byte offset of character `char_index`, or the byte size
    /// when the string is shorter (`rb_str_offset`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("añb");
    ///
    /// assert_eq!(string.byte_offset(1), 1);
    /// assert_eq!(string.byte_offset(2), 3);
    /// assert_eq!(string.byte_offset(10), 4);
    /// ```
    pub fn byte_offset(&self, char_index: usize) -> usize {
        string::byte_offset(self.value(), char_index)
    }

    /// Returns how many characters the first `byte_offset` bytes hold (all
    /// of the string when it is shorter, `rb_str_sublen`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("añb");
    ///
    /// assert_eq!(string.char_index(3), 2);
    /// assert_eq!(string.char_index(100), 3);
    /// ```
    pub fn char_index(&self, byte_offset: usize) -> usize {
        string::char_index(self.value(), byte_offset)
    }

    /// Returns the successor of the string, incrementing its rightmost
    /// alphanumeric (Ruby's `succ`, `rb_str_succ`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// assert_eq!(RString::new_utf8("az").succ().to_str(), "ba");
    /// assert_eq!(RString::new_utf8("zz99").succ().to_str(), "aaa00");
    /// ```
    pub fn succ(&self) -> RString {
        RString::from(string::succ(self.value()))
    }

    /// Returns the string with its non-printable characters escaped, in
    /// double quotes (Ruby's `dump`, `rb_str_dump`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// assert_eq!(RString::new_utf8("tab\t é").dump().to_str(), r#""tab\t \u00E9""#);
    /// ```
    pub fn dump(&self) -> RString {
        RString::from(string::dump(self.value()))
    }

    /// Returns the `Encoding::CompatibilityError` if the string's encoding is
    /// not ASCII compatible, such as UTF-16 (`rb_must_asciicompat`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, RString, VM};
    /// # VM::init();
    /// VM::init_loadpath(); // Needed for the encoding database
    /// VM::require("enc/encdb");
    ///
    /// assert!(RString::new_utf8("text").check_ascii_compatible().is_ok());
    ///
    /// let utf16 = RString::from_bytes(b"a\x00", &Encoding::find("UTF-16LE").unwrap());
    /// assert!(utf16.check_ascii_compatible().is_err());
    /// ```
    pub fn check_ascii_compatible(&self) -> Result<(), AnyException> {
        let string = self.value();

        vm::protect_value(|| {
            string::must_ascii_compatible(string);

            string
        })
        .map(|_| ())
        .map_err(AnyException::from)
    }

    /// Returns the string converted to the default external encoding, as
    /// for writing it outside Ruby (`rb_str_export`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, EncodingSupport, RString, VM};
    /// # VM::init();
    ///
    /// let exported = RString::new_utf8("out").export();
    ///
    /// assert_eq!(exported.to_str(), "out");
    /// assert_eq!(exported.encoding(), Encoding::default_external());
    /// ```
    pub fn export(&self) -> RString {
        RString::from(string::export(self.value()))
    }

    /// Converts `object` to a `String` with its `to_str` method, as Ruby
    /// does where a string is expected (`rb_str_to_str`). Unlike
    /// [`RString::convert`](#method.convert), `to_s` is not used; returns the
    /// `TypeError` when `object` has no `to_str`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, RString, VM};
    /// # VM::init();
    ///
    /// let path = VM::eval("Struct.new(:to_str).new('a/path')").unwrap();
    ///
    /// assert_eq!(RString::implicit_convert(&path).unwrap().to_str(), "a/path");
    /// assert!(RString::implicit_convert(&Fixnum::new(1)).is_err());
    /// ```
    pub fn implicit_convert<T: Object>(object: &T) -> Result<Self, AnyException> {
        let object = object.value();

        vm::protect_value(|| string::to_str(object))
            .map(Self::from)
            .map_err(AnyException::from)
    }

    /// Formats `arguments` with the format string `format`, as Ruby's
    /// `format` (`Kernel#sprintf`, `rb_f_sprintf`) does. Returns the error
    /// (such as `ArgumentError`) for a bad format or arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Float, Object, RString, VM};
    /// # VM::init();
    ///
    /// let formatted = RString::format(
    ///     "%-5s|%03d|%.2f",
    ///     &[RString::new_utf8("ab").into(), Fixnum::new(7).into(), Float::new(3.14159).into()],
    /// )
    /// .unwrap();
    ///
    /// assert_eq!(formatted.to_str(), "ab   |007|3.14");
    /// assert!(RString::format("%d", &[]).is_err());
    /// ```
    pub fn format(format: &str, arguments: &[AnyObject]) -> Result<RString, AnyException> {
        let format = string::new_utf8(format);
        let mut values = Vec::with_capacity(arguments.len() + 1);
        values.push(format);
        values.extend(arguments.iter().map(Object::value));

        let result = vm::protect_value(|| string::format(&values))
            .map(RString::from)
            .map_err(AnyException::from);

        // `values` is on the heap, out of the GC's sight: keep `format` on
        // the stack until Ruby is done with it.
        unsafe { std::ptr::read_volatile(&format) };

        result
    }

    /// Returns the last component of the string as a path, without trailing
    /// separators, as Ruby's `File.basename` does (`ruby_enc_find_basename`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// assert_eq!(RString::new_utf8("/home/crab/notes.txt").path_basename().to_str(), "notes.txt");
    /// assert_eq!(RString::new_utf8("dir/sub/").path_basename().to_str(), "sub");
    /// assert_eq!(RString::new_utf8("/").path_basename().to_str(), "/");
    /// ```
    pub fn path_basename(&self) -> RString {
        let bytes = self.to_bytes_unchecked();

        match string::path_basename(bytes, self.encoding().value()) {
            Some((start, len, _)) => self.byte_slice(start, len).unwrap(),
            // A UNC root on Windows (such as `//`): Ruby answers with a `"/"`
            // of its own instead of a part of the string.
            None if is_separators(bytes) => RString::from_bytes(b"/", &self.encoding()),
            None => self.byte_slice(0, bytes.len()).unwrap(),
        }
    }

    /// Returns the extension (with its dot) of the string's last path
    /// component, or an empty string, as Ruby's `File.extname` does
    /// (`ruby_enc_find_extname`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// assert_eq!(RString::new_utf8("/src/lib.rs").path_extname().to_str(), ".rs");
    /// assert_eq!(RString::new_utf8("archive.tar.gz").path_extname().to_str(), ".gz");
    /// assert_eq!(RString::new_utf8(".profile").path_extname().to_str(), "");
    /// ```
    pub fn path_extname(&self) -> RString {
        let bytes = self.to_bytes_unchecked();

        match string::path_extname(bytes, self.encoding().value()) {
            Some((start, len)) => self.byte_slice(start, len).unwrap(),
            None => self.byte_slice(0, 0).unwrap(),
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
    /// let result = RString::from_bytes(&[0xff], &Encoding::utf8());
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
    /// result = "\xff".force_encoding(Encoding::UTF_8)
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
    use crate::{CodeRange, Encoding, EncodingSupport, Exception, Object, RString, VM};
    use std::{
        cmp::Ordering,
        ffi::CStr,
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

    fn ruby_string(code: &str) -> RString {
        VM::eval(code).unwrap().try_convert_to::<RString>().unwrap()
    }

    #[test]
    fn test_string_constructors_and_interning() {
        crate::on_ruby_thread(|| {
            let external = RString::new_external(b"plain");
            assert_eq!(external.to_str(), "plain");
            assert_eq!(external.encoding(), Encoding::default_external());

            // Non-ASCII bytes cannot be US-ASCII: Ruby makes them binary.
            let external = RString::new_external("é".as_bytes());
            assert_eq!(external.to_bytes_unchecked(), "é".as_bytes());
            if Encoding::default_external() == Encoding::us_ascii() {
                assert_eq!(external.encoding(), Encoding::ascii_8bit());
            }

            assert_eq!(RString::new_locale(b"x").encoding(), Encoding::locale());
            assert_eq!(
                RString::new_filesystem(b"x").encoding(),
                Encoding::filesystem()
            );
            let tagged = RString::new_external_with_encoding(b"\xFF", &Encoding::ascii_8bit());
            assert_eq!(tagged.encoding(), Encoding::ascii_8bit());

            static BYTES: [u8; 4] = [1, 2, 3, 0];
            let bytes = CStr::from_bytes_with_nul(&BYTES).unwrap();
            let mut fixed = RString::new_static_with_encoding(bytes, &Encoding::ascii_8bit());
            fixed.concat_bytes(&[4], &Encoding::ascii_8bit()).unwrap();
            assert_eq!(fixed.to_bytes_unchecked(), &[1, 2, 3, 4]);
            // Modifying the string copied the bytes rather than writing them.
            assert_eq!(BYTES, [1, 2, 3, 0]);

            let literal = RString::new_static(CStr::from_bytes_with_nul(b"lit\0").unwrap());
            assert!(!literal.is_frozen());
            assert_eq!(literal.to_str(), "lit");

            // Copied: UTF-16 needs a two-byte terminator.
            VM::init_loadpath();
            VM::require("enc/encdb");
            let utf16 = Encoding::find("UTF-16LE").unwrap();
            let wide = RString::new_static_with_encoding(
                CStr::from_bytes_with_nul(b"a\x01\0").unwrap(),
                &utf16,
            );
            assert_eq!(wide.to_bytes_unchecked(), b"a\x01");
            assert_eq!(wide.encoding(), utf16);
            assert_eq!(wide.to_string_unchecked(), "a\u{1}");

            let interned = RString::interned("rutie interned");
            assert!(interned.is_frozen());
            assert_eq!(interned.encoding(), Encoding::utf8());
            assert_eq!(
                RString::new_utf8("rutie interned")
                    .to_interned()
                    .value()
                    .value,
                interned.value().value
            );
            // Equal bytes in another encoding are another string.
            let binary = RString::interned_bytes(b"rutie interned", &Encoding::ascii_8bit());
            assert_ne!(binary.value().value, interned.value().value);
            assert_eq!(binary.encoding(), Encoding::ascii_8bit());
        });
    }

    #[test]
    fn test_string_editing() {
        crate::on_ruby_thread(|| {
            let mut string = RString::new_utf8("ab");
            string.append(&RString::new_utf8("ç")).unwrap();
            string.concat_codepoint(0x1F980).unwrap();
            assert_eq!(string.to_str(), "abç🦀");

            let mut ascii = RString::from_bytes(b"x", &Encoding::us_ascii());
            ascii.append(&RString::new_utf8("é")).unwrap();
            assert_eq!(ascii.encoding(), Encoding::utf8());
            let mut binary = RString::from_bytes(b"x", &Encoding::ascii_8bit());
            binary.concat_codepoint(0xFF).unwrap();
            assert_eq!(binary.to_bytes_unchecked(), b"x\xFF");
            assert!(binary.concat_codepoint(0x100).is_err());

            let mut frozen = RString::new_utf8("f").freeze();
            assert!(frozen.append(&RString::new_utf8("x")).is_err());
            assert!(frozen.reserve(10).is_err());
            assert!(frozen.splice(0, 1, &RString::new_utf8("x")).is_err());
            assert!(frozen.concat_codepoint(0x41).is_err());

            let mut buffer = RString::new_utf8("");
            buffer.reserve(4096).unwrap();
            assert!(buffer.capacity() >= 4096);
            assert_eq!(buffer.bytesize(), 0);

            let mut multibyte = RString::new_utf8("éa");
            multibyte.drop_bytes(1).unwrap();
            assert_eq!(multibyte.coderange(), CodeRange::Broken);
            multibyte.drop_bytes(usize::MAX).unwrap();
            assert_eq!(multibyte.bytesize(), 0);

            let mut spliced = RString::new_utf8("añb");
            spliced.splice(1, 1, &RString::new_utf8("NN")).unwrap();
            assert_eq!(spliced.to_str(), "aNNb");
            spliced
                .splice(4, usize::MAX, &RString::new_utf8("!"))
                .unwrap();
            assert_eq!(spliced.to_str(), "aNNb!");
            spliced
                .splice(0, usize::MAX, &RString::new_utf8(""))
                .unwrap();
            assert_eq!(spliced.to_str(), "");
            assert!(spliced.splice(-1, 0, &RString::new_utf8("x")).is_err());
            let binary = RString::from_bytes(b"\xFF", &Encoding::ascii_8bit());
            let mut utf8 = RString::new_utf8("é");
            assert!(utf8.splice(0, 0, &binary).is_err());
        });
    }

    #[test]
    fn test_string_offsets_and_comparison() {
        crate::on_ruby_thread(|| {
            let string = RString::new_utf8("aé🦀b");
            let offsets: Vec<usize> = (0..6).map(|i| string.byte_offset(i)).collect();
            assert_eq!(offsets, [0, 1, 3, 7, 8, 8]);
            assert_eq!(string.byte_offset(usize::MAX), 8);
            let indexes: Vec<usize> = [0, 1, 3, 7, 8, 9]
                .iter()
                .map(|&b| string.char_index(b))
                .collect();
            assert_eq!(indexes, [0, 1, 2, 3, 4, 4]);
            assert_eq!(string.char_index(usize::MAX), 4);
            assert_eq!(RString::new_utf8("").byte_offset(3), 0);

            let ascii = RString::new_utf8("abc");
            let ascii_binary = RString::from_bytes(b"abc", &Encoding::ascii_8bit());
            assert!(ascii.is_comparable(&ascii_binary));
            assert!(ascii.is_eql(&ascii_binary));
            assert!(RString::new_utf8("")
                .is_comparable(&RString::from_bytes(b"\xFF", &Encoding::ascii_8bit())));
            assert!(!ascii.is_eql(&RString::new_utf8("abd")));

            assert_eq!(RString::new_utf8("Az").succ().to_str(), "Ba");
            assert_eq!(RString::new_utf8("").succ().to_str(), "");
            assert_eq!(RString::new_utf8("a\0\"").dump().to_str(), r#""a\x00\"""#);
        });
    }

    #[test]
    fn test_string_conversions_and_format() {
        crate::on_ruby_thread(|| {
            assert!(RString::new_utf8("x").check_ascii_compatible().is_ok());
            let utf16 =
                ruby_string("'x'.encode('UTF-16LE') rescue 'x'.dup.force_encoding('UTF-16LE')");
            assert!(utf16.check_ascii_compatible().is_err());

            let exported = RString::new_utf8("e").export();
            assert_eq!(exported.encoding(), Encoding::default_external());

            let string = RString::new_utf8("same");
            let converted = RString::implicit_convert(&string).unwrap();
            assert!(converted.is_equal(&string));
            let error = RString::implicit_convert(&crate::Symbol::new("sym")).unwrap_err();
            assert!(crate::Class::from_existing("TypeError").case_equals(&error));

            let formatted = RString::format(
                "%s and %p: %x",
                &[
                    RString::new_utf8("str").into(),
                    crate::Symbol::new("sym").into(),
                    crate::Fixnum::new(255).into(),
                ],
            )
            .unwrap();
            assert_eq!(formatted.to_str(), "str and :sym: ff");
            assert!(RString::format("%d", &[RString::new_utf8("x").into()]).is_err());
            assert_eq!(RString::format("100%%", &[]).unwrap().to_str(), "100%");
        });
    }

    #[test]
    fn test_string_paths_match_file() {
        crate::on_ruby_thread(|| {
            let paths = [
                "",
                "/",
                "//",
                "a",
                "a/",
                "/a/b.rb",
                "a/b/",
                "dir/.hidden",
                ".hidden.rb",
                "a.tar.gz",
                "a/b.",
                "a/.b.c",
                "x//y//",
                "é/ü.ñ",
                "a/b c.d e",
                "...",
                "a..b",
            ];

            for path in paths.iter() {
                let string = RString::new_utf8(path);
                let basename = ruby_string(&format!("File.basename({:?})", path));
                let extname = ruby_string(&format!("File.extname({:?})", path));

                assert_eq!(
                    string.path_basename().to_str(),
                    basename.to_str(),
                    "basename of {:?}",
                    path
                );
                assert_eq!(
                    string.path_extname().to_str(),
                    extname.to_str(),
                    "extname of {:?}",
                    path
                );
            }

            // Bytes after a NUL are not part of the path.
            let nul = RString::from_bytes(b"a.b\0.c", &Encoding::utf8());
            assert_eq!(nul.path_extname().to_str(), ".b");
        });
    }
}
