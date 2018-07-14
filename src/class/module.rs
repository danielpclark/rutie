use std::convert::From;

use binding::{module, class};
use binding::global::rb_cObject;
use typed_data::DataTypeWrapper;
use types::{Value, ValueType, Callback};

use {AnyObject, Array, Object, Class, VerifiedObject};

/// `Module`
///
/// Also see `def`, `def_self`, `define` and some more functions from `Object` trait.
///
/// ```rust
/// #[macro_use] extern crate rutie;
///
/// use std::error::Error;
///
/// use rutie::{Module, Fixnum, Object, Exception, VM};
/// 
/// module!(Example);
///
/// methods!(
///    Example,
///    itself,
///
///     fn square(exp: Fixnum) -> Fixnum {
///         // `exp` is not a valid `Fixnum`, raise an exception
///         if let Err(ref error) = exp {
///             VM::raise(error.class(), &error.message());
///         }
///
///         // We can safely unwrap here, because an exception was raised if `exp` is `Err`
///         let exp = exp.unwrap().to_i64();
///
///         Fixnum::new(exp * exp)
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     Module::new("Example").define(|itself| {
///         itself.def("square", square);
///     });
/// }
/// ```
///
/// Ruby:
///
/// ```ruby
/// module Example
///   def square(exp)
///     raise TypeError unless exp.is_a?(Fixnum)
///
///     exp * exp
///   end
/// end
/// ```
#[derive(Debug, PartialEq)]
pub struct Module {
    value: Value,
}

impl Module {
    /// Creates a new `Module`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Module, VM};
    /// # VM::init();
    ///
    /// let basic_record_module = Module::new("BasicRecord");
    ///
    /// assert_eq!(basic_record_module, Module::from_existing("BasicRecord"));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// module BasicRecord
    /// end
    /// ```
    pub fn new(name: &str) -> Self {
        Self::from(module::define_module(name))
    }

    /// Retrieves an existing `Module` object.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Module, VM};
    /// # VM::init();
    ///
    /// let module = Module::new("Record");
    ///
    /// assert_eq!(module, Module::from_existing("Record"));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// module Record
    /// end
    ///
    /// # get module
    ///
    /// Record
    ///
    /// # or
    ///
    /// Object.const_get('Record')
    /// ```
    pub fn from_existing(name: &str) -> Self {
        let object_module = unsafe { rb_cObject };

        Self::from(class::const_get(object_module, name))
    }

    /// Returns a Vector of ancestors of current module
    ///
    /// # Examples
    ///
    /// ### Getting all the ancestors
    ///
    /// ```
    /// use rutie::{Module, VM};
    /// # VM::init();
    ///
    /// let process_module_ancestors = Module::from_existing("Process").ancestors();
    ///
    /// let expected_ancestors = vec![
    ///     Module::from_existing("Process")
    /// ];
    ///
    /// assert_eq!(process_module_ancestors, expected_ancestors);
    /// ```
    ///
    /// ### Searching for an ancestor
    ///
    /// ```
    /// use rutie::{Module, VM};
    /// # VM::init();
    ///
    /// let record_module = Module::new("Record");
    ///
    /// let ancestors = record_module.ancestors();
    ///
    /// assert!(ancestors.iter().any(|module| *module == record_module));
    /// ```
    // Using unsafe conversions is ok, because MRI guarantees to return an `Array` of `Module`es
    pub fn ancestors(&self) -> Vec<Module> {
        let ancestors = Array::from(class::ancestors(self.value()));

        ancestors
            .into_iter()
            .map(|module| unsafe { module.to::<Self>() })
            .collect()
    }

    /// Retrieves a `Module` nested to current `Module`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Module, Object, VM};
    /// # VM::init();
    ///
    /// Module::new("Outer").define(|itself| {
    ///     itself.define_nested_module("Inner");
    /// });
    ///
    /// Module::from_existing("Outer").get_nested_module("Inner");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// module Outer
    ///   module Inner
    ///   end
    /// end
    ///
    /// Outer::Inner
    ///
    /// # or
    ///
    /// Outer.const_get('Inner')
    /// ```
    pub fn get_nested_module(&self, name: &str) -> Self {
        Self::from(class::const_get(self.value(), name))
    }

    /// Retrieves a `Class` nested to current `Module`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// Module::new("Outer").define(|itself| {
    ///     itself.define_nested_class("Inner", None);
    /// });
    ///
    /// Module::from_existing("Outer").get_nested_class("Inner");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// module Outer
    ///   class Inner
    ///   end
    /// end
    ///
    /// Outer::Inner
    ///
    /// # or
    ///
    /// Outer.const_get('Inner')
    /// ```
    pub fn get_nested_class(&self, name: &str) -> Class {
        Class::from(class::const_get(self.value(), name))
    }

    /// Creates a new `Module` nested into current `Module`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Module, Object, VM};
    /// # VM::init();
    ///
    /// Module::new("Outer").define(|itself| {
    ///     itself.define_nested_module("Inner");
    /// });
    ///
    /// Module::from_existing("Outer").get_nested_module("Inner");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// module Outer
    ///   module Inner
    ///   end
    /// end
    ///
    /// Outer::Inner
    ///
    /// # or
    ///
    /// Outer.const_get('Inner')
    /// ```
    pub fn define_nested_module(&mut self, name: &str) -> Self {
        Self::from(module::define_nested_module(self.value(), name))
    }

    /// Creates a new `Class` nested into current module.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// Module::new("Outer").define(|itself| {
    ///     itself.define_nested_class("Inner", None);
    /// });
    ///
    /// Module::from_existing("Outer").get_nested_class("Inner");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// module Outer
    ///   class Inner
    ///   end
    /// end
    ///
    /// Outer::Inner
    ///
    /// # or
    ///
    /// Outer.const_get('Inner')
    /// ```
    pub fn define_nested_class(&mut self, name: &str, superclass: Option<&Class>) -> Class {
        let superclass = Self::superclass_to_value(superclass);

        Class::from(class::define_nested_class(self.value(), name, superclass))
    }

    /// Defines an instance method for the given module.
    ///
    /// Use `methods!` macro to define a `callback`.
    ///
    /// You can also use `def()` alias for this function combined with `Module::define()` for a
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
    /// use rutie::{Boolean, Module, Class, Object, RString, VM};
    ///
    /// methods!(
    ///    RString,
    ///    itself,
    ///
    ///    fn is_blank() -> Boolean {
    ///        Boolean::new(itself.to_str().chars().all(|c| c.is_whitespace()))
    ///    }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Module::new("Blank").define(|itself| {
    ///         itself.mod_func("blank?", is_blank);
    ///     });
    ///
    ///     Class::from_existing("String").include("Blank");
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// module Blank
    ///   def blank?
    ///     # simplified
    ///     self.chars.all? { |c| c == ' ' }
    ///   end
    ///   module_function :blank?
    /// end
    ///
    /// String.include Blank
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
    /// use rutie::{Module, Fixnum, Object, Exception, VM};
    ///
    /// methods!(
    ///     Fixnum,
    ///     itself,
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
    ///         Fixnum::new(itself.to_i64().pow(exp))
    ///     }
    ///
    ///     fn pow_with_default_argument(exp: Fixnum) -> Fixnum {
    ///         let default_exp = 0;
    ///         let exp = exp.map(|exp| exp.to_i64()).unwrap_or(default_exp);
    ///
    ///         let result = itself.to_i64().pow(exp as u32);
    ///
    ///         Fixnum::new(result)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Module::from_existing("Fixnum").define(|itself| {
    ///         itself.mod_func("pow", pow);
    ///         itself.mod_func("pow_with_default_argument", pow_with_default_argument);
    ///     });
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// module Fixnum
    ///   def pow(exp)
    ///     raise ArgumentError unless exp.is_a?(Fixnum)
    ///
    ///     self ** exp
    ///   end
    ///   module_function :pow
    ///
    ///   def pow_with_default_argument(exp)
    ///     default_exp = 0
    ///     exp = default_exp unless exp.is_a?(Fixnum)
    ///
    ///     self ** exp
    ///   end
    ///   module_function :pow_with_default_argument
    /// end
    /// ```
    pub fn define_module_function<I: Object, O: Object>(&mut self, name: &str, callback: Callback<I, O>) {
        module::define_module_function(self.value(), name, callback);
    }

    /// An alias for `define_module_function` (similar to Ruby `module_function :some_method`).
    pub fn mod_func<I: Object, O: Object>(&mut self, name: &str, callback: Callback<I, O>) {
        self.define_module_function(name, callback);
    }

    /// Retrieves a constant from module.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Module, Object, RString, VM};
    /// # VM::init();
    ///
    /// Module::new("Greeter").define(|itself| {
    ///     itself.const_set("GREETING", &RString::new("Hello, World!"));
    /// });
    ///
    /// let greeting = Module::from_existing("Greeter")
    ///     .const_get("GREETING")
    ///     .try_convert_to::<RString>()
    ///     .unwrap();
    ///
    /// assert_eq!(greeting.to_str(), "Hello, World!");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// module Greeter
    ///   GREETING = 'Hello, World!'
    /// end
    ///
    /// # or
    ///
    /// Greeter = Module.new
    /// Greeter.const_set('GREETING', 'Hello, World!')
    ///
    /// # ...
    ///
    /// Greeter::GREETING == 'Hello, World!'
    ///
    /// # or
    ///
    /// Greeter.const_get('GREETING') == 'Hello, World'
    /// ```
    pub fn const_get(&self, name: &str) -> AnyObject {
        let value = class::const_get(self.value(), name);

        AnyObject::from(value)
    }

    /// Defines a constant for module.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Module, Object, RString, VM};
    /// # VM::init();
    ///
    /// Module::new("Greeter").define(|itself| {
    ///     itself.const_set("GREETING", &RString::new("Hello, World!"));
    /// });
    ///
    /// let greeting = Module::from_existing("Greeter")
    ///     .const_get("GREETING")
    ///     .try_convert_to::<RString>()
    ///     .unwrap();
    ///
    /// assert_eq!(greeting.to_str(), "Hello, World!");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// module Greeter
    ///   GREETING = 'Hello, World!'
    /// end
    ///
    /// # or
    ///
    /// Greeter = Module.new
    /// Greeter.const_set('GREETING', 'Hello, World!')
    ///
    /// # ...
    ///
    /// Greeter::GREETING == 'Hello, World!'
    ///
    /// # or
    ///
    /// Greeter.const_get('GREETING') == 'Hello, World'
    /// ```
    pub fn const_set<T: Object>(&mut self, name: &str, value: &T) {
        class::const_set(self.value(), name, value.value());
    }

    /// Includes module into current module
    /// 
    /// # Examples
    /// 
    /// ```
    /// use rutie::{Module, VM};
    /// # VM::init();
    /// 
    /// Module::new("A");
    /// Module::new("B").include("A");
    ///
    /// let b_module_ancestors = Module::from_existing("B").ancestors();
    ///
    /// let expected_ancestors = vec![
    ///     Module::from_existing("B"),
    ///     Module::from_existing("A")
    /// ];
    ///
    /// assert_eq!(b_module_ancestors, expected_ancestors);
    /// ```
    pub fn include(&self, md: &str) {
        module::include_module(self.value(), md);
    }

    /// Prepends module into current module
    /// 
    /// # Examples
    /// 
    /// ```
    /// use rutie::{Module, VM};
    /// # VM::init();
    /// 
    /// Module::new("A");
    /// Module::new("B").prepend("A");
    ///
    /// let b_module_ancestors = Module::from_existing("B").ancestors();
    ///
    /// let expected_ancestors = vec![
    ///     Module::from_existing("A"),
    ///     Module::from_existing("B")
    /// ];
    ///
    /// assert_eq!(b_module_ancestors, expected_ancestors);
    /// ```
    pub fn prepend(&self, md: &str) {
        module::prepend_module(self.value(), md);
    }

    /// Defines an `attr_reader` for module
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Module, Object, VM};
    /// # VM::init();
    ///
    /// Module::new("Test").define(|itself| {
    ///     itself.attr_reader("reader");
    /// });
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// module Test
    ///   attr_reader :reader
    /// end
    /// ```
    pub fn attr_reader(&mut self, name: &str) {
        class::define_attribute(self.value(), name, true, false);
    }

    /// Defines an `attr_writer` for module
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Module, Object, VM};
    /// # VM::init();
    ///
    /// Module::new("Test").define(|itself| {
    ///     itself.attr_writer("writer");
    /// });
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// module Test
    ///   attr_writer :writer
    /// end
    /// ```
    pub fn attr_writer(&mut self, name: &str) {
        class::define_attribute(self.value(), name, false, true);
    }

    /// Defines an `attr_accessor` for module
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Module, Object, VM};
    /// # VM::init();
    ///
    /// Module::new("Test").define(|itself| {
    ///     itself.attr_accessor("accessor");
    /// });
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// module Test
    ///   attr_accessor :accessor
    /// end
    /// ```
    pub fn attr_accessor(&mut self, name: &str) {
        class::define_attribute(self.value(), name, true, true);
    }

    /// Wraps Rust structure into a new Ruby object of the current module.
    ///
    /// See the documentation for `wrappable_struct!` macro for more information.
    ///
    /// # Examples
    ///
    /// Wrap `Server` structs to `RubyServer` objects.  Note: Example shows use
    /// with class but the method still applies to module.
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
    ///     itself,
    ///
    ///     fn ruby_server_new(host: RString, port: Fixnum) -> AnyObject {
    ///         let server = Server::new(host.unwrap().to_string(),
    ///                                  port.unwrap().to_i64() as u16);
    ///
    ///         Class::from_existing("RubyServer").wrap_data(server, &*SERVER_WRAPPER)
    ///     }
    ///
    ///     fn ruby_server_host() -> RString {
    ///         let host = itself.get_data(&*SERVER_WRAPPER).host();
    ///
    ///         RString::new(host)
    ///     }
    ///
    ///     fn ruby_server_port() -> Fixnum {
    ///         let port = itself.get_data(&*SERVER_WRAPPER).port();
    ///
    ///         Fixnum::new(port as i64)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     let data_class = Class::from_existing("Object");
    ///
    ///     Class::new("RubyServer", None).define(|itself| {
    ///         itself.def_self("new", ruby_server_new);
    ///
    ///         itself.def("host", ruby_server_host);
    ///         itself.def("port", ruby_server_port);
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
    pub fn wrap_data<T, O: Object>(&self, data: T, wrapper: &DataTypeWrapper<T>) -> O {
        let value = class::wrap_data(self.value(), data, wrapper);

        O::from(value)
    }

    fn superclass_to_value(superclass: Option<&Class>) -> Value {
        match superclass {
            Some(class) => class.value(),
            None => unsafe { rb_cObject },
        }
    }
}

impl From<Value> for Module {
    fn from(value: Value) -> Self {
        Module { value: value }
    }
}

impl Object for Module {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Module {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.value().ty() == ValueType::Module
    }

    fn error_message() -> &'static str {
        "Error converting to Module"
    }
}
