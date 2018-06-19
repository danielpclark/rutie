use std::convert::From;

use binding::string;
use types::{Value, ValueType};

use {Object, VerifiedObject};

/// `String`
#[derive(Debug, PartialEq)]
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
    pub fn new(string: &str) -> Self {
        Self::from(string::new(string))
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

    /// Retrieves underlying Rust `String` from Ruby `String` object.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new("Hello, World!");
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
    /// let string = RString::new("Hello,\0World!");
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
    /// let string = RString::new("Hello,\0World!");
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
    /// let string = RString::new("Hello,\0World!");
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
    /// let string = RString::new("Hello,\0World!");
    ///
    /// assert_eq!(string.to_bytes_unchecked(), b"Hello,\0World!");
    /// ```
    pub fn to_bytes_unchecked(&self) -> &[u8] {
        let value = self.value();

        string::value_to_bytes_unchecked(value)
    }

    /// Returns the length of the string in bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new("Hello, World!");
    /// let utf8_string = RString::new("⓯");
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
}

impl From<Value> for RString {
    fn from(value: Value) -> Self {
        RString { value: value }
    }
}

impl From<String> for RString {
    fn from(string: String) -> Self {
        Self::new(string.as_str())
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
