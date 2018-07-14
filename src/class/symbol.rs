use std::convert::From;

use binding::symbol;
use types::{Value, ValueType};

use {Object, VerifiedObject};

/// `Symbol`
#[derive(Debug, PartialEq)]
pub struct Symbol {
    value: Value,
}

impl Symbol {
    /// Creates a new instance of Ruby `Symbol`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Symbol, VM};
    /// # VM::init();
    ///
    /// let symbol = Symbol::new("hello");
    ///
    /// assert_eq!(symbol.to_str(), "hello");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// sym = :hello
    ///
    /// sym.to_s == 'hello'
    /// ```
    pub fn new(string: &str) -> Self {
        let id = symbol::internal_id(string);

        Self::from(symbol::id_to_sym(id))
    }

    /// Retrieves the Rust `&str` corresponding to `Symbol` object (Ruby `Symbol#to_s`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Symbol, VM};
    /// # VM::init();
    ///
    /// let symbol = Symbol::new("hello");
    ///
    /// assert_eq!(symbol.to_str(), "hello");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// sym = :hello
    ///
    /// sym.to_s == 'hello'
    /// ```
    pub fn to_str(&self) -> &str {
        symbol::value_to_str(self.value())
    }

    /// Retrieves the Rust `String` corresponding to `Symbol` object (Ruby `Symbol#to_s`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Symbol, VM};
    /// # VM::init();
    ///
    /// let symbol = Symbol::new("hello");
    ///
    /// assert_eq!(symbol.to_string(), "hello");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// sym = :hello
    ///
    /// sym.to_s == 'hello'
    /// ```
    pub fn to_string(&self) -> String {
        symbol::value_to_string(self.value())
    }
}

impl From<Value> for Symbol {
    fn from(value: Value) -> Self {
        Symbol { value: value }
    }
}

impl Object for Symbol {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Symbol {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.value().ty() == ValueType::Symbol
    }

    fn error_message() -> &'static str {
        "Error converting to Symbol"
    }
}
