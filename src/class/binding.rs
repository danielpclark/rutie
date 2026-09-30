use std::convert::From;

use crate::{
    binding::rproc,
    types::{Value, ValueType},
    AnyException, AnyObject, Array, Class, Object, RString, Symbol, VerifiedObject,
};

/// `Integer`
#[derive(Debug)]
pub struct Binding {
    value: Value,
}

impl Binding {
    /// Creates a new `Binding`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Binding, VM};
    /// # VM::init();
    ///
    /// let _ = Binding::new();
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// binding
    /// ```
    pub fn new() -> Self {
        Binding {
            value: rproc::binding_new(),
        }
    }

    /// Returns the value of local variable `name` in the binding, or the
    /// `NameError` if it is not defined (Ruby's `local_variable_get`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Binding, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let binding = VM::eval("count = 3; binding").unwrap().try_convert_to::<Binding>().unwrap();
    ///
    /// assert_eq!(binding.local_variable_get("count").unwrap().try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
    /// assert!(binding.local_variable_get("missing").is_err());
    /// ```
    pub fn local_variable_get(&self, name: &str) -> Result<AnyObject, AnyException> {
        self.protect_send("local_variable_get", &[Symbol::new(name).into()])
    }

    /// Sets local variable `name` in the binding, creating it if needed
    /// (Ruby's `local_variable_set`), or returns the `NameError` for an
    /// invalid name.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Binding, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let binding = VM::eval("binding").unwrap().try_convert_to::<Binding>().unwrap();
    ///
    /// binding.local_variable_set("answer", Fixnum::new(42)).unwrap();
    ///
    /// assert_eq!(binding.eval("answer").unwrap().try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));
    /// assert!(binding.local_variable_set("Not Valid", Fixnum::new(1)).is_err());
    /// ```
    pub fn local_variable_set<T: Object>(
        &self,
        name: &str,
        value: T,
    ) -> Result<AnyObject, AnyException> {
        self.protect_send(
            "local_variable_set",
            &[Symbol::new(name).into(), value.to_any_object()],
        )
    }

    /// Returns `true` if local variable `name` is defined in the binding
    /// (Ruby's `local_variable_defined?`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Binding, Object, VM};
    /// # VM::init();
    ///
    /// let binding = VM::eval("present = 1; binding").unwrap().try_convert_to::<Binding>().unwrap();
    ///
    /// assert!(binding.is_local_variable_defined("present"));
    /// assert!(!binding.is_local_variable_defined("absent"));
    /// ```
    pub fn is_local_variable_defined(&self, name: &str) -> bool {
        self.protect_send("local_variable_defined?", &[Symbol::new(name).into()])
            .map(|defined| defined.value().is_true())
            .unwrap_or(false)
    }

    /// Returns the names of the binding's local variables as `Symbol`s
    /// (Ruby's `local_variables`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Binding, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let binding = VM::eval("a = 1; binding").unwrap().try_convert_to::<Binding>().unwrap();
    ///
    /// assert!(binding.local_variables().includes(&Symbol::new("a")));
    /// ```
    pub fn local_variables(&self) -> Array {
        Array::from(unsafe { self.send("local_variables", &[]) }.value())
    }

    /// Returns the binding's `self` (Ruby's `receiver`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Binding, Object, RString, VM};
    /// # VM::init();
    ///
    /// let binding = VM::eval("'owner'.instance_eval { binding }").unwrap().try_convert_to::<Binding>().unwrap();
    ///
    /// assert_eq!(binding.receiver().try_convert_to::<RString>().unwrap().to_str(), "owner");
    /// ```
    pub fn receiver(&self) -> AnyObject {
        unsafe { self.send("receiver", &[]) }
    }

    /// Evaluates `code` in the binding (Ruby's `eval`), returning the result
    /// or the exception raised.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Binding, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let binding = VM::eval("x = 20; binding").unwrap().try_convert_to::<Binding>().unwrap();
    ///
    /// assert_eq!(binding.eval("x + 22").unwrap().try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));
    /// assert!(binding.eval("raise 'oops'").is_err());
    /// ```
    pub fn eval(&self, code: &str) -> Result<AnyObject, AnyException> {
        self.protect_send("eval", &[RString::new_utf8(code).into()])
    }
}

impl From<Value> for Binding {
    fn from(value: Value) -> Self {
        Binding { value }
    }
}

impl Into<Value> for Binding {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Binding {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for Binding {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Binding {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        Class::binding().case_equals(object)
    }

    fn error_message() -> &'static str {
        "Error converting to Integer"
    }
}

impl PartialEq for Binding {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Binding, Fixnum, Object, Symbol, VM};

    #[test]
    fn test_binding_locals() {
        crate::on_ruby_thread(|| {
            let binding = VM::eval("first = 1; binding")
                .unwrap()
                .try_convert_to::<Binding>()
                .unwrap();

            binding
                .local_variable_set("second", Fixnum::new(2))
                .unwrap();
            assert!(binding.is_local_variable_defined("second"));
            assert_eq!(binding.local_variables().length(), 2);

            let sum = binding.eval("first + second").unwrap();
            assert_eq!(sum.try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));

            assert!(binding.local_variable_get("third").is_err());
            assert!(!binding.is_local_variable_defined("Third"));
            assert!(binding.eval("undefined_method_here").is_err());

            let receiver = binding.receiver();
            assert_eq!(receiver.class().name().unwrap().to_str(), "Object");
            assert!(binding.local_variables().includes(&Symbol::new("first")));
        });
    }
}
