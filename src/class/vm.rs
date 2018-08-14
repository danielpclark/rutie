use binding::vm;
use types::{Argc, Value};

use {AnyObject, AnyException, Class, Object, Proc, NilClass, Array};

/// Virtual Machine and helpers
pub struct VM;

impl VM {
    /// Initializes Ruby virtual machine.
    ///
    /// This function should **ONLY** be used if you write a standalone application which calls
    /// Ruby itself, for example:
    ///
    /// - Sidekiq-like background processing
    ///
    /// - Unicorn-like web server
    ///
    /// In these cases it should be called before any interaction with Ruby.
    ///
    /// If you write a library which is being connected to Ruby in runtime (e.g. some gem), this
    /// function should not be used.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, VM};
    ///
    /// VM::init();
    ///
    /// // VM started, able to use Ruby now
    /// // ...
    ///
    /// Class::new("SomeClass", None); // etc
    /// ```
    pub fn init() {
        vm::init();
    }

    /// Initializes Ruby load path.
    ///
    /// This enables more of Ruby's internal features such as making additional encodings
    /// available.
    ///
    /// This function, like `VM::init`, should **ONLY** be used if you write a standalone
    /// application which calls Ruby itself.
    ///
    /// If you write a library which is being connected to Ruby in runtime (e.g. some gem), this
    /// function should not be used.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, Encoding, EncodingSupport, VM, Object};
    /// # VM::init();
    /// VM::init_loadpath(); // Needed for alternate encodings
    /// VM::require("enc/encdb");
    /// VM::require("enc/trans/transdb");
    ///
    /// let bytes = [254, 255, 1, 65, 0, 97, 1, 66] ;
    ///
    /// let enc = Encoding::find("UTF-16").unwrap();
    ///
    /// let mut string = RString::from_bytes(&bytes, &enc);
    ///
    /// assert_eq!(string.to_bytes_unchecked(), bytes);
    /// assert!(string.encoding().equals(&enc), "not equal!");
    /// ```
    pub fn init_loadpath() {
        vm::init_loadpath();
    }

    /// Requires Ruby source file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rutie::VM;
    /// # VM::init();
    ///
    /// VM::require("some_ruby_file");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// require 'some_ruby_file'
    /// ```
    pub fn require(name: &str) {
        vm::require(name);
    }

    /// Raises an exception.
    ///
    /// # Examples
    ///
    /// ### Built-in exceptions
    ///
    /// ```no_run
    /// use rutie::{Class, VM};
    /// # VM::init();
    ///
    /// VM::raise(Class::from_existing("ArgumentError"), "Wrong argument");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// raise ArgumentError, 'Wrong argument'
    /// ```
    ///
    /// ### Custom exceptions
    ///
    /// ```no_run
    /// use rutie::{Class, VM};
    /// # VM::init();
    ///
    /// let standard_error = Class::from_existing("StandardError");
    /// let custom_exception = Class::new("CustomException", Some(&standard_error));
    ///
    /// VM::raise(custom_exception, "Something went wrong");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class CustomException < StandardError
    /// end
    ///
    /// raise CustomException, 'Something went wrong'
    /// ```
    pub fn raise(exception: Class, message: &str) {
        vm::raise(exception.value(), message);
    }

    /// Raises an exception from a native `AnyException` object.
    ///
    /// # Examples
    ///
    /// ### Built-in exceptions
    ///
    /// ```no_run
    /// use rutie::{Class, VM, Exception, AnyException};
    /// # VM::init();
    ///
    /// VM::raise_ex(AnyException::new("StandardError", Some("something went wrong")));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// raise StandardError, 'something went wrong'
    /// ```
    ///
    /// ### Custom exceptions
    ///
    /// ```no_run
    /// use rutie::{Class, VM, Exception, AnyException};
    /// # VM::init();
    ///
    /// let standard_error = Class::from_existing("StandardError");
    /// Class::new("CustomException", Some(&standard_error));
    ///
    /// let exception = AnyException::new("CustomException", Some("something went wrong"));
    ///
    /// VM::raise_ex(exception);
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class CustomException < StandardError
    /// end
    ///
    /// raise CustomException, 'Something went wrong'
    /// ```
    pub fn raise_ex(exception: AnyException) {
        vm::raise_ex(exception.value());
    }

    /// Evals string and returns an Result<AnyObject, c_int>
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use]
    /// extern crate rutie;
    ///
    /// use rutie::{Class, Fixnum, Object, VM};
    ///
    /// fn main() {
    ///     # VM::init();
    ///
    ///     // Successful example
    ///
    ///     let result = VM::eval("2+2").ok().unwrap().try_convert_to::<Fixnum>();
    ///
    ///     assert_eq!(result, Ok(Fixnum::new(4)));
    ///
    ///     // Error example
    ///
    ///     let result = VM::eval("raise 'flowers'");
    ///
    ///     assert!(result.is_err());
    /// }
    /// ```
    ///
    /// `Err` will return an `AnyObject` of the exception class raised.
    ///
    ///
    /// ```
    /// #[macro_use]
    /// extern crate rutie;
    ///
    /// use rutie::{Class, Fixnum, Object, Exception, RString, VM};
    ///
    /// fn main() {
    ///     # VM::init();
    ///
    ///     let result = VM::eval("raise IndexError, 'flowers'");
    ///
    ///     match result {
    ///       Err(ao) => {
    ///         let err = ao.message();
    ///         assert_eq!(err, "flowers");
    ///       },
    ///       _ => { unreachable!() }
    ///     }
    /// }
    /// ```
    ///
    /// Be aware when checking for equality amongst types like strings, that even
    /// with the same content in Ruby, they will evaluate to different values in
    /// C/Rust.
    pub fn eval(string: &str) -> Result<AnyObject, AnyException> {
        vm::eval_string_protect(string).map(|v|
            AnyObject::from(v)
        ).map_err(|_| {
            let output = AnyException::from(vm::errinfo());

            // error cleanup
            vm::set_errinfo(NilClass::new().value());

            output
        })
    }

    /// Evals string and returns an AnyObject
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use]
    /// extern crate rutie;
    ///
    /// use rutie::{Class, Fixnum, Object, VM};
    ///
    /// fn main() {
    ///     # VM::init();
    ///
    ///     let result = unsafe { VM::eval_str("2+2").try_convert_to::<Fixnum>() };
    ///
    ///     assert_eq!(result, Ok(Fixnum::new(4)));
    /// }
    /// ```
    ///
    /// Be aware when checking for equality amongst types like strings, that even
    /// with the same content in Ruby, they will evaluate to different values in
    /// C/Rust.
    ///
    /// Marked unsafe because "evaluation can raise an exception."
    pub unsafe fn eval_str(string: &str) -> AnyObject {
        AnyObject::from(
            vm::eval_string(string)
        )
    }

    /// Converts a block given to current method to a `Proc`
    ///
    /// It works similarly to `def method(&block)` which converts block to `Proc`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// #[macro_use]
    /// extern crate rutie;
    ///
    /// use rutie::{Class, Object, Proc, RString, VM};
    ///
    /// class!(Greeter);
    ///
    /// methods!(
    ///     Greeter,
    ///     itself,
    ///
    ///     fn greet_rust_with() -> RString {
    ///         let greeting_template = VM::block_proc();
    ///         let name = RString::new_utf8("Rust").to_any_object();
    ///
    ///         greeting_template.call(Some(&[name])).try_convert_to::<RString>().unwrap()
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
    ///   def self.greet_rust_with(&greeting_template)
    ///     greeting_template.call('Rust')
    ///   end
    /// end
    ///
    /// Greeter.greet_rust_with do |name|
    ///   "Hello, #{name}!"
    /// end
    /// # => "Hello, Rust!"
    /// ```
    pub fn block_proc() -> Proc {
        Proc::from(vm::block_proc())
    }

    /// Checks if a block is given to current method.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Fixnum, Object, VM};
    ///
    /// class!(Calculator);
    ///
    /// methods!(
    ///     Calculator,
    ///     itself,
    ///
    ///     fn calculate(a: Fixnum, b: Fixnum) -> Fixnum {
    ///         let a = a.unwrap();
    ///         let b = b.unwrap();
    ///
    ///         if VM::is_block_given() {
    ///             let arguments = [a.to_any_object(), b.to_any_object()];
    ///             let result = VM::block_proc().call(Some(&arguments));
    ///
    ///             result.try_convert_to::<Fixnum>().unwrap()
    ///         } else {
    ///             Fixnum::new(a.to_i64() + b.to_i64())
    ///         }
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///
    ///     Class::new("Calculator", None).define(|itself| {
    ///         itself.def("calculate", calculate);
    ///     });
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Calculator
    ///   def calculate(a, b, &block)
    ///     if block_given?
    ///       block.call(a, b)
    ///     else
    ///       a + b
    ///     end
    ///   end
    /// end
    /// ```
    pub fn is_block_given() -> bool {
        vm::is_block_given()
    }

    /// Yield object to block
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Fixnum, Object, VM};
    ///
    /// class!(Calculator);
    ///
    /// methods!(
    ///     Calculator,
    ///     itself,
    ///
    ///     fn calculate(a: Fixnum) -> Fixnum {
    ///         let a = a.map_err(|e| VM::raise_ex(e) ).unwrap();
    ///
    ///         if VM::is_block_given() {
    ///             let argument = a.to_any_object();
    ///             let result = VM::yield_object(a);
    ///
    ///             result.try_convert_to::<Fixnum>().unwrap()
    ///         } else {
    ///             VM::raise(Class::from_existing("LocalJumpError"), "no block given (yield)");
    ///             unreachable!();
    ///         }
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///
    ///     Class::new("Calculator", None).define(|itself| {
    ///         itself.def("calculate", calculate);
    ///     });
    ///
    ///     let result = VM::eval(" Calculator.new().calculate(4) { |n| n * n } ").unwrap();
    ///     let num = result.try_convert_to::<Fixnum>().unwrap().to_i64();
    ///     assert_eq!(num, 16);
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Calculator
    ///   def calculate(a)
    ///     if block_given?
    ///       yield a
    ///     else
    ///       raise LocalJumpError, "no block given (yield)"
    ///     end
    ///   end
    /// end
    ///
    /// result = Calculator.new.calculate(4) { |n| n * n }
    /// result == 16
    /// ```
    pub fn yield_object(object: impl Object) -> AnyObject {
        AnyObject::from(vm::yield_object(object.value()))
    }

    /// Yield splat from array of Ruby objects to block
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Array, Class, Fixnum, Object, VM};
    ///
    /// class!(Calculator);
    ///
    /// methods!(
    ///     Calculator,
    ///     itself,
    ///
    ///     fn calculate(a: Array) -> Fixnum {
    ///         let a = a.map_err(|e| VM::raise_ex(e) ).unwrap();
    ///
    ///         if VM::is_block_given() {
    ///             let argument = a.to_any_object();
    ///             let result = VM::yield_splat(a);
    ///
    ///             result.try_convert_to::<Fixnum>().unwrap()
    ///         } else {
    ///             VM::raise(Class::from_existing("LocalJumpError"), "no block given (yield)");
    ///             unreachable!();
    ///         }
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///
    ///     Class::new("Calculator", None).define(|itself| {
    ///         itself.def("calculate", calculate);
    ///     });
    ///
    ///     let result = VM::eval(" Calculator.new().calculate([4,6,8]) { |a,b,c| a*b-c } ").unwrap();
    ///     let num = result.try_convert_to::<Fixnum>().unwrap().to_i64();
    ///     assert_eq!(num, 16);
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Calculator
    ///   def calculate(a)
    ///     if block_given?
    ///       yield a
    ///     else
    ///       raise LocalJumpError, "no block given (yield)"
    ///     end
    ///   end
    /// end
    ///
    /// result = Calculator.new.calculate([4,6,8]) { |a,b,c| a*b-c }
    /// result == 16
    /// ```
    pub fn yield_splat(objects: Array) -> AnyObject {
        AnyObject::from(vm::yield_splat(objects.value()))
    }

    // TODO: Change result to return `AnyObject` instead of `Value`.
    // Do this by next :SEMVER:MINOR: change.
    pub fn protect<F>(func: F) -> Result<Value, i32>
    where
        F: FnMut() -> Value,
    {
        vm::protect(func)
    }
}
