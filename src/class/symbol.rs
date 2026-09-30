use std::{
    convert::From,
    hash::{Hash, Hasher},
};

use crate::{
    binding::{string, symbol},
    types::{Value, ValueType},
    AnyObject, Object, Proc, RString, VerifiedObject,
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

    /// Returns the existing symbol named `name`, or `None` if no such
    /// symbol has been created yet (`rb_check_id`).
    ///
    /// Unlike `Symbol::new`, this never creates a symbol, so it is safe to
    /// call with untrusted input: Ruby 2.5 to 2.7 never garbage collect
    /// symbols created from C.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Symbol, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Symbol::find("puts"), Some(Symbol::new("puts")));
    /// assert!(Symbol::find("rutie_never_interned_symbol").is_none());
    /// ```
    pub fn find(name: &str) -> Option<Self> {
        symbol::check_symbol(string::new_utf8(name)).map(Self::from)
    }

    /// Returns the symbol for a Ruby `String`, keeping its encoding (Ruby's
    /// `String#to_sym`, `rb_to_symbol`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, Symbol, VM};
    /// # VM::init();
    ///
    /// let name = RString::new_utf8("résumé");
    ///
    /// assert_eq!(Symbol::from_rstring(&name).to_str(), "résumé");
    /// ```
    pub fn from_rstring(name: &RString) -> Self {
        Self::from(symbol::to_symbol(name.value()))
    }

    /// Returns the symbol's name as a frozen Ruby `String` (`rb_sym2str`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let name = Symbol::new("hello").to_rstring();
    ///
    /// assert_eq!(name.to_str(), "hello");
    /// assert!(name.is_frozen());
    /// ```
    pub fn to_rstring(&self) -> RString {
        RString::from(symbol::sym_to_str(self.value()))
    }

    /// Returns `true` if the symbol is a valid constant name, such as
    /// `:Name` (`rb_is_const_id`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Symbol, VM};
    /// # VM::init();
    ///
    /// assert!(Symbol::new("Name").is_const_name());
    /// assert!(!Symbol::new("name").is_const_name());
    /// ```
    pub fn is_const_name(&self) -> bool {
        symbol::is_const_name(self.value())
    }

    /// Returns `true` if the symbol is a valid instance variable name, such
    /// as `:@name` (`rb_is_instance_id`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Symbol, VM};
    /// # VM::init();
    ///
    /// assert!(Symbol::new("@name").is_instance_variable_name());
    /// assert!(!Symbol::new("name").is_instance_variable_name());
    /// ```
    pub fn is_instance_variable_name(&self) -> bool {
        symbol::is_instance_variable_name(self.value())
    }

    /// Returns `true` if the symbol is a valid class variable name, such as
    /// `:@@name` (`rb_is_class_id`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Symbol, VM};
    /// # VM::init();
    ///
    /// assert!(Symbol::new("@@name").is_class_variable_name());
    /// assert!(!Symbol::new("@name").is_class_variable_name());
    /// ```
    pub fn is_class_variable_name(&self) -> bool {
        symbol::is_class_variable_name(self.value())
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

impl Into<Value> for Symbol {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Symbol {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
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

#[cfg(test)]
mod tests {
    use crate::{Object, RString, Symbol, VM};

    #[test]
    fn test_symbol_lookup_and_names() {
        crate::on_ruby_thread(|| {
            assert!(Symbol::find("rutie_symbol_lookup_test").is_none());

            let created = Symbol::new("rutie_symbol_lookup_test");
            assert_eq!(Symbol::find("rutie_symbol_lookup_test"), Some(created));

            // Symbols made by Ruby code are found too.
            VM::eval(":rutie_symbol_from_ruby").unwrap();
            assert!(Symbol::find("rutie_symbol_from_ruby").is_some());

            let name = RString::new_utf8("from_string");
            let symbol = Symbol::from_rstring(&name);
            assert_eq!(symbol, Symbol::new("from_string"));
            assert_eq!(symbol.to_rstring(), name);
            assert!(symbol.to_rstring().is_frozen());

            assert!(Symbol::new("CONST").is_const_name());
            assert!(Symbol::new("@ivar").is_instance_variable_name());
            assert!(Symbol::new("@@cvar").is_class_variable_name());
            assert!(!Symbol::new("@@cvar").is_instance_variable_name());
            assert!(!Symbol::new("local").is_const_name());
        });
    }
}
