use std::convert::From;

use binding::rproc;
use types::Value;
use util;

use {AnyObject, Class, Object, VerifiedObject, Boolean};

/// `Proc` (works with `Lambda` as well)
#[derive(Debug)]
pub struct Proc {
    value: Value,
}

impl Proc {
    /// Calls a proc with given arguments
    ///
    /// # Examples
    ///
    /// ```no_run
    /// #[macro_use]
    /// extern crate rutie;
    ///
    /// use rutie::{Class, Object, Proc, RString};
    ///
    /// class!(Greeter);
    ///
    /// methods!(
    ///     Greeter,
    ///     itself,
    ///
    ///     fn greet_rust_with(greeting_template: Proc) -> RString {
    ///         let name = RString::new_utf8("Rust").to_any_object();
    ///         let rendered_template = greeting_template.unwrap().call(Some(&[name]));
    ///
    ///         rendered_template.try_convert_to::<RString>().unwrap()
    ///     }
    /// );
    ///
    /// fn main() {
    ///     Class::new("Greeter", None).define(|itself| {
    ///         itself.def_self("greet_rust_with", greet_rust_with);
    ///     });
    /// }
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
    pub fn call(&self, arguments: Option<&[AnyObject]>) -> AnyObject {
        let arguments = util::arguments_to_values(arguments).unwrap_or_default();
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
        Boolean::from(self.send("lambda?", None).value()).to_bool()
    }
}

impl From<Value> for Proc {
    fn from(value: Value) -> Self {
        Proc { value: value }
    }
}

impl Into<Value> for Proc {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Proc {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
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
