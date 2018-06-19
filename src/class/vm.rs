use std::slice;

use binding::vm;
use types::{Argc, Value};

use {AnyObject, AnyException, Class, Object, Proc, NilClass};

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
    /// use rutie::{Class, Fixnum, Object, RString, VM};
    ///
    /// fn main() {
    ///     # VM::init();
    ///
    ///     let result = VM::eval("raise IndexError, 'flowers'");
    ///
    ///     match result {
    ///       Err(ao) => {
    ///         let err = Class::from(ao.value());
    ///         let message = err.send("message", None);
    ///         let s = message.try_convert_to::<RString>();
    ///         assert_eq!(s.ok().unwrap().to_string(), "flowers");
    ///       },
    ///       _ => { unreachable!() }
    ///     }
    /// }
    /// ```
    ///
    /// Be aware when checking for equality amongst types like strings, that even
    /// with the same content in Ruby, they will evaluate to different values in
    /// C/Rust.
    pub fn eval(string: &str) -> Result<AnyObject, AnyObject> {
        vm::eval_string_protect(string).map(|v|
            AnyObject::from(v)
        ).map_err(|_| {
            let output = AnyObject::from(vm::errinfo());

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
    ///         let name = RString::new("Rust").to_any_object();
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

    // TODO: Move to other struct
    /// Converts a pointer to array of `AnyObject`s to `Vec<AnyObject>`.
    ///
    /// This function is a helper for callbacks, do not use it directly.
    ///
    /// It will be moved to other struct, because it is not related to VM itself.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rutie::types::Argc;
    /// use rutie::{AnyObject, Boolean, Class, Object, RString, VM};
    ///
    /// #[no_mangle]
    /// pub extern fn string_eq(argc: Argc, argv: *const AnyObject, itself: RString) -> Boolean {
    ///     let argv = VM::parse_arguments(argc, argv);
    ///     let other_string = argv[0].try_convert_to::<RString>().unwrap();
    ///
    ///     Boolean::new(itself.to_str() == other_string.to_str())
    /// }
    ///
    /// fn main() {
    ///     Class::from_existing("String").define_method("==", string_eq);
    /// }
    /// ```
    pub fn parse_arguments(argc: Argc, arguments: *const AnyObject) -> Vec<AnyObject> {
        unsafe { slice::from_raw_parts(arguments, argc as usize).to_vec() }
    }

    /// Release GVL for current thread.
    ///
    /// **Warning!** Due to MRI limitations, interaction with Ruby objects is not allowed while
    /// GVL is released, it may cause unexpected behaviour.
    /// [Read more at Ruby documentation](https://github.com/ruby/ruby/blob/2fc5210f31ad23463d7b0a0e36bcfbeee7b41b3e/thread.c#L1314-L1398)
    ///
    /// You should extract all the information from Ruby world before invoking
    /// `thread_call_without_gvl`.
    ///
    /// GVL will be re-acquired when the closure is finished.
    ///
    /// # Examples
    ///
    /// ```no_run
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
    ///     fn heavy_computation() -> Fixnum {
    ///         let computation = || { 2 * 2 };
    ///         let unblocking_function = || {};
    ///
    ///         // release GVL for current thread until `computation` is completed
    ///         let result = VM::thread_call_without_gvl(
    ///             computation,
    ///             Some(unblocking_function)
    ///         );
    ///
    ///         // GVL is re-acquired, we can interact with Ruby-world
    ///         Fixnum::new(result)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     Class::new("Calculator", None).define(|itself| {
    ///         itself.def("heavy_computation", heavy_computation);
    ///     });
    /// }
    /// ```
    #[deprecated(since = "0.9.2", note = "Use `Thread::call_without_gvl()` instead")]
    pub fn thread_call_without_gvl<F, R, G>(func: F, unblock_func: Option<G>) -> R
    where
        F: FnOnce() -> R,
        G: FnOnce(),
    {
        vm::thread_call_without_gvl(func, unblock_func)
    }

    #[deprecated(since = "0.9.2", note = "Use `Thread::call_without_gvl2()` instead")]
    pub fn thread_call_without_gvl2<F, R, G>(func: F, unblock_func: Option<G>) -> R
    where
        F: FnOnce() -> R,
        G: FnOnce(),
    {
        vm::thread_call_without_gvl2(func, unblock_func)
    }

    #[deprecated(since = "0.9.2", note = "Use `Thread::call_with_gvl()` instead")]
    pub fn thread_call_with_gvl<F, R>(func: F) -> R
    where
        F: FnOnce() -> R,
    {
        vm::thread_call_with_gvl(func)
    }

    pub fn protect<F>(func: F) -> Result<Value, i32>
    where
        F: FnOnce(),
    {
        vm::protect(func)
    }
}
