use std::{
    convert::From,
    hash::{Hash, Hasher},
};

use crate::{
    binding::symbol,
    types::{Value, ValueType},
    AnyObject, Object, Proc, VerifiedObject,
};

/// `Symbol`
#[derive(Debug)]
#[repr(C)]
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

    /// Converts `Symbol` to `Proc`
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Symbol, Proc, VM, VerifiedObject};
    /// # VM::init();
    ///
    /// let symbol = Symbol::new("hello");
    ///
    /// assert!(Proc::is_correct_type(&symbol.to_proc()), "not correct type!");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// sym = :hello
    ///
    /// sym.to_s == 'hello'
    /// ```
    pub fn to_proc(&self) -> Proc {
        Proc::from(unsafe { self.send("to_proc", &[]) }.value())
    }
}

impl From<Value> for Symbol {
    fn from(value: Value) -> Self {
        Symbol { value }
    }
}

impl From<Symbol> for Value {
    fn from(val: Symbol) -> Self {
        val.value
    }
}

impl From<Symbol> for AnyObject {
    fn from(val: Symbol) -> Self {
        AnyObject::from(val.value)
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

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

impl Eq for Symbol {}

impl Hash for Symbol {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.value().value.hash(state);
    }
}
