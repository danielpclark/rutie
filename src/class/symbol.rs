use std::{
    convert::From,
    hash::{Hash, Hasher},
};

use crate::{
    binding::{string, symbol, vm},
    types::{Value, ValueType},
    AnyException, AnyObject, Array, Encoding, Object, Proc, RString, VerifiedObject,
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
    /// call with untrusted input: Ruby never garbage collects symbols created
    /// from C (`rb_intern`).
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

    /// Returns the symbol named by `name` in encoding `enc`, creating it if
    /// needed (`rb_intern3`). Returns the `EncodingError` when `name` is not
    /// valid in `enc`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, EncodingSupport, Symbol, VM};
    /// # VM::init();
    ///
    /// let symbol = Symbol::new_with_encoding("café".as_bytes(), &Encoding::utf8()).unwrap();
    ///
    /// assert_eq!(symbol.to_str(), "café");
    /// assert_eq!(symbol.to_rstring().encoding(), Encoding::utf8());
    /// assert!(Symbol::new_with_encoding(b"\xFF", &Encoding::utf8()).is_err());
    /// ```
    pub fn new_with_encoding(name: &[u8], enc: &Encoding) -> Result<Self, AnyException> {
        let enc = enc.value();

        vm::protect_value(|| symbol::intern_with_encoding(name, enc))
            .map(Self::from)
            .map_err(AnyException::from)
    }

    /// Returns the existing symbol named by `name` in encoding `enc`, or
    /// `None` without creating one (`rb_check_symbol_cstr`). Returns the
    /// `EncodingError` when `name` is not valid in `enc`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, Symbol, VM};
    /// # VM::init();
    ///
    /// let utf8 = Encoding::utf8();
    ///
    /// assert_eq!(Symbol::find_with_encoding(b"puts", &utf8).unwrap(), Some(Symbol::new("puts")));
    /// assert!(Symbol::find_with_encoding(b"rutie_never_interned", &utf8).unwrap().is_none());
    /// assert!(Symbol::find_with_encoding(b"\xFF", &utf8).is_err());
    /// ```
    pub fn find_with_encoding(name: &[u8], enc: &Encoding) -> Result<Option<Self>, AnyException> {
        let enc = enc.value();

        vm::protect_value(|| symbol::check_symbol_with_encoding(name, enc))
            .map(|found| {
                if found.is_nil() {
                    None
                } else {
                    Some(Self::from(found))
                }
            })
            .map_err(AnyException::from)
    }

    /// Returns `true` if `name` can be written as a symbol literal without
    /// quotes, such as `:name`, `:Const`, `:@ivar`, `:+` or `:[]=`
    /// (`rb_enc_symname2_p`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Symbol, VM};
    /// # VM::init();
    ///
    /// assert!(Symbol::is_literal_name("name?"));
    /// assert!(Symbol::is_literal_name("[]="));
    /// assert!(!Symbol::is_literal_name("two words"));
    /// assert!(!Symbol::is_literal_name("1st"));
    /// ```
    pub fn is_literal_name(name: &str) -> bool {
        symbol::is_symbol_name(name.as_bytes(), Encoding::utf8().value())
    }

    /// Returns the setter name for the symbol, `:name` → `:name=`
    /// (`rb_id_attrset`). Returns the `NameError` for operators such as
    /// `:+`, which have none.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Symbol, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Symbol::new("name").to_setter().unwrap(), Symbol::new("name="));
    /// assert!(Symbol::new("+").to_setter().is_err());
    /// ```
    pub fn to_setter(&self) -> Result<Symbol, AnyException> {
        let symbol = self.value();

        vm::protect_value(|| symbol::attrset(symbol))
            .map(Self::from)
            .map_err(AnyException::from)
    }

    /// Returns `true` if the symbol is a setter name, such as `:name=`
    /// (`rb_is_attrset_id`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Symbol, VM};
    /// # VM::init();
    ///
    /// assert!(Symbol::new("name=").is_setter_name());
    /// assert!(!Symbol::new("name").is_setter_name());
    /// ```
    pub fn is_setter_name(&self) -> bool {
        symbol::is_attrset_name(self.value())
    }

    /// Returns `true` if the symbol is a valid global variable name, such as
    /// `:$name` (`rb_is_global_id`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Symbol, VM};
    /// # VM::init();
    ///
    /// assert!(Symbol::new("$stdout").is_global_variable_name());
    /// assert!(!Symbol::new("stdout").is_global_variable_name());
    /// ```
    pub fn is_global_variable_name(&self) -> bool {
        symbol::is_global_name(self.value())
    }

    /// Returns `true` if the symbol is a valid local variable (or method)
    /// name, such as `:name` (`rb_is_local_id`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Symbol, VM};
    /// # VM::init();
    ///
    /// assert!(Symbol::new("name").is_local_name());
    /// assert!(!Symbol::new("Name").is_local_name());
    /// assert!(!Symbol::new("@name").is_local_name());
    /// ```
    pub fn is_local_name(&self) -> bool {
        symbol::is_local_name(self.value())
    }

    /// Returns `true` if the symbol is a name of none of the variable,
    /// constant or setter kinds, such as a method name ending in `!` or `?`
    /// (`rb_is_junk_id`). Operators such as `:+` are not junk names.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Symbol, VM};
    /// # VM::init();
    ///
    /// assert!(Symbol::new("save!").is_junk_name());
    /// assert!(Symbol::new("rutie_valid?").is_junk_name());
    /// assert!(!Symbol::new("name").is_junk_name());
    /// assert!(!Symbol::new("+").is_junk_name());
    /// ```
    pub fn is_junk_name(&self) -> bool {
        symbol::is_junk_name(self.value())
    }

    /// Returns every symbol Ruby knows (Ruby's `Symbol.all_symbols`,
    /// `rb_sym_all_symbols`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let all = Symbol::all();
    /// let puts = Symbol::new("puts");
    ///
    /// assert!(all.length() > 100);
    /// assert!((0..all.length()).any(|i| all.at(i as i64).value() == puts.value()));
    /// ```
    pub fn all() -> Array {
        Array::from(symbol::all_symbols())
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
    use crate::{Encoding, Object, RString, Symbol, VM};

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

    #[test]
    fn test_symbol_encodings_and_kinds() {
        crate::on_ruby_thread(|| {
            let utf8 = Encoding::utf8();

            let created = Symbol::new_with_encoding("rutie_sym_é".as_bytes(), &utf8).unwrap();
            assert_eq!(created.to_str(), "rutie_sym_é");
            let found = Symbol::find_with_encoding("rutie_sym_é".as_bytes(), &utf8).unwrap();
            assert_eq!(found.unwrap().value(), created.value());
            // The same bytes in another encoding are another symbol.
            assert!(
                Symbol::find_with_encoding("rutie_sym_é".as_bytes(), &Encoding::ascii_8bit())
                    .unwrap()
                    .is_none()
            );
            assert!(Symbol::find_with_encoding(b"rutie_sym_missing", &utf8)
                .unwrap()
                .is_none());
            // ASCII-only names in different encodings are the same symbol.
            let binary =
                Symbol::new_with_encoding(b"rutie_plain", &Encoding::ascii_8bit()).unwrap();
            assert_eq!(binary, Symbol::new("rutie_plain"));

            assert!(Symbol::is_literal_name("Const"));
            assert!(Symbol::is_literal_name("@@cvar"));
            assert!(Symbol::is_literal_name("$0"));
            assert!(Symbol::is_literal_name("<=>"));
            assert!(Symbol::is_literal_name("ünïcode"));
            assert!(!Symbol::is_literal_name(""));
            assert!(!Symbol::is_literal_name("a-b"));

            assert_eq!(
                Symbol::new("Name").to_setter().unwrap(),
                Symbol::new("Name=")
            );
            assert!(Symbol::new("name=").to_setter().is_ok());
            assert_eq!(Symbol::new("[]").to_setter().unwrap(), Symbol::new("[]="));
            assert!(Symbol::new("==").to_setter().is_err());
            assert!(Symbol::new("Name=").is_setter_name());
            assert!(Symbol::new("$x").is_global_variable_name());
            assert!(!Symbol::new("@x").is_global_variable_name());
            assert!(Symbol::new("_x").is_local_name());
            assert!(Symbol::new("save!").is_junk_name());
            assert!(!Symbol::new("<<").is_junk_name());
            assert!(!Symbol::new("Name").is_junk_name());

            let all = Symbol::all();
            let created = created.value();
            assert!((0..all.length()).any(|i| all.at(i as i64).value() == created));
        });
    }
}
