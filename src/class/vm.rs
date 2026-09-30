use crate::{
    binding::{class, exception, hash, io, object, symbol, variable, vm},
    helpers::scan_args::{KeywordArgs, ScanArgsFormat, ScannedArgs},
    rubysys::{exception::rb_eStandardError, rproc},
    types::{Argc, Id, Value, VmPointer},
};

use crate::{
    util, AnyException, AnyObject, Array, Class, Exception, GlobalVariable, Hash, NilClass, Object,
    Proc, RString, TryConvert,
};

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
    pub fn raise_ex<E>(exception: E)
    where
        E: Into<AnyException>,
    {
        vm::raise_ex(exception.into().value());
    }

    /// Evals string and returns an Result<AnyObject, AnyException>
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
        vm::eval_string_protect(string)
            .map(|v| AnyObject::from(v))
            .map_err(|_| {
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
        AnyObject::from(vm::eval_string(string))
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
    ///     rtself,
    ///
    ///     fn greet_rust_with() -> RString {
    ///         let greeting_template = VM::block_proc();
    ///         let name = RString::new_utf8("Rust").to_any_object();
    ///
    ///         greeting_template.call(&[name]).try_convert_to::<RString>().unwrap()
    ///     }
    /// );
    ///
    /// fn main() {
    ///     Class::new("Greeter", None).define(|klass| {
    ///         klass.def_self("greet_rust_with", greet_rust_with);
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
    ///     rtself,
    ///
    ///     fn calculate(a: Fixnum, b: Fixnum) -> Fixnum {
    ///         let a = a.unwrap();
    ///         let b = b.unwrap();
    ///
    ///         if VM::is_block_given() {
    ///             let arguments = [a.to_any_object(), b.to_any_object()];
    ///             let result = VM::block_proc().call(&arguments);
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
    ///     Class::new("Calculator", None).define(|klass| {
    ///         klass.def("calculate", calculate);
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
    ///     rtself,
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
    ///     Class::new("Calculator", None).define(|klass| {
    ///         klass.def("calculate", calculate);
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
    ///     rtself,
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
    ///     Class::new("Calculator", None).define(|klass| {
    ///         klass.def("calculate", calculate);
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

    /// Yields several values to the block of the current method call, as
    /// separate block arguments (`rb_yield_values2`), like Ruby's
    /// `yield a, b`.
    ///
    /// Raises `LocalJumpError` when there is no block.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Fixnum, Object, VM};
    ///
    /// class!(Pairs);
    ///
    /// methods!(
    ///     Pairs,
    ///     rtself,
    ///
    ///     fn pairs_each_pair() -> Fixnum {
    ///         VM::need_block();
    ///
    ///         VM::yield_values(&[Fixnum::new(1).into(), Fixnum::new(2).into()]);
    ///
    ///         Fixnum::new(0)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::new("Pairs", None).define(|klass| {
    ///         klass.def("each_pair", pairs_each_pair);
    ///     });
    ///
    ///     let sum = VM::eval("s = 0; Pairs.new.each_pair { |a, b| s = a + b }; s").unwrap();
    ///
    ///     assert_eq!(sum.try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
    ///     assert!(VM::eval("Pairs.new.each_pair").is_err());
    /// }
    /// ```
    pub fn yield_values(values: &[AnyObject]) -> AnyObject {
        let values = util::arguments_to_values(values);

        AnyObject::from(vm::yield_values(&values))
    }

    /// Raises `LocalJumpError` ("no block given") unless the current method
    /// was called with a block (`rb_need_block`).
    ///
    /// See [`VM::yield_values`](#method.yield_values) for an example.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, VM};
    /// # VM::init();
    ///
    /// let result = VM::protect(|| {
    ///     VM::need_block();
    ///     rutie::NilClass::new().into()
    /// });
    ///
    /// assert!(result.is_err());
    /// assert!(Class::from_existing("LocalJumpError").case_equals(&VM::error_pop().unwrap()));
    /// ```
    pub fn need_block() {
        vm::need_block()
    }

    /// Run a `closure` and protect from panic during raised exceptions
    /// by returning `Err<i32>`.
    ///
    /// # Examples
    ///
    /// ```text
    /// fn protect_send(&self, method: &str, arguments: &[AnyObject]) -> Result<AnyObject, AnyException> {
    ///     let closure = || self.send(&method, arguments.as_ref());
    ///
    ///     let result = VM::protect(closure);
    ///
    ///     result.map_err(|_| {
    ///         let output = VM::error_info().unwrap();
    ///
    ///         // error cleanup
    ///         VM::clear_error_info();
    ///
    ///         output
    ///     })
    /// }
    /// ```
    pub fn protect<F>(func: F) -> Result<AnyObject, i32>
    where
        F: FnMut() -> AnyObject,
    {
        vm::protect(func)
    }

    /// Get current VM error info.
    ///
    /// # Examples
    ///
    /// ```text
    /// fn protect_send(&self, method: &str, arguments: &[AnyObject]) -> Result<AnyObject, AnyException> {
    ///     let closure = || self.send(&method, arguments.as_ref()).into();
    ///
    ///     let result = VM::protect(closure);
    ///
    ///     result.map_err(|_| {
    ///         let output = VM::error_info().unwrap();
    ///
    ///         // error cleanup
    ///         VM::clear_error_info();
    ///
    ///         output
    ///     })
    /// }
    /// ```
    pub fn error_info() -> Result<AnyException, NilClass> {
        AnyException::try_convert(AnyObject::from(vm::errinfo()))
    }

    /// Get current VM error info and reset it.  If no error exists
    /// then `Err(NilClass::new())` is returned.
    ///
    /// ```
    /// use rutie::{VM, Exception, AnyException, Object};
    /// # VM::init();
    ///
    /// let closure = || unsafe { VM::eval_str("raise 'hello world!'").into() };
    /// let result = VM::protect(closure);
    ///
    /// let exception = VM::error_pop().expect("nil should not have occurred here!");
    /// assert_eq!("hello world!", exception.message());
    /// ```
    pub fn error_pop() -> Result<AnyException, NilClass> {
        VM::error_info().map(|exc| {
            VM::clear_error_info();
            exc
        })
    }

    /// Clear current VM error info.
    ///
    /// # Examples
    ///
    /// ```text
    /// fn protect_send(&self, method: &str, arguments: &[AnyObject]) -> Result<AnyObject, AnyException> {
    ///     let closure = || self.send(&method, arguments.as_ref()).into();
    ///
    ///     let result = VM::protect(closure);
    ///
    ///     result.map_err(|_| {
    ///         let output = VM::error_info().unwrap();
    ///
    ///         // error cleanup
    ///         VM::clear_error_info();
    ///
    ///         output
    ///     })
    /// }
    /// ```
    pub fn clear_error_info() {
        vm::set_errinfo(NilClass::new().value());
    }

    /// Exit with Ruby VM with status code.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate rutie;
    /// use rutie::VM;
    /// # VM::init();
    ///
    /// VM::exit(0)
    /// ```
    pub fn exit(status: i32) {
        vm::exit(status)
    }

    /// Exits the process immediately. No exit handlers are
    /// run. `status` is returned to the underlying system as the
    /// exit status.
    ///
    /// ```text
    /// call-seq:
    ///   Process.exit!(status=false)
    /// ```
    ///
    /// > Note: Because the VM is exiting — having a return object is not a viable option and therefore you
    /// >       must account for any exceptions that may arise yourself.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate rutie;
    /// use rutie::{VM,Boolean};
    /// # VM::init();
    ///
    /// unsafe { VM::exit_bang(&[Boolean::new(true).into()]) }
    /// ```
    ///
    /// ```ruby
    /// Process.exit!(true)
    /// ```
    ///
    /// Since invalid arguments can raise an exception this is marked as unsafe.  Simply use `VM::protect`
    /// and `VM::error_pop` to handle potential exceptions.
    ///
    /// ```
    /// extern crate rutie;
    /// use rutie::{VM, Symbol, NilClass, Object, AnyException, Exception};
    /// # VM::init();
    ///
    /// VM::protect(|| {
    ///     unsafe { VM::exit_bang(&[Symbol::new("asdf").into()]) };
    ///
    ///     NilClass::new().into()
    /// });
    ///
    /// let error = VM::error_pop();
    /// assert_eq!(error.unwrap().inspect(), "#<TypeError: no implicit conversion of Symbol into Integer>");
    /// ```
    pub unsafe fn exit_bang(arguments: &[AnyObject]) {
        crate::Module::process().send("exit!", arguments.as_ref());
    }

    /// Terminate execution immediately, effectively by calling
    /// `Kernel.exit(false)`. If _msg_ is given, it is written
    /// to STDERR prior to terminating.
    ///
    /// ```text
    /// call-seq:
    ///     abort
    ///     Kernel::abort([msg])
    ///     Process.abort([msg])
    /// ```
    ///
    /// > Note: Because the VM is aborting — having a return object is not a viable option and therefore you
    /// >       must account for any exceptions that may arise yourself.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate rutie;
    /// use rutie::{VM, NilClass, AnyException, Exception, RString};
    /// # VM::init();
    ///
    /// VM::protect(|| {
    ///     unsafe { VM::abort(&[RString::new_utf8("Goodbye cruel world!").into()]) }
    ///
    ///     NilClass::new().into()
    /// });
    ///
    /// let error = VM::error_pop();
    /// assert_eq!(error.unwrap().inspect(), "#<SystemExit: Goodbye cruel world!>");
    /// ```
    ///
    /// ```ruby
    /// abort "Goodbye cruel world!"
    /// ```
    ///
    /// Since invalid arguments can raise an exception this is marked as unsafe.  Simply use `VM::protect`
    /// and `VM::error_pop` to handle potential exceptions.
    ///
    /// ```
    /// extern crate rutie;
    /// use rutie::{VM, Symbol, NilClass, Object, AnyException, Exception};
    /// # VM::init();
    ///
    /// VM::protect(|| {
    ///     unsafe { VM::abort(&[Symbol::new("asdf").into()]) };
    ///
    ///     NilClass::new().into()
    /// });
    ///
    /// let error = VM::error_pop();
    /// assert_eq!(error.unwrap().inspect(), "#<TypeError: no implicit conversion of Symbol into String>");
    /// ```
    pub unsafe fn abort(arguments: &[AnyObject]) {
        let arguments = util::arguments_to_values(arguments);

        vm::abort(&arguments)
    }

    /// Specifies the handling of signals. The first parameter is a signal name (a string such as “SIGALRM”,
    /// “SIGUSR1”, and so on) or a signal number. The characters “SIG” may be omitted from the signal name.
    /// The command or block specifies code to be run when the signal is raised. If the command is the
    /// string “IGNORE” or “SIG_IGN”, the signal will be ignored. If the command is “DEFAULT” or “SIG_DFL”,
    /// the Ruby’s default handler will be invoked. If the command is “EXIT”, the script will be terminated
    /// by the signal. If the command is “SYSTEM_DEFAULT”, the operating system’s default handler will be
    /// invoked. Otherwise, the given command or block will be run. The special signal name “EXIT” or signal
    /// number zero will be invoked just prior to program termination. trap returns the previous handler for
    /// the given signal.
    ///
    /// ```ruby
    /// Signal.trap(0, proc { puts "Terminating: #{$$}" })
    /// Signal.trap("CLD")  { puts "Child died" }
    /// fork && Process.wait
    /// ```
    ///
    /// produces:
    ///
    /// ```text
    /// Terminating: 27461
    /// Child died
    /// Terminating: 27460
    /// ```
    pub fn trap(arguments: &[AnyObject]) -> Result<AnyObject, AnyException> {
        Class::from_existing("Signal").protect_send("trap", arguments)
    }

    /// Registers `func` to be called when the Ruby VM runs its `at_exit` handlers.
    ///
    /// This is Ruby's `Kernel#at_exit` for Rust closures (`rb_set_end_proc`).
    /// Handlers run in reverse order of registration, together with the ones
    /// registered from Ruby, when the interpreter shuts down: at the end of a
    /// `ruby` process that loaded your extension, or when an embedding
    /// program calls [`VM::cleanup`](#method.cleanup).
    ///
    /// The closure is kept alive until it runs, so it must be `'static`. A
    /// panic inside it does not cross into Ruby; it is reported as a
    /// `RuntimeError` in the `at_exit` handler, like an exception raised from
    /// a Ruby `at_exit` block.
    ///
    /// The `VmPointer` argument is always null; it is kept for compatibility.
    ///
    /// Before 0.11 this method called `func` immediately; that behaviour is
    /// still available as [`VM::call_protected`](#method.call_protected).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    ///
    /// static CALLS: AtomicUsize = AtomicUsize::new(0);
    ///
    /// # VM::init();
    /// VM::at_exit(|_vm| {
    ///     // Handlers run last-in, first-out: this one runs second.
    ///     assert_eq!(CALLS.fetch_add(1, Ordering::SeqCst), 1);
    /// });
    /// VM::at_exit(|_vm| {
    ///     assert_eq!(CALLS.fetch_add(1, Ordering::SeqCst), 0);
    /// });
    ///
    /// // Nothing runs until the VM shuts down.
    /// assert_eq!(CALLS.load(Ordering::SeqCst), 0);
    ///
    /// unsafe { VM::cleanup() };
    ///
    /// assert_eq!(CALLS.load(Ordering::SeqCst), 2);
    /// ```
    ///
    /// A panicking handler does not stop the others from running:
    ///
    /// ```
    /// use rutie::VM;
    /// use std::sync::atomic::{AtomicBool, Ordering};
    ///
    /// static RAN: AtomicBool = AtomicBool::new(false);
    ///
    /// # VM::init();
    /// VM::at_exit(|_vm| RAN.store(true, Ordering::SeqCst));
    /// VM::at_exit(|_vm| panic!("handler failed"));
    ///
    /// unsafe { VM::cleanup() };
    ///
    /// assert!(RAN.load(Ordering::SeqCst));
    /// ```
    pub fn at_exit<F>(func: F)
    where
        F: FnOnce(VmPointer) + 'static,
    {
        vm::at_exit(func)
    }

    /// Calls `func` immediately inside `rb_protect`, ignoring any exception
    /// it raises (the error info is left set).
    ///
    /// This is what `VM::at_exit` did before 0.11. To run code when the VM
    /// shuts down use [`VM::at_exit`](#method.at_exit); to handle exceptions
    /// use [`VM::protect`](#method.protect).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    /// # VM::init();
    ///
    /// let mut calls = 0;
    ///
    /// VM::call_protected(|_vm| calls += 1);
    ///
    /// assert_eq!(calls, 1);
    /// ```
    pub fn call_protected<F>(func: F)
    where
        F: FnMut(VmPointer),
    {
        vm::call_protected(func)
    }

    /// Shuts down the Ruby VM (`ruby_cleanup`) and returns the exit status
    /// Ruby would have exited with.
    ///
    /// This runs the `at_exit` handlers (both [`VM::at_exit`](#method.at_exit)
    /// ones and Ruby's), terminates the other Ruby threads, runs finalizers
    /// and frees the VM. Call it once, at the end of a program that started
    /// the VM with [`VM::init`](#method.init). Libraries loaded into a
    /// running Ruby must not call it.
    ///
    /// # Safety
    ///
    /// After this returns no Ruby object or Rutie API may be used again, and
    /// `VM::init` must not be called again in the same process.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    ///
    /// VM::init();
    ///
    /// // ... use Ruby ...
    ///
    /// let status = unsafe { VM::cleanup() };
    ///
    /// assert_eq!(status, 0);
    /// ```
    pub unsafe fn cleanup() -> i32 {
        vm::cleanup(0)
    }

    /// Calls `body`, then always calls `ensure`, even when `body` raises a
    /// Ruby exception (Ruby's `begin`/`ensure`, `rb_ensure`).
    ///
    /// Returns the result of `body`. When `body` raises, `ensure` runs and
    /// the exception keeps propagating, so this raises too; wrap the call in
    /// [`VM::protect`](#method.protect) or [`VM::rescue`](#method.rescue) to
    /// handle it. A panic in either closure is raised as a `RuntimeError`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, VM};
    /// use std::cell::Cell;
    /// # VM::init();
    ///
    /// let ensured = Cell::new(false);
    ///
    /// let result = VM::ensure(|| Fixnum::new(1).into(), || ensured.set(true));
    ///
    /// assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// assert!(ensured.get());
    ///
    /// // The ensure closure also runs when the body raises.
    /// ensured.set(false);
    ///
    /// let result = VM::protect(|| {
    ///     VM::ensure(|| unsafe { VM::eval_str("raise 'oops'") }, || ensured.set(true))
    /// });
    ///
    /// assert!(result.is_err());
    /// assert!(ensured.get());
    /// assert_eq!(VM::error_pop().unwrap().to_string(), "#<RuntimeError: oops>");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// begin
    ///   raise 'oops'
    /// ensure
    ///   ensured = true
    /// end
    /// ```
    pub fn ensure<B, E>(body: B, ensure: E) -> AnyObject
    where
        B: FnOnce() -> AnyObject,
        E: FnOnce(),
    {
        AnyObject::from(vm::ensure(|| body().value(), ensure))
    }

    /// Calls `body`; if it raises a `StandardError`, calls `handler` with the
    /// exception and returns its result instead (Ruby's `begin`/`rescue`,
    /// `rb_rescue`).
    ///
    /// Exceptions that are not a `StandardError` (such as `SystemExit` or
    /// `Interrupt`) keep propagating. To rescue other classes use
    /// [`VM::rescue_from`](#method.rescue_from). A panic in `body` is raised
    /// as a `RuntimeError`, so it is rescued too.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Exception, Object, RString, VM};
    /// # VM::init();
    ///
    /// let result = VM::rescue(
    ///     || unsafe { VM::eval_str("raise ArgumentError, 'bad'") },
    ///     |exception| RString::new_utf8(&exception.message()).into(),
    /// );
    ///
    /// assert_eq!(result.try_convert_to::<RString>().unwrap().to_str(), "bad");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// begin
    ///   raise ArgumentError, 'bad'
    /// rescue => exception
    ///   exception.message
    /// end
    /// ```
    pub fn rescue<B, R>(body: B, handler: R) -> AnyObject
    where
        B: FnOnce() -> AnyObject,
        R: FnOnce(AnyException) -> AnyObject,
    {
        let standard_error = unsafe { rb_eStandardError };

        VM::rescue_values(body, handler, &[standard_error])
    }

    /// Calls `body`; if it raises an exception that is kind of one of
    /// `classes`, calls `handler` with the exception and returns its result
    /// instead (Ruby's `rescue ClassA, ClassB => e`, `rb_rescue2`).
    ///
    /// Any other exception keeps propagating.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let classes = [Class::from_existing("KeyError"), Class::from_existing("IndexError")];
    ///
    /// let result = VM::rescue_from(
    ///     &classes,
    ///     || unsafe { VM::eval_str("{}.fetch(:missing)") },
    ///     |_exception| Fixnum::new(0).into(),
    /// );
    ///
    /// assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(0)));
    ///
    /// // A `TypeError` is not in `classes`, so it propagates.
    /// let result = VM::protect(|| {
    ///     VM::rescue_from(
    ///         &classes,
    ///         || unsafe { VM::eval_str("Integer([])") },
    ///         |_exception| Fixnum::new(0).into(),
    ///     )
    /// });
    ///
    /// assert!(result.is_err());
    /// assert!(Class::from_existing("TypeError").case_equals(&VM::error_pop().unwrap()));
    /// ```
    pub fn rescue_from<B, R>(classes: &[Class], body: B, handler: R) -> AnyObject
    where
        B: FnOnce() -> AnyObject,
        R: FnOnce(AnyException) -> AnyObject,
    {
        let classes: Vec<Value> = classes.iter().map(Object::value).collect();

        VM::rescue_values(body, handler, &classes)
    }

    fn rescue_values<B, R>(body: B, handler: R, classes: &[Value]) -> AnyObject
    where
        B: FnOnce() -> AnyObject,
        R: FnOnce(AnyException) -> AnyObject,
    {
        let result = vm::rescue(
            || body().value(),
            |exception| handler(AnyException::from(exception)).value(),
            classes,
        );

        AnyObject::from(result)
    }

    /// Calls `body` with `tag`; a [`VM::throw`](#method.throw) of the same
    /// tag from inside `body` (or anything it calls) returns the thrown
    /// value from `catch` (Ruby's `catch`/`throw`, `rb_catch_obj`).
    ///
    /// Returns the result of `body` when nothing is thrown.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let result = VM::catch(Symbol::new("done"), |tag| {
    ///     VM::throw(tag, Fixnum::new(42));
    /// });
    ///
    /// assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));
    ///
    /// let result = VM::catch(Symbol::new("done"), |_tag| Fixnum::new(1).into());
    ///
    /// assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// catch(:done) do |tag|
    ///   throw tag, 42
    /// end
    /// ```
    pub fn catch<T, F>(tag: T, body: F) -> AnyObject
    where
        T: Object,
        F: FnOnce(AnyObject) -> AnyObject,
    {
        let result = vm::catch(tag.value(), |tag| body(AnyObject::from(tag)).value());

        AnyObject::from(result)
    }

    /// Transfers control to the end of the active [`VM::catch`](#method.catch)
    /// block waiting for `tag`, which returns `value` (Ruby's `throw`,
    /// `rb_throw_obj`).
    ///
    /// Raises `UncaughtThrowError` (an `ArgumentError`) when there is no
    /// matching `catch`. Rust values alive in the frames being unwound are
    /// not dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, NilClass, Object, RString, Symbol, VM};
    /// # VM::init();
    ///
    /// let result = VM::catch(Symbol::new("found"), |_tag| {
    ///     for name in &["a", "b", "c"] {
    ///         if *name == "b" {
    ///             VM::throw(Symbol::new("found"), RString::new_utf8(name));
    ///         }
    ///     }
    ///
    ///     NilClass::new().into()
    /// });
    ///
    /// assert_eq!(result.try_convert_to::<RString>().unwrap().to_str(), "b");
    ///
    /// // Without a matching `catch`:
    /// let result = VM::protect(|| VM::throw(Symbol::new("nowhere"), NilClass::new()));
    ///
    /// assert!(result.is_err());
    /// assert!(Class::from_existing("ArgumentError").case_equals(&VM::error_pop().unwrap()));
    /// ```
    pub fn throw<T, V>(tag: T, value: V) -> !
    where
        T: Object,
        V: Object,
    {
        vm::throw(tag.value(), value.value())
    }

    /// Breaks out of the method that yielded to the current block, making it
    /// return `nil` (Ruby's `break` inside a block, `rb_iter_break`).
    ///
    /// # Safety
    ///
    /// Must only be called from inside a block, such as the closure given to
    /// [`Object::send_with_block`](trait.Object.html#method.send_with_block);
    /// anywhere else Ruby may abort the process. Rust values alive in the
    /// frames being unwound are not dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, NilClass, Object, VM};
    /// # VM::init();
    ///
    /// let array: Array = (1..=5).map(|i| Fixnum::new(i).to_any_object()).collect();
    /// let mut seen = 0;
    ///
    /// let result = unsafe {
    ///     array.send_with_block("each", &[], |_values| {
    ///         seen += 1;
    ///
    ///         if seen == 2 {
    ///             VM::iter_break();
    ///         }
    ///
    ///         NilClass::new().into()
    ///     })
    /// };
    ///
    /// assert!(result.is_nil());
    /// assert_eq!(seen, 2);
    /// ```
    pub unsafe fn iter_break() -> ! {
        vm::iter_break()
    }

    /// Breaks out of the method that yielded to the current block, making it
    /// return `value` (Ruby's `break value` inside a block,
    /// `rb_iter_break_value`).
    ///
    /// # Safety
    ///
    /// The same as [`VM::iter_break`](#method.iter_break).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, NilClass, Object, VM};
    /// # VM::init();
    ///
    /// let array: Array = (1..=5).map(|i| Fixnum::new(i).to_any_object()).collect();
    ///
    /// // Like `[1, 2, 3, 4, 5].each { |x| break x * 10 if x == 3 }`
    /// let result = unsafe {
    ///     array.send_with_block("each", &[], |values| {
    ///         let x = values[0].try_convert_to::<Fixnum>().unwrap().to_i64();
    ///
    ///         if x == 3 {
    ///             VM::iter_break_value(Fixnum::new(x * 10));
    ///         }
    ///
    ///         NilClass::new().into()
    ///     })
    /// };
    ///
    /// assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(30)));
    /// ```
    pub unsafe fn iter_break_value<T: Object>(value: T) -> ! {
        vm::iter_break_value(value.value())
    }

    /// Resumes a non-local jump (usually a raised exception) that
    /// [`VM::protect`](#method.protect) stopped, given the state it returned
    /// (`rb_jump_tag`).
    ///
    /// # Safety
    ///
    /// `state` must be a non-zero state returned by `VM::protect`, and for a
    /// raised exception the error info must not have been cleared (for
    /// example by `VM::error_pop`). Rust values alive in the frames being
    /// unwound are not dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Exception, NilClass, Object, VM};
    /// # VM::init();
    ///
    /// let outer = VM::protect(|| {
    ///     let inner = VM::protect(|| unsafe { VM::eval_str("raise 'again'") });
    ///
    ///     if let Err(state) = inner {
    ///         // Clean up, then let the exception continue.
    ///         unsafe { VM::jump_tag(state) };
    ///     }
    ///
    ///     NilClass::new().into()
    /// });
    ///
    /// assert!(outer.is_err());
    /// assert_eq!(VM::error_pop().unwrap().message(), "again");
    /// ```
    pub unsafe fn jump_tag(state: i32) -> ! {
        vm::jump_tag(state)
    }

    /// Returns `true` when the Ruby method currently running was called with
    /// keyword arguments (`rb_keyword_given_p`).
    ///
    /// Only available on Ruby 2.7, where it is needed to tell keywords from
    /// a trailing positional `Hash`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    /// # VM::init();
    ///
    /// // Not inside a method called with keywords.
    /// # #[cfg(ruby_gte_2_7)]
    /// assert!(!VM::is_keyword_given());
    /// ```
    #[cfg(ruby_gte_2_7)]
    pub fn is_keyword_given() -> bool {
        vm::is_keyword_given()
    }

    /// Emits a Ruby warning (`rb_warn`), printed to `$stderr` as
    /// `warning: message` unless warnings are disabled (`$VERBOSE` is `nil`).
    ///
    /// `message` is never interpreted as a format string. It is cut at its
    /// first NUL byte, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// // Collect warnings instead of printing them.
    /// VM::eval("$warnings = []; def Warning.warn(message); $warnings << message; end").unwrap();
    ///
    /// VM::warn("100% deprecated");
    ///
    /// let warnings = VM::eval("$warnings.join").unwrap().try_convert_to::<RString>().unwrap();
    ///
    /// assert!(warnings.to_str().ends_with("warning: 100% deprecated\n"));
    /// ```
    pub fn warn(message: &str) {
        exception::warn(message)
    }

    /// Emits a Ruby warning only in verbose mode (`$VERBOSE` is `true`,
    /// `rb_warning`), printed to `$stderr` as `warning: message`.
    ///
    /// `message` is never interpreted as a format string. It is cut at its
    /// first NUL byte, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// VM::eval("$warnings = []; def Warning.warn(message); $warnings << message; end").unwrap();
    ///
    /// VM::eval("$VERBOSE = false").unwrap();
    /// VM::warning("quiet");
    ///
    /// VM::eval("$VERBOSE = true").unwrap();
    /// VM::warning("loud");
    ///
    /// let warnings = VM::eval("$warnings.join").unwrap().try_convert_to::<RString>().unwrap();
    ///
    /// assert!(!warnings.to_str().contains("quiet"));
    /// assert!(warnings.to_str().ends_with("warning: loud\n"));
    /// ```
    pub fn warning(message: &str) {
        exception::warning(message)
    }

    /// Checks that `argc` arguments fit a method taking `min` to `max`
    /// arguments (`max` of `-1` means no upper limit), like `rb_check_arity`.
    ///
    /// Returns `argc`, or an `ArgumentError` with Ruby's usual message
    /// without raising it. To raise it, see
    /// [`VM::raise_arity_error`](#method.raise_arity_error).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Exception, VM};
    /// # VM::init();
    ///
    /// assert_eq!(VM::check_arity(2, 1, 3), Ok(2));
    /// assert_eq!(VM::check_arity(5, 1, -1), Ok(5));
    ///
    /// let error = VM::check_arity(0, 1, 2).unwrap_err();
    ///
    /// assert_eq!(error.message(), "wrong number of arguments (given 0, expected 1..2)");
    /// ```
    pub fn check_arity(argc: i32, min: i32, max: i32) -> Result<i32, AnyException> {
        rproc::check_arity(argc, min, max)
    }

    /// Raises an `ArgumentError` for a call with `argc` arguments to a
    /// method taking `min` to `max` arguments (`max` of `-1` means no upper
    /// limit), with Ruby's usual message (`rb_error_arity`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Exception, VM};
    /// # VM::init();
    ///
    /// let result = VM::protect(|| VM::raise_arity_error(3, 1, 2));
    ///
    /// assert!(result.is_err());
    /// assert_eq!(
    ///     VM::error_pop().unwrap().message(),
    ///     "wrong number of arguments (given 3, expected 1..2)"
    /// );
    /// ```
    pub fn raise_arity_error(argc: i32, min: i32, max: i32) -> ! {
        exception::error_arity(argc, min, max)
    }

    /// Raises `Interrupt`, as if the process got `SIGINT` (`rb_interrupt`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, VM};
    /// # VM::init();
    ///
    /// let result = VM::protect(|| VM::raise_interrupt());
    ///
    /// assert!(result.is_err());
    /// assert!(Class::interrupt().case_equals(&VM::error_pop().unwrap()));
    /// ```
    pub fn raise_interrupt() -> ! {
        exception::interrupt()
    }

    /// Raises `ZeroDivisionError` with Ruby's usual message (`rb_num_zerodiv`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Exception, Object, VM};
    /// # VM::init();
    ///
    /// let result = VM::protect(|| VM::raise_zero_division());
    ///
    /// assert!(result.is_err());
    ///
    /// let error = VM::error_pop().unwrap();
    ///
    /// assert!(Class::from_existing("ZeroDivisionError").case_equals(&error));
    /// assert_eq!(error.message(), "divided by 0");
    /// ```
    pub fn raise_zero_division() -> ! {
        exception::num_zerodiv()
    }

    /// Raises `NotImplementedError` for the Ruby method currently running
    /// (`rb_notimplement`), for features the platform does not support.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, NilClass, Object, VM};
    ///
    /// class!(Forker);
    ///
    /// methods!(
    ///     Forker,
    ///     rtself,
    ///
    ///     fn forker_fork() -> NilClass {
    ///         VM::not_implemented()
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::new("Forker", None).define(|klass| {
    ///         klass.def("fork", forker_fork);
    ///     });
    ///
    ///     let error = VM::eval("Forker.new.fork").unwrap_err();
    ///
    ///     assert!(Class::from_existing("NotImplementedError").case_equals(&error));
    /// }
    /// ```
    pub fn not_implemented() -> ! {
        exception::not_implemented()
    }

    /// Raises the `SystemCallError` subclass (`Errno::*`) for the current
    /// OS error (`errno`), with `message` added to its message, like
    /// `rb_sys_fail`.
    ///
    /// Unlike `rb_sys_fail`, which aborts the process when `errno` is `0`,
    /// this raises a plain `SystemCallError` in that case.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Exception, Object, VM};
    /// use std::fs::File;
    /// # VM::init();
    ///
    /// let result = VM::protect(|| {
    ///     match File::open("/this/path/does/not/exist") {
    ///         Ok(_) => unreachable!(),
    ///         Err(_) => VM::sys_fail("/this/path/does/not/exist"),
    ///     }
    /// });
    ///
    /// assert!(result.is_err());
    ///
    /// let error = VM::error_pop().unwrap();
    ///
    /// assert!(Class::from_existing("Errno").get_nested_class("ENOENT").case_equals(&error));
    /// assert!(error.message().contains("/this/path/does/not/exist"));
    /// ```
    pub fn sys_fail(message: &str) -> ! {
        let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);

        exception::syserr_fail(errno, message)
    }

    /// Splits the `arguments` of a Ruby method call according to an
    /// `rb_scan_args` format, checking the number of arguments.
    ///
    /// The format is `[required[optional]][*][post][:][&]`:
    ///
    ///  - `required` — number of leading mandatory arguments (one digit)
    ///  - `optional` — number of optional arguments (one digit, only after `required`)
    ///  - `*` — collect the remaining arguments in an `Array`
    ///  - `post` — number of trailing mandatory arguments (one digit)
    ///  - `:` — take keyword arguments as a `Hash`
    ///  - `&` — take the block as a `Proc`
    ///
    /// For example `"12"` is one required and two optional arguments, and
    /// `"1*:&"` is one required argument, a splat, keywords and a block.
    ///
    /// Keywords follow the running Ruby's rules for `rb_scan_args`: a
    /// trailing `Hash` is taken as keywords on Ruby 2.5 and 2.6, and Ruby 2.7
    /// applies its keyword-argument transition rules (and warnings).
    ///
    /// Returns an `ArgumentError`, without raising it, when the format is
    /// invalid or the arguments do not fit it.
    ///
    /// This is meant for methods that take a variable number of arguments,
    /// defined with a plain `extern` function (see the example) since
    /// `methods!` gives each argument its own parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{AnyObject, Class, Exception, Fixnum, Object, RString, VM};
    /// use rutie::types::Argc;
    ///
    /// // def greet(name, greeting = "Hello", *rest)
    /// pub extern "C" fn greet(argc: Argc, argv: *const AnyObject, _rtself: AnyObject) -> RString {
    ///     let arguments = rutie::util::parse_arguments(argc, argv);
    ///
    ///     let args = VM::scan_args(&arguments, "11*");
    ///
    ///     if let Err(ref error) = args {
    ///         VM::raise(error.class(), &error.message());
    ///     }
    ///
    ///     // We can safely unwrap here
    ///     let args = args.unwrap();
    ///
    ///     let name = args.required[0].try_convert_to::<RString>().unwrap();
    ///     let greeting = args.optional[0]
    ///         .as_ref()
    ///         .map(|greeting| greeting.try_convert_to::<RString>().unwrap().to_string())
    ///         .unwrap_or_else(|| "Hello".to_string());
    ///     let rest = args.splat.unwrap().length();
    ///
    ///     RString::new_utf8(&format!("{} {} (+{})", greeting, name.to_str(), rest))
    /// }
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def("greet", greet);
    ///     });
    ///
    ///     let greeting = |code| VM::eval(code).unwrap().try_convert_to::<RString>().unwrap().to_string();
    ///
    ///     assert_eq!(greeting("greet('Ruby')"), "Hello Ruby (+0)");
    ///     assert_eq!(greeting("greet('Ruby', 'Hi', 1, 2)"), "Hi Ruby (+2)");
    ///
    ///     let error = VM::eval("greet").unwrap_err();
    ///     assert_eq!(error.message(), "wrong number of arguments (given 0, expected 1+)");
    /// }
    /// ```
    ///
    /// Scanning keywords and a block:
    ///
    /// ```
    /// use rutie::{Fixnum, Hash, Object, Symbol, VM, Exception};
    /// # VM::init();
    ///
    /// let mut options = Hash::new();
    /// options.store(Symbol::new("verbose"), Fixnum::new(1));
    ///
    /// let arguments = [Fixnum::new(1).into(), options.into()];
    /// let args = VM::scan_args(&arguments, "1:&").unwrap();
    ///
    /// assert_eq!(args.required.len(), 1);
    /// assert_eq!(args.keywords.unwrap().at(&Symbol::new("verbose")), Fixnum::new(1).into());
    /// assert!(args.block.is_none());
    ///
    /// let error = VM::scan_args(&arguments, "bogus").unwrap_err();
    /// assert_eq!(error.message(), "bad scan arg format: bogus");
    /// ```
    pub fn scan_args(arguments: &[AnyObject], format: &str) -> Result<ScannedArgs, AnyException> {
        let spec = ScanArgsFormat::parse(format)
            .map_err(|message| AnyException::new("ArgumentError", Some(&message)))?;

        let arguments = util::arguments_to_values(arguments);
        let format = util::str_to_cstring(format);
        let mut out = [NilClass::new().value(); class::SCAN_ARGS_MAX_VARIABLES];
        let mut positional = 0;

        vm::protect_value(|| {
            positional = class::scan_args(&arguments, &format, &mut out);

            NilClass::new().value()
        })
        .map_err(AnyException::from)?;

        let mut values = out.iter().map(|value| AnyObject::from(*value));
        let mut take = |count: usize| -> Vec<AnyObject> { values.by_ref().take(count).collect() };

        let required = take(spec.required);
        let given_optional = (positional as usize)
            .saturating_sub(spec.required + spec.post)
            .min(spec.optional);
        let optional = take(spec.optional)
            .into_iter()
            .enumerate()
            .map(|(i, value)| {
                if i < given_optional {
                    Some(value)
                } else {
                    None
                }
            })
            .collect();
        let splat = take(spec.splat as usize)
            .pop()
            .map(|value| Array::from(value.value()));
        let post = take(spec.post);
        let keywords = take(spec.keywords as usize)
            .pop()
            .filter(|value| !value.is_nil())
            .map(|value| Hash::from(value.value()));
        let block = take(spec.block as usize)
            .pop()
            .filter(|value| !value.is_nil())
            .map(|value| Proc::from(value.value()));

        Ok(ScannedArgs {
            required,
            optional,
            splat,
            post,
            keywords,
            block,
        })
    }

    /// Looks up keyword arguments by name in `keywords` (the `Hash` from
    /// [`VM::scan_args`](#method.scan_args), or `None` when no keywords were
    /// passed), like `rb_get_kwargs`.
    ///
    /// `keywords` is not modified. When `allow_extra` is `true`, keywords
    /// that are neither `required` nor `optional` are returned in `rest`;
    /// otherwise they are an error.
    ///
    /// Returns an `ArgumentError`, without raising it, when a required
    /// keyword is missing or an unknown keyword is given.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Exception, Fixnum, Hash, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let mut keywords = Hash::new();
    /// keywords.store(Symbol::new("host"), Fixnum::new(1));
    /// keywords.store(Symbol::new("extra"), Fixnum::new(2));
    ///
    /// let kwargs = VM::get_kwargs(Some(&keywords), &["host"], &["port"], true).unwrap();
    ///
    /// assert_eq!(kwargs.required, vec![Fixnum::new(1).into()]);
    /// assert_eq!(kwargs.optional, vec![None]);
    /// assert_eq!(kwargs.rest.unwrap().length(), 1);
    ///
    /// // Unknown keywords are an error unless `allow_extra` is set.
    /// let error = VM::get_kwargs(Some(&keywords), &["host"], &["port"], false).unwrap_err();
    /// assert!(error.message().starts_with("unknown keyword"));
    ///
    /// let error = VM::get_kwargs(None, &["host"], &[], false).unwrap_err();
    /// assert!(error.message().starts_with("missing keyword"));
    ///
    /// // The original hash is left alone.
    /// assert_eq!(keywords.length(), 2);
    /// ```
    pub fn get_kwargs(
        keywords: Option<&Hash>,
        required: &[&str],
        optional: &[&str],
        allow_extra: bool,
    ) -> Result<KeywordArgs, AnyException> {
        let hash = match keywords {
            Some(keywords) => Hash::from(hash::dup(keywords.value())),
            None => Hash::new(),
        };
        let table: Vec<Id> = required
            .iter()
            .chain(optional.iter())
            .map(|name| symbol::internal_id(name))
            .collect();
        let mut values = vec![NilClass::new().value(); table.len()];

        vm::protect_value(|| {
            class::get_kwargs(
                hash.value(),
                &table,
                required.len(),
                allow_extra,
                &mut values,
            );

            NilClass::new().value()
        })
        .map_err(AnyException::from)?;

        let optional_values = values
            .split_off(required.len())
            .into_iter()
            .map(|value| {
                if value.is_undef() {
                    None
                } else {
                    Some(AnyObject::from(value))
                }
            })
            .collect();

        Ok(KeywordArgs {
            required: values.into_iter().map(AnyObject::from).collect(),
            optional: optional_values,
            rest: if allow_extra { Some(hash) } else { None },
        })
    }

    /// Returns the value of the global variable `name`, such as `"$stdout"`
    /// (`rb_gv_get`). An undefined global is `nil`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("$rutie_answer = 42").unwrap();
    ///
    /// assert_eq!(VM::global_get("$rutie_answer").try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));
    /// assert!(VM::global_get("$rutie_undefined").is_nil());
    /// ```
    pub fn global_get(name: &str) -> AnyObject {
        AnyObject::from(variable::global_get(name))
    }

    /// Sets the global variable `name`, such as `"$verbose_mode"`, to `value`
    /// and returns it (`rb_gv_set`).
    ///
    /// Raises `NameError` for a read-only global such as `"$$"`; see
    /// [`VM::protect_global_set`](#method.protect_global_set).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// VM::global_set("$rutie_answer", Fixnum::new(42));
    ///
    /// let answer = VM::eval("$rutie_answer").unwrap();
    ///
    /// assert_eq!(answer.try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));
    /// ```
    pub fn global_set<T: Object>(name: &str, value: T) -> AnyObject {
        AnyObject::from(variable::global_set(name, value.value()))
    }

    /// Like [`VM::global_set`](#method.global_set), but returns the
    /// exception (such as the `NameError` for a read-only global) instead of
    /// raising it.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// assert!(VM::protect_global_set("$rutie_answer", Fixnum::new(42)).is_ok());
    /// assert!(VM::protect_global_set("$$", Fixnum::new(1)).is_err());
    /// ```
    pub fn protect_global_set<T: Object>(name: &str, value: T) -> Result<AnyObject, AnyException> {
        let value = value.value();

        vm::protect_value(|| variable::global_set(name, value))
            .map(AnyObject::from)
            .map_err(AnyException::from)
    }

    /// Defines the global variable `name` (such as `"$counter"`) stored in
    /// memory owned by Rust, starting as `initial` (`rb_define_variable`).
    ///
    /// The returned [`GlobalVariable`](struct.GlobalVariable.html) reads and
    /// writes the value directly. Ruby code can read and assign it as usual.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let counter = VM::define_variable("$rutie_counter", Fixnum::new(0));
    ///
    /// VM::eval("$rutie_counter += 5").unwrap();
    ///
    /// assert_eq!(counter.get().try_convert_to::<Fixnum>(), Ok(Fixnum::new(5)));
    /// ```
    pub fn define_variable<T: Object>(name: &str, initial: T) -> GlobalVariable {
        GlobalVariable::new(variable::define_variable(name, initial.value(), false))
    }

    /// Like [`VM::define_variable`](#method.define_variable), but Ruby code
    /// cannot assign it (`rb_define_readonly_variable`); only Rust can,
    /// through the returned [`GlobalVariable`](struct.GlobalVariable.html).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let version = VM::define_readonly_variable("$rutie_version", Fixnum::new(1));
    ///
    /// assert!(VM::eval("$rutie_version = 2").is_err());
    ///
    /// version.set(Fixnum::new(2));
    ///
    /// let value = VM::eval("$rutie_version").unwrap();
    ///
    /// assert_eq!(value.try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// ```
    pub fn define_readonly_variable<T: Object>(name: &str, initial: T) -> GlobalVariable {
        GlobalVariable::new(variable::define_variable(name, initial.value(), true))
    }

    /// Defines the global variable `name` (such as `"$now"`) whose value is
    /// computed by `getter` each time it is read, and whose assignment calls
    /// `setter` (`rb_define_hooked_variable`). Without a setter the variable
    /// is read-only and assigning it raises `NameError`.
    ///
    /// The closures are kept for the life of the process. A panic in either
    /// is raised as a Ruby `RuntimeError`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{AnyObject, Fixnum, Object, VM};
    /// use std::{cell::Cell, rc::Rc};
    /// # VM::init();
    ///
    /// let stored = Rc::new(Cell::new(10));
    /// let (read, write) = (stored.clone(), stored.clone());
    ///
    /// VM::define_virtual_variable(
    ///     "$rutie_virtual",
    ///     move || Fixnum::new(read.get()).into(),
    ///     Some(move |value: AnyObject| {
    ///         write.set(value.try_convert_to::<Fixnum>().unwrap().to_i64() * 2);
    ///     }),
    /// );
    ///
    /// VM::eval("$rutie_virtual = 21").unwrap();
    ///
    /// assert_eq!(stored.get(), 42);
    ///
    /// let value = VM::eval("$rutie_virtual").unwrap();
    ///
    /// assert_eq!(value.try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));
    ///
    /// // Read-only without a setter:
    /// VM::define_virtual_variable("$rutie_constant", || Fixnum::new(1).into(), None::<fn(AnyObject)>);
    ///
    /// assert!(VM::eval("$rutie_constant = 2").is_err());
    /// ```
    pub fn define_virtual_variable<G, S>(name: &str, mut getter: G, setter: Option<S>)
    where
        G: FnMut() -> AnyObject + 'static,
        S: FnMut(AnyObject) + 'static,
    {
        let setter = setter.map(|mut setter| move |value: Value| setter(AnyObject::from(value)));

        variable::define_virtual_variable(name, move || getter().value(), setter)
    }

    /// Defines the constant `name` on `Object`, making it visible
    /// everywhere (`rb_define_global_const`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// VM::define_global_const("RUTIE_LIMIT", Fixnum::new(100));
    ///
    /// let limit = VM::eval("RUTIE_LIMIT").unwrap();
    ///
    /// assert_eq!(limit.try_convert_to::<Fixnum>(), Ok(Fixnum::new(100)));
    /// ```
    pub fn define_global_const<T: Object>(name: &str, value: T) {
        class::define_global_const(name, value.value())
    }

    /// Formats `arguments` with the Ruby format string `format` (Ruby's
    /// `format`/`sprintf`, `rb_str_format`), returning the string or the
    /// `ArgumentError` for a bad format.
    ///
    /// This is Ruby-level formatting: the format is never passed to C's
    /// `printf`, so untrusted formats are fine.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Float, Object, RString, VM};
    /// # VM::init();
    ///
    /// let formatted = VM::format(
    ///     "%s has %03d items costing %.2f",
    ///     &[RString::new_utf8("cart").into(), Fixnum::new(7).into(), Float::new(9.5).into()],
    /// );
    ///
    /// assert_eq!(formatted.unwrap().to_str(), "cart has 007 items costing 9.50");
    ///
    /// assert!(VM::format("%d", &[]).is_err());
    /// ```
    pub fn format(format: &str, arguments: &[AnyObject]) -> Result<RString, AnyException> {
        let format = RString::new_utf8(format);
        let arguments = util::arguments_to_values(arguments);

        vm::protect_value(|| object::format(format.value(), &arguments))
            .map(RString::from)
            .map_err(AnyException::from)
    }

    /// Prints the `inspect` form of `object` and a newline to `$stdout`
    /// (Ruby's `p`, `rb_p`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// VM::p(&Fixnum::new(42)); // prints "42"
    /// ```
    pub fn p<T: Object>(object: &T) {
        object::p(object.value())
    }

    /// Loads and runs the Ruby file at `path` (Ruby's `load`,
    /// `rb_load_protect`), returning the exception it raises. With `wrap`,
    /// the file runs in an anonymous module so its methods and constants do
    /// not leak into the global namespace.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let path = std::env::temp_dir().join("rutie_vm_load_example.rb");
    /// std::fs::write(&path, "$rutie_loaded = 42").unwrap();
    ///
    /// VM::load(path.to_str().unwrap(), false).unwrap();
    ///
    /// assert_eq!(VM::global_get("$rutie_loaded").try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));
    /// assert!(VM::load("/no/such/file.rb", false).is_err());
    /// # std::fs::remove_file(path).unwrap();
    /// ```
    pub fn load(path: &str, wrap: bool) -> Result<(), AnyException> {
        let path = RString::new_utf8(path);

        io::load_protect(path.value(), wrap).map_err(|_| {
            VM::error_pop().unwrap_or_else(|_| AnyException::new("LoadError", Some("load failed")))
        })
    }

    /// Requires the feature `name` (Ruby's `require`, `rb_f_require`),
    /// returning `true` if it was loaded now, `false` if it already was, or
    /// the `LoadError` (or any exception it raised) otherwise.
    ///
    /// Unlike [`VM::require`](#method.require), a missing feature is an
    /// error value instead of an unrescued exception.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Object, VM};
    /// # VM::init();
    /// VM::init_loadpath();
    ///
    /// assert!(VM::protect_require("set").is_ok());
    /// assert_eq!(VM::protect_require("set").unwrap(), false);
    ///
    /// let error = VM::protect_require("no_such_rutie_feature").unwrap_err();
    ///
    /// assert!(Class::load_error().case_equals(&error));
    /// ```
    pub fn protect_require(name: &str) -> Result<bool, AnyException> {
        let name = RString::new_utf8(name);

        vm::protect_value(|| io::require(name.value()))
            .map(|loaded| loaded.is_true())
            .map_err(AnyException::from)
    }

    /// Records `feature` in `$LOADED_FEATURES` (`rb_provide`), the way a
    /// C extension announces the feature it implements.
    ///
    /// Give the name with its extension (`"name.so"` or `"name.rb"`), as
    /// Ruby's own extensions do, and ask [`VM::is_provided`](#method.is_provided)
    /// with the same name.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Object, RString, VM};
    /// # VM::init();
    ///
    /// assert!(!VM::is_provided("rutie_provided_example.so"));
    ///
    /// VM::provide("rutie_provided_example.so");
    ///
    /// assert!(VM::is_provided("rutie_provided_example.so"));
    ///
    /// let features = VM::global_get("$LOADED_FEATURES").try_convert_to::<Array>().unwrap();
    /// assert!(features.includes(&RString::new_utf8("rutie_provided_example.so")));
    /// ```
    pub fn provide(feature: &str) {
        io::provide(feature)
    }

    /// Returns `true` if `feature` has been required or provided
    /// (`rb_provided`). Without an extension only `.rb` features match, so
    /// ask for `"name.so"` to find a provided extension.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    /// # VM::init();
    ///
    /// assert!(VM::is_provided("enumerator"));
    /// ```
    pub fn is_provided(feature: &str) -> bool {
        io::is_provided(feature)
    }

    /// Adds the directory `path` to the front of `$LOAD_PATH`
    /// (`ruby_incpush`; several directories may be joined with the
    /// platform's path separator).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// let dir = std::env::temp_dir().join("rutie_load_path_example");
    /// std::fs::create_dir_all(&dir).unwrap();
    /// std::fs::write(dir.join("rutie_feature.rb"), "RUTIE_FEATURE = :loaded").unwrap();
    ///
    /// VM::add_load_path(dir.to_str().unwrap());
    ///
    /// assert!(VM::find_file("rutie_feature.rb").is_some());
    /// assert_eq!(VM::protect_require("rutie_feature").unwrap(), true);
    /// # std::fs::remove_dir_all(dir).unwrap();
    /// ```
    pub fn add_load_path(path: &str) {
        io::add_load_path(path)
    }

    /// Returns the full path of `name` found in `$LOAD_PATH`, or `None`
    /// (`rb_find_file`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    /// # VM::init();
    ///
    /// assert!(VM::find_file("no_such_rutie_file.rb").is_none());
    /// ```
    pub fn find_file(name: &str) -> Option<RString> {
        io::find_file(RString::new_utf8(name).value()).map(RString::from)
    }

    /// Call super
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Fixnum, Object, VM, Exception};
    ///
    /// class!(Adder);
    ///
    /// methods!(
    ///     Adder,
    ///     rtself,
    ///
    ///     fn adder_add(a: Fixnum, b: Fixnum) -> Fixnum {
    ///         if let Err(ref error) = a {
    ///             VM::raise(error.class(), &error.message());
    ///         }
    ///         if let Err(ref error) = b {
    ///             VM::raise(error.class(), &error.message());
    ///         }
    ///
    ///         // We can safely unwrap here
    ///         let a = a.unwrap().to_i64();
    ///         // We can safely unwrap here
    ///         let b = b.unwrap().to_i64();
    ///
    ///         Fixnum::new(a + b)
    ///     }
    /// );
    ///
    /// class!(DoAdder);
    ///
    /// methods!(
    ///     DoAdder,
    ///     rtself,
    ///
    ///     fn do_adder_add(a: Fixnum, b: Fixnum) -> Fixnum {
    ///         if let Err(ref error) = a {
    ///             VM::raise(error.class(), &error.message());
    ///         }
    ///         if let Err(ref error) = b {
    ///             VM::raise(error.class(), &error.message());
    ///         }
    ///
    ///         unsafe {
    ///             VM::call_super(&[
    ///                 a.unwrap().into(),
    ///                 b.unwrap().into()
    ///             ]).to::<Fixnum>()
    ///         }
    ///     }
    /// );
    ///
    ///
    /// fn main() {
    ///     # VM::init();
    ///
    ///     Class::new("Adder", None).define(|klass| {
    ///         klass.def("add", adder_add);
    ///     });
    ///     Class::new("DoAdder", Some(&Class::from_existing("Adder"))).define(|klass| {
    ///         klass.def("add", do_adder_add);
    ///     });
    ///
    ///     let result = VM::eval(" DoAdder.new().add(4, 4) ").unwrap();
    ///     let num = result.try_convert_to::<Fixnum>().unwrap().to_i64();
    ///     assert_eq!(num, 8);
    /// }
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// class Adder
    ///   def add(a, b)
    ///     a + b
    ///   end
    /// end
    ///
    /// class DoAdder < Adder
    ///   def add(a, b)
    ///     super(a, b)
    ///   end
    /// end
    ///
    /// result = DoAdder.new.add(4, 4)
    /// result == 8
    /// ```
    pub unsafe fn call_super(arguments: &[AnyObject]) -> AnyObject {
        let arguments = util::arguments_to_values(arguments);

        let result = vm::call_super(&arguments);

        AnyObject::from(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        types::ValueType, AnyObject, Array, Class, Exception, Fixnum, Hash, NilClass, Object,
        RString, Symbol, VM,
    };
    use std::{
        cell::Cell,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
    };

    fn eval_raising(code: &str) -> AnyObject {
        unsafe { VM::eval_str(code) }
    }

    fn fixnums(values: &[i64]) -> Array {
        values
            .iter()
            .map(|&value| Fixnum::new(value).to_any_object())
            .collect()
    }

    // cargo test at_exit -- --nocapture
    #[test]
    fn test_at_exit() {
        crate::on_ruby_thread(|| {
            let closure = |_vm| {
                println!("test class::vm::tests::test_at_exit worked!");
            };

            VM::at_exit(closure);
        });
    }

    #[test]
    fn test_at_exit_does_not_run_closure_immediately() {
        crate::on_ruby_thread(|| {
            let calls = Arc::new(AtomicUsize::new(0));
            let counter = calls.clone();

            VM::at_exit(move |_vm| {
                counter.fetch_add(1, Ordering::SeqCst);
            });

            // The closure runs when the VM shuts down, not now.
            assert_eq!(calls.load(Ordering::SeqCst), 0);
        });
    }

    #[test]
    fn test_call_protected_calls_capturing_closure() {
        crate::on_ruby_thread(|| {
            let mut calls = 0;
            VM::call_protected(|_vm| calls += 1);

            assert_eq!(calls, 1);
        });
    }

    #[test]
    fn test_raise_does_not_use_message_as_format() {
        crate::on_ruby_thread(|| {
            let result = VM::protect(|| {
                VM::raise(Class::from_existing("RuntimeError"), "100%s %d %n done");

                NilClass::new().into()
            });

            assert!(result.is_err());
            assert_eq!(VM::error_pop().unwrap().message(), "100%s %d %n done");
        });
    }

    #[test]
    fn test_ensure_runs_on_success_and_on_raise() {
        crate::on_ruby_thread(|| {
            let ensured = Cell::new(0);

            let result = VM::ensure(|| Fixnum::new(7).into(), || ensured.set(ensured.get() + 1));
            assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(7)));

            let result = VM::protect(|| {
                VM::ensure(
                    || eval_raising("raise IOError, 'closed'"),
                    || ensured.set(ensured.get() + 1),
                )
            });
            assert!(result.is_err());
            assert_eq!(ensured.get(), 2);

            let error = VM::error_pop().unwrap();
            assert!(Class::from_existing("IOError").case_equals(&error));
        });
    }

    #[test]
    fn test_rescue_standard_error_only() {
        crate::on_ruby_thread(|| {
            let result = VM::rescue(
                || eval_raising("raise 'rescued'"),
                |error| RString::new_utf8(&error.message()).into(),
            );
            assert_eq!(
                result.try_convert_to::<RString>().unwrap().to_str(),
                "rescued"
            );

            // Nothing raised: the handler is not called.
            let result = VM::rescue(|| Fixnum::new(1).into(), |_| unreachable!());
            assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));

            // `Exception` is not a `StandardError`, so it propagates.
            let result = VM::protect(|| {
                VM::rescue(
                    || eval_raising("raise Exception, 'not standard'"),
                    |_| NilClass::new().into(),
                )
            });
            assert!(result.is_err());
            assert_eq!(VM::error_pop().unwrap().message(), "not standard");

            // A panic in the body becomes a `RuntimeError` that can be rescued.
            let result = VM::rescue(
                || panic!("in body"),
                |error| RString::new_utf8(&error.message()).into(),
            );
            assert_eq!(
                result.try_convert_to::<RString>().unwrap().to_str(),
                "Rust panic: in body"
            );
        });
    }

    #[test]
    fn test_rescue_from_classes() {
        crate::on_ruby_thread(|| {
            let classes = [Class::from_existing("ZeroDivisionError")];

            let result = VM::rescue_from(
                &classes,
                || eval_raising("1 / 0"),
                |_| Fixnum::new(-1).into(),
            );
            assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(-1)));

            // Subclasses match too.
            let result = VM::rescue_from(
                &[Class::from_existing("StandardError")],
                || eval_raising("1 / 0"),
                |_| Fixnum::new(-2).into(),
            );
            assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(-2)));

            let result = VM::protect(|| {
                VM::rescue_from(
                    &classes,
                    || eval_raising("raise 'other'"),
                    |_| unreachable!(),
                )
            });
            assert!(result.is_err());
            assert_eq!(VM::error_pop().unwrap().message(), "other");
        });
    }

    #[test]
    fn test_catch_and_throw() {
        crate::on_ruby_thread(|| {
            // An inner catch does not stop a throw to an outer tag.
            let result = VM::catch(Symbol::new("outer"), |_| {
                VM::catch(Symbol::new("inner"), |_| {
                    VM::throw(Symbol::new("outer"), Fixnum::new(1));
                });

                Fixnum::new(2).into()
            });
            assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));

            // Ruby code can throw to a catch made in Rust.
            let result = VM::catch(Symbol::new("from_ruby"), |_| {
                eval_raising("throw :from_ruby, 'thrown'")
            });
            assert_eq!(
                result.try_convert_to::<RString>().unwrap().to_str(),
                "thrown"
            );

            let result = VM::protect(|| VM::throw(Symbol::new("missing"), NilClass::new()));
            assert!(result.is_err());
            assert!(
                Class::from_existing("UncaughtThrowError").case_equals(&VM::error_pop().unwrap())
            );
        });
    }

    #[test]
    fn test_send_with_block_and_iter_break() {
        crate::on_ruby_thread(|| {
            let array = fixnums(&[1, 2, 3, 4]);

            let evens = unsafe {
                array.send_with_block("select", &[], |values| {
                    let x = values[0].try_convert_to::<Fixnum>().unwrap().to_i64();

                    crate::Boolean::new(x % 2 == 0).into()
                })
            };
            assert_eq!(evens.try_convert_to::<Array>().unwrap(), fixnums(&[2, 4]));

            // Arguments are passed on: [1, 2, 3, 4].inject(10) { |sum, x| sum + x }
            let sum = unsafe {
                array.send_with_block("inject", &[Fixnum::new(10).into()], |values| {
                    let sum = values[0].try_convert_to::<Fixnum>().unwrap().to_i64();
                    let x = values[1].try_convert_to::<Fixnum>().unwrap().to_i64();

                    Fixnum::new(sum + x).into()
                })
            };
            assert_eq!(sum.try_convert_to::<Fixnum>(), Ok(Fixnum::new(20)));

            let found = unsafe {
                array.send_with_block("each", &[], |values| {
                    if values[0].try_convert_to::<Fixnum>().unwrap().to_i64() == 3 {
                        VM::iter_break_value(RString::new_utf8("three"));
                    }

                    NilClass::new().into()
                })
            };
            assert_eq!(found.try_convert_to::<RString>().unwrap().to_str(), "three");

            let result = array.protect_send_with_block("each", &[], |_| panic!("in block"));
            assert_eq!(result.unwrap_err().message(), "Rust panic: in block");
        });
    }

    #[test]
    fn test_jump_tag_resumes_exception() {
        crate::on_ruby_thread(|| {
            let result = VM::protect(|| {
                if let Err(state) = VM::protect(|| eval_raising("raise KeyError, 'resumed'")) {
                    unsafe { VM::jump_tag(state) };
                }

                NilClass::new().into()
            });

            assert!(result.is_err());

            let error = VM::error_pop().unwrap();
            assert!(Class::from_existing("KeyError").case_equals(&error));
            assert_eq!(error.message(), "resumed");
        });
    }

    #[test]
    fn test_warn_and_warning() {
        crate::on_ruby_thread(|| {
            VM::eval(
                "$rutie_warnings = []
                 $rutie_verbose = $VERBOSE
                 Warning.singleton_class.send(:alias_method, :rutie_original_warn, :warn)
                 def Warning.warn(message); $rutie_warnings << message; end",
            )
            .unwrap();

            let warnings = || {
                VM::eval("$rutie_warnings.join")
                    .unwrap()
                    .try_convert_to::<RString>()
                    .unwrap()
                    .to_string()
            };

            VM::eval("$VERBOSE = nil").unwrap();
            VM::warn("silenced");
            VM::warning("silenced");
            assert_eq!(warnings(), "");

            VM::eval("$VERBOSE = false").unwrap();
            VM::warn("%s %d warn\0 after NUL");
            VM::warning("not verbose");
            assert!(
                warnings().ends_with("warning: %s %d warn\n"),
                "{}",
                warnings()
            );

            VM::eval("$VERBOSE = true").unwrap();
            VM::warning("verbose");
            assert!(warnings().ends_with("warning: verbose\n"), "{}", warnings());

            VM::eval(
                "Warning.singleton_class.send(:alias_method, :warn, :rutie_original_warn)
                 $VERBOSE = $rutie_verbose",
            )
            .unwrap();
        });
    }

    #[test]
    fn test_arity_helpers() {
        crate::on_ruby_thread(|| {
            assert_eq!(VM::check_arity(1, 1, 1), Ok(1));
            assert_eq!(
                VM::check_arity(3, 0, 2).unwrap_err().message(),
                "wrong number of arguments (given 3, expected 0..2)"
            );

            assert!(VM::protect(|| VM::raise_arity_error(0, 2, -1)).is_err());
            assert_eq!(
                VM::error_pop().unwrap().message(),
                "wrong number of arguments (given 0, expected 2+)"
            );
        });
    }

    #[test]
    fn test_raising_helpers() {
        crate::on_ruby_thread(|| {
            assert!(VM::protect(|| VM::raise_zero_division()).is_err());
            assert!(
                Class::from_existing("ZeroDivisionError").case_equals(&VM::error_pop().unwrap())
            );

            assert!(VM::protect(|| VM::not_implemented()).is_err());
            assert!(
                Class::from_existing("NotImplementedError").case_equals(&VM::error_pop().unwrap())
            );
        });
    }

    #[test]
    fn test_check_frozen_and_check_type() {
        crate::on_ruby_thread(|| {
            let array = fixnums(&[1]);
            array.check_frozen();
            array.check_type(ValueType::Array);

            let frozen = fixnums(&[1]).freeze();
            assert!(VM::protect(|| {
                frozen.check_frozen();
                NilClass::new().into()
            })
            .is_err());

            let error = VM::error_pop().unwrap();
            assert!(Class::from_existing("FrozenError").case_equals(&error));

            assert!(VM::protect(|| {
                array.check_type(ValueType::Hash);
                NilClass::new().into()
            })
            .is_err());
            assert_eq!(
                VM::error_pop().unwrap().message(),
                "wrong argument type Array (expected Hash)"
            );
        });
    }

    #[test]
    fn test_scan_args() {
        crate::on_ruby_thread(|| {
            let arguments: Vec<AnyObject> = (1..=5).map(|i| Fixnum::new(i).into()).collect();

            let args = VM::scan_args(&arguments, "21*1").unwrap();
            assert_eq!(args.required, arguments[0..2].to_vec());
            assert_eq!(args.optional, vec![Some(arguments[2].clone())]);
            assert_eq!(args.splat.unwrap(), fixnums(&[4]));
            assert_eq!(args.post, vec![arguments[4].clone()]);
            assert!(args.keywords.is_none());
            assert!(args.block.is_none());

            // Missing optional arguments are `None`, not `Some(nil)`.
            let args = VM::scan_args(&arguments[0..3], "12*1").unwrap();
            assert_eq!(args.optional, vec![Some(arguments[1].clone()), None]);
            assert_eq!(args.splat.unwrap().length(), 0);
            assert_eq!(args.post, vec![arguments[2].clone()]);

            let args = VM::scan_args(&[NilClass::new().into()], "02").unwrap();
            assert_eq!(args.optional, vec![Some(NilClass::new().into()), None]);

            assert_eq!(
                VM::scan_args(&arguments, "2").unwrap_err().message(),
                "wrong number of arguments (given 5, expected 2)"
            );
            assert_eq!(
                VM::scan_args(&arguments, "9").unwrap_err().message(),
                "wrong number of arguments (given 5, expected 9)"
            );
            assert_eq!(
                VM::scan_args(&arguments, "1:*").unwrap_err().message(),
                "bad scan arg format: 1:*"
            );

            let mut keywords = Hash::new();
            keywords.store(Symbol::new("mode"), Symbol::new("fast"));

            let args =
                VM::scan_args(&[arguments[0].clone(), keywords.to_any_object()], "1:").unwrap();
            assert_eq!(args.required, vec![arguments[0].clone()]);
            assert_eq!(
                args.keywords.unwrap().at(&Symbol::new("mode")),
                Symbol::new("fast").into()
            );

            let args = VM::scan_args(&arguments[0..1], "1:").unwrap();
            assert!(args.keywords.is_none());
        });
    }

    #[test]
    fn test_scan_args_in_method_with_block() {
        crate::on_ruby_thread(|| {
            extern "C" fn rutie_scan_block(
                argc: crate::types::Argc,
                argv: *const AnyObject,
                _rtself: AnyObject,
            ) -> AnyObject {
                let arguments = crate::util::parse_arguments(argc, argv);
                let args = VM::scan_args(&arguments, "1&");

                if let Err(ref error) = args {
                    VM::raise(error.class(), &error.message());
                }

                let args = args.unwrap();

                match args.block {
                    Some(block) => block.call(&args.required),
                    None => Symbol::new("no_block").into(),
                }
            }

            Class::from_existing("Object").define(|klass| {
                klass.def_private("rutie_scan_block", rutie_scan_block);
            });

            let result = VM::eval("rutie_scan_block(20) { |x| x + 1 }").unwrap();
            assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(21)));

            let result = VM::eval("rutie_scan_block(20)").unwrap();
            assert_eq!(
                result.try_convert_to::<Symbol>(),
                Ok(Symbol::new("no_block"))
            );

            let error = VM::eval("rutie_scan_block").unwrap_err();
            assert!(Class::from_existing("ArgumentError").case_equals(&error));
        });
    }

    #[cfg(ruby_gte_2_7)]
    #[test]
    fn test_is_keyword_given() {
        crate::on_ruby_thread(|| {
            extern "C" fn rutie_keyword_given(
                _argc: crate::types::Argc,
                _argv: *const AnyObject,
                _rtself: AnyObject,
            ) -> crate::Boolean {
                crate::Boolean::new(VM::is_keyword_given())
            }

            Class::from_existing("Object").define(|klass| {
                klass.def_private("rutie_keyword_given", rutie_keyword_given);
            });

            let given = |code| {
                VM::eval(code)
                    .unwrap()
                    .try_convert_to::<crate::Boolean>()
                    .unwrap()
                    .to_bool()
            };

            assert!(given("rutie_keyword_given(a: 1)"));
            assert!(!given("rutie_keyword_given({ a: 1 })"));
            assert!(!given("rutie_keyword_given(1)"));
        });
    }

    #[test]
    fn test_get_kwargs() {
        crate::on_ruby_thread(|| {
            let mut keywords = Hash::new();
            keywords.store(Symbol::new("a"), Fixnum::new(1));
            keywords.store(Symbol::new("c"), Fixnum::new(3));

            let kwargs = VM::get_kwargs(Some(&keywords), &["a"], &["b", "c"], false).unwrap();
            assert_eq!(kwargs.required, vec![Fixnum::new(1).into()]);
            assert_eq!(kwargs.optional, vec![None, Some(Fixnum::new(3).into())]);
            assert!(kwargs.rest.is_none());

            let kwargs = VM::get_kwargs(Some(&keywords), &[], &["a"], true).unwrap();
            assert_eq!(kwargs.optional, vec![Some(Fixnum::new(1).into())]);
            assert_eq!(
                kwargs.rest.unwrap().at(&Symbol::new("c")),
                Fixnum::new(3).into()
            );

            let kwargs = VM::get_kwargs(None, &[], &["a"], false).unwrap();
            assert_eq!(kwargs.optional, vec![None]);

            // Ruby 2.7 formats the names as symbols (`:z`), 2.5 and 2.6 do not.
            let message = VM::get_kwargs(Some(&keywords), &["a", "z"], &[], true)
                .unwrap_err()
                .message();
            assert!(message.starts_with("missing keyword: ") && message.ends_with("z"));

            // A frozen hash is fine: it is never modified.
            let frozen = keywords.freeze();
            assert!(VM::get_kwargs(Some(&frozen), &["a"], &["c"], false).is_ok());
            assert_eq!(frozen.length(), 2);
        });
    }

    #[test]
    fn test_global_variables() {
        crate::on_ruby_thread(|| {
            VM::global_set("$rutie_test_global", Fixnum::new(1));
            let value = VM::eval("$rutie_test_global").unwrap();
            assert_eq!(value.try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
            assert!(VM::global_get("$rutie_test_undefined_global").is_nil());

            assert!(VM::protect_global_set("$$", Fixnum::new(1)).is_err());

            let variable = VM::define_variable("$rutie_test_defined", RString::new_utf8("a"));
            VM::eval("$rutie_test_defined += 'b'").unwrap();
            // Survives a full GC: the storage is a GC root.
            crate::GC::start();
            assert_eq!(
                variable.get().try_convert_to::<RString>().unwrap().to_str(),
                "ab"
            );

            let readonly = VM::define_readonly_variable("$rutie_test_readonly", Fixnum::new(1));
            assert!(VM::eval("$rutie_test_readonly = 2").is_err());
            readonly.set(Fixnum::new(3));
            assert_eq!(
                VM::global_get("$rutie_test_readonly").try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(3))
            );

            let reads = Arc::new(AtomicUsize::new(0));
            let counter = reads.clone();
            VM::define_virtual_variable(
                "$rutie_test_virtual",
                move || Fixnum::new(counter.fetch_add(1, Ordering::SeqCst) as i64).into(),
                None::<fn(AnyObject)>,
            );
            VM::eval("$rutie_test_virtual; $rutie_test_virtual").unwrap();
            assert_eq!(reads.load(Ordering::SeqCst), 2);

            let error = VM::eval("$rutie_test_virtual = 1").unwrap_err();
            assert_eq!(
                error.message(),
                "$rutie_test_virtual is a read-only variable"
            );

            VM::define_virtual_variable(
                "$rutie_test_panicking",
                || panic!("getter failed"),
                None::<fn(AnyObject)>,
            );
            let error = VM::eval("$rutie_test_panicking").unwrap_err();
            assert_eq!(error.message(), "Rust panic: getter failed");
        });
    }

    #[test]
    fn test_format_and_global_const() {
        crate::on_ruby_thread(|| {
            let formatted = VM::format(
                "%-5s|%+d",
                &[Symbol::new("ab").into(), Fixnum::new(3).into()],
            );
            assert_eq!(formatted.unwrap().to_str(), "ab   |+3");

            // Formats with C conversions are handled by Ruby, not printf.
            assert!(VM::format("%n %s", &[Fixnum::new(1).into()]).is_err());

            VM::define_global_const("RUTIE_TEST_CONST", Fixnum::new(8));
            let value = VM::eval("RUTIE_TEST_CONST").unwrap();
            assert_eq!(value.try_convert_to::<Fixnum>(), Ok(Fixnum::new(8)));
        });
    }

    #[test]
    fn test_load_require_and_provide() {
        crate::on_ruby_thread(|| {
            let dir = std::env::temp_dir().join("rutie_vm_load_unit_test");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("rutie_unit_feature.rb"), "$rutie_unit_feature = 1").unwrap();
            std::fs::write(dir.join("rutie_unit_broken.rb"), "raise 'broken on load'").unwrap();
            std::fs::write(dir.join("rutie_unit_wrapped.rb"), "RUTIE_WRAPPED_CONST = 1").unwrap();

            VM::add_load_path(dir.to_str().unwrap());

            assert!(VM::find_file("rutie_unit_feature.rb").is_some());
            assert_eq!(VM::protect_require("rutie_unit_feature").unwrap(), true);
            assert_eq!(VM::protect_require("rutie_unit_feature").unwrap(), false);
            assert!(VM::is_provided("rutie_unit_feature"));

            let error = VM::protect_require("rutie_unit_broken").unwrap_err();
            assert_eq!(error.message(), "broken on load");

            // A wrapped load keeps constants out of Object.
            let wrapped = dir.join("rutie_unit_wrapped.rb");
            VM::load(wrapped.to_str().unwrap(), true).unwrap();
            assert!(!Class::object().is_const_defined_at("RUTIE_WRAPPED_CONST"));
            VM::load(wrapped.to_str().unwrap(), false).unwrap();
            assert!(Class::object().is_const_defined_at("RUTIE_WRAPPED_CONST"));

            let error = VM::load("/no/such/rutie/file.rb", false).unwrap_err();
            assert!(Class::load_error().case_equals(&error));

            VM::provide("rutie_unit_virtual_feature.so");
            assert!(VM::is_provided("rutie_unit_virtual_feature.so"));
            assert!(!VM::is_provided("rutie_unit_never_provided.so"));

            std::fs::remove_dir_all(dir).unwrap();
        });
    }

    #[test]
    fn test_define_alias_undef_method_and_alloc_func() {
        crate::on_ruby_thread(|| {
            let mut klass = Class::new("RutieVmTestAliases", None);
            VM::eval("class RutieVmTestAliases; def original; :original; end; end").unwrap();

            klass.define_alias("copy", "original");
            let result = VM::eval("RutieVmTestAliases.new.copy").unwrap();
            assert_eq!(
                result.try_convert_to::<Symbol>(),
                Ok(Symbol::new("original"))
            );

            klass.undef_method("original");
            assert!(VM::eval("RutieVmTestAliases.new.original").is_err());
            assert!(VM::eval("RutieVmTestAliases.new.copy").is_ok());

            klass.undef_alloc_func();
            let error = VM::eval("RutieVmTestAliases.new").unwrap_err();
            assert!(Class::from_existing("TypeError").case_equals(&error));
        });
    }
}
