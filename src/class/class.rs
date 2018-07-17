use std::convert::From;

use binding::{class, module};
use binding::global::rb_cObject;
use typed_data::DataTypeWrapper;
use types::{Value, ValueType};
use util;

use {AnyObject, Array, Object, Module, VerifiedObject};

/// `Class`
///
/// Also see `def`, `def_self`, `define` and some more functions from `Object` trait.
///
/// ```rust
/// #[macro_use] extern crate rutie;
///
/// use std::error::Error;
///
/// use rutie::{Class, Fixnum, Object, Exception, VM};
///
/// methods!(
///    Fixnum,
///    itself,
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
/// );
///
/// fn main() {
///     # VM::init();
///     Class::from_existing("Fixnum").define(|itself| {
///         itself.def("pow", pow);
///     });
/// }
/// ```
///
/// Ruby:
///
/// ```ruby
/// class Fixnum
///   def pow(exp)
///     raise TypeError unless exp.is_a?(Fixnum)
///
///     self ** exp
///   end
/// end
/// ```
#[derive(Debug, PartialEq)]
pub struct Class {
    value: Value,
}

impl Class {
    /// Creates a new `Class`.
    ///
    /// `superclass` can receive the following values:
    ///
    ///  - `None` to inherit from `Object` class
    ///     (standard Ruby behavior when superclass is not given explicitly);
    ///  - `Some(&Class)` to inherit from the given class
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, VM};
    /// # VM::init();
    ///
    /// let basic_record_class = Class::new("BasicRecord", None);
    ///
    /// assert_eq!(basic_record_class, Class::from_existing("BasicRecord"));
    /// assert_eq!(basic_record_class.superclass(), Some(Class::from_existing("Object")));
    ///
    /// let record_class = Class::new("Record", Some(&basic_record_class));
    ///
    /// assert_eq!(record_class, Class::from_existing("Record"));
    /// assert_eq!(record_class.superclass(), Some(Class::from_existing("BasicRecord")));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class BasicRecord
    /// end
    ///
    /// class Record < BasicRecord
    /// end
    ///
    /// BasicRecord.superclass == Object
    ///
    /// Record.superclass == BasicRecord
    /// ```
    pub fn new(name: &str, superclass: Option<&Self>) -> Self {
        let superclass = Self::superclass_to_value(superclass);

        Self::from(class::define_class(name, superclass))
    }

    /// Retrieves an existing `Class` object.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, VM};
    /// # VM::init();
    ///
    /// let class = Class::new("Record", None);
    ///
    /// assert_eq!(class, Class::from_existing("Record"));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Record
    /// end
    ///
    /// # get class
    ///
    /// Record
    ///
    /// # or
    ///
    /// Object.const_get('Record')
    /// ```
    pub fn from_existing(name: &str) -> Self {
        let object_class = unsafe { rb_cObject };

        Self::from(class::const_get(object_class, name))
    }

    /// Creates a new instance of `Class`
    ///
    /// Arguments must be passed as a vector of `AnyObject` (see example).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rutie::{Class, Fixnum, Object};
    ///
    /// // Without arguments
    /// Class::from_existing("Hello").new_instance(None);
    ///
    /// // With arguments passing arguments to constructor
    /// let arguments = [
    ///     Fixnum::new(1).to_any_object(),
    ///     Fixnum::new(2).to_any_object()
    /// ];
    ///
    /// Class::from_existing("Worker").new_instance(Some(&arguments));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// Hello.new
    ///
    /// Worker.new(1, 2)
    /// ```
    pub fn new_instance(&self, arguments: Option<&[AnyObject]>) -> AnyObject {
        let arguments = util::arguments_to_values(arguments).unwrap_or_default();
        let instance = class::new_instance(self.value(), &arguments);

        AnyObject::from(instance)
    }

    /// Creates a new instance of `Class`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rutie::{Class, Object};
    ///
    /// Class::from_existing("String").allocate();
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// String.allocate
    /// ```
    pub fn allocate(&self) -> Class {
        Class::from(self.send("allocate", None).value())
    }

    /// Returns a superclass of the current class
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, VM};
    /// # VM::init();
    ///
    /// assert_eq!(
    ///     Class::from_existing("Array").superclass(),
    ///     Some(Class::from_existing("Object"))
    /// );
    ///
    /// assert_eq!(Class::from_existing("BasicObject").superclass(), None);
    /// ```
    pub fn superclass(&self) -> Option<Class> {
        let superclass_value = class::superclass(self.value());

        if superclass_value.is_nil() {
            None
        } else {
            Some(Self::from(superclass_value))
        }
    }

    /// Returns a Vector of ancestors of current class
    ///
    /// # Examples
    ///
    /// ### Getting all the ancestors
    ///
    /// ```
    /// use rutie::{Class, VM};
    /// # VM::init();
    ///
    /// let true_class_ancestors = Class::from_existing("TrueClass").ancestors();
    ///
    /// let expected_ancestors = vec![
    ///     Class::from_existing("TrueClass"),
    ///     Class::from_existing("Object"),
    ///     Class::from_existing("Kernel"),
    ///     Class::from_existing("BasicObject")
    /// ];
    ///
    /// assert_eq!(true_class_ancestors, expected_ancestors);
    /// ```
    ///
    /// ### Searching for an ancestor
    ///
    /// ```
    /// use rutie::{Class, VM};
    /// # VM::init();
    ///
    /// let basic_record_class = Class::new("BasicRecord", None);
    /// let record_class = Class::new("Record", Some(&basic_record_class));
    ///
    /// let ancestors = record_class.ancestors();
    ///
    /// assert!(ancestors.iter().any(|class| *class == basic_record_class));
    /// ```
    // Using unsafe conversions is ok, because MRI guarantees to return an `Array` of `Class`es
    pub fn ancestors(&self) -> Vec<Class> {
        let ancestors = Array::from(class::ancestors(self.value()));

        ancestors
            .into_iter()
            .map(|class| unsafe { class.to::<Self>() })
            .collect()
    }

    /// Retrieves a `Class` nested to current `Class`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, VM};
    /// # VM::init();
    ///
    /// Class::new("Outer", None).define(|itself| {
    ///     itself.define_nested_class("Inner", None);
    /// });
    ///
    /// Class::from_existing("Outer").get_nested_class("Inner");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Outer
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
    pub fn get_nested_class(&self, name: &str) -> Self {
        Self::from(class::const_get(self.value(), name))
    }

    /// Retrieves a `Module` nested to current `Class`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// Class::new("Outer", None).define(|itself| {
    ///     itself.define_nested_module("Inner");
    /// });
    ///
    /// Class::from_existing("Outer").get_nested_module("Inner");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Outer
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
    pub fn get_nested_module(&self, name: &str) -> Module {
        Module::from(class::const_get(self.value(), name))
    }

    /// Creates a new `Class` nested into current class.
    ///
    /// `superclass` can receive the following values:
    ///
    ///  - `None` to inherit from `Object` class
    ///     (standard Ruby behavior when superclass is not given explicitly);
    ///  - `Some(&class)` to inherit from the given class
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, VM};
    /// # VM::init();
    ///
    /// Class::new("Outer", None).define(|itself| {
    ///     itself.define_nested_class("Inner", None);
    /// });
    ///
    /// Class::from_existing("Outer").get_nested_class("Inner");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Outer
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
    pub fn define_nested_class(&mut self, name: &str, superclass: Option<&Class>) -> Self {
        let superclass = Self::superclass_to_value(superclass);

        Self::from(class::define_nested_class(self.value(), name, superclass))
    }

    /// Creates a new `Module` nested into current `Class`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// Class::new("Outer", None).define(|itself| {
    ///     itself.define_nested_module("Inner");
    /// });
    ///
    /// Module::from_existing("Outer").get_nested_module("Inner");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Outer
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
    pub fn define_nested_module(&mut self, name: &str) -> Module {
        Module::from(module::define_nested_module(self.value(), name))
    }

    /// Retrieves a constant from class.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, RString, VM};
    /// # VM::init();
    ///
    /// Class::new("Greeter", None).define(|itself| {
    ///     itself.const_set("GREETING", &RString::new("Hello, World!"));
    /// });
    ///
    /// let greeting = Class::from_existing("Greeter")
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
    /// class Greeter
    ///   GREETING = 'Hello, World!'
    /// end
    ///
    /// # or
    ///
    /// Greeter = Class.new
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

    /// Defines a constant for class.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, RString, VM};
    /// # VM::init();
    ///
    /// Class::new("Greeter", None).define(|itself| {
    ///     itself.const_set("GREETING", &RString::new("Hello, World!"));
    /// });
    ///
    /// let greeting = Class::from_existing("Greeter")
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
    /// class Greeter
    ///   GREETING = 'Hello, World!'
    /// end
    ///
    /// # or
    ///
    /// Greeter = Class.new
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

    /// Includes module into current class
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, VM};
    /// # VM::init();
    ///
    /// let a_module = Module::new("A");
    /// Class::new("B", None).include("A");
    ///
    /// let b_class_ancestors = Class::from_existing("B").ancestors();
    /// let expected_ancestors = vec![Module::from_existing("A")];
    ///
    /// assert!(expected_ancestors.iter().any(|anc| *anc == a_module));
    /// ```
    pub fn include(&self, md: &str) {
        module::include_module(self.value(), md);
    }

    /// Prepends module into current class
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, VM};
    /// # VM::init();
    ///
    /// let a_module = Module::new("A");
    /// Class::new("B", None).prepend("A");
    ///
    /// let b_class_ancestors = Class::from_existing("B").ancestors();
    /// let expected_ancestors = vec![Module::from_existing("A")];
    ///
    /// assert!(expected_ancestors.iter().any(|anc| *anc == a_module));
    /// ```
    pub fn prepend(&self, md: &str) {
        module::prepend_module(self.value(), md);
    }

    /// Defines an `attr_reader` for class
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, VM};
    /// # VM::init();
    ///
    /// Class::new("Test", None).define(|itself| {
    ///     itself.attr_reader("reader");
    /// });
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Test
    ///   attr_reader :reader
    /// end
    /// ```
    pub fn attr_reader(&mut self, name: &str) {
        class::define_attribute(self.value(), name, true, false);
    }

    /// Defines an `attr_writer` for class
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, VM};
    /// # VM::init();
    ///
    /// Class::new("Test", None).define(|itself| {
    ///     itself.attr_writer("writer");
    /// });
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Test
    ///   attr_writer :writer
    /// end
    /// ```
    pub fn attr_writer(&mut self, name: &str) {
        class::define_attribute(self.value(), name, false, true);
    }

    /// Defines an `attr_accessor` for class
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, VM};
    /// # VM::init();
    ///
    /// Class::new("Test", None).define(|itself| {
    ///     itself.attr_accessor("accessor");
    /// });
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Test
    ///   attr_accessor :accessor
    /// end
    /// ```
    pub fn attr_accessor(&mut self, name: &str) {
        class::define_attribute(self.value(), name, true, true);
    }

    /// Wraps Rust structure into a new Ruby object of the current class.
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
    ///     Class::new("RubyServer", Some(&data_class)).define(|itself| {
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

impl From<Value> for Class {
    fn from(value: Value) -> Self {
        Class { value: value }
    }
}

impl Object for Class {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Class {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        object.value().ty() == ValueType::Class
    }

    fn error_message() -> &'static str {
        "Error converting to Class"
    }
}
