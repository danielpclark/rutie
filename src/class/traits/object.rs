use std::{cmp::Ordering, convert::From};

use crate::{
    binding::{class, enumerator, exception, global::ValueType, object, rproc, vm},
    typed_data::DataTypeWrapper,
    types::{Callback, Value},
    util,
};

use crate::{
    AnyException, AnyObject, Array, Boolean, Class, Exception, Integer, Method, NilClass, Proc,
    RString, VerifiedObject, VM,
};

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
    ///     value: Value
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
    ///
    /// # rutie::VM::init();
    /// let ruby_array = rutie::VM::eval("[1, 2]").unwrap();
    /// let array = Array::from(ruby_array.value());
    ///
    /// assert_eq!(array.value(), ruby_array.value());
    /// assert_eq!(array.inspect_object().to_str(), "[1, 2]");
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
    /// #[macro_use] extern crate rutie;
    /// #[macro_use] extern crate lazy_static;
    ///
    /// use rutie::{AnyObject, Class, Fixnum, Object, RString, VM};
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
    ///     fn ruby_server_new(host: RString, port: Fixnum) -> AnyObject {
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
    ///     fn ruby_server_port() -> Fixnum {
    ///         let port = rtself.get_data(&*SERVER_WRAPPER).port();
    ///
    ///         Fixnum::new(port as i64)
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
    ///
    ///     let server = VM::eval("RubyServer.new('127.0.0.1', 3000)").unwrap();
    ///     assert_eq!(server.get_data(&*SERVER_WRAPPER).port(), 3000);
    ///     assert_eq!(server.get_data(&*SERVER_WRAPPER).host(), "127.0.0.1");
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
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    /// #[macro_use] extern crate lazy_static;
    ///
    /// use rutie::{AnyObject, Class, Object, VM};
    ///
    /// pub struct Counter {
    ///     count: u32,
    /// }
    ///
    /// wrappable_struct!(Counter, CounterWrapper, COUNTER_WRAPPER);
    ///
    /// fn main() {
    ///     # VM::init();
    ///     let mut counter: AnyObject =
    ///         Class::new("Counter", None).wrap_data(Counter { count: 0 }, &*COUNTER_WRAPPER);
    ///
    ///     counter.get_data_mut(&*COUNTER_WRAPPER).count += 2;
    ///
    ///     assert_eq!(counter.get_data(&*COUNTER_WRAPPER).count, 2);
    /// }
    /// ```
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
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Fixnum, Object, RString, VM};
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
    ///     # VM::init();
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
    ///
    ///     let greeting = VM::eval("Hello::Nested.nested_greeting").unwrap();
    ///     assert_eq!(greeting.try_convert_to::<RString>().unwrap().to_str(), "Greeting from nested class");
    ///     assert!(VM::eval("Hello.new.reader").unwrap().is_nil());
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
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{AnyObject, Class, Fixnum, Object, RString, VM};
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
    /// of some class (like `Fixnum`, `String`, `Array` etc).
    ///
    /// Use this method only on classes (or singleton classes of objects).
    ///
    /// # Examples
    ///
    /// ### The famous String#blank? method
    ///
    /// ```rust
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Boolean, Class, Object, RString, VM};
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
    ///         klass.define_method("blank?", is_blank);
    ///     });
    ///
    ///     assert!(VM::eval("'   '.blank?").unwrap().value().is_true());
    ///     assert!(VM::eval("' x '.blank?").unwrap().value().is_false());
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
    /// Raise `Fixnum` to the power of `exp`.
    ///
    /// ```rust
    /// #[macro_use] extern crate rutie;
    ///
    /// use std::error::Error;
    ///
    /// use rutie::{Class, Fixnum, Object, Exception, VM};
    ///
    /// methods!(
    ///     Fixnum,
    ///     rtself,
    ///
    ///     fn pow(exp: Fixnum) -> Fixnum {
    ///         // `exp` is not a valid `Fixnum`, raise an exception
    ///         if let Err(ref error) = exp {
    ///             VM::raise(error.class(), &error.message());
    ///         }
    ///
    ///         // We can safely unwrap here, because an exception was raised if `exp` is `Err`
    ///         let exp = exp.unwrap().to_i64() as u32;
    ///
    ///         Fixnum::new(rtself.to_i64().pow(exp))
    ///     }
    ///
    ///     fn pow_with_default_argument(exp: Fixnum) -> Fixnum {
    ///         let default_exp = 0;
    ///         let exp = exp.map(|exp| exp.to_i64()).unwrap_or(default_exp);
    ///
    ///         let result = rtself.to_i64().pow(exp as u32);
    ///
    ///         Fixnum::new(result)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Fixnum").define(|klass| {
    ///         klass.def("pow", pow);
    ///         klass.def("pow_with_default_argument", pow_with_default_argument);
    ///     });
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Fixnum
    ///   def pow(exp)
    ///     raise ArgumentError unless exp.is_a?(Fixnum)
    ///
    ///     self ** exp
    ///   end
    ///
    ///   def pow_with_default_argument(exp)
    ///     default_exp = 0
    ///     exp = default_exp unless exp.is_a?(Fixnum)
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
    /// of some class (like `Fixnum`, `String`, `Array` etc).
    ///
    /// Use this method only on classes (or singleton classes of objects).
    ///
    /// # Examples
    ///
    /// ### The famous String#blank? method
    ///
    /// ```rust
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Boolean, Class, Object, RString, VM};
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
    ///         klass.define_private_method("blank?", is_blank);
    ///     });
    ///
    ///     // Callable from inside the class, not from outside.
    ///     assert!(VM::eval("' '.send(:blank?)").unwrap().value().is_true());
    ///     assert!(VM::eval("' '.blank?").is_err());
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
    /// Raise `Fixnum` to the power of `exp`.
    ///
    /// ```rust
    /// #[macro_use] extern crate rutie;
    ///
    /// use std::error::Error;
    ///
    /// use rutie::{Class, Fixnum, Object, Exception, VM};
    ///
    /// methods!(
    ///     Fixnum,
    ///     rtself,
    ///
    ///     fn pow(exp: Fixnum) -> Fixnum {
    ///         // `exp` is not a valid `Fixnum`, raise an exception
    ///         if let Err(ref error) = exp {
    ///             VM::raise(error.class(), &error.message());
    ///         }
    ///
    ///         // We can safely unwrap here, because an exception was raised if `exp` is `Err`
    ///         let exp = exp.unwrap().to_i64() as u32;
    ///
    ///         Fixnum::new(rtself.to_i64().pow(exp))
    ///     }
    ///
    ///     fn pow_with_default_argument(exp: Fixnum) -> Fixnum {
    ///         let default_exp = 0;
    ///         let exp = exp.map(|exp| exp.to_i64()).unwrap_or(default_exp);
    ///
    ///         let result = rtself.to_i64().pow(exp as u32);
    ///
    ///         Fixnum::new(result)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Fixnum").define(|klass| {
    ///         klass.def_private("pow", pow);
    ///         klass.def_private("pow_with_default_argument", pow_with_default_argument);
    ///     });
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Fixnum
    ///   private
    ///   def pow(exp)
    ///     raise ArgumentError unless exp.is_a?(Fixnum)
    ///
    ///     self ** exp
    ///   end
    ///
    ///   def pow_with_default_argument(exp)
    ///     default_exp = 0
    ///     exp = default_exp unless exp.is_a?(Fixnum)
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
    ///
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
    /// use rutie::{AnyObject, Class, Fixnum, Object, RString, VM};
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
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Object, RString, VM};
    ///
    /// class!(Greeter);
    ///
    /// methods!(
    ///     Greeter,
    ///     rtself,
    ///
    ///     fn hello() -> RString {
    ///         RString::new_utf8("hello")
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::new("Greeter", None).define(|klass| {
    ///         klass.def("hello", hello);
    ///     });
    ///
    ///     let result = VM::eval("Greeter.new.hello").unwrap();
    ///     assert_eq!(result.try_convert_to::<RString>().unwrap().to_str(), "hello");
    /// }
    /// ```
    fn def<I: Object, O: Object>(&mut self, name: &str, callback: Callback<I, O>) {
        self.define_method(name, callback);
    }

    /// An alias for `define_private_method` (similar to Ruby syntax `private def some_method`).
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Object, RString, VM};
    ///
    /// class!(Greeter);
    ///
    /// methods!(
    ///     Greeter,
    ///     rtself,
    ///
    ///     fn hello() -> RString {
    ///         RString::new_utf8("hello")
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::new("Greeter", None).define(|klass| {
    ///         klass.def_private("hello", hello);
    ///     });
    ///
    ///     assert!(VM::eval("Greeter.new.hello").is_err());
    ///     let result = VM::eval("Greeter.new.send(:hello)").unwrap();
    ///     assert_eq!(result.try_convert_to::<RString>().unwrap().to_str(), "hello");
    /// }
    /// ```
    fn def_private<I: Object, O: Object>(&mut self, name: &str, callback: Callback<I, O>) {
        self.define_private_method(name, callback);
    }

    /// An alias for `define_singleton_method` (similar to Ruby `def self.some_method`).
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Object, RString, VM};
    ///
    /// class!(Greeter);
    ///
    /// methods!(
    ///     Greeter,
    ///     rtself,
    ///
    ///     fn hello() -> RString {
    ///         RString::new_utf8("hello")
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::new("Greeter", None).define(|klass| {
    ///         klass.def_self("hello", hello);
    ///     });
    ///
    ///     let result = VM::eval("Greeter.hello").unwrap();
    ///     assert_eq!(result.try_convert_to::<RString>().unwrap().to_str(), "hello");
    /// }
    /// ```
    fn def_self<I: Object, O: Object>(&mut self, name: &str, callback: Callback<I, O>) {
        self.define_singleton_method(name, callback);
    }

    /// Calls a given method on an object similarly to Ruby `Object#send` method
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, RString, VM};
    /// # VM::init();
    ///
    /// let array = Array::new().push(Fixnum::new(1));
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
    /// use rutie::{Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let a = Fixnum::new(4);
    /// let b = Fixnum::new(7);
    /// let c = Fixnum::new(4);
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
    /// use rutie::{Fixnum, Object, Class, VM};
    /// # VM::init();
    ///
    /// let a = Fixnum::new(4);
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
    /// use rutie::{Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let a = Fixnum::new(4);
    /// let b = Fixnum::new(7);
    /// let c = Fixnum::new(4);
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
    /// use rutie::{RString, Fixnum, Object, Exception, Class, VM, Boolean};
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
    /// use rutie::{RString, Fixnum, Object, Exception, Class, VM, Boolean};
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

    /// Calls a given method on an object with a Rust closure as its block,
    /// like Ruby's `object.method(*arguments) { |*values| ... }`
    /// (`rb_block_call`).
    ///
    /// The closure receives the values yielded to the block and its result
    /// is the block's value. Inside the closure [`VM::iter_break`] and
    /// [`VM::iter_break_value`] work like Ruby's `break`. A panic in the
    /// closure is raised as a Ruby `RuntimeError`.
    ///
    /// [`VM::iter_break`]: struct.VM.html#method.iter_break
    /// [`VM::iter_break_value`]: struct.VM.html#method.iter_break_value
    ///
    /// # Safety
    ///
    /// Like `send`, an exception raised by the method or the block is not
    /// caught; see
    /// [`protect_send_with_block`](#method.protect_send_with_block).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let array: Array = (1..=3).map(|i| Fixnum::new(i).to_any_object()).collect();
    ///
    /// // [1, 2, 3].map { |x| x * 2 }
    /// let doubled = unsafe {
    ///     array.send_with_block("map", &[], |values| {
    ///         let x = values[0].try_convert_to::<Fixnum>().unwrap().to_i64();
    ///
    ///         Fixnum::new(x * 2).into()
    ///     })
    /// };
    ///
    /// let doubled = doubled.try_convert_to::<Array>().unwrap();
    ///
    /// assert_eq!(doubled.at(2).try_convert_to::<Fixnum>(), Ok(Fixnum::new(6)));
    ///
    /// // All yielded values are passed: [[:a, 1]].each_with_index { |pair, index| ... }
    /// let mut count = 0;
    ///
    /// unsafe {
    ///     array.send_with_block("each_with_index", &[], |values| {
    ///         assert_eq!(values.len(), 2);
    ///         count += 1;
    ///
    ///         values[1].clone()
    ///     })
    /// };
    ///
    /// assert_eq!(count, 3);
    /// ```
    unsafe fn send_with_block<F>(
        &self,
        method: &str,
        arguments: &[AnyObject],
        mut block: F,
    ) -> AnyObject
    where
        F: FnMut(&[AnyObject]) -> AnyObject,
    {
        let arguments = util::arguments_to_values(arguments);
        let result = vm::call_method_with_block(self.value(), method, &arguments, |values| {
            // `AnyObject` is a `#[repr(C)]` wrapper around a single `Value`.
            let values =
                std::slice::from_raw_parts(values.as_ptr() as *const AnyObject, values.len());

            block(values).value()
        });

        AnyObject::from(result)
    }

    /// Like [`send_with_block`](#method.send_with_block), but an exception
    /// raised by the method or the block is returned as `Err` instead of
    /// propagating.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Exception, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let array: Array = (1..=3).map(|i| Fixnum::new(i).to_any_object()).collect();
    ///
    /// let sum = array.protect_send_with_block("sum", &[], |values| values[0].clone());
    ///
    /// assert_eq!(sum.unwrap().try_convert_to::<Fixnum>(), Ok(Fixnum::new(6)));
    ///
    /// let result = array.protect_send_with_block("each", &[], |_values| {
    ///     unsafe { VM::eval_str("raise 'stop'") }
    /// });
    ///
    /// assert_eq!(result.unwrap_err().message(), "stop");
    ///
    /// // Panics are turned into Ruby exceptions too.
    /// let result = array.protect_send_with_block("each", &[], |_values| panic!("boom"));
    ///
    /// assert_eq!(result.unwrap_err().message(), "Rust panic: boom");
    /// ```
    fn protect_send_with_block<F>(
        &self,
        method: &str,
        arguments: &[AnyObject],
        block: F,
    ) -> Result<AnyObject, AnyException>
    where
        F: FnMut(&[AnyObject]) -> AnyObject,
    {
        vm::protect_value(|| unsafe { self.send_with_block(method, arguments, block) }.value())
            .map(AnyObject::from)
            .map_err(AnyException::from)
    }

    /// Calls the object's `initialize` method with `arguments`, passing on
    /// the block of the current method call, if any (`rb_obj_call_init`).
    ///
    /// Used by custom allocating constructors, see
    /// [`Class::define_alloc_func`](struct.Class.html#method.define_alloc_func).
    ///
    /// # Safety
    ///
    /// Like `send`, an exception raised by `initialize` is not caught.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{AnyObject, Class, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("class Point; attr_reader :x; def initialize(x); @x = x; end; end").unwrap();
    ///
    /// let point = unsafe { Class::from_existing("Point").send("allocate", &[]) };
    ///
    /// unsafe { point.call_init(&[Fixnum::new(3).into()]) };
    ///
    /// let x = unsafe { point.send("x", &[]) };
    ///
    /// assert_eq!(x.try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
    /// ```
    unsafe fn call_init(&self, arguments: &[AnyObject]) {
        let arguments = util::arguments_to_values(arguments);

        vm::call_init(self.value(), &arguments)
    }

    /// Raises `FrozenError` if the object is frozen (`rb_check_frozen`).
    ///
    /// Call it at the start of a Rust method that modifies its receiver.
    /// To check without raising, use [`is_frozen`](#method.is_frozen).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("mutable");
    /// string.check_frozen(); // does nothing
    ///
    /// let frozen = RString::new_utf8("frozen").freeze();
    ///
    /// let result = VM::protect(|| {
    ///     frozen.check_frozen();
    ///     frozen.to_any_object()
    /// });
    ///
    /// assert!(result.is_err());
    /// assert!(Class::from_existing("FrozenError").case_equals(&VM::error_pop().unwrap()));
    /// ```
    fn check_frozen(&self) {
        exception::check_frozen(self.value())
    }

    /// Raises `TypeError` unless the object has the given built-in type
    /// (`rb_check_type`, C's `Check_Type`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{AnyException, Exception, Fixnum, Object, RString, VM};
    /// use rutie::types::ValueType;
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("text");
    /// string.check_type(ValueType::RString); // does nothing
    ///
    /// let result = VM::protect(|| {
    ///     Fixnum::new(1).check_type(ValueType::RString);
    ///     string.to_any_object()
    /// });
    ///
    /// assert!(result.is_err());
    /// assert_eq!(
    ///     VM::error_pop().unwrap().message(),
    ///     "wrong argument type Integer (expected String)"
    /// );
    /// ```
    fn check_type(&self, value_type: ValueType) {
        exception::check_type(self.value(), value_type)
    }

    /// Returns a shallow copy of the object, without its singleton class or
    /// frozen state (Ruby's `dup`, `rb_obj_dup`).
    ///
    /// Raises `TypeError` for objects that cannot be copied.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// let original = RString::new_utf8("text").freeze();
    /// let copy = original.dup();
    ///
    /// assert!(copy.equals(&original));
    /// assert!(!copy.is_equal(&original));
    /// assert!(!copy.is_frozen());
    /// ```
    fn dup(&self) -> Self {
        Self::from(object::dup(self.value()))
    }

    /// Returns a shallow copy of the object, including its singleton class
    /// and frozen state (Ruby's `clone`, `rb_obj_clone`).
    ///
    /// Named `clone_object` so it does not clash with `Clone::clone`.
    /// Raises `TypeError` for objects that cannot be copied.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// let original = RString::new_utf8("text").freeze();
    /// let copy = original.clone_object();
    ///
    /// assert!(copy.equals(&original));
    /// assert!(!copy.is_equal(&original));
    /// assert!(copy.is_frozen());
    /// ```
    fn clone_object(&self) -> Self {
        Self::from(object::clone(self.value()))
    }

    /// Returns the object's `object_id` (`rb_obj_id`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("text");
    /// let same = string.to_any_object();
    ///
    /// assert_eq!(string.object_id(), same.object_id());
    /// assert_ne!(string.object_id(), RString::new_utf8("text").object_id());
    /// ```
    fn object_id(&self) -> Integer {
        Integer::from(object::id(self.value()))
    }

    /// Returns the result of the object's `inspect` method as a string
    /// (`rb_inspect`).
    ///
    /// Named `inspect_object` so it does not clash with `Exception::inspect`.
    /// Raises whatever `inspect` raises.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, RString, Symbol, VM};
    /// # VM::init();
    ///
    /// let array = Array::new().push(Fixnum::new(1)).push(Symbol::new("a")).push(RString::new_utf8("b"));
    ///
    /// assert_eq!(array.inspect_object().to_str(), r#"[1, :a, "b"]"#);
    /// ```
    fn inspect_object(&self) -> RString {
        RString::from(object::inspect(self.value()))
    }

    /// Returns the object's `to_s` as a string, falling back to the default
    /// `#<Class:0x...>` form when `to_s` does not return a `String`
    /// (`rb_obj_as_string`, what string interpolation uses).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Fixnum::new(42).as_string().to_str(), "42");
    /// assert_eq!(Symbol::new("name").as_string().to_str(), "name");
    /// ```
    fn as_string(&self) -> RString {
        RString::from(object::as_string(self.value()))
    }

    /// Returns `true` if `klass` (a class or module) is the object's class,
    /// one of its ancestors or a module included in them (Ruby's `kind_of?`,
    /// `rb_obj_is_kind_of`).
    ///
    /// Raises `TypeError` if `klass` is not a class or module.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Fixnum, Module, Object, VM};
    /// # VM::init();
    ///
    /// let number = Fixnum::new(1);
    ///
    /// assert!(number.is_kind_of(&Class::from_existing("Integer")));
    /// assert!(number.is_kind_of(&Class::from_existing("Numeric")));
    /// assert!(number.is_kind_of(&Module::from_existing("Comparable")));
    /// assert!(!number.is_kind_of(&Class::from_existing("String")));
    /// ```
    fn is_kind_of<T: Object>(&self, klass: &T) -> bool {
        class::is_kind_of(self.value(), klass.value())
    }

    /// Returns `true` if the object's class is exactly `klass` (Ruby's
    /// `instance_of?`, `rb_obj_is_instance_of`).
    ///
    /// Raises `TypeError` if `klass` is not a class or module.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let number = Fixnum::new(1);
    ///
    /// assert!(number.is_instance_of(&Class::from_existing("Integer")));
    /// assert!(!number.is_instance_of(&Class::from_existing("Numeric")));
    /// ```
    fn is_instance_of<T: Object>(&self, klass: &T) -> bool {
        object::is_instance_of(self.value(), klass.value())
    }

    /// Returns the object's method `name` as a [`Method`](struct.Method.html)
    /// (Ruby's `method`, `rb_obj_method`), or the `NameError` if there is no
    /// such method.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Exception, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let method = Fixnum::new(20).method("+").unwrap();
    /// let result = method.call(&[Fixnum::new(22).into()]);
    ///
    /// assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));
    ///
    /// let error = Fixnum::new(1).method("no_such_method").unwrap_err();
    ///
    /// assert!(error.message().contains("no_such_method"));
    /// ```
    fn method(&self, name: &str) -> Result<Method, AnyException> {
        let object = self.value();

        vm::protect_value(|| object::method(object, name))
            .map(Method::from)
            .map_err(AnyException::from)
    }

    /// Returns the arity of the object's method `name`, or `0` if it is not
    /// defined (`rb_obj_method_arity`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Object, VM};
    /// # VM::init();
    ///
    /// let array = Array::new();
    ///
    /// assert_eq!(array.method_arity("push"), -1);
    /// assert_eq!(array.method_arity("length"), 0);
    /// ```
    fn method_arity(&self, name: &str) -> i32 {
        rproc::object_method_arity(self.value(), name)
    }

    /// Compares the object with `other` using Ruby's `<=>`, returning the
    /// `ArgumentError` Ruby raises when they cannot be compared (`<=>`
    /// returns `nil`), via `rb_cmpint`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Exception, Fixnum, Float, Object, RString, VM};
    /// use std::cmp::Ordering;
    /// # VM::init();
    ///
    /// assert_eq!(Fixnum::new(1).try_compare(&Float::new(1.5)).unwrap(), Ordering::Less);
    ///
    /// let error = Fixnum::new(1).try_compare(&RString::new_utf8("1")).unwrap_err();
    ///
    /// assert!(error.message().starts_with("comparison of Integer with String failed"));
    /// ```
    fn try_compare<T: Object>(&self, other: &T) -> Result<Ordering, AnyException> {
        let (object, other) = (self.value(), other.value());
        let mut ordering = 0;

        vm::protect_value(|| {
            let result = vm::call_method(object, "<=>", &[other]);
            ordering = enumerator::cmpint(result, object, other);

            NilClass::new().value()
        })
        .map(|_| ordering.cmp(&0))
        .map_err(AnyException::from)
    }

    /// Calls `method` if the object responds to it, returning `None` when
    /// it does not (`rb_check_funcall`). A `respond_to_missing?` or
    /// `method_missing` defined by the object is respected.
    ///
    /// # Safety
    ///
    /// Like `send`, an exception raised by the method is not caught.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, RString, VM};
    /// # VM::init();
    ///
    /// let string = RString::new_utf8("abc");
    ///
    /// let length = unsafe { string.check_send("length", &[]) };
    /// assert_eq!(length.unwrap().try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
    ///
    /// let missing = unsafe { string.check_send("no_such_method", &[]) };
    /// assert!(missing.is_none());
    /// ```
    unsafe fn check_send(&self, method: &str, arguments: &[AnyObject]) -> Option<AnyObject> {
        let arguments = util::arguments_to_values(arguments);

        object::check_funcall(self.value(), method, &arguments).map(AnyObject::from)
    }

    /// Calls a given method on an object, passing `block` as its block
    /// (Ruby's `object.method(*arguments, &block)`, `rb_funcall_with_block`).
    ///
    /// # Safety
    ///
    /// Like `send`, an exception raised by the method is not caught.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, Proc, VM};
    /// # VM::init();
    ///
    /// let array: Array = (1..=3).map(|i| Fixnum::new(i).to_any_object()).collect();
    /// let double = VM::eval("proc { |x| x * 2 }").unwrap().try_convert_to::<Proc>().unwrap();
    ///
    /// let doubled = unsafe { array.send_with_proc("map", &[], &double) };
    /// let doubled = doubled.try_convert_to::<Array>().unwrap();
    ///
    /// assert_eq!(doubled.at(2).try_convert_to::<Fixnum>(), Ok(Fixnum::new(6)));
    /// ```
    unsafe fn send_with_proc(
        &self,
        method: &str,
        arguments: &[AnyObject],
        block: &Proc,
    ) -> AnyObject {
        let arguments = util::arguments_to_values(arguments);

        AnyObject::from(vm::call_method_with_proc(
            self.value(),
            method,
            &arguments,
            block.value(),
        ))
    }

    /// Calls a given method on an object with `keywords` passed as keyword
    /// arguments (`rb_funcallv_kw`), like Ruby's
    /// `object.method(*arguments, **keywords)`.
    ///
    /// Only available on Ruby 2.7. On Ruby 2.5 and 2.6 a trailing `Hash`
    /// argument given to `send` is taken as keywords.
    ///
    /// # Safety
    ///
    /// Like `send`, an exception raised by the method is not caught.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Hash, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// # #[cfg(ruby_gte_2_7)]
    /// # {
    /// VM::eval("def rutie_kw(a, b: 0); a + b; end").unwrap();
    ///
    /// let mut keywords = Hash::new();
    /// keywords.store(Symbol::new("b"), Fixnum::new(2));
    ///
    /// let object = VM::eval("self").unwrap();
    /// let result = unsafe { object.send_with_keywords("rutie_kw", &[Fixnum::new(40).into()], keywords) };
    ///
    /// assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));
    /// # }
    /// ```
    #[cfg(ruby_gte_2_7)]
    unsafe fn send_with_keywords(
        &self,
        method: &str,
        arguments: &[AnyObject],
        keywords: crate::Hash,
    ) -> AnyObject {
        let mut arguments = util::arguments_to_values(arguments);
        arguments.push(keywords.value());

        AnyObject::from(vm::call_method_with_keywords(
            self.value(),
            method,
            &arguments,
        ))
    }

    /// Returns the names of the object's instance variables as an `Array`
    /// of `Symbol`s (`rb_obj_instance_variables`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let mut object = VM::eval("Object.new").unwrap();
    /// object.instance_variable_set("@count", Fixnum::new(1));
    ///
    /// let names = object.instance_variables();
    ///
    /// assert_eq!(names.length(), 1);
    /// assert_eq!(names.at(0).try_convert_to::<Symbol>(), Ok(Symbol::new("@count")));
    /// ```
    fn instance_variables(&self) -> Array {
        Array::from(object::instance_variables(self.value()))
    }

    /// Returns `true` if the instance variable `name` (such as `"@count"`)
    /// is set on the object (`rb_ivar_defined`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let mut object = VM::eval("Object.new").unwrap();
    ///
    /// assert!(!object.is_instance_variable_defined("@count"));
    ///
    /// object.instance_variable_set("@count", Fixnum::new(1));
    ///
    /// assert!(object.is_instance_variable_defined("@count"));
    /// ```
    fn is_instance_variable_defined(&self, name: &str) -> bool {
        object::is_instance_variable_defined(self.value(), name)
    }

    /// Removes the instance variable `name` (such as `"@count"`) from the
    /// object and returns its value, or `None` if it was not set
    /// (`rb_obj_remove_instance_variable`).
    ///
    /// Raises `FrozenError` if the object is frozen and the variable is set.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let mut object = VM::eval("Object.new").unwrap();
    /// object.instance_variable_set("@count", Fixnum::new(1));
    ///
    /// let removed = object.remove_instance_variable("@count");
    ///
    /// assert_eq!(removed.unwrap().try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// assert!(object.remove_instance_variable("@count").is_none());
    /// ```
    fn remove_instance_variable(&mut self, name: &str) -> Option<AnyObject> {
        if !self.is_instance_variable_defined(name) {
            return None;
        }

        Some(AnyObject::from(object::remove_instance_variable(
            self.value(),
            name,
        )))
    }

    /// Returns the object's `hash` value, the one `Hash` keys use (`rb_hash`).
    ///
    /// Named `hash_value` so it does not clash with `std::hash::Hash::hash`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// let a = RString::new_utf8("same");
    /// let b = RString::new_utf8("same");
    ///
    /// assert!(a.hash_value().equals(&b.hash_value()));
    /// assert!(!a.hash_value().equals(&RString::new_utf8("other").hash_value()));
    /// ```
    fn hash_value(&self) -> Integer {
        Integer::from(object::hash(self.value()))
    }

    /// Evaluates `code` with the object as `self` (Ruby's `instance_eval`,
    /// `rb_obj_instance_eval`), returning the result or the exception raised.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Exception, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let mut object = VM::eval("Object.new").unwrap();
    /// object.instance_variable_set("@secret", Fixnum::new(42));
    ///
    /// let secret = object.instance_eval("@secret").unwrap();
    ///
    /// assert_eq!(secret.try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));
    ///
    /// let error = object.instance_eval("raise 'no'").unwrap_err();
    ///
    /// assert_eq!(error.message(), "no");
    /// ```
    fn instance_eval(&self, code: &str) -> Result<AnyObject, AnyException> {
        let object = self.value();

        vm::protect_value(|| object::instance_eval(object, code))
            .map(AnyObject::from)
            .map_err(AnyException::from)
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
    /// use rutie::{Array, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let array = Array::new().push(Fixnum::new(1));
    /// let args = [Fixnum::new(1).to_any_object()];
    /// let index = unsafe { array.send("find_index", &args) }
    ///                           .try_convert_to::<Fixnum>();
    ///
    /// assert_eq!(index, Ok(Fixnum::new(0)));
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
    /// use rutie::{AnyObject, Class, Fixnum, Object, VM};
    ///
    /// class!(Counter);
    ///
    /// methods!(
    ///     Counter,
    ///     rtself,
    ///
    ///     fn counter_initialize() -> AnyObject {
    ///         rtself.instance_variable_set("@state", Fixnum::new(0))
    ///     }
    ///
    ///     fn counter_increment() -> AnyObject {
    ///         // Using unsafe conversion, because we are sure that `@state` is always a `Fixnum`
    ///         // and we don't provide an interface to set the value externally
    ///         let state = unsafe {
    ///             rtself.instance_variable_get("@state").to::<Fixnum>().to_i64()
    ///         };
    ///
    ///         rtself.instance_variable_set("@state", Fixnum::new(state + 1))
    ///     }
    ///
    ///     fn counter_state() -> Fixnum {
    ///         unsafe { rtself.instance_variable_get("@state").to::<Fixnum>() }
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
    ///     let new_state = unsafe { counter.send("state", &[]) }.try_convert_to::<Fixnum>();
    ///
    ///     assert_eq!(new_state, Ok(Fixnum::new(1)));
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
    /// #[macro_use]
    /// extern crate rutie;
    ///
    /// use rutie::{AnyObject, Class, Fixnum, Object, VM};
    ///
    /// class!(Counter);
    ///
    /// methods!(
    ///     Counter,
    ///     rtself,
    ///
    ///     fn counter_initialize() -> AnyObject {
    ///         rtself.instance_variable_set("@state", Fixnum::new(0))
    ///     }
    ///
    ///     fn counter_increment() -> AnyObject {
    ///         // Using unsafe conversion, because we are sure that `@state` is always a `Fixnum`
    ///         // and we don't provide an interface to set the value externally
    ///         let state = unsafe {
    ///             rtself.instance_variable_get("@state").to::<Fixnum>().to_i64()
    ///         };
    ///
    ///         rtself.instance_variable_set("@state", Fixnum::new(state + 1))
    ///     }
    ///
    ///     fn counter_state() -> Fixnum {
    ///         unsafe { rtself.instance_variable_get("@state").to::<Fixnum>() }
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
    ///     let new_state = unsafe { counter.send("state", &[]) }.try_convert_to::<Fixnum>();
    ///
    ///     assert_eq!(new_state, Ok(Fixnum::new(1)));
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
    /// use rutie::{AnyObject, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let fixnum_as_any_object = Fixnum::new(1).to_any_object();
    ///
    /// let fixnum = unsafe { fixnum_as_any_object.to::<Fixnum>() };
    ///
    /// assert_eq!(fixnum.to_i64(), 1);
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
    /// use rutie::{AnyException, Exception, Fixnum, Object, RString, VM};
    /// # VM::init();
    ///
    /// let fixnum_as_any_object = Fixnum::new(1).to_any_object();
    /// let converted_fixnum = fixnum_as_any_object.try_convert_to::<Fixnum>();
    ///
    /// assert_eq!(converted_fixnum, Ok(Fixnum::new(1)));
    ///
    /// let string = RString::new_utf8("string");
    /// let string_as_fixnum = string.try_convert_to::<Fixnum>();
    /// let expected_error = AnyException::new("TypeError", Some("Error converting to Fixnum"));
    ///
    /// assert_eq!(string_as_fixnum, Err(expected_error));
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
    ///  - `address[:port]` is not a `Fixnum`
    ///
    /// ```
    /// #[macro_use]
    /// extern crate rutie;
    ///
    /// use rutie::{Class, Fixnum, Hash, NilClass, Object, Symbol, VM};
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
    ///             .and_then(|port| port.try_convert_to::<Fixnum>())
    ///             .map(|port| port.to_i64())
    ///             .unwrap_or(default_port);
    ///
    ///         // Start server...
    ///         rtself.instance_variable_set("@port", Fixnum::new(port));
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
    ///
    ///     let port = |code: &str| VM::eval(code).unwrap().try_convert_to::<Fixnum>().unwrap().to_i64();
    ///     assert_eq!(port("s = Server.new; s.start(port: 3000); s.instance_variable_get(:@port)"), 3000);
    ///     assert_eq!(port("s = Server.new; s.start('localhost'); s.instance_variable_get(:@port)"), 8080);
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
    ///       if address.is_a?(Hash) && address[:port].is_a?(Fixnum)
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

#[cfg(test)]
mod tests {
    use crate::{
        AnyObject, Array, Class, Exception, Fixnum, Float, Hash, Integer, Module, NilClass, Object,
        Proc, RString, Symbol, VM,
    };

    #[test]
    fn test_copies_identity_and_strings() {
        crate::on_ruby_thread(|| {
            let object =
                VM::eval("o = Object.new; def o.extra; :singleton; end; o.freeze").unwrap();

            let dup = object.dup();
            assert!(!dup.is_frozen());
            assert!(!dup.respond_to("extra"));
            assert_ne!(dup.object_id().to_i64(), object.object_id().to_i64());

            let clone = object.clone_object();
            assert!(clone.is_frozen());
            assert!(clone.respond_to("extra"));

            let hash = VM::eval("{ a: 1 }").unwrap();
            assert_eq!(hash.inspect_object().to_str(), "{:a=>1}");
            assert_eq!(Fixnum::new(7).as_string().to_str(), "7");

            // `as_string` falls back to the default form when `to_s` is not a String.
            let odd = VM::eval("o = Object.new; def o.to_s; 1; end; o").unwrap();
            assert!(odd.as_string().to_str().starts_with("#<Object:"));

            let a = RString::new_utf8("key");
            let b = RString::new_utf8("key");
            assert!(a.hash_value().equals(&b.hash_value()));
        });
    }

    #[test]
    fn test_kind_of_and_instance_of() {
        crate::on_ruby_thread(|| {
            let array = Array::new();
            let enumerable = Module::from_existing("Enumerable");
            let object = Class::from_existing("Object");

            assert!(array.is_kind_of(&enumerable));
            assert!(array.is_kind_of(&object));
            assert!(!array.is_instance_of(&object));
            assert!(array.is_instance_of(&Class::from_existing("Array")));
        });
    }

    #[test]
    fn test_method_check_send_and_send_with_proc() {
        crate::on_ruby_thread(|| {
            let string = RString::new_utf8("abc");

            let upcase = string.method("upcase").unwrap();
            let result = unsafe { upcase.send("call", &[]) };
            assert_eq!(result.try_convert_to::<RString>().unwrap().to_str(), "ABC");
            assert!(string.method("nope").is_err());

            // `respond_to_missing?` and `method_missing` are honoured.
            let ghost = VM::eval(
                "o = Object.new
                 def o.respond_to_missing?(name, _ = false); name == :ghost; end
                 def o.method_missing(name, *args); name == :ghost ? :boo : super; end
                 o",
            )
            .unwrap();
            let boo = unsafe { ghost.check_send("ghost", &[]) }.unwrap();
            assert_eq!(boo.try_convert_to::<Symbol>(), Ok(Symbol::new("boo")));
            assert!(unsafe { ghost.check_send("other", &[]) }.is_none());

            let add = VM::eval("proc { |sum, x| sum + x }")
                .unwrap()
                .try_convert_to::<Proc>()
                .unwrap();
            let array: Array = (1..=4).map(|i| Fixnum::new(i).to_any_object()).collect();
            let sum = unsafe { array.send_with_proc("inject", &[Fixnum::new(0).into()], &add) };
            assert_eq!(sum.try_convert_to::<Fixnum>(), Ok(Fixnum::new(10)));
        });
    }

    #[cfg(ruby_gte_2_7)]
    #[test]
    fn test_send_with_keywords() {
        crate::on_ruby_thread(|| {
            VM::eval("def rutie_kw_test(*args, **kw); [args.size, kw[:x]]; end").unwrap();

            let mut keywords = Hash::new();
            keywords.store(Symbol::new("x"), Fixnum::new(9));

            let object = VM::eval("self").unwrap();
            let result = unsafe {
                object.send_with_keywords("rutie_kw_test", &[Fixnum::new(1).into()], keywords)
            };
            let result = result.try_convert_to::<Array>().unwrap();

            assert_eq!(result.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
            assert_eq!(result.at(1).try_convert_to::<Fixnum>(), Ok(Fixnum::new(9)));
        });
    }

    #[test]
    fn test_instance_variables_and_instance_eval() {
        crate::on_ruby_thread(|| {
            let mut object = VM::eval("Object.new").unwrap();

            object.instance_variable_set("@a", Fixnum::new(1));
            object.instance_variable_set("@b", Fixnum::new(2));
            assert_eq!(object.instance_variables().length(), 2);
            assert!(object.is_instance_variable_defined("@a"));

            let sum = object.instance_eval("@a + @b").unwrap();
            assert_eq!(sum.try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));

            let removed = object.remove_instance_variable("@a").unwrap();
            assert_eq!(removed.try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
            assert!(object.remove_instance_variable("@a").is_none());
            assert!(!object.is_instance_variable_defined("@a"));

            let error = object.instance_eval("undefined_thing").unwrap_err();
            assert!(Class::from_existing("NameError").case_equals(&error));
        });
    }

    #[test]
    fn test_kernel_conversions() {
        crate::on_ruby_thread(|| {
            assert_eq!(RString::convert(&Float::new(1.5)).unwrap().to_str(), "1.5");
            assert_eq!(Array::convert(&NilClass::new()).unwrap().length(), 0);
            assert_eq!(
                Integer::convert(&RString::new_utf8("0b101"))
                    .unwrap()
                    .to_i64(),
                5
            );
            assert!(Integer::convert(&NilClass::new()).is_err());
            assert_eq!(
                Float::convert(&RString::new_utf8("1e3")).unwrap().to_f64(),
                1000.0
            );
            assert!(Float::convert(&RString::new_utf8("")).is_err());
            assert_eq!(Hash::convert(&Array::new()).unwrap().length(), 0);
            assert!(Hash::convert(&RString::new_utf8("x")).is_err());

            let unconvertible: AnyObject = VM::eval("BasicObject.new").unwrap();
            assert!(RString::convert(&unconvertible).is_err());
            assert!(Integer::convert(&unconvertible)
                .unwrap_err()
                .message()
                .contains("BasicObject"));
        });
    }

    crate::class!(RutieObjectDefs);

    crate::methods!(
        RutieObjectDefs,
        rtself,
        fn rutie_obj_public() -> Symbol {
            Symbol::new("public")
        },
        fn rutie_obj_private() -> Symbol {
            Symbol::new("private")
        },
        fn rutie_obj_singleton() -> Symbol {
            Symbol::new("singleton")
        },
        fn rutie_obj_initialize(value: Fixnum) -> NilClass {
            rtself.instance_variable_set("@value", value.unwrap());
            NilClass::new()
        }
    );

    pub struct RutieObjectBox {
        count: i64,
    }

    crate::wrappable_struct!(RutieObjectBox, RutieObjectBoxWrapper, RUTIE_OBJECT_BOX);

    #[test]
    fn test_method_definition_and_dispatch() {
        crate::on_ruby_thread(|| {
            let mut class = Class::new("RutieObjectDefs", None);
            class.define_method("public_one", rutie_obj_public);
            class.define_private_method("private_one", rutie_obj_private);
            class.def_self("class_one", rutie_obj_singleton);
            class.define_method("initialize", rutie_obj_initialize);

            let mut instance = class.new_instance(&[Fixnum::new(5).into()]);
            instance.define_singleton_method("only_me", rutie_obj_singleton);

            let public = instance.protect_public_send("public_one", &[]).unwrap();
            assert_eq!(
                public.try_convert_to::<Symbol>().unwrap().to_str(),
                "public"
            );

            // Private methods are only reachable with `send`.
            let error = instance
                .protect_public_send("private_one", &[])
                .unwrap_err();
            assert!(Class::no_method_error().case_equals(&error));
            let private = instance.protect_send("private_one", &[]).unwrap();
            assert_eq!(
                private.try_convert_to::<Symbol>().unwrap().to_str(),
                "private"
            );

            let class_one = class.protect_public_send("class_one", &[]).unwrap();
            assert_eq!(
                class_one.try_convert_to::<Symbol>().unwrap().to_str(),
                "singleton"
            );
            assert!(instance.protect_public_send("only_me", &[]).is_ok());
            let other = class.new_instance(&[Fixnum::new(1).into()]);
            assert!(other.protect_public_send("only_me", &[]).is_err());

            assert_eq!(
                instance
                    .instance_variable_get("@value")
                    .try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(5))
            );

            // `call_init` runs `initialize` on an allocated object.
            let allocated = class.allocate();
            unsafe { allocated.call_init(&[Fixnum::new(9).into()]) };
            assert_eq!(
                allocated
                    .instance_variable_get("@value")
                    .try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(9))
            );
        });
    }

    #[test]
    fn test_types_equality_and_data() {
        crate::on_ruby_thread(|| {
            use crate::types::ValueType;

            assert_eq!(RString::new_utf8("a").ty(), ValueType::RString);
            assert_eq!(Fixnum::new(1).ty(), ValueType::Fixnum);
            assert_eq!(Array::new().ty(), ValueType::Array);
            assert_eq!(Hash::new().ty(), ValueType::Hash);
            assert_eq!(NilClass::new().ty(), ValueType::Nil);
            assert_eq!(Float::new(1.5).ty(), ValueType::Float);
            assert_eq!(Class::object().ty(), ValueType::Class);
            assert_eq!(Module::kernel().ty(), ValueType::Module);
            assert_eq!(Symbol::new("s").ty(), ValueType::Symbol);

            assert!(Fixnum::new(1).is_eql(&Fixnum::new(1)));
            assert!(!Fixnum::new(1).is_eql(&Float::new(1.0)));
            assert!(RString::new_utf8("a").is_eql(&RString::new_utf8("a")));

            let any = RString::new_utf8("cast").to_any_object();
            let string: RString = unsafe { any.to::<RString>() };
            assert_eq!(string.to_str(), "cast");

            let mut boxed: AnyObject = Class::new("RutieObjectBoxClass", None)
                .wrap_data(RutieObjectBox { count: 1 }, &*RUTIE_OBJECT_BOX);
            boxed.get_data_mut(&*RUTIE_OBJECT_BOX).count += 41;
            crate::GC::start();
            assert_eq!(boxed.get_data(&*RUTIE_OBJECT_BOX).count, 42);
        });
    }
}
