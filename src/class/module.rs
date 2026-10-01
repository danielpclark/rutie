use std::convert::From;

use crate::{
    binding::{class, global::rb_cObject, module, vm},
    typed_data::DataTypeWrapper,
    types::{Callback, Value, ValueType},
    AnyException, AnyObject, Array, Class, Exception, NilClass, Object, RString, Symbol,
    VerifiedObject,
};

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
///    rtself,
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
///     Module::new("Example").define(|klass| {
///         klass.def("square", square);
///     });
/// }
/// ```
///
/// Ruby:
///
/// ```ruby
/// module Example
///   def square(exp)
///     raise TypeError unless exp.is_a?(Integer)
///
///     exp * exp
///   end
/// end
/// ```
#[derive(Debug)]
#[repr(C)]
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

    /// Creates an anonymous `Refinement`, a module not attached to any
    /// class (`rb_refinement_new`).
    ///
    /// Ruby 3.1+.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// let refinement = Module::new_refinement();
    ///
    /// assert_eq!(refinement.class(), Class::refinement());
    /// assert!(refinement.name().is_none());
    /// ```
    #[cfg(ruby_gte_3_1)]
    pub fn new_refinement() -> Self {
        Self::from(class::refinement_new())
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
    /// Module::new("Outer").define(|klass| {
    ///     klass.define_nested_module("Inner");
    /// });
    ///
    /// let inner = Module::from_existing("Outer").get_nested_module("Inner");
    ///
    /// assert_eq!(inner.name().unwrap().to_str(), "Outer::Inner");
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
    /// Module::new("Outer").define(|klass| {
    ///     klass.define_nested_class("Inner", None);
    /// });
    ///
    /// let inner = Module::from_existing("Outer").get_nested_class("Inner");
    ///
    /// assert_eq!(inner.name().unwrap().to_str(), "Outer::Inner");
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
    /// let mut outer = Module::new("Outer");
    /// let inner = outer.define_nested_module("Inner");
    ///
    /// assert_eq!(inner.name().unwrap().to_str(), "Outer::Inner");
    /// assert!(Module::from_existing("Outer").get_nested_module("Inner") == inner);
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
    /// let mut outer = Module::new("Outer");
    /// let inner = outer.define_nested_class("Inner", None);
    ///
    /// assert_eq!(inner.name().unwrap().to_str(), "Outer::Inner");
    /// assert_eq!(inner.superclass(), Some(Class::object()));
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
    ///    rtself,
    ///
    ///    fn is_blank() -> Boolean {
    ///        Boolean::new(rtself.to_str().chars().all(|c| c.is_whitespace()))
    ///    }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Module::new("Blank").define(|klass| {
    ///         klass.define_module_function("blank?", is_blank);
    ///     });
    ///
    ///     Class::from_existing("String").include("Blank");
    ///
    ///     // Callable on the module, and privately inside includers.
    ///     let blank = VM::eval("Blank.instance_method(:blank?).bind(' ').call").unwrap();
    ///     assert!(blank.try_convert_to::<Boolean>().unwrap().to_bool());
    ///     assert!(VM::eval("' '.blank?").is_err());
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
    ///     Module::from_existing("Integer").define(|klass| {
    ///         klass.mod_func("pow", pow);
    ///         klass.mod_func("pow_with_default_argument", pow_with_default_argument);
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
    ///   module_function :pow
    ///
    ///   def pow_with_default_argument(exp)
    ///     default_exp = 0
    ///     exp = default_exp unless exp.is_a?(Integer)
    ///
    ///     self ** exp
    ///   end
    ///   module_function :pow_with_default_argument
    /// end
    /// ```
    pub fn define_module_function<I: Object, O: Object>(
        &mut self,
        name: &str,
        callback: Callback<I, O>,
    ) {
        module::define_module_function(self.value(), name, callback);
    }

    /// An alias for `define_module_function` (similar to Ruby `module_function :some_method`).
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Fixnum, Module, Object, VM};
    ///
    /// module!(Doubler);
    ///
    /// methods!(
    ///     Doubler,
    ///     rtself,
    ///
    ///     fn double(number: Fixnum) -> Fixnum {
    ///         Fixnum::new(number.unwrap().to_i64() * 2)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Module::new("Doubler").define(|module| {
    ///         module.mod_func("double", double);
    ///     });
    ///
    ///     let result = VM::eval("Doubler.double(21)").unwrap();
    ///     assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));
    /// }
    /// ```
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
    /// Module::new("Greeter").define(|klass| {
    ///     klass.const_set("GREETING", &RString::new_utf8("Hello, World!"));
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
    /// Module::new("Greeter").define(|klass| {
    ///     klass.const_set("GREETING", &RString::new_utf8("Hello, World!"));
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
    /// use rutie::{Fixnum, Module, Object, VM};
    /// # VM::init();
    ///
    /// Module::new("Test").define(|klass| {
    ///     klass.attr_reader("reader");
    /// });
    ///
    /// let object = VM::eval("o = Object.new.extend(Test); o.instance_variable_set(:@reader, 1); o").unwrap();
    /// assert_eq!(unsafe { object.send("reader", &[]) }.try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
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
    /// use rutie::{Fixnum, Module, Object, VM};
    /// # VM::init();
    ///
    /// Module::new("Test").define(|klass| {
    ///     klass.attr_writer("writer");
    /// });
    ///
    /// let object = VM::eval("o = Object.new.extend(Test); o.writer = 2; o").unwrap();
    /// assert_eq!(object.instance_variable_get("@writer").try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
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
    /// use rutie::{Fixnum, Module, Object, VM};
    /// # VM::init();
    ///
    /// Module::new("Test").define(|klass| {
    ///     klass.attr_accessor("accessor");
    /// });
    ///
    /// let object = VM::eval("o = Object.new.extend(Test); o.accessor = 3; o").unwrap();
    /// assert_eq!(unsafe { object.send("accessor", &[]) }.try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
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

    /// Makes `new_name` a copy of the method `old_name` (`rb_define_alias`,
    /// Ruby's `alias_method`).
    ///
    /// Raises `NameError` if `old_name` is not defined.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Module, Object, RString, VM};
    /// # VM::init();
    ///
    /// VM::eval("module Aliased; def hello; 'hello'; end; end").unwrap();
    ///
    /// Module::from_existing("Aliased").define_alias("greet", "hello");
    ///
    /// let greeting = VM::eval("Class.new { include Aliased }.new.greet").unwrap();
    ///
    /// assert_eq!(greeting.try_convert_to::<RString>().unwrap().to_str(), "hello");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// module Aliased
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
    /// use rutie::{Module, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("module Undefined; def to_s; 'custom'; end; end").unwrap();
    ///
    /// Module::from_existing("Undefined").undef_method("to_s");
    ///
    /// assert!(VM::eval("Class.new { include Undefined }.new.to_s").is_err());
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// module Undefined
    ///   undef_method :to_s
    /// end
    /// ```
    pub fn undef_method(&mut self, name: &str) {
        class::undef_method(self.value(), name);
    }

    /// Returns the module's name, or `None` for an anonymous module
    /// (Ruby's `name`, `rb_mod_name`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// assert_eq!(Module::from_existing("Kernel").name().unwrap().to_str(), "Kernel");
    ///
    /// let anonymous = VM::eval("Module.new").unwrap().try_convert_to::<Module>().unwrap();
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

    /// Returns the module's full path, such as `"A::B"`, or a
    /// `"#<Module:0x...>"` description for an anonymous module
    /// (`rb_class_path`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("module Outer; module Inner; end; end").unwrap();
    ///
    /// let inner = Module::from_existing("Outer").get_nested_module("Inner");
    ///
    /// assert_eq!(inner.path().to_str(), "Outer::Inner");
    /// ```
    pub fn path(&self) -> RString {
        RString::from(class::class_path(self.value()))
    }

    /// Returns the module named by a path such as `"A::B"`
    /// (`rb_path2class`), or an error if nothing is defined there
    /// (`ArgumentError`) or it is not a module (`TypeError`).
    ///
    /// Unlike `from_existing`, nested paths work and a missing constant is
    /// returned as an error instead of raising. No symbols are created for
    /// unknown names, so untrusted paths are fine.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Module, Exception, Object, VM};
    /// # VM::init();
    ///
    /// let found = Module::from_path("File::Constants").unwrap();
    ///
    /// assert_eq!(found.path().to_str(), "File::Constants");
    ///
    /// assert!(Module::from_path("No::Such::Thing").is_err());
    ///
    /// let error = Module::from_path("String").unwrap_err();
    ///
    /// assert!(error.message().contains("is not a module"));
    /// ```
    pub fn from_path(path: &str) -> Result<Self, AnyException> {
        let path = ::std::ffi::CString::new(path).map_err(|_| {
            AnyException::new("ArgumentError", Some("class path contains a NUL byte"))
        })?;

        let found =
            vm::protect_value(|| class::path_to_class(&path)).map_err(AnyException::from)?;

        if found.ty() == ValueType::Module {
            Ok(Self::from(found))
        } else {
            let message = format!("{} is not a module", path.to_string_lossy());

            Err(AnyException::new("TypeError", Some(&message)))
        }
    }

    /// Returns `true` if instances respond to the method `name` defined in
    /// this module or its ancestors (`rb_method_boundp`). Private methods
    /// count only when `include_private` is `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("module Sample; @@count = 1; LIMIT = 10; def visible; end; private def hidden; end; end").unwrap();
    ///
    /// let sample = Module::from_existing("Sample");
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
    /// use rutie::{Module, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("module Arities; def two(a, b); end; def many(a, *rest); end; end").unwrap();
    ///
    /// let arities = Module::from_existing("Arities");
    ///
    /// assert_eq!(arities.instance_method_arity("two"), 2);
    /// assert_eq!(arities.instance_method_arity("many"), -2);
    /// ```
    pub fn instance_method_arity(&self, name: &str) -> i32 {
        crate::binding::rproc::module_method_arity(self.value(), name)
    }

    /// Compares this module with `other` in the class hierarchy, like
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
    /// let enumerable = Module::from_existing("Enumerable");
    /// let comparable = Module::from_existing("Comparable");
    /// let array = Class::from_existing("Array");
    ///
    /// assert_eq!(enumerable.inherits(&enumerable), Some(true));
    /// assert_eq!(enumerable.inherits(&array), Some(false));
    /// assert_eq!(enumerable.inherits(&comparable), None);
    /// ```
    pub fn inherits<T: Object>(&self, other: &T) -> Option<bool> {
        let result = class::inherited_p(self.value(), other.value());

        if result.is_nil() {
            None
        } else {
            Some(result.is_true())
        }
    }

    /// Returns `true` if `module` is included in this module or one of its
    /// ancestors (Ruby's `include?`, `rb_mod_include_p`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("module Outer; include Comparable; end").unwrap();
    ///
    /// let outer = Module::from_existing("Outer");
    ///
    /// assert!(outer.includes_module(&Module::from_existing("Comparable")));
    /// assert!(!outer.includes_module(&Module::from_existing("Enumerable")));
    /// ```
    pub fn includes_module(&self, module: &Module) -> bool {
        class::include_p(self.value(), module.value())
    }

    /// Evaluates `code` in the context of this module (Ruby's
    /// `module_eval`/`class_eval`, `rb_mod_module_eval`), returning the result
    /// or the exception raised.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Module, Exception, Object, RString, VM};
    /// # VM::init();
    ///
    /// let mut evaluated = Module::new("Evaluated");
    ///
    /// evaluated.module_eval("def greet; 'hi'; end").unwrap();
    ///
    /// let greeting = VM::eval("Object.new.extend(Evaluated).greet").unwrap();
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
    /// use rutie::{Module, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// VM::eval("module Sample; @@count = 1; LIMIT = 10; def visible; end; private def hidden; end; end").unwrap();
    ///
    /// let methods = Module::from_existing("Sample").instance_methods(false);
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
    /// use rutie::{Module, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("module Sample; @@count = 1; LIMIT = 10; def visible; end; private def hidden; end; end").unwrap();
    ///
    /// let sample = Module::from_existing("Sample");
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
    /// a class variable name, or a `FrozenError` if the module is frozen.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Module, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let mut counter = Module::new("Counter");
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
    /// VM::eval("module Sample; @@count = 1; LIMIT = 10; def visible; end; private def hidden; end; end").unwrap();
    ///
    /// let sample = Module::from_existing("Sample");
    ///
    /// assert!(sample.is_class_variable_defined("@@count"));
    /// assert!(!sample.is_class_variable_defined("@@missing"));
    /// ```
    pub fn is_class_variable_defined(&self, name: &str) -> bool {
        class::is_class_variable_defined(self.value(), name)
    }

    /// Returns `true` if the constant `name` is visible from this module:
    /// defined in it, its ancestors or `Object` (`rb_const_defined`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("module Sample; @@count = 1; LIMIT = 10; def visible; end; private def hidden; end; end").unwrap();
    ///
    /// let sample = Module::from_existing("Sample");
    ///
    /// assert!(sample.is_const_defined("LIMIT"));
    /// assert!(sample.is_const_defined("String"));
    /// assert!(!sample.is_const_defined("MISSING"));
    /// ```
    pub fn is_const_defined(&self, name: &str) -> bool {
        class::is_const_defined(self.value(), name)
    }

    /// Returns `true` if the constant `name` is defined directly in this
    /// module, not inherited (`rb_const_defined_at`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("module Sample; @@count = 1; LIMIT = 10; def visible; end; private def hidden; end; end").unwrap();
    ///
    /// let sample = Module::from_existing("Sample");
    ///
    /// assert!(sample.is_const_defined_at("LIMIT"));
    /// assert!(!sample.is_const_defined_at("String"));
    /// ```
    pub fn is_const_defined_at(&self, name: &str) -> bool {
        class::is_const_defined_at(self.value(), name)
    }

    /// Removes the constant `name` defined directly in this module and
    /// returns its value, or `None` if there is no such constant
    /// (`rb_const_remove`).
    ///
    /// Raises `FrozenError` if the module is frozen and the constant exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Module, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("module Sample; @@count = 1; LIMIT = 10; def visible; end; private def hidden; end; end").unwrap();
    ///
    /// let mut sample = Module::from_existing("Sample");
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
    /// with the module (or class) that defines it, searching this module and
    /// its ancestors (`rb_cvar_find`), or returns the error: a `NameError`
    /// if it is not defined, or a `RuntimeError` if both this module and an
    /// ancestor define it.
    ///
    /// Ruby 3.1+.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Module, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("module Base; @@level = 1; end; module Extended; include Base; end").unwrap();
    ///
    /// let extended = Module::from_existing("Extended");
    /// let (level, owner) = extended.class_variable_find("@@level").unwrap();
    ///
    /// assert_eq!(level.try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// assert_eq!(owner, Module::from_existing("Base"));
    /// assert!(extended.class_variable_find("@@missing").is_err());
    /// ```
    #[cfg(ruby_gte_3_1)]
    pub fn class_variable_find(&self, name: &str) -> Result<(AnyObject, Module), AnyException> {
        Class::from(self.value()).class_variable_find(name)
    }

    /// Marks the constant `name` defined directly in this module as
    /// deprecated (Ruby's `deprecate_constant`, `rb_deprecate_constant`):
    /// using it warns when deprecation warnings are enabled. Returns the
    /// error: a `NameError` if the module does not define the constant, or a
    /// `FrozenError` if it is frozen.
    ///
    /// Ruby 3.1+.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Module, Object, RString, VM};
    /// # VM::init();
    ///
    /// let mut settings = Module::new("Settings");
    ///
    /// settings.const_set("LEGACY", &Fixnum::new(1));
    /// settings.deprecate_constant("LEGACY").unwrap();
    ///
    /// VM::eval("Warning[:deprecated] = true
    ///           def Warning.warn(message, category: nil) = ($warned = message)").unwrap();
    /// VM::eval("Settings::LEGACY").unwrap();
    ///
    /// let warned = VM::eval("$warned").unwrap().try_convert_to::<RString>().unwrap();
    ///
    /// assert!(warned.to_str().contains("Settings::LEGACY is deprecated"));
    /// assert!(settings.deprecate_constant("MISSING").is_err());
    /// ```
    #[cfg(ruby_gte_3_1)]
    pub fn deprecate_constant(&mut self, name: &str) -> Result<(), AnyException> {
        crate::class::class::deprecate_constant(self.value(), name)
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
    ///     Class::new("RubyServer", None).define(|klass| {
    ///         klass.def_self("new", ruby_server_new);
    ///
    ///         klass.def("host", ruby_server_host);
    ///         klass.def("port", ruby_server_port);
    ///     });
    ///
    ///     let host = VM::eval("RubyServer.new('127.0.0.1', 3000).host").unwrap();
    ///     assert_eq!(host.try_convert_to::<RString>().unwrap().to_str(), "127.0.0.1");
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

impl From<Value> for Module {
    fn from(value: Value) -> Self {
        Module { value }
    }
}

impl Into<Value> for Module {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Module {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
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

impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{AnyObject, Array, Class, Fixnum, Module, Object, RString, Symbol, GC, VM};

    crate::module!(RutieTestModule);

    crate::methods!(
        RutieTestModule,
        rtself,
        fn rutie_module_double(number: Fixnum) -> Fixnum {
            Fixnum::new(number.unwrap().to_i64() * 2)
        },
        fn rutie_module_name() -> RString {
            RString::new_utf8("helper")
        }
    );

    fn eval_array(code: &str) -> Array {
        VM::eval(code).unwrap().try_convert_to::<Array>().unwrap()
    }

    #[test]
    fn test_module_definitions() {
        crate::on_ruby_thread(|| {
            let mut module = Module::new("RutieTestModuleOne");
            assert_eq!(Module::from_existing("RutieTestModuleOne"), module);

            module.define_module_function("double", rutie_module_double);
            module.mod_func("helper_name", rutie_module_name);
            GC::start();

            let doubled = VM::eval("RutieTestModuleOne.double(21)").unwrap();
            assert_eq!(doubled.try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));

            // Module functions are also private instance methods.
            let result = eval_array(
                "c = Class.new { include RutieTestModuleOne; def call; helper_name; end }
                 [c.new.call, c.new.respond_to?(:helper_name), RutieTestModuleOne.helper_name]",
            );
            assert_eq!(
                result.at(0).try_convert_to::<RString>().unwrap().to_str(),
                "helper"
            );
            assert!(!result.at(1).value().is_true());
            assert_eq!(
                result.at(2).try_convert_to::<RString>().unwrap().to_str(),
                "helper"
            );

            let nested = module.define_nested_module("Inner");
            assert_eq!(module.get_nested_module("Inner"), nested);
            let nested_class = module.define_nested_class("Thing", None);
            assert_eq!(module.get_nested_class("Thing"), nested_class);
            assert_eq!(
                nested_class.name().unwrap().to_str(),
                "RutieTestModuleOne::Thing"
            );

            module.const_set("LIMIT", &Fixnum::new(3));
            assert_eq!(
                module.const_get("LIMIT").try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(3))
            );
            let from_ruby = VM::eval("RutieTestModuleOne::LIMIT").unwrap();
            assert_eq!(from_ruby.try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
        });
    }

    #[test]
    fn test_module_composition_and_attrs() {
        crate::on_ruby_thread(|| {
            VM::eval("module RutieTestModuleBase; def base; :base; end; end").unwrap();
            VM::eval("module RutieTestModuleFront; def base; :front; end; end").unwrap();

            let mut module = Module::new("RutieTestModuleTwo");
            module.include("RutieTestModuleBase");
            module.prepend("RutieTestModuleFront");

            let ancestors: Vec<String> = module
                .ancestors()
                .iter()
                .map(|m| m.name().unwrap().to_string())
                .collect();
            assert_eq!(
                ancestors,
                vec![
                    "RutieTestModuleFront",
                    "RutieTestModuleTwo",
                    "RutieTestModuleBase"
                ]
            );

            module.attr_reader("reader");
            module.attr_writer("writer");
            module.attr_accessor("both");

            let klass = Class::new("RutieTestModuleUser", None);
            klass.include("RutieTestModuleTwo");

            let result = eval_array(
                "o = RutieTestModuleUser.new
                 o.instance_variable_set(:@reader, 1)
                 o.writer = 2
                 o.both = 3
                 [o.reader, o.instance_variable_get(:@writer), o.both, o.base]",
            );
            assert_eq!(result.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
            assert_eq!(result.at(1).try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
            assert_eq!(result.at(2).try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
            assert_eq!(
                result.at(3).try_convert_to::<Symbol>().unwrap().to_str(),
                "front"
            );

            let any: AnyObject = module.to_any_object();
            assert!(any.try_convert_to::<Module>().is_ok());
            assert!(Class::object()
                .to_any_object()
                .try_convert_to::<Module>()
                .is_err());
        });
    }

    #[test]
    #[cfg(ruby_gte_3_1)]
    fn test_module_refinement_cvar_find_and_deprecate() {
        crate::on_ruby_thread(|| {
            let refinement = Module::new_refinement();
            assert_eq!(refinement.class(), Class::refinement());
            assert!(refinement.name().is_none());
            assert!(Module::new_refinement() != refinement);

            VM::eval(
                "module RutieCvarModule; @@flag = :on; end
                 class RutieCvarModuleUser; include RutieCvarModule; end
                 class RutieCvarModuleChild < RutieCvarModuleUser; end",
            )
            .unwrap();
            let module = Module::from_existing("RutieCvarModule");
            let (value, owner) = module.class_variable_find("@@flag").unwrap();
            assert_eq!(value.try_convert_to::<Symbol>(), Ok(Symbol::new("on")));
            assert_eq!(owner, module);

            // Found through the module's iclass in a subclass's ancestors.
            let (_, owner) = Class::from_existing("RutieCvarModuleChild")
                .class_variable_find("@@flag")
                .unwrap();
            assert_eq!(owner, module);
            assert!(module.class_variable_find("@@nope").is_err());

            let mut deprecating = Module::new("RutieDeprecatingModule");
            deprecating.const_set("GONE", &Fixnum::new(0));
            assert!(deprecating.deprecate_constant("GONE").is_ok());
            assert!(deprecating.deprecate_constant("NEVER_DEFINED").is_err());
        });
    }
}
