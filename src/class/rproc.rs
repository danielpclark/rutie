use std::convert::From;

use crate::{
    binding::rproc, types::Value, util, AnyObject, Boolean, Class, Object, VerifiedObject,
};

/// `Proc` (works with `Lambda` as well)
#[derive(Debug)]
#[repr(C)]
pub struct Proc {
    value: Value,
}

impl Proc {
    /// Calls a proc with given arguments
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rutie::{Class, Object, Proc, RString, class, methods};
    ///
    /// class!(Greeter);
    ///
    /// methods!(
    ///     Greeter,
    ///     rtself,
    ///
    ///     fn greet_rust_with(greeting_template: Proc) -> RString {
    ///         let name = RString::new_utf8("Rust").to_any_object();
    ///         let rendered_template = greeting_template.unwrap().call(&[name]);
    ///
    ///         rendered_template.try_convert_to::<RString>().unwrap()
    ///     }
    /// );
    ///
    /// Class::new("Greeter", None).define(|klass| {
    ///     klass.def_self("greet_rust_with", greet_rust_with);
    /// });
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Greeter
    ///   def self.greet_rust_with(greeting_template)
    ///     greeting_template.call('Rust')
    ///   end
    /// end
    ///
    /// greeting_template = -> (name) { "Hello, #{name}!" }
    ///
    /// Greeter.greet_rust_with(greeting_template) # => "Hello, Rust!"
    /// ```
    pub fn call(&self, arguments: &[AnyObject]) -> AnyObject {
        let arguments = util::arguments_to_values(arguments);
        let result = rproc::call(self.value(), &arguments);

        AnyObject::from(result)
    }

    /// Check if Proc is a lambda
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Proc, VM, VerifiedObject};
    /// # VM::init();
    ///
    /// let procish = VM::eval("lambda {|a,b| a + b }").unwrap();
    ///
    /// assert!(Proc::is_correct_type(&procish), "not Proc!");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// procish = lambda {|a,b| a + b }
    ///
    /// procish.lambda? # => true
    /// ```
    pub fn is_lambda(&self) -> bool {
        Boolean::from(unsafe { self.send("lambda?", &[]) }.value()).to_bool()
    }
}

impl From<Value> for Proc {
    fn from(value: Value) -> Self {
        Proc { value }
    }
}

impl From<Proc> for Value {
    fn from(val: Proc) -> Self {
        val.value
    }
}

impl From<Proc> for AnyObject {
    fn from(val: Proc) -> Self {
        AnyObject::from(val.value)
    }
}

impl Object for Proc {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Proc {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        Class::from_existing("Proc").case_equals(object)
    }

    fn error_message() -> &'static str {
        "Error converting to Proc"
    }
}

impl PartialEq for Proc {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}
