use std::convert::From;

use binding::{encoding, string, vm};
use binding::class::is_frozen;
use types::{Value, ValueType};

use {
  Object,
  VerifiedObject,
  NilClass,
  AnyObject,
  EncodingSupport,
  Encoding,
  AnyException,
  Exception,
  Boolean,
  TryConvert,
  Hash,
  Array,
  CodepointIterator,
  Integer,
};

/// `String`
#[derive(Debug)]
pub struct RString {
    value: Value,
}

impl RString {
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
    #[deprecated(since="0.3.2", note="please use `new_usascii_unchecked` or `new_utf8` instead")]
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
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_usascii_unchecked("Hello, World!");
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
    /// ```no_run
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
    /// ```no_run
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
    /// ```no_run
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
    /// ```no_run
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
    /// ```no_run
    /// use rutie::{Object, RString, Array, Fixnum, Encoding, EncodingSupport, VM};
    /// # VM::init();
    /// # VM::init_loadpath(); // Needed for alternate encodings
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// let string = RString::from_bytes(b"foo\x93_a", &Encoding::find("cp932").unwrap());
    ///
    /// let codepoints: Array = [102, 111, 111, 37727, 97].
    ///   into_iter().map(|cp| Fixnum::new(*cp as i64).to_any_object()).collect();
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
    #[cfg(not(target_os = "windows"))]
    pub fn codepoints(&self) -> Array {
        Array::from(string::codepoints(self.value()))
    }

    #[cfg(target_os = "windows")]
    pub fn codepoints(&self) -> Array {
        CodepointIterator::new(self).
            into_iter().
            map(|n| Integer::new(n as i64).to_any_object()).
            collect()
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
        Encoding::from(encoding::from_encoding_index(encoding::enc_get_index(self.value())))
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
    // TODO: See comment in method definition below.
    // ```
    // use rutie::{RString, VM, EncodingSupport, Encoding, Object, Exception};
    // # VM::init();
    //
    // let mut string = RString::new_utf8("Hello");
    // string.force_encoding(Encoding::utf8());
    // string.freeze();
    // let result = string.force_encoding(Encoding::us_ascii());
    //
    // match result {
    //     Ok(_) => assert_eq!("This is a bad path.", "You shouldn't get this message."),
    //     Err(happy_path) => assert_eq!(happy_path.message(), "can\'t modify frozen String"),
    // }
    // ```
    fn force_encoding(&mut self, enc: Encoding) -> Result<Self, AnyException> {
        if string::is_lockedtmp(self.value()) {
            return Err(AnyException::new("RuntimeError", Some("can't modify string; temporarily locked")));
        }

        // TODO: Ruby 2.3.7 & 2.4.4 fail on CI servers for all OSes because of the `is_frozen` check
        // here.  Works with Ruby 2.5.1 everywhere though and on my machine or Docker with all
        // versions.  May be CI binaries related but that doesn't explain why `is_frozen` works
        // elsewhere on the CI same systems.  Either get this to work on the CI servers or wait
        // till EOL for Ruby 2.3 and 2.4.
        //
        // if self.is_frozen() {
        //     return Err(AnyException::new("FrozenError", Some("can't modify frozen String")));
        // }

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
                let ecflags = encoding::econv_prepare_opts(options.value(), &nil);

                encoding::encode(
                    self.value(),
                    enc.value(),
                    ecflags,
                    options.value()
                )
            },
            None => {
                encoding::encode(self.value(), enc.value(), 0, nil)
            },
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
        let result = self.send("valid_encoding?", None);
        result.try_convert_to::<Boolean>().unwrap().to_bool()
    }
}

impl From<Value> for RString {
    fn from(value: Value) -> Self {
        RString { value: value }
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
            Err( NilClass::from(result) )
        } else {
            Ok( Self::from(result) )
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

impl PartialEq for RString {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}
