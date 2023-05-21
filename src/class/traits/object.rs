use std::convert::From;

use crate::{
    binding::{class, global::ValueType, vm},
    typed_data::DataTypeWrapper,
    types::{Callback, Value},
    util,
};

use crate::{AnyException, AnyObject, Boolean, Class, Exception, NilClass, VerifiedObject, VM};

/// `Object`
///
/// Trait consists methods of Ruby `Object` class. Every struct like `Array`, `Hash` etc implements
/// this trait.
///
/// `class!` macro automatically implements this trait for custom classes.
pub trait Object: From<Value> {
    /// Returns internal `value` of current object.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::types::Value;
    /// use rutie::Object;
    ///
    /// struct Array {
    ///   value: Value
    /// }
    ///
    /// impl From<Value> for Array {
    ///     fn from(value: Value) -> Self {
    ///         Array {
    ///             value: value
    ///         }
    ///     }
    /// }
    ///
    /// impl Object for Array {
    ///     fn value(&self) -> Value {
    ///         self.value
    ///     }
    /// }
    /// ```
    fn value(&self) -> Value;

    /// Returns a class of current object.
    ///
    /// # Examples
    /// ```
    /// use rutie::{Array, Object, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Array::new().class(), Array::new().class());
    /// ```
    fn class(&self) -> Class {
        let class = class::object_class(self.value());

        Class::from(class)
    }

    /// Returns a singleton class of current object.
    ///
    /// # Examples
    ///
    /// ### Getting singleton class
    ///
    /// ```
    /// use rutie::{Array, Object, VM};
    /// # VM::init();
    ///
    /// let array = Array::new();
    /// let another_array = Array::new();
    ///
    /// assert!(array.singleton_class() != another_array.singleton_class());
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// array = []
    /// another_array = []
    ///
    /// array.singleton_class != another_array.singleton_class
    /// ```
    ///
    /// ### Modifying singleton class
    ///
    /// ```
    /// use rutie::{Array, Object, VM};
    /// # VM::init();
    ///
    /// let array = Array::new();
    /// let another_array = Array::new();
    ///
    /// array.singleton_class().define(|klass| {
    ///     klass.attr_reader("modified");
    /// });
    ///
    /// assert!(array.respond_to("modified"));
    /// assert!(!another_array.respond_to("modified"));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// array = []
    ///
    /// class << array
    ///   attr_reader :modified
    /// end
    ///
    /// array.respond_to?(:modified)
    /// ```
    fn singleton_class(&self) -> Class {
        let class = class::singleton_class(self.value());

        Class::from(class)
    }

    /// Gets an immutable reference to the Rust structure which is wrapped into a Ruby object.
    ///
    /// See the documentation for `wrappable_struct!` macro for more information.
    ///
    /// # Examples
    ///
    /// Wrap `Server` structs to `RubyServer` objects
    ///
    /// ```
    /// use rutie::{AnyObject, Class, Integer, Object, RString, VM, class, methods, wrappable_struct};
    ///
    /// // The structure which we want to wrap
    /// pub struct Server {
    ///     host: String,
    ///     port: u16,
    /// }
    ///
    /// impl Server {
    ///     fn new(host: String, port: u16) -> Self {
    ///         Server {
    ///             host: host,
    ///             port: port,
    ///         }
    ///     }
    ///
    ///     fn host(&self) -> &str {
    ///         &self.host
    ///     }
    ///
    ///     fn port(&self) -> u16 {
    ///         self.port
    ///     }
    /// }
    ///
    /// wrappable_struct!(Server, ServerWrapper, SERVER_WRAPPER);
    ///
    /// class!(RubyServer);
    ///
    /// methods!(
    ///     RubyServer,
    ///     rtself,
    ///
    ///     fn ruby_server_new(host: RString, port: Integer) -> AnyObject {
    ///         let server = Server::new(host.unwrap().to_string(),
    ///                                  port.unwrap().to_i64() as u16);
    ///
    ///         Class::from_existing("RubyServer").wrap_data(server, &*SERVER_WRAPPER)
    ///     }
    ///
    ///     fn ruby_server_host() -> RString {
    ///         let host = rtself.get_data(&*SERVER_WRAPPER).host();
    ///
    ///         RString::new_utf8(host)
    ///     }
    ///
    ///     fn ruby_server_port() -> Integer {
    ///         let port = rtself.get_data(&*SERVER_WRAPPER).port();
    ///
    ///         Integer::new(port as i64)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     let data_class = Class::from_existing("Object");
    ///
    ///     Class::new("RubyServer", Some(&data_class)).define(|klass| {
    ///         klass.def_self("new", ruby_server_new);
    ///
    ///         klass.def("host", ruby_server_host);
    ///         klass.def("port", ruby_server_port);
    ///     });
    /// }
    /// ```
    ///
    /// To use the `RubyServer` class in Ruby:
    ///
    /// ```ruby
    /// server = RubyServer.new("127.0.0.1", 3000)
    ///
    /// server.host == "127.0.0.1"
    /// server.port == 3000
    /// ```
    fn get_data<'a, T>(&'a self, wrapper: &'a dyn DataTypeWrapper<T>) -> &T {
        class::get_data(self.value(), wrapper)
    }

    /// Gets a mutable reference to the Rust structure which is wrapped into a Ruby object.
    fn get_data_mut<'a, T>(&'a mut self, wrapper: &'a dyn DataTypeWrapper<T>) -> &mut T {
        class::get_data(self.value(), wrapper)
    }

    /// Wraps calls to the object.
    ///
    /// Mostly used to have Ruby-like class definition DSL.
    ///
    /// # Examples
    ///
    /// ### Defining class
    ///
    /// ```no_run
    /// use rutie::{Class, Object, RString, methods, class};
    ///
    /// class!(Hello);
    /// class!(Nested);
    ///
    /// methods!(
    ///     Hello,
    ///     rtself,
    ///
    ///     fn greeting() -> RString {
    ///         RString::new_utf8("Greeting from class")
    ///     }
    ///
    ///     fn many_greetings() -> RString {
    ///         RString::new_utf8("Many greetings from instance")
    ///     }
    /// );
    ///
    /// methods!(
    ///     Nested,
    ///     rtself,
    ///
    ///     fn nested_greeting() -> RString {
    ///         RString::new_utf8("Greeting from nested class")
    ///     }
    /// );
    ///
    /// fn main() {
    ///     Class::new("Hello", None).define(|klass| {
    ///         klass.attr_reader("reader");
    ///
    ///         klass.def_self("greeting", greeting);
    ///         klass.def("many_greetings", many_greetings);
    ///
    ///         klass.define_nested_class("Nested", None).define(|klass| {
    ///             klass.def_self("nested_greeting", nested_greeting);
    ///         });
    ///     });
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Hello
    ///   attr_reader :reader
    ///
    ///   def self.greeting
    ///     'Greeting from class'
    ///   end
    ///
    ///   def many_greetings
    ///     'Many greetings from instance'
    ///   end
    ///
    ///   class Nested
    ///     def self.nested_greeting
    ///       'Greeting from nested class'
    ///     end
    ///   end
    /// end
    /// ```
    ///
    /// ### Defining singleton method for an object
    ///
    /// ```rust
    /// use rutie::{AnyObject, Class, Integer, Object, RString, VM, methods};
    ///
    /// methods!(
    ///     RString,
    ///     rtself,
    ///
    ///     fn greeting() -> RString {
    ///         RString::new_utf8("Greeting!")
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     let mut string = RString::new_utf8("Some string");
    ///
    ///     // The same can be done by modifying `string.singleton_class()`
    ///     // or usiang `string.define_singleton_method("greeting", greeting)`
    ///     string.define(|klass| {
    ///         klass.define_singleton_method("greeting", greeting);
    ///     });
    ///
    ///     assert!(string.respond_to("greeting"));
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// string = "Some string"
    ///
    /// class << string
    ///   def greeting
    ///     'Greeting!'
    ///   end
    /// end
    ///
    /// string.respond_to?("greeting")
    /// ```
    fn define<F: Fn(&mut Self)>(&mut self, f: F) -> &Self {
        f(self);

        self
    }

    /// Defines an instance method for the given class or object.
    ///
    /// Use `methods!` macro to define a `callback`.
    ///
    /// You can also use `def()` alias for this function combined with `Class::define()` for a
    /// nicer DSL.
    ///
    /// # Panics
    ///
    /// Ruby can raise an exception if you try to define instance method directly on an instance
    /// of some class (like `Integer`, `String`, `Array` etc).
    ///
    /// Use this method only on classes (or singleton classes of objects).
    ///
    /// # Examples
    ///
    /// ### The famous String#blank? method
    ///
    /// ```rust
    /// use rutie::{Boolean, Class, Object, RString, VM, methods};
    ///
    /// methods!(
    ///    RString,
    ///    rtself,
    ///
    ///    fn is_blank() -> Boolean {
    ///        Boolean::new(rtself.to_str().chars().all(|c| c.is_whitespace()))
    ///    }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("String").define(|klass| {
    ///         klass.def("blank?", is_blank);
    ///     });
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class String
    ///   def blank?
    ///     # simplified
    ///     self.chars.all? { |c| c == ' ' }
    ///   end
    /// end
    /// ```
    ///
    /// ### Receiving arguments
    ///
    /// Raise `Integer` to the power of `exp`.
    ///
    /// ```rust
    /// use std::error::Error;
    /// use rutie::{Class, Integer, Object, Exception, VM, methods};
    ///
    /// methods!(
    ///     Integer,
    ///     rtself,
    ///
    ///     fn pow(exp: Integer) -> Integer {
    ///         // `exp` is not a valid `Integer`, raise an exception
    ///         if let Err(ref error) = exp {
    ///             VM::raise(error.class(), &error.message());
    ///         }
    ///
    ///         // We can safely unwrap here, because an exception was raised if `exp` is `Err`
    ///         let exp = exp.unwrap().to_i64() as u32;
    ///
    ///         Integer::new(rtself.to_i64().pow(exp))
    ///     }
    ///
    ///     fn pow_with_default_argument(exp: Integer) -> Integer {
    ///         let default_exp = 0;
    ///         let exp = exp.map(|exp| exp.to_i64()).unwrap_or(default_exp);
    ///
    ///         let result = rtself.to_i64().pow(exp as u32);
    ///
    ///         Integer::new(result)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Integer").define(|klass| {
    ///         klass.def("pow", pow);
    ///         klass.def("pow_with_default_argument", pow_with_default_argument);
    ///     });
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Integer
    ///   def pow(exp)
    ///     raise ArgumentError unless exp.is_a?(Integer)
    ///
    ///     self ** exp
    ///   end
    ///
    ///   def pow_with_default_argument(exp)
    ///     default_exp = 0
    ///     exp = default_exp unless exp.is_a?(Integer)
    ///
    ///     self ** exp
    ///   end
    /// end
    /// ```
    fn define_method<I: Object, O: Object>(&mut self, name: &str, callback: Callback<I, O>) {
        class::define_method(self.value(), name, callback);
    }

    /// Defines a private instance method for the given class or object.
    ///
    /// Use `methods!` macro to define a `callback`.
    ///
    /// You can also use `def_private()` alias for this function combined with `Class::define()` for a
    /// nicer DSL.
    ///
    /// # Panics
    ///
    /// Ruby can raise an exception if you try to define instance method directly on an instance
    /// of some class (like `Integer`, `String`, `Array` etc).
    ///
    /// Use this method only on classes (or singleton classes of objects).
    ///
    /// # Examples
    ///
    /// ### The famous String#blank? method
    ///
    /// ```rust
    /// use rutie::{Boolean, Class, Object, RString, VM, methods};
    ///
    /// methods!(
    ///    RString,
    ///    rtself,
    ///
    ///    fn is_blank() -> Boolean {
    ///        Boolean::new(rtself.to_str().chars().all(|c| c.is_whitespace()))
    ///    }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("String").define(|klass| {
    ///         klass.def_private("blank?", is_blank);
    ///     });
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class String
    ///   private def blank?
    ///     # simplified
    ///     self.chars.all? { |c| c == ' ' }
    ///   end
    /// end
    /// ```
    ///
    /// ### Receiving arguments
    ///
    /// Raise `Integer` to the power of `exp`.
    ///
    /// ```rust
    /// use std::error::Error;
    /// use rutie::{Class, Integer, Object, Exception, VM, methods};
    ///
    /// methods!(
    ///     Integer,
    ///     rtself,
    ///
    ///     fn pow(exp: Integer) -> Integer {
    ///         // `exp` is not a valid `Integer`, raise an exception
    ///         if let Err(ref error) = exp {
    ///             VM::raise(error.class(), &error.message());
    ///         }
    ///
    ///         // We can safely unwrap here, because an exception was raised if `exp` is `Err`
    ///         let exp = exp.unwrap().to_i64() as u32;
    ///
    ///         Integer::new(rtself.to_i64().pow(exp))
    ///     }
    ///
    ///     fn pow_with_default_argument(exp: Integer) -> Integer {
    ///         let default_exp = 0;
    ///         let exp = exp.map(|exp| exp.to_i64()).unwrap_or(default_exp);
    ///
    ///         let result = rtself.to_i64().pow(exp as u32);
    ///
    ///         Integer::new(result)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Integer").define(|klass| {
    ///         klass.def_private("pow", pow);
    ///         klass.def_private("pow_with_default_argument", pow_with_default_argument);
    ///     });
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Integer
    ///   private
    ///   def pow(exp)
    ///     raise ArgumentError unless exp.is_a?(Integer)
    ///
    ///     self ** exp
    ///   end
    ///
    ///   def pow_with_default_argument(exp)
    ///     default_exp = 0
    ///     exp = default_exp unless exp.is_a?(Integer)
    ///
    ///     self ** exp
    ///   end
    /// end
    /// ```
    fn define_private_method<I: Object, O: Object>(
        &mut self,
        name: &str,
        callback: Callback<I, O>,
    ) {
        class::define_private_method(self.value(), name, callback);
    }

    /// Defines a class method for given class or singleton method for object.
    ///
    /// Use `methods!` macro to define a `callback`.
    ///
    /// You can also use `def_self()` alias for this function combined with `Class::define()` a for
    /// nicer DSL.
    ///
    /// # Examples
    ///
    /// ### Defining a class method
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use std::error::Error;
    /// use rutie::{Class, Object, Exception, RString, Symbol, VM};
    ///
    /// methods!(
    ///     Symbol,
    ///     rtself,
    ///
    ///     fn from_string(string: RString) -> Symbol {
    ///         // `string` is not a valid `String`, raise an exception
    ///         if let Err(ref error) = string {
    ///             VM::raise(error.class(), &error.message());
    ///         }
    ///
    ///         Symbol::new(&string.unwrap().to_string())
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Symbol").define(|klass| {
    ///         klass.def_self("from_string", from_string);
    ///     });
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Symbol
    ///   def self.from_string(string)
    ///     raise ArgumentError unless string.is_a?(String)
    ///
    ///     # simplified
    ///     string.to_sym
    ///   end
    /// end
    /// ```
    ///
    /// ### Defining a singleton method for an object
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{AnyObject, Class, Object, RString, VM};
    ///
    /// methods!(
    ///     RString,
    ///     rtself,
    ///
    ///     fn greeting() -> RString {
    ///         RString::new_utf8("Greeting!")
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     let mut string = RString::new_utf8("Some string");
    ///
    ///     // The same can be done by modifying `string.singleton_class()`
    ///     // or using `string.define_singleton_method("greeting", greeting)`
    ///     string.define(|klass| {
    ///         klass.define_singleton_method("greeting", greeting);
    ///     });
    ///
    ///     assert!(string.respond_to("greeting"));
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    ///
    /// string = "Some string"
    ///
    /// class << string
    ///   def greeting
    ///     'Greeting!'
    ///   end
    /// end
    ///
    /// string.respond_to?("greeting")
    /// ```
    fn define_singleton_method<I: Object, O: Object>(
        &mut self,
        name: &str,
        callback: Callback<I, O>,
    ) {
        class::define_singleton_method(self.value(), name, callback);
    }

    /// An alias for `define_method` (similar to Ruby syntax `def some_method`).
    fn def<I: Object, O: Object>(&mut self, name: &str, callback: Callback<I, O>) {
        self.define_method(name, callback);
    }

    /// An alias for `define_private_method` (similar to Ruby syntax `private def some_method`).
    fn def_private<I: Object, O: Object>(&mut self, name: &str, callback: Callback<I, O>) {
        self.define_private_method(name, callback);
    }

    /// An alias for `define_singleton_method` (similar to Ruby `def self.some_method`).
    fn def_self<I: Object, O: Object>(&mut self, name: &str, callback: Callback<I, O>) {
        self.define_singleton_method(name, callback);
    }

    /// Calls a given method on an object similarly to Ruby `Object#send` method
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Integer, Object, RString, VM};
    /// # VM::init();
    ///
    /// let array = Array::new().push(Integer::new(1));
    /// let array_string = unsafe { array.send("to_s", &[]) }
    ///                                  .try_convert_to::<RString>()
    ///                                  .unwrap();
    ///
    /// assert_eq!(array_string.to_str(), "[1]");
    /// ```
    unsafe fn send(&self, method: &str, arguments: &[AnyObject]) -> AnyObject {
        let arguments = util::arguments_to_values(arguments);
        let result = vm::call_method(self.value(), method, &arguments);

        AnyObject::from(result)
    }

    /// Alias for Ruby's `==`
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, Object, VM};
    /// # VM::init();
    ///
    /// let a = Integer::new(4);
    /// let b = Integer::new(7);
    /// let c = Integer::new(4);
    ///
    /// assert!(!a.equals(&b));
    /// assert!(a.equals(&c));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// a = 4
    /// b = 7
    /// c = 4
    ///
    /// a == b # false
    /// a == c # true
    /// ```
    fn equals<T: Object>(&self, other: &T) -> bool {
        class::equals(self.value(), other.value()).is_true()
    }

    /// Alias for Ruby's `===`
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, Object, Class, VM};
    /// # VM::init();
    ///
    /// let a = Integer::new(4);
    /// let b = Class::from_existing("Integer");
    ///
    /// assert!(!a.case_equals(&b));
    /// assert!(b.case_equals(&a));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// a = 4
    ///
    /// a === Integer # false
    /// Integer === a # true
    /// ```
    fn case_equals<T: Object>(&self, other: &T) -> bool {
        let v = self.value();
        let m = "===";
        let a = [other.value()];

        vm::call_method(v, m, &a).is_true()
    }

    /// Alias for Ruby's `eql?`
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Integer, Object, VM};
    /// # VM::init();
    ///
    /// let a = Integer::new(4);
    /// let b = Integer::new(7);
    /// let c = Integer::new(4);
    ///
    /// assert!(!a.is_eql(&b));
    /// assert!(a.is_eql(&c));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// a = 4
    /// b = 7
    /// c = 4
    ///
    ///
    /// a.eql?(b)
    /// a.eql?(c)
    /// ```
    fn is_eql<T: Object>(&self, other: &T) -> bool {
        class::is_eql(self.value(), other.value()).is_true()
    }

    /// Alias for Ruby's `equal?`
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Array, VM};
    /// # VM::init();
    ///
    /// let values: Array = VM::eval("a='a';b=a;c=a.dup;[a,b,c]").unwrap().try_convert_to::<Array>().unwrap();
    /// let a = values.at(0);
    /// let b = values.at(1);
    /// let c = values.at(2);
    ///
    /// assert!(a.is_equal(&b));
    /// assert!(!a.is_equal(&c));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// a = "a"
    /// b = a
    /// c = a.dup
    ///
    ///
    /// a.equal?(b) # true
    /// a.equal?(c) # false
    /// ```
    fn is_equal<T: Object>(&self, other: &T) -> bool {
        self.value() == other.value()
    }

    /// Checks whether the object responds to given method
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Object, VM};
    /// # VM::init();
    ///
    /// let array = Array::new();
    ///
    /// assert!(array.respond_to("push"));
    /// assert!(!array.respond_to("something_else"));
    /// ```
    fn respond_to(&self, method: &str) -> bool {
        class::respond_to(self.value(), method)
    }

    /// `protect_send` returns Result<AnyObject, AnyObject>
    ///
    /// Protects against crash with `send` when exception object raised which will
    /// be returned in the `Err` result.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, Object, Exception, Class, VM, Boolean};
    /// # VM::init();
    ///
    /// let kernel = Class::from_existing("Kernel");
    ///
    /// let result = kernel.protect_send("nil?", &[]);
    ///
    /// if let Ok(r) = result {
    ///     assert!(!r.try_convert_to::<Boolean>().unwrap().to_bool());
    /// } else {
    ///     unreachable!()
    /// }
    ///
    /// let kernel = Class::from_existing("Kernel");
    ///
    /// let result = kernel.protect_send(
    ///     "raise",
    ///     &[RString::new_utf8("flowers").to_any_object()]
    /// );
    ///
    /// if let Err(error) = result {
    ///     assert_eq!(
    ///         error.message(),
    ///         "flowers"
    ///     );
    /// } else {
    ///     unreachable!()
    /// }
    /// ```
    fn protect_send(
        &self,
        method: &str,
        arguments: &[AnyObject],
    ) -> Result<AnyObject, AnyException> {
        let closure = || unsafe { self.send(&method, arguments.as_ref()) };

        let result = VM::protect(closure);

        result.map_err(|_| {
            let output = VM::error_info().unwrap();

            // error cleanup
            VM::clear_error_info();

            output
        })
    }

    /// `protect_public_send` returns Result<AnyObject, AnyObject>
    ///
    /// Protects against crash with `public_send` when exception object raised which will
    /// be returned in the `Err` result.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, Object, Exception, Class, VM, Boolean};
    /// # VM::init();
    ///
    /// let kernel = Class::from_existing("Kernel");
    ///
    /// let result = kernel.protect_public_send("nil?", &[]);
    ///
    /// if let Ok(r) = result {
    ///     assert!(!r.try_convert_to::<Boolean>().unwrap().to_bool());
    /// } else {
    ///     unreachable!()
    /// }
    ///
    /// let kernel = Class::from_existing("Kernel");
    ///
    /// let result = kernel.protect_public_send(
    ///     "raise",
    ///     &[RString::new_utf8("flowers").to_any_object()]
    /// );
    ///
    /// if let Err(error) = result {
    ///     assert_eq!(
    ///         error.message(),
    ///         "flowers"
    ///     );
    /// } else {
    ///     unreachable!()
    /// }
    /// ```
    fn protect_public_send(
        &self,
        method: &str,
        arguments: &[AnyObject],
    ) -> Result<AnyObject, AnyException> {
        let v = self.value();
        let arguments = util::arguments_to_values(arguments);

        let closure = || vm::call_public_method(v, &method, &arguments).into();

        let result = VM::protect(closure);

        result.map_err(|_| {
            let output = VM::error_info().unwrap();

            // error cleanup
            VM::clear_error_info();

            output
        })
    }

    /// Checks whether the object is `nil`
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Hash, NilClass, Object, VM};
    /// # VM::init();
    ///
    /// assert!(NilClass::new().is_nil());
    /// assert!(!Hash::new().is_nil());
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// nil.nil? == true
    /// {}.nil? == false
    /// ```
    fn is_nil(&self) -> bool {
        self.value().is_nil()
    }

    /// Converts struct to `AnyObject`
    ///
    /// See docs for `AnyObject` class for more details.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Integer, Object, VM};
    /// # VM::init();
    ///
    /// let array = Array::new().push(Integer::new(1));
    /// let args = [Integer::new(1).to_any_object()];
    /// let index = unsafe { array.send("find_index", &args) }
    ///                           .try_convert_to::<Integer>();
    ///
    /// assert_eq!(index, Ok(Integer::new(0)));
    /// ```
    fn to_any_object(&self) -> AnyObject {
        AnyObject::from(self.value())
    }

    /// Gets an instance variable of object
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use]
    /// extern crate rutie;
    ///
    /// use rutie::{AnyObject, Class, Integer, Object, VM};
    ///
    /// class!(Counter);
    ///
    /// methods!(
    ///     Counter,
    ///     rtself,
    ///
    ///     fn counter_initialize() -> AnyObject {
    ///         rtself.instance_variable_set("@state", Integer::new(0))
    ///     }
    ///
    ///     fn counter_increment() -> AnyObject {
    ///         // Using unsafe conversion, because we are sure that `@state` is always a `Integer`
    ///         // and we don't provide an interface to set the value externally
    ///         let state = unsafe {
    ///             rtself.instance_variable_get("@state").to::<Integer>().to_i64()
    ///         };
    ///
    ///         rtself.instance_variable_set("@state", Integer::new(state + 1))
    ///     }
    ///
    ///     fn counter_state() -> Integer {
    ///         unsafe { rtself.instance_variable_get("@state").to::<Integer>() }
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     let counter = Class::new("Counter", None).define(|klass| {
    ///         klass.def("initialize", counter_initialize);
    ///         klass.def("increment!", counter_increment);
    ///         klass.def("state", counter_state);
    ///     }).new_instance(&[]);
    ///
    ///     unsafe { counter.send("increment!", &[]) };
    ///
    ///     let new_state = unsafe { counter.send("state", &[]) }.try_convert_to::<Integer>();
    ///
    ///     assert_eq!(new_state, Ok(Integer::new(1)));
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Counter
    ///   def initialize
    ///     @state = 0
    ///   end
    ///
    ///   def increment!
    ///     @state += 1
    ///   end
    ///
    ///   def state
    ///     @state
    ///   end
    /// end
    ///
    /// counter = Counter.new
    /// counter.increment!
    ///
    /// new_state = counter.state
    ///
    /// new_state == 1
    /// ```
    fn instance_variable_get(&self, variable: &str) -> AnyObject {
        let result = class::instance_variable_get(self.value(), variable);

        AnyObject::from(result)
    }

    /// Sets an instance variable for object
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{AnyObject, Class, Integer, Object, VM, class, methods};
    ///
    /// class!(Counter);
    ///
    /// methods!(
    ///     Counter,
    ///     rtself,
    ///
    ///     fn counter_initialize() -> AnyObject {
    ///         rtself.instance_variable_set("@state", Integer::new(0))
    ///     }
    ///
    ///     fn counter_increment() -> AnyObject {
    ///         // Using unsafe conversion, because we are sure that `@state` is always a `Integer`
    ///         // and we don't provide an interface to set the value externally
    ///         let state = unsafe {
    ///             rtself.instance_variable_get("@state").to::<Integer>().to_i64()
    ///         };
    ///
    ///         rtself.instance_variable_set("@state", Integer::new(state + 1))
    ///     }
    ///
    ///     fn counter_state() -> Integer {
    ///         unsafe { rtself.instance_variable_get("@state").to::<Integer>() }
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     let counter = Class::new("Counter", None).define(|klass| {
    ///         klass.def("initialize", counter_initialize);
    ///         klass.def("increment!", counter_increment);
    ///         klass.def("state", counter_state);
    ///     }).new_instance(&[]);
    ///
    ///     unsafe { counter.send("increment!", &[]) };
    ///
    ///     let new_state = unsafe { counter.send("state", &[]) }.try_convert_to::<Integer>();
    ///
    ///     assert_eq!(new_state, Ok(Integer::new(1)));
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Counter
    ///   def initialize
    ///     @state = 0
    ///   end
    ///
    ///   def increment!
    ///     @state += 1
    ///   end
    ///
    ///   def state
    ///     @state
    ///   end
    /// end
    ///
    /// counter = Counter.new
    /// counter.increment!
    ///
    /// new_state = counter.state
    ///
    /// new_state == 1
    /// ```
    fn instance_variable_set<T: Object>(&mut self, variable: &str, value: T) -> AnyObject {
        let result = class::instance_variable_set(self.value(), variable, value.value());

        AnyObject::from(result)
    }

    /// Returns the freeze status of the object.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// let frozen_string = RString::new_utf8("String").freeze();
    ///
    /// assert!(frozen_string.is_frozen());
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// frozen_string = 'String'.freeze
    ///
    /// frozen_string.frozen? == true
    /// ```
    fn is_frozen(&self) -> bool {
        self.value().is_frozen()
    }

    /// Prevents further modifications to the object.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// let mut string = RString::new_utf8("String");
    ///
    /// assert!(!string.is_frozen(), "String should not be frozen");
    ///
    /// let frozen_string = RString::new_utf8("String").freeze();
    ///
    /// assert!(frozen_string.is_frozen(), "String should be frozen");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// string = 'String'
    ///
    /// string.frozen? == false
    ///
    /// frozen_string = 'String'.freeze
    ///
    /// frozen_string.frozen? == true
    /// ```
    fn freeze(&mut self) -> Self {
        let result = class::freeze(self.value());

        Self::from(result)
    }

    /// Unsafely casts current object to the specified Ruby type
    ///
    /// This operation in unsafe, because it does not perform any validations on the object, but
    /// it is faster than `try_convert_to()`.
    ///
    /// Use it when:
    ///
    ///  - you own the Ruby code which passes the object to Rust;
    ///  - you are sure that the object always has correct type;
    ///  - Ruby code has a good test coverage.
    ///
    /// This function is used by `unsafe_methods!` macro for argument casting.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{AnyObject, Integer, Object, VM};
    /// # VM::init();
    ///
    /// let integer_as_any_object = Integer::new(1).to_any_object();
    ///
    /// let integer = unsafe { integer_as_any_object.to::<Integer>() };
    ///
    /// assert_eq!(integer.to_i64(), 1);
    /// ```
    unsafe fn to<T: Object>(&self) -> T {
        T::from(self.value())
    }

    /// Safely casts current object to the specified Ruby type
    ///
    /// This function is used by `methods!` macro for argument casting.
    ///
    /// See documentation for `VerifiedObject` trait to enable safe conversions
    /// for custom classes.
    ///
    /// # Examples
    ///
    /// ### Basic conversions
    ///
    /// ```
    /// use rutie::{AnyException, Exception, Integer, Object, RString, VM};
    /// # VM::init();
    ///
    /// let integer_as_any_object = Integer::new(1).to_any_object();
    /// let converted_integer = integer_as_any_object.try_convert_to::<Integer>();
    ///
    /// assert_eq!(converted_integer, Ok(Integer::new(1)));
    ///
    /// let string = RString::new_utf8("string");
    /// let string_as_integer = string.try_convert_to::<Integer>();
    /// let expected_error = AnyException::new("TypeError", Some("Error converting to Integer"));
    ///
    /// assert_eq!(string_as_integer, Err(expected_error));
    /// ```
    ///
    /// ### Method arguments
    ///
    /// To launch a server in Rust, you plan to write a simple `Server` class
    ///
    /// ```ruby
    /// class Server
    ///   def start(address)
    ///     # ...
    ///   end
    /// end
    /// ```
    ///
    /// The `address` must be `Hash` with the following structure:
    ///
    /// ```ruby
    /// {
    ///   host: 'localhost',
    ///   port: 8080,
    /// }
    /// ```
    ///
    /// You want to extract port from it. Default port is `8080` in case when:
    ///
    ///  - `address` is not a `Hash`
    ///  - `address[:port]` is not present
    ///  - `address[:port]` is not a `Integer`
    ///
    /// ```no_run
    /// #[macro_use]
    /// extern crate rutie;
    ///
    /// use rutie::{Class, Integer, Hash, NilClass, Object, Symbol, VM, class, methods};
    ///
    /// class!(Server);
    ///
    /// methods!(
    ///     Server,
    ///     rtself,
    ///
    ///     fn start(address: Hash) -> NilClass {
    ///         let default_port = 8080;
    ///
    ///         let port = address
    ///             .map(|hash| hash.at(&Symbol::new("port")))
    ///             .and_then(|port| port.try_convert_to::<Integer>())
    ///             .map(|port| port.to_i64())
    ///             .unwrap_or(default_port);
    ///
    ///         // Start server...
    ///
    ///         NilClass::new()
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::new("Server", None).define(|klass| {
    ///         klass.def("start", start);
    ///     });
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Server
    ///   def start(address)
    ///     default_port = 8080
    ///
    ///     port =
    ///       if address.is_a?(Hash) && address[:port].is_a?(Integer)
    ///         address[:port]
    ///       else
    ///         default_port
    ///       end
    ///
    ///     # Start server...
    ///   end
    /// end
    /// ```
    fn try_convert_to<T: VerifiedObject>(&self) -> Result<T, AnyException> {
        if T::is_correct_type(self) {
            let converted_object = unsafe { self.to::<T>() };

            Ok(converted_object)
        } else {
            Err(AnyException::new("TypeError", Some(T::error_message())))
        }
    }

    /// Determines the value type of the object
    ///
    /// # Example
    ///
    /// ```
    /// use rutie::{AnyObject, Fixnum, Object, VM};
    /// use rutie::types::ValueType;
    /// # VM::init();
    ///
    /// let any_object = Fixnum::new(1).to_any_object();
    ///
    /// assert_eq!(any_object.ty(), ValueType::Fixnum);
    /// ```
    fn ty(&self) -> ValueType {
        self.value().ty()
    }
}

impl<Obj: Object> Object for Option<Obj>
where
    Option<Obj>: From<Value>,
{
    fn value(&self) -> Value {
        match self {
            Some(val) => val.value(),
            None => NilClass::new().into(),
        }
    }
}
