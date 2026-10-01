use std::convert::From;

use crate::{
    binding::{class, global::rb_cObject, module, vm},
    rubysys::class::AllocFunction,
    typed_data::DataTypeWrapper,
    types::{Value, ValueType},
    util, AnyException, AnyObject, Array, Exception, Module, NilClass, Object, RString, Symbol,
    VerifiedObject,
};

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
///    rtself,
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
/// );
///
/// fn main() {
///     # VM::init();
///     Class::from_existing("Integer").define(|klass| {
///         klass.def("pow", pow);
///     });
/// }
/// ```
///
/// Ruby:
///
/// ```ruby
/// class Integer
///   def pow(exp)
///     raise TypeError unless exp.is_a?(Integer)
///
///     self ** exp
///   end
/// end
/// ```
#[derive(Debug)]
#[repr(C)]
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
    /// ```
    /// use rutie::{Class, Fixnum, Object, VM};
    /// # VM::init();
    /// # VM::eval("class Hello; end").unwrap();
    /// # VM::eval("class Worker; attr_reader :a, :b; def initialize(a, b); @a, @b = a, b; end; end").unwrap();
    ///
    /// // Without arguments
    /// let hello = Class::from_existing("Hello").new_instance(&[]);
    /// assert!(hello.class() == Class::from_existing("Hello"));
    ///
    /// // With arguments passing arguments to constructor
    /// let arguments = [
    ///     Fixnum::new(1).to_any_object(),
    ///     Fixnum::new(2).to_any_object()
    /// ];
    ///
    /// let worker = Class::from_existing("Worker").new_instance(&arguments);
    /// assert_eq!(unsafe { worker.send("b", &[]) }.try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// Hello.new
    ///
    /// Worker.new(1, 2)
    /// ```
    pub fn new_instance(&self, arguments: &[AnyObject]) -> AnyObject {
        let arguments = util::arguments_to_values(arguments);
        let instance = class::new_instance(self.value(), &arguments);

        AnyObject::from(instance)
    }

    /// Creates a new instance of `Class`
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, RString, VM};
    /// # VM::init();
    ///
    /// // The object is created without calling `initialize`.
    /// let string = Class::from_existing("String").allocate();
    ///
    /// assert_eq!(string.try_convert_to::<RString>().unwrap().to_str(), "");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// String.allocate
    /// ```
    pub fn allocate(&self) -> Class {
        Class::from(unsafe { self.send("allocate", &[]) }.value())
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
    /// Class::new("Outer", None).define(|klass| {
    ///     klass.define_nested_class("Inner", None);
    /// });
    ///
    /// let inner = Class::from_existing("Outer").get_nested_class("Inner");
    ///
    /// assert_eq!(inner.name().unwrap().to_str(), "Outer::Inner");
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
    /// Class::new("Outer", None).define(|klass| {
    ///     klass.define_nested_module("Inner");
    /// });
    ///
    /// let inner = Class::from_existing("Outer").get_nested_module("Inner");
    ///
    /// assert_eq!(inner.name().unwrap().to_str(), "Outer::Inner");
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
    /// let mut outer = Class::new("Outer", None);
    /// let inner = outer.define_nested_class("Inner", None);
    ///
    /// assert_eq!(inner.name().unwrap().to_str(), "Outer::Inner");
    /// assert_eq!(inner.superclass(), Some(Class::object()));
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
    /// let mut outer = Class::new("Outer", None);
    /// let inner = outer.define_nested_module("Inner");
    ///
    /// assert_eq!(inner.name().unwrap().to_str(), "Outer::Inner");
    /// assert!(Module::from_existing("Outer").get_nested_module("Inner") == inner);
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
    /// Class::new("Greeter", None).define(|klass| {
    ///     klass.const_set("GREETING", &RString::new_utf8("Hello, World!"));
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
    /// Class::new("Greeter", None).define(|klass| {
    ///     klass.const_set("GREETING", &RString::new_utf8("Hello, World!"));
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
    /// use rutie::{Class, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// Class::new("Test", None).define(|klass| {
    ///     klass.attr_reader("reader");
    /// });
    ///
    /// let object = VM::eval("t = Test.new; t.instance_variable_set(:@reader, 1); t").unwrap();
    /// assert_eq!(unsafe { object.send("reader", &[]) }.try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// assert!(!object.respond_to("reader="));
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
    /// use rutie::{Class, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// Class::new("Test", None).define(|klass| {
    ///     klass.attr_writer("writer");
    /// });
    ///
    /// let object = VM::eval("t = Test.new; t.writer = 2; t").unwrap();
    /// assert_eq!(object.instance_variable_get("@writer").try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// assert!(!object.respond_to("writer"));
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
    /// use rutie::{Class, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// Class::new("Test", None).define(|klass| {
    ///     klass.attr_accessor("accessor");
    /// });
    ///
    /// let object = VM::eval("t = Test.new; t.accessor = 3; t").unwrap();
    /// assert_eq!(unsafe { object.send("accessor", &[]) }.try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
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

    /// Makes `new_name` a copy of the method `old_name` (`rb_define_alias`,
    /// Ruby's `alias_method`).
    ///
    /// Raises `NameError` if `old_name` is not defined.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, RString, VM};
    /// # VM::init();
    ///
    /// VM::eval("class Aliased; def hello; 'hello'; end; end").unwrap();
    ///
    /// Class::from_existing("Aliased").define_alias("greet", "hello");
    ///
    /// let greeting = VM::eval("Aliased.new.greet").unwrap();
    ///
    /// assert_eq!(greeting.try_convert_to::<RString>().unwrap().to_str(), "hello");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Aliased
    ///   alias_method :greet, :hello
    /// end
    /// ```
    pub fn define_alias(&mut self, new_name: &str, old_name: &str) {
        class::define_alias(self.value(), new_name, old_name);
    }

    /// Prevents instances from responding to the method `name`, including
    /// one inherited from an ancestor (`rb_undef_method`, Ruby's
    /// `undef_method`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("class Undefined; def to_s; 'custom'; end; end").unwrap();
    ///
    /// Class::from_existing("Undefined").undef_method("to_s");
    ///
    /// assert!(VM::eval("Undefined.new.to_s").is_err());
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Undefined
    ///   undef_method :to_s
    /// end
    /// ```
    pub fn undef_method(&mut self, name: &str) {
        class::undef_method(self.value(), name);
    }

    /// Sets the function Ruby calls to allocate instances of this class
    /// (`rb_define_alloc_func`); `new` calls it and then `initialize`.
    ///
    /// Use it to make every instance wrap Rust data (see
    /// `wrappable_struct!`) so that `Class#new`, `allocate`, `dup` and
    /// subclasses work as usual. The function must not panic.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    /// #[macro_use] extern crate lazy_static;
    ///
    /// use rutie::{AnyObject, Class, Fixnum, NilClass, Object, VM};
    ///
    /// pub struct Counter {
    ///     count: i64,
    /// }
    ///
    /// wrappable_struct!(Counter, CounterWrapper, COUNTER_WRAPPER);
    ///
    /// class!(RubyCounter);
    ///
    /// // `extern "C"` (`extern "C-unwind"` on Windows), see `rutie_callback!`.
    /// rutie_callback! {
    ///     fn counter_alloc(klass: Class) -> AnyObject {
    ///         klass.wrap_data(Counter { count: 0 }, &*COUNTER_WRAPPER)
    ///     }
    /// }
    ///
    /// methods!(
    ///     RubyCounter,
    ///     rtself,
    ///
    ///     fn counter_increment() -> Fixnum {
    ///         let counter = rtself.get_data_mut(&*COUNTER_WRAPPER);
    ///         counter.count += 1;
    ///
    ///         Fixnum::new(counter.count)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::new("RubyCounter", None).define(|klass| {
    ///         klass.define_alloc_func(counter_alloc);
    ///         klass.def("increment", counter_increment);
    ///     });
    ///
    ///     let count = VM::eval("c = RubyCounter.new; c.increment; c.increment").unwrap();
    ///
    ///     assert_eq!(count.try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// }
    /// ```
    pub fn define_alloc_func(&mut self, func: rutie_callback!(type fn(Class) -> AnyObject)) {
        // `Class` and `AnyObject` are `#[repr(C)]` wrappers around a `Value`.
        let func: AllocFunction = unsafe { ::std::mem::transmute(func) };

        class::define_alloc_func(self.value(), func);
    }

    /// Removes the allocator of this class (`rb_undef_alloc_func`), so
    /// `new` and `allocate` raise `TypeError`.
    ///
    /// Use it for classes whose instances can only be made from Rust, such
    /// as ones wrapping Rust data with a custom constructor.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, VM};
    /// # VM::init();
    ///
    /// Class::new("NoAllocate", None).undef_alloc_func();
    ///
    /// assert!(VM::eval("NoAllocate.new").is_err());
    /// assert!(VM::eval("NoAllocate.allocate").is_err());
    /// ```
    pub fn undef_alloc_func(&mut self) {
        class::undef_alloc_func(self.value());
    }

    /// Returns the class's name, or `None` for an anonymous class
    /// (Ruby's `name`, `rb_mod_name`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Class::from_existing("String").name().unwrap().to_str(), "String");
    ///
    /// let anonymous = VM::eval("Class.new").unwrap().try_convert_to::<Class>().unwrap();
    ///
    /// assert!(anonymous.name().is_none());
    /// ```
    pub fn name(&self) -> Option<RString> {
        let name = class::module_name(self.value());

        if name.is_nil() {
            None
        } else {
            Some(RString::from(name))
        }
    }

    /// Returns the class's full path, such as `"A::B"`, or a
    /// `"#<Class:0x...>"` description for an anonymous class
    /// (`rb_class_path`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// let nested = Class::from_existing("Errno").get_nested_class("ENOENT");
    ///
    /// assert_eq!(nested.path().to_str(), "Errno::ENOENT");
    ///
    /// let anonymous = VM::eval("Class.new").unwrap().try_convert_to::<Class>().unwrap();
    ///
    /// assert!(anonymous.path().to_str().starts_with("#<Class:"));
    /// ```
    pub fn path(&self) -> RString {
        RString::from(class::class_path(self.value()))
    }

    /// Returns the class named by a path such as `"A::B"`
    /// (`rb_path2class`), or an error if nothing is defined there
    /// (`ArgumentError`) or it is not a class (`TypeError`).
    ///
    /// Unlike `from_existing`, nested paths work and a missing constant is
    /// returned as an error instead of raising. No symbols are created for
    /// unknown names, so untrusted paths are fine.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Exception, Object, VM};
    /// # VM::init();
    ///
    /// let found = Class::from_path("Errno::ENOENT").unwrap();
    ///
    /// assert_eq!(found.path().to_str(), "Errno::ENOENT");
    ///
    /// assert!(Class::from_path("No::Such::Thing").is_err());
    ///
    /// let error = Class::from_path("Comparable").unwrap_err();
    ///
    /// assert!(error.message().contains("is not a class"));
    /// ```
    pub fn from_path(path: &str) -> Result<Self, AnyException> {
        let path = ::std::ffi::CString::new(path).map_err(|_| {
            AnyException::new("ArgumentError", Some("class path contains a NUL byte"))
        })?;

        let found =
            vm::protect_value(|| class::path_to_class(&path)).map_err(AnyException::from)?;

        if found.ty() == ValueType::Class {
            Ok(Self::from(found))
        } else {
            let message = format!("{} is not a class", path.to_string_lossy());

            Err(AnyException::new("TypeError", Some(&message)))
        }
    }

    /// Returns `true` if instances respond to the method `name` defined in
    /// this class or its ancestors (`rb_method_boundp`). Private methods
    /// count only when `include_private` is `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("class Sample; @@count = 1; LIMIT = 10; def visible; end; private def hidden; end; end").unwrap();
    ///
    /// let sample = Class::from_existing("Sample");
    ///
    /// assert!(sample.is_method_defined("visible", false));
    /// assert!(!sample.is_method_defined("hidden", false));
    /// assert!(sample.is_method_defined("hidden", true));
    /// assert!(!sample.is_method_defined("missing", true));
    /// ```
    pub fn is_method_defined(&self, name: &str, include_private: bool) -> bool {
        class::is_method_defined(self.value(), name, include_private)
    }

    /// Returns the arity of the instance method `name`, or `0` if it is not
    /// defined (`rb_mod_method_arity`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("class Arities; def two(a, b); end; def many(a, *rest); end; end").unwrap();
    ///
    /// let arities = Class::from_existing("Arities");
    ///
    /// assert_eq!(arities.instance_method_arity("two"), 2);
    /// assert_eq!(arities.instance_method_arity("many"), -2);
    /// ```
    pub fn instance_method_arity(&self, name: &str) -> i32 {
        crate::binding::rproc::module_method_arity(self.value(), name)
    }

    /// Compares this class with `other` in the class hierarchy, like
    /// Ruby's `self <= other` (`rb_class_inherited_p`): `Some(true)` if this
    /// is `other` or inherits from or includes it, `Some(false)` if `other`
    /// inherits from or includes this, and `None` if they are unrelated.
    ///
    /// Raises `TypeError` if `other` is not a class or module.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// let integer = Class::from_existing("Integer");
    /// let numeric = Class::from_existing("Numeric");
    /// let string = Class::from_existing("String");
    ///
    /// assert_eq!(integer.inherits(&numeric), Some(true));
    /// assert_eq!(integer.inherits(&integer), Some(true));
    /// assert_eq!(numeric.inherits(&integer), Some(false));
    /// assert_eq!(integer.inherits(&string), None);
    /// ```
    pub fn inherits<T: Object>(&self, other: &T) -> Option<bool> {
        let result = class::inherited_p(self.value(), other.value());

        if result.is_nil() {
            None
        } else {
            Some(result.is_true())
        }
    }

    /// Returns `true` if `module` is included in this class or one of its
    /// ancestors (Ruby's `include?`, `rb_mod_include_p`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// let integer = Class::from_existing("Integer");
    ///
    /// assert!(integer.includes_module(&Module::from_existing("Comparable")));
    /// assert!(!integer.includes_module(&Module::from_existing("Enumerable")));
    /// ```
    pub fn includes_module(&self, module: &Module) -> bool {
        class::include_p(self.value(), module.value())
    }

    /// Evaluates `code` in the context of this class (Ruby's
    /// `module_eval`/`class_eval`, `rb_mod_module_eval`), returning the result
    /// or the exception raised.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Exception, Object, RString, VM};
    /// # VM::init();
    ///
    /// let mut evaluated = Class::new("Evaluated", None);
    ///
    /// evaluated.module_eval("def greet; 'hi'; end").unwrap();
    ///
    /// let greeting = VM::eval("Evaluated.new.greet").unwrap();
    ///
    /// assert_eq!(greeting.try_convert_to::<RString>().unwrap().to_str(), "hi");
    ///
    /// assert!(evaluated.module_eval("raise 'bad'").is_err());
    /// ```
    pub fn module_eval(&mut self, code: &str) -> Result<AnyObject, AnyException> {
        let module = self.value();

        vm::protect_value(|| class::module_eval(module, code))
            .map(AnyObject::from)
            .map_err(AnyException::from)
    }

    /// Returns the names of the public and protected instance methods as an
    /// `Array` of `Symbol`s, including inherited ones when
    /// `include_inherited` is `true` (Ruby's `instance_methods`,
    /// `rb_class_instance_methods`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// VM::eval("class Sample; @@count = 1; LIMIT = 10; def visible; end; private def hidden; end; end").unwrap();
    ///
    /// let methods = Class::from_existing("Sample").instance_methods(false);
    ///
    /// assert_eq!(methods.length(), 1);
    /// assert_eq!(methods.at(0).try_convert_to::<Symbol>(), Ok(Symbol::new("visible")));
    /// ```
    pub fn instance_methods(&self, include_inherited: bool) -> Array {
        Array::from(class::instance_methods(self.value(), include_inherited))
    }

    /// Returns the value of the class variable `name` (such as `"@@count"`),
    /// or `None` if it is not defined (`rb_cvar_get`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("class Sample; @@count = 1; LIMIT = 10; def visible; end; private def hidden; end; end").unwrap();
    ///
    /// let sample = Class::from_existing("Sample");
    ///
    /// let count = sample.class_variable_get("@@count").unwrap();
    ///
    /// assert_eq!(count.try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// assert!(sample.class_variable_get("@@missing").is_none());
    /// ```
    pub fn class_variable_get(&self, name: &str) -> Option<AnyObject> {
        if class::is_class_variable_defined(self.value(), name) {
            Some(AnyObject::from(class::class_variable_get(
                self.value(),
                name,
            )))
        } else {
            None
        }
    }

    /// Sets the class variable `name` (such as `"@@count"`) to `value`
    /// (`rb_cvar_set`), or returns the error: a `NameError` if `name` is not
    /// a class variable name, or a `FrozenError` if the class is frozen.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let mut counter = Class::new("Counter", None);
    ///
    /// counter.class_variable_set("@@count", &Fixnum::new(5)).unwrap();
    ///
    /// let count = VM::eval("Counter.class_variable_get(:@@count)").unwrap();
    ///
    /// assert_eq!(count.try_convert_to::<Fixnum>(), Ok(Fixnum::new(5)));
    ///
    /// assert!(counter.class_variable_set("count", &Fixnum::new(5)).is_err());
    /// ```
    pub fn class_variable_set<T: Object>(
        &mut self,
        name: &str,
        value: &T,
    ) -> Result<(), AnyException> {
        if !Symbol::new(name).is_class_variable_name() {
            let message = format!("`{}' is not allowed as a class variable name", name);

            return Err(AnyException::new("NameError", Some(&message)));
        }

        let (module, value) = (self.value(), value.value());

        vm::protect_value(|| {
            class::class_variable_set(module, name, value);

            NilClass::new().value()
        })
        .map(|_| ())
        .map_err(AnyException::from)
    }

    /// Returns `true` if the class variable `name` (such as `"@@count"`) is
    /// defined here or in an ancestor (`rb_cvar_defined`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("class Sample; @@count = 1; LIMIT = 10; def visible; end; private def hidden; end; end").unwrap();
    ///
    /// let sample = Class::from_existing("Sample");
    ///
    /// assert!(sample.is_class_variable_defined("@@count"));
    /// assert!(!sample.is_class_variable_defined("@@missing"));
    /// ```
    pub fn is_class_variable_defined(&self, name: &str) -> bool {
        class::is_class_variable_defined(self.value(), name)
    }

    /// Returns `true` if the constant `name` is visible from this class:
    /// defined in it, its ancestors (`rb_const_defined`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("class Sample; @@count = 1; LIMIT = 10; def visible; end; private def hidden; end; end").unwrap();
    ///
    /// let sample = Class::from_existing("Sample");
    ///
    /// assert!(sample.is_const_defined("LIMIT"));
    /// assert!(sample.is_const_defined("String"));
    /// assert!(!sample.is_const_defined("MISSING"));
    /// ```
    pub fn is_const_defined(&self, name: &str) -> bool {
        class::is_const_defined(self.value(), name)
    }

    /// Returns `true` if the constant `name` is defined directly in this
    /// class, not inherited (`rb_const_defined_at`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("class Sample; @@count = 1; LIMIT = 10; def visible; end; private def hidden; end; end").unwrap();
    ///
    /// let sample = Class::from_existing("Sample");
    ///
    /// assert!(sample.is_const_defined_at("LIMIT"));
    /// assert!(!sample.is_const_defined_at("String"));
    /// ```
    pub fn is_const_defined_at(&self, name: &str) -> bool {
        class::is_const_defined_at(self.value(), name)
    }

    /// Removes the constant `name` defined directly in this class and
    /// returns its value, or `None` if there is no such constant
    /// (`rb_const_remove`).
    ///
    /// Raises `FrozenError` if the class is frozen and the constant exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("class Sample; @@count = 1; LIMIT = 10; def visible; end; private def hidden; end; end").unwrap();
    ///
    /// let mut sample = Class::from_existing("Sample");
    ///
    /// let limit = sample.const_remove("LIMIT").unwrap();
    ///
    /// assert_eq!(limit.try_convert_to::<Fixnum>(), Ok(Fixnum::new(10)));
    /// assert!(!sample.is_const_defined_at("LIMIT"));
    /// assert!(sample.const_remove("LIMIT").is_none());
    /// ```
    pub fn const_remove(&mut self, name: &str) -> Option<AnyObject> {
        if !class::is_const_defined_at(self.value(), name) {
            return None;
        }

        Some(AnyObject::from(class::const_remove(self.value(), name)))
    }

    /// Returns the class variable `name` (such as `"@@count"`) together
    /// with the class or module that defines it, searching this class and
    /// its ancestors (`rb_cvar_find`), or returns the error: a `NameError`
    /// if it is not defined, or a `RuntimeError` if both this class and an
    /// ancestor define it.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Fixnum, Module, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("module Counted; @@count = 3; end; class Widget; include Counted; end").unwrap();
    ///
    /// let widget = Class::from_existing("Widget");
    /// let (count, owner) = widget.class_variable_find("@@count").unwrap();
    ///
    /// assert_eq!(count.try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
    /// assert_eq!(owner, Module::from_existing("Counted"));
    /// assert!(widget.class_variable_find("@@missing").is_err());
    /// ```
    pub fn class_variable_find(&self, name: &str) -> Result<(AnyObject, Module), AnyException> {
        let klass = self.value();
        let mut owner = Value::from(0);

        vm::protect_value(|| {
            let (value, front) = class::class_variable_find(klass, name);
            owner = front;

            value
        })
        .map(|value| (AnyObject::from(value), Module::from(owner)))
        .map_err(AnyException::from)
    }

    /// Marks the constant `name` defined directly in this class as
    /// deprecated (Ruby's `deprecate_constant`, `rb_deprecate_constant`):
    /// using it warns when deprecation warnings are enabled. Returns the
    /// error: a `NameError` if the class does not define the constant, or a
    /// `FrozenError` if it is frozen.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Fixnum, Object, RString, VM};
    /// # VM::init();
    ///
    /// let mut config = Class::new("Config", None);
    ///
    /// config.const_set("OLD_LIMIT", &Fixnum::new(10));
    /// config.deprecate_constant("OLD_LIMIT").unwrap();
    ///
    /// VM::eval("Warning[:deprecated] = true
    ///           def Warning.warn(message, category: nil) = ($warned = message)").unwrap();
    /// VM::eval("Config::OLD_LIMIT").unwrap();
    ///
    /// let warned = VM::eval("$warned").unwrap().try_convert_to::<RString>().unwrap();
    ///
    /// assert!(warned.to_str().contains("Config::OLD_LIMIT is deprecated"));
    /// assert!(config.deprecate_constant("MISSING").is_err());
    /// ```
    pub fn deprecate_constant(&mut self, name: &str) -> Result<(), AnyException> {
        deprecate_constant(self.value(), name)
    }

    /// Returns the direct subclasses of this class (Ruby's
    /// `Class#subclasses`, `rb_class_subclasses`), most recently defined
    /// first. Singleton classes are not included.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, VM};
    /// # VM::init();
    ///
    /// let shape = Class::new("Shape", None);
    /// let circle = Class::new("Circle", Some(&shape));
    /// let square = Class::new("Square", Some(&shape));
    ///
    /// Class::new("Ball", Some(&circle));
    ///
    /// let subclasses = shape.subclasses();
    ///
    /// assert_eq!(subclasses.len(), 2);
    /// assert!(subclasses.contains(&circle));
    /// assert!(subclasses.contains(&square));
    /// assert!(square.subclasses().is_empty());
    /// ```
    pub fn subclasses(&self) -> Vec<Class> {
        Array::from(class::subclasses(self.value()))
            .into_iter()
            .map(|class| Class::from(class.value()))
            .collect()
    }

    /// Returns the object this singleton class is attached to (Ruby's
    /// `Class#attached_object`, `rb_class_attached_object`), or the
    /// `TypeError` if this is not a singleton class.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, VM};
    /// # VM::init();
    ///
    /// let object = VM::eval("Object.new").unwrap();
    /// let singleton = object.singleton_class();
    ///
    /// assert!(singleton.attached_object().unwrap().is_equal(&object));
    /// assert!(Class::string().attached_object().is_err());
    /// ```
    #[cfg(ruby_gte_3_2)]
    pub fn attached_object(&self) -> Result<AnyObject, AnyException> {
        let klass = self.value();

        vm::protect_value(|| class::attached_object(klass))
            .map(AnyObject::from)
            .map_err(AnyException::from)
    }

    /// Defines an anonymous `Data` class with the given members (Ruby's
    /// `Data.define`, `rb_data_define`), inheriting from `superclass` (a
    /// `Data` class; `None` for `Data`). Its instances are frozen value
    /// objects with a reader for each member. Returns the error: a
    /// `TypeError` if `superclass` is not a `Data` class, or an
    /// `ArgumentError` for a duplicate member or a name containing a NUL.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let point = Class::data_define(None, &["x", "y"]).unwrap();
    ///
    /// Class::object().const_set("Point", &point);
    ///
    /// let y = VM::eval("Point.new(x: 1, y: 2).y").unwrap();
    ///
    /// assert_eq!(y.try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// assert!(VM::eval("Point.new(1, 2).frozen?").unwrap().is_true());
    ///
    /// let point3d = Class::data_define(Some(&point), &["x", "y", "z"]).unwrap();
    ///
    /// assert_eq!(point3d.superclass(), Some(point));
    /// assert!(Class::data_define(None, &["x", "x"]).is_err());
    /// assert!(Class::data_define(Some(&Class::string()), &["x"]).is_err());
    /// ```
    #[cfg(ruby_gte_3_3)]
    pub fn data_define(
        superclass: Option<&Class>,
        members: &[&str],
    ) -> Result<Class, AnyException> {
        use std::ffi::{CStr, CString};

        use crate::binding::rstruct;

        let data = Class::from(class::const_get(unsafe { rb_cObject }, "Data"));
        let superclass = superclass.unwrap_or(&data);

        if superclass.inherits(&data) != Some(true) {
            let message = format!("{} is not a Data class", superclass.path().to_str());

            return Err(AnyException::new("TypeError", Some(&message)));
        }

        let names = members
            .iter()
            .map(|name| CString::new(*name))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| {
                AnyException::new("ArgumentError", Some("member name contains a NUL byte"))
            })?;
        let (data, superclass) = (data.value(), superclass.value());

        vm::protect_value(|| {
            if names.len() <= rstruct::DATA_DEFINE_MAX_MEMBERS {
                let names: Vec<&CStr> = names.iter().map(|name| name.as_c_str()).collect();

                rstruct::data_define(superclass, &names)
            } else {
                // Too many for one variadic call: `Data.define`, run on
                // `superclass` (which may have undefined `define`).
                let define = vm::call_method(
                    class::singleton_class(data),
                    "instance_method",
                    &[Symbol::new("define").value()],
                );
                let mut arguments = vec![superclass];

                arguments.extend(members.iter().map(|name| Symbol::new(name).value()));

                vm::call_method(define, "bind_call", &arguments)
            }
        })
        .map(Class::from)
        .map_err(AnyException::from)
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
    ///     let port = VM::eval("RubyServer.new('127.0.0.1', 3000).port").unwrap();
    ///     assert_eq!(port.try_convert_to::<Fixnum>(), Ok(Fixnum::new(3000)));
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
    pub fn wrap_data<T, O: Object>(&self, data: T, wrapper: &dyn DataTypeWrapper<T>) -> O {
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

// `rb_deprecate_constant` for `Class` and `Module`.
pub(crate) fn deprecate_constant(module: Value, name: &str) -> Result<(), AnyException> {
    let name = ::std::ffi::CString::new(name).map_err(|_| {
        AnyException::new("ArgumentError", Some("constant name contains a NUL byte"))
    })?;

    vm::protect_value(|| {
        class::deprecate_constant(module, &name);

        NilClass::new().value()
    })
    .map(|_| ())
    .map_err(AnyException::from)
}

impl From<Value> for Class {
    fn from(value: Value) -> Self {
        Class { value }
    }
}

impl Into<Value> for Class {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Class {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
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

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{AnyObject, Class, Exception, Fixnum, Module, Object, RString, Symbol, VM};

    #[test]
    fn test_class_introspection() {
        crate::on_ruby_thread(|| {
            VM::eval(
                "class RutieIntrospect
                   @@shared = :cvar
                   TOP = 1
                   def one; end
                   protected def two; end
                   private def three; end
                 end
                 class RutieIntrospectChild < RutieIntrospect; end",
            )
            .unwrap();

            let parent = Class::from_existing("RutieIntrospect");
            let child = Class::from_path("RutieIntrospectChild").unwrap();

            assert_eq!(child.name().unwrap().to_str(), "RutieIntrospectChild");
            assert_eq!(child.path().to_str(), "RutieIntrospectChild");
            assert_eq!(child.inherits(&parent), Some(true));
            assert_eq!(parent.inherits(&child), Some(false));
            assert_eq!(parent.inherits(&Class::from_existing("String")), None);
            assert!(child.includes_module(&Module::from_existing("Kernel")));

            assert!(child.is_method_defined("one", false));
            assert!(child.is_method_defined("two", false));
            assert!(!child.is_method_defined("three", false));
            assert!(child.is_method_defined("three", true));

            assert_eq!(parent.instance_methods(false).length(), 2);
            assert_eq!(child.instance_methods(false).length(), 0);
            assert!(child.instance_methods(true).length() > 2);

            // Class variables and constants are inherited.
            assert!(child.is_class_variable_defined("@@shared"));
            assert_eq!(
                child
                    .class_variable_get("@@shared")
                    .unwrap()
                    .try_convert_to::<Symbol>(),
                Ok(Symbol::new("cvar"))
            );
            assert!(child.is_const_defined("TOP"));
            assert!(!child.is_const_defined_at("TOP"));
        });
    }

    #[test]
    fn test_class_mutation() {
        crate::on_ruby_thread(|| {
            let mut klass = Class::new("RutieMutated", None);

            klass
                .module_eval("def value; @@value; end; CONST = :c")
                .unwrap();
            klass
                .class_variable_set("@@value", &Fixnum::new(3))
                .unwrap();

            let value = VM::eval("RutieMutated.new.value").unwrap();
            assert_eq!(value.try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));

            assert!(klass.class_variable_set("@value", &Fixnum::new(1)).is_err());

            assert_eq!(
                klass
                    .const_remove("CONST")
                    .unwrap()
                    .try_convert_to::<Symbol>(),
                Ok(Symbol::new("c"))
            );
            assert!(klass.const_remove("CONST").is_none());

            let error = klass
                .module_eval("raise ArgumentError, 'eval'")
                .unwrap_err();
            assert_eq!(error.message(), "eval");

            klass.freeze();
            let error = klass
                .class_variable_set("@@value", &Fixnum::new(4))
                .unwrap_err();
            assert!(Class::from_existing("FrozenError").case_equals(&error));
        });
    }

    #[test]
    fn test_from_path() {
        crate::on_ruby_thread(|| {
            assert_eq!(
                Class::from_path("Errno::EACCES").unwrap().path().to_str(),
                "Errno::EACCES"
            );
            assert!(Class::from_path("Comparable").is_err());
            assert!(Class::from_path("").is_err());
            assert!(Class::from_path("Bad\0Path").is_err());
            assert!(Class::from_path("#<Class:0x0>").is_err());
            assert!(Class::from_path("Errno::").is_err());

            assert_eq!(
                Module::from_path("Comparable")
                    .unwrap()
                    .name()
                    .unwrap()
                    .to_str(),
                "Comparable"
            );
            assert!(Module::from_path("String").is_err());

            // Looking up unknown names must not create symbols.
            assert!(Class::from_path("RutieNeverDefinedConstant").is_err());
            assert!(Symbol::find("RutieNeverDefinedConstant").is_none());
        });
    }

    #[test]
    fn test_anonymous_class_names() {
        crate::on_ruby_thread(|| {
            let anonymous = VM::eval("Class.new")
                .unwrap()
                .try_convert_to::<Class>()
                .unwrap();

            assert!(anonymous.name().is_none());
            assert!(anonymous.path().to_str().starts_with("#<Class:"));

            let named: AnyObject = VM::eval("RutieNamedLater = Class.new").unwrap();
            let named = named.try_convert_to::<Class>().unwrap();
            assert_eq!(named.name().unwrap(), RString::new_utf8("RutieNamedLater"));
        });
    }

    pub struct RutieAllocCounter {
        count: i64,
    }

    crate::wrappable_struct!(
        RutieAllocCounter,
        RutieAllocCounterWrapper,
        RUTIE_ALLOC_COUNTER
    );

    rutie_callback! {
        fn counter_alloc(klass: Class) -> AnyObject {
            klass.wrap_data(RutieAllocCounter { count: 41 }, &*RUTIE_ALLOC_COUNTER)
        }
    }

    #[test]
    fn test_class_structure() {
        crate::on_ruby_thread(|| {
            let mut outer = Class::new("RutieStructOuter", None);
            let inner = outer.define_nested_class("Inner", None);
            let nested_module = outer.define_nested_module("Helpers");

            assert_eq!(outer.get_nested_class("Inner"), inner);
            assert_eq!(outer.get_nested_module("Helpers"), nested_module);
            assert_eq!(
                Class::from_existing("RutieStructOuter")
                    .get_nested_class("Inner")
                    .name()
                    .unwrap()
                    .to_str(),
                "RutieStructOuter::Inner"
            );

            let child = Class::new("RutieStructChild", Some(&outer));
            assert_eq!(
                child.superclass(),
                Some(Class::from_existing("RutieStructOuter"))
            );
            assert!(Class::basic_object().superclass().is_none());

            let ancestors = child.ancestors();
            assert_eq!(ancestors[0], child);
            assert_eq!(ancestors[1], outer);
            assert!(ancestors.contains(&Class::object()));

            outer.const_set("ANSWER", &Fixnum::new(42));
            assert_eq!(
                outer.const_get("ANSWER").try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(42))
            );
            assert_eq!(
                child.const_get("ANSWER").try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(42))
            );

            VM::eval("module RutieStructMixin; def mixed; :mixed; end; end").unwrap();
            VM::eval("module RutieStructPrepended; def who; :prepended; end; end").unwrap();
            VM::eval("class RutieStructChild; def who; :child; end; end").unwrap();

            child.include("RutieStructMixin");
            child.prepend("RutieStructPrepended");

            let instance = child.new_instance(&[]);
            let mixed = unsafe { instance.send("mixed", &[]) };
            assert_eq!(mixed.try_convert_to::<Symbol>().unwrap().to_str(), "mixed");
            let who = unsafe { instance.send("who", &[]) };
            assert_eq!(
                who.try_convert_to::<Symbol>().unwrap().to_str(),
                "prepended"
            );
            assert_eq!(
                child.ancestors()[0].name().unwrap().to_str(),
                "RutieStructPrepended"
            );

            // `allocate` skips `initialize`.
            VM::eval("class RutieStructChild; def initialize; @set = true; end; end").unwrap();
            let allocated = child.allocate();
            assert!(allocated.instance_variable_get("@set").is_nil());
            assert!(child
                .new_instance(&[])
                .instance_variable_get("@set")
                .value()
                .is_true());
        });
    }

    #[test]
    fn test_class_attrs_and_alloc_func() {
        crate::on_ruby_thread(|| {
            let mut class = Class::new("RutieAttrs", None);
            class.attr_reader("reader");
            class.attr_writer("writer");
            class.attr_accessor("both");

            let object = VM::eval(
                "o = RutieAttrs.new
                 o.instance_variable_set(:@reader, 1)
                 o.writer = 2
                 o.both = 3
                 [o.reader, o.instance_variable_get(:@writer), o.both, o.respond_to?(:writer), o.respond_to?(:reader=)]",
            )
            .unwrap()
            .try_convert_to::<crate::Array>()
            .unwrap();

            assert_eq!(object.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
            assert_eq!(object.at(1).try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
            assert_eq!(object.at(2).try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
            assert!(!object.at(3).value().is_true());
            assert!(!object.at(4).value().is_true());

            let mut counter = Class::new("RutieAllocCounterClass", None);
            counter.define_alloc_func(counter_alloc);

            let instance = counter.new_instance(&[]);
            crate::GC::start();
            assert_eq!(instance.get_data(&*RUTIE_ALLOC_COUNTER).count, 41);

            let error = VM::eval("RutieAttrs.const_get(:Missing)").unwrap_err();
            assert!(Class::name_error().case_equals(&error));
            assert!(error.message().contains("Missing"));
        });
    }

    #[test]
    fn test_class_subclasses_and_cvar_find() {
        crate::on_ruby_thread(|| {
            let base = Class::new("RutieSubBase", None);
            let left = Class::new("RutieSubLeft", Some(&base));
            let right = Class::new("RutieSubRight", Some(&base));

            // Most recent first.
            assert_eq!(base.subclasses(), vec![right, left]);
            assert!(Class::basic_object()
                .subclasses()
                .contains(&Class::object()));

            VM::eval(
                "class RutieCvarTop; @@shared = :top; end
                 class RutieCvarMid < RutieCvarTop; end
                 class RutieCvarLow < RutieCvarMid; end
                 class RutieCvarOwn; end
                 class RutieCvarOvertaken < RutieCvarOwn; end",
            )
            .unwrap();

            let (value, owner) = Class::from_existing("RutieCvarLow")
                .class_variable_find("@@shared")
                .unwrap();
            assert_eq!(value.try_convert_to::<Symbol>(), Ok(Symbol::new("top")));
            assert_eq!(
                owner,
                Module::from(Class::from_existing("RutieCvarTop").value())
            );

            let error = Class::from_existing("RutieCvarLow")
                .class_variable_find("@@missing")
                .unwrap_err();
            assert!(Class::name_error().case_equals(&error));

            // An ancestor that defines it later overtakes the subclass's.
            VM::eval(
                "class RutieCvarOvertaken; @@own = 2; end
                 class RutieCvarOwn; @@own = 3; end",
            )
            .unwrap();
            let error = Class::from_existing("RutieCvarOvertaken")
                .class_variable_find("@@own")
                .unwrap_err();
            assert!(Class::runtime_error().case_equals(&error));
            assert!(error.message().contains("overtaken"));
        });
    }

    #[test]
    fn test_class_deprecate_constant() {
        crate::on_ruby_thread(|| {
            let mut klass = Class::new("RutieDeprecating", None);

            klass.const_set("OLD", &Fixnum::new(1));
            klass.deprecate_constant("OLD").unwrap();

            VM::eval(
                "$rutie_deprecated_was = Warning[:deprecated]
                 Warning[:deprecated] = true
                 def Warning.warn(message, category: nil) = ($rutie_warned = [message, category])",
            )
            .unwrap();
            let warned = VM::eval(
                "begin
                   RutieDeprecating::OLD
                   $rutie_warned
                 ensure
                   Warning.singleton_class.send(:remove_method, :warn)
                   Warning[:deprecated] = $rutie_deprecated_was
                 end",
            )
            .unwrap()
            .try_convert_to::<crate::Array>()
            .unwrap();
            assert!(warned
                .at(0)
                .try_convert_to::<RString>()
                .unwrap()
                .to_str()
                .contains("constant RutieDeprecating::OLD is deprecated"));
            assert_eq!(
                warned.at(1).try_convert_to::<Symbol>(),
                Ok(Symbol::new("deprecated"))
            );

            let missing = klass.deprecate_constant("RutieNeverDefined").unwrap_err();
            assert!(Class::name_error().case_equals(&missing));
            assert!(klass.deprecate_constant("O\0LD").is_err());

            klass.freeze();
            let frozen = klass.deprecate_constant("OLD").unwrap_err();
            assert!(Class::frozen_error().case_equals(&frozen));
        });
    }

    #[cfg(ruby_gte_3_2)]
    #[test]
    fn test_class_attached_object() {
        crate::on_ruby_thread(|| {
            let klass = Class::new("RutieAttached", None);

            let attached = klass.singleton_class().attached_object().unwrap();
            assert_eq!(Class::from(attached.value()), klass);

            let error = klass.attached_object().unwrap_err();
            assert!(Class::type_error().case_equals(&error));
        });
    }

    #[cfg(ruby_gte_3_3)]
    #[test]
    fn test_class_data_define() {
        crate::on_ruby_thread(|| {
            let pair = Class::data_define(None, &["left", "right"]).unwrap();
            Class::object().const_set("RutieDataPair", &pair);

            let inspected = VM::eval("RutieDataPair.new(left: 1, right: 2).inspect").unwrap();
            assert_eq!(
                inspected.try_convert_to::<RString>().unwrap().to_str(),
                "#<data RutieDataPair left=1, right=2>"
            );
            assert!(pair.inherits(&Class::from_existing("Data")) == Some(true));

            let empty = Class::data_define(None, &[]).unwrap();
            let members = unsafe { empty.send("members", &[]) };
            assert_eq!(
                members.try_convert_to::<crate::Array>().unwrap().length(),
                0
            );

            // Through `Data.define` past the variadic call's limit, also
            // under a `Data` class (which undefines `define`).
            let names: Vec<String> = (0..40).map(|i| format!("m{}", i)).collect();
            let names: Vec<&str> = names.iter().map(|name| name.as_str()).collect();
            let wide = Class::data_define(Some(&pair), &names).unwrap();
            assert_eq!(wide.superclass(), Some(Class::from(pair.value())));
            let members = unsafe { wide.send("members", &[]) };
            assert_eq!(
                members.try_convert_to::<crate::Array>().unwrap().length(),
                40
            );

            let mut duplicated = names.clone();
            duplicated.push("m0");
            assert!(Class::data_define(None, &duplicated).is_err());

            let error = Class::data_define(None, &["a", "a"]).unwrap_err();
            assert!(Class::argument_error().case_equals(&error));
            assert!(Class::data_define(None, &["a\0b"]).is_err());

            let error = Class::data_define(Some(&Class::object()), &["a"]).unwrap_err();
            assert!(Class::type_error().case_equals(&error));
        });
    }
}
