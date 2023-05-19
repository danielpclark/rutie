use crate::{
    binding::vm,
    types::{Argc, Value, VmPointer},
};

use crate::{util, AnyException, AnyObject, Array, Class, NilClass, Object, Proc, TryConvert};

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
    /// use rutie::{Class, Fixnum, Object, VM, eval};
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
    /// use rutie::{Class, Fixnum, Object, Exception, RString, VM, eval};
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
    /// use rutie::{Class, Object, Proc, RString, VM, class, methods};
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
    /// use rutie::{Class, Fixnum, Object, VM, class, methods};
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
    /// use rutie::{Class, Fixnum, Object, VM, class, methods};
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
    /// use rutie::{Array, Class, Fixnum, Object, VM, class, methods};
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
        Class::from_existing("Process").send("exit!", arguments.as_ref());
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

    /// `at_exit` is run AFTER the VM is shut down
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    ///
    /// # VM::init();
    ///
    /// let closure = |_vm| {
    ///     println!("at_exit worked!");
    /// };
    ///
    /// VM::at_exit(closure);
    /// ```
    pub fn at_exit<F>(func: F)
    where
        F: FnMut(VmPointer) -> (),
    {
        vm::at_exit(func)
    }

    /// Call super
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Fixnum, Object, VM, Exception, class, methods};
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
    use crate::{LOCK_FOR_TEST, VM};
    use rb_sys_test_helpers::ruby_test;

    // cargo test at_exit -- --nocapture
    #[ruby_test]
    fn test_at_exit() {
        let closure = |_vm| {
            println!("test class::vm::tests::test_at_exit worked!");
        };

        VM::at_exit(closure);
    }
}
