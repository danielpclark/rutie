use crate::{
    binding::encoding,
    types::{Value, ValueType},
    AnyException, AnyObject, Class, Exception, NilClass, Object, RString, VerifiedObject,
};

#[derive(Debug)]
#[repr(C)]
pub struct Encoding {
    value: Value,
}

impl Encoding {
    /// Creates a UTF-8 instance of `Encoding`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Encoding, VM};
    /// # VM::init();
    ///
    /// Encoding::utf8();
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
    /// Encoding::us_ascii();
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
    /// use rutie::{Encoding, VM};
    /// # VM::init();
    ///
    /// Encoding::default_external();
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
    /// Encoding::default_internal();
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

impl From<Encoding> for Value {
    fn from(val: Encoding) -> Self {
        val.value
    }
}

impl From<Encoding> for AnyObject {
    fn from(val: Encoding) -> Self {
        AnyObject::from(val.value)
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
        object.value().ty() == ValueType::Class
            && Class::from_existing("Encoding").case_equals(object)
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
