use std::ffi::CString;

use crate::{
    binding::{class, debug, exception, hash, io, object, symbol, variable, vm},
    helpers::scan_args::{KeywordArgs, ScanArgsFormat, ScannedArgs},
    rubysys::{exception::rb_eStandardError, rproc},
    types::{c_void, Argc, Id, Value, VmPointer},
};

use crate::{
    types::{Callback, ValueType},
    util, AnyException, AnyObject, Array, Class, Exception, GlobalVariable, Hash, Module, NilClass,
    Object, Proc, ProfileFrame, RString, Symbol, TryConvert,
};

/// The category of a warning, for [`VM::warn_category`](struct.VM.html#method.warn_category)
/// and friends (`rb_warning_category_t`). Ruby prints a category's warnings
/// only while it is enabled (`Warning[:deprecated] = true`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum WarningCategory {
    /// Deprecated features (`:deprecated`, disabled by default).
    Deprecated = crate::rubysys::exception::RB_WARN_CATEGORY_DEPRECATED,
    /// Experimental features (`:experimental`).
    Experimental = crate::rubysys::exception::RB_WARN_CATEGORY_EXPERIMENTAL,
    /// Performance issues (`:performance`, disabled by default).
    Performance = crate::rubysys::exception::RB_WARN_CATEGORY_PERFORMANCE,
    /// Blocks passed to methods that do not use them (`:strict_unused_block`,
    /// disabled by default).
    StrictUnusedBlock = crate::rubysys::exception::RB_WARN_CATEGORY_STRICT_UNUSED_BLOCK,
}

/// What a non-blocking IO operation would have waited for, for
/// [`VM::raise_wait_syserr`](struct.VM.html#method.raise_wait_syserr)
/// (`enum rb_io_wait_readwrite`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum IoWait {
    /// Waiting to read: the exception is extended with `IO::WaitReadable`.
    Readable = crate::rubysys::exception::RB_IO_WAIT_READABLE,
    /// Waiting to write: the exception is extended with `IO::WaitWritable`.
    Writable = crate::rubysys::exception::RB_IO_WAIT_WRITABLE,
}

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
    /// Calling it again while the VM is running does nothing (`ruby_setup` returns early), so it
    /// is safe to call more than once from the thread that started the VM. It must not be called
    /// after [`VM::cleanup`](#method.cleanup). It exits the process if Ruby fails to boot; use
    /// [`VM::try_init`](#method.try_init) to get the error instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, VM};
    ///
    /// VM::init();
    ///
    /// // VM started, able to use Ruby now
    /// assert!(VM::is_initialized());
    ///
    /// let class = Class::new("SomeClass", None); // etc
    /// assert_eq!(class.name().unwrap().to_str(), "SomeClass");
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

    /// Marks the methods this thread defines from now on as Ractor-safe
    /// (`rb_ext_ractor_safe(true)`), so Ruby lets any Ractor call them, in
    /// parallel with other Ractors.
    ///
    /// Methods are not Ractor-safe by default: Ruby raises
    /// `Ractor::UnsafeError` when one is called outside the main Ractor.
    /// That holds for an extension loaded with `require` (Ruby resets the
    /// setting for each one) and for a VM started by
    /// [`VM::init`](#method.init). Use
    /// [`VM::ext_ractor_unsafe`](#method.ext_ractor_unsafe) to switch back.
    ///
    /// # Safety
    ///
    /// Each method defined while this is on may run on several Ractors, and
    /// so several OS threads, at once. It must not touch Rust state shared
    /// between calls without synchronisation (a `static mut`, or wrapped data
    /// that is not `Sync`), and must only use Ruby objects it was given or
    /// created.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Fixnum, Object, RString, VM};
    ///
    /// methods!(
    ///     Fixnum,
    ///     rtself,
    ///
    ///     // Uses only its receiver, so it is safe on any Ractor.
    ///     fn double() -> Fixnum {
    ///         Fixnum::new(rtself.to_i64() * 2)
    ///     }
    ///
    ///     fn triple() -> Fixnum {
    ///         Fixnum::new(rtself.to_i64() * 3)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Integer").define(|klass| {
    ///         unsafe { VM::ext_ractor_safe(true) };
    ///         klass.def("double", double);
    ///         VM::ext_ractor_unsafe();
    ///
    ///         klass.def("triple", triple);
    ///     });
    ///
    ///     let in_ractor = |code: &str| {
    ///         let code = format!(
    ///             "Warning[:experimental] = false
    ///              Ractor.new {{ begin; {}; rescue => e; e.class.name; end }}.value.to_s",
    ///             code
    ///         );
    ///         VM::eval(&code).unwrap().try_convert_to::<RString>().unwrap().to_string()
    ///     };
    ///
    ///     assert_eq!(in_ractor("21.double"), "42");
    ///     assert_eq!(in_ractor("21.triple"), "Ractor::UnsafeError");
    ///
    ///     // The main Ractor can call both.
    ///     assert_eq!(VM::eval("21.triple").unwrap().try_convert_to::<Fixnum>().unwrap().to_i64(), 63);
    /// }
    /// ```
    pub unsafe fn ext_ractor_safe(flag: bool) {
        vm::ext_ractor_safe(flag);
    }

    /// Marks the methods this thread defines from now on as not Ractor-safe
    /// (`rb_ext_ractor_safe(false)`): calling one outside the main Ractor
    /// raises `Ractor::UnsafeError`. This is the default; it undoes
    /// [`VM::ext_ractor_safe`](#method.ext_ractor_safe).
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Fixnum, Object, RString, VM};
    ///
    /// methods!(
    ///     Fixnum,
    ///     rtself,
    ///
    ///     fn negate() -> Fixnum {
    ///         Fixnum::new(-rtself.to_i64())
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     unsafe { VM::ext_ractor_safe(true) };
    ///     VM::ext_ractor_unsafe();
    ///
    ///     Class::from_existing("Integer").define(|klass| {
    ///         klass.def("negate", negate);
    ///     });
    ///
    ///     let result = VM::eval(
    ///         "Warning[:experimental] = false
    ///          Ractor.new { begin; 1.negate; rescue => e; e.class.name; end }.value",
    ///     )
    ///     .unwrap();
    ///
    ///     assert_eq!(result.try_convert_to::<RString>().unwrap().to_str(), "Ractor::UnsafeError");
    /// }
    /// ```
    pub fn ext_ractor_unsafe() {
        vm::ext_ractor_safe(false);
    }

    /// Requires Ruby source file.
    ///
    /// # Examples
    ///
    /// A missing file raises `LoadError`, which ends a program that is not
    /// inside `VM::protect`; [`VM::protect_require`](#method.protect_require)
    /// returns it instead.
    ///
    /// ```
    /// use rutie::{Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let dir = std::env::temp_dir().join(format!("rutie_require_{}", std::process::id()));
    /// std::fs::create_dir_all(&dir).unwrap();
    /// std::fs::write(dir.join("some_ruby_file.rb"), "$loaded = 1").unwrap();
    /// VM::add_load_path(dir.to_str().unwrap());
    ///
    /// VM::require("some_ruby_file");
    ///
    /// assert_eq!(VM::global_get("$loaded").try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
    /// std::fs::remove_dir_all(dir).unwrap();
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
    /// ```
    /// use rutie::{Class, Exception, NilClass, Object, VM};
    /// # VM::init();
    ///
    /// // Raising ends the program unless something rescues it.
    /// let result = VM::protect(|| {
    ///     VM::raise(Class::from_existing("ArgumentError"), "Wrong argument");
    ///     NilClass::new().into()
    /// });
    ///
    /// assert!(result.is_err());
    /// let error = VM::error_pop().unwrap();
    /// assert!(Class::argument_error().case_equals(&error));
    /// assert_eq!(error.message(), "Wrong argument");
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
    /// ```
    /// use rutie::{Class, Exception, NilClass, Object, VM};
    /// # VM::init();
    ///
    /// let standard_error = Class::from_existing("StandardError");
    /// let custom_exception = Class::new("CustomException", Some(&standard_error));
    ///
    /// let result = VM::protect(|| {
    ///     VM::raise(Class::from_existing("CustomException"), "Something went wrong");
    ///     NilClass::new().into()
    /// });
    ///
    /// assert!(result.is_err());
    /// assert!(custom_exception.case_equals(&VM::error_pop().unwrap()));
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
    ///
    /// # Message format
    ///
    /// The message is passed to `rb_raise` as its printf format, as in C:
    /// `%` starts a conversion, so write `%%` for a literal `%`, and never
    /// pass text you did not write (a stray `%s` reads memory it shouldn't).
    /// [`VM::raise_message`](#method.raise_message) takes the message as
    /// plain text instead.
    pub fn raise(exception: Class, message: &str) {
        vm::raise(exception.value(), message);
    }

    /// Raises `exception` with `message` as plain text: unlike
    /// [`VM::raise`](#method.raise), `%` has no special meaning, so any
    /// string (such as one containing user input) is safe to pass. The
    /// exception is built before raising, so no Rust allocation is leaked
    /// by the jump.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Exception, VM};
    /// # VM::init();
    ///
    /// let result = VM::protect(|| {
    ///     VM::raise_message(Class::argument_error(), "100% of 5%s");
    /// });
    ///
    /// assert!(result.is_err());
    /// assert_eq!(VM::error_pop().unwrap().message(), "100% of 5%s");
    /// ```
    pub fn raise_message(exception: Class, message: &str) -> ! {
        vm::raise_message(exception.value(), message)
    }

    /// Raises an exception from a native `AnyException` object.
    ///
    /// # Examples
    ///
    /// ### Built-in exceptions
    ///
    /// ```
    /// use rutie::{Class, VM, Exception, AnyException, NilClass};
    /// # VM::init();
    ///
    /// let result = VM::protect(|| {
    ///     VM::raise_ex(AnyException::new("StandardError", Some("something went wrong")));
    ///     NilClass::new().into()
    /// });
    ///
    /// assert!(result.is_err());
    /// assert_eq!(VM::error_pop().unwrap().message(), "something went wrong");
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
    /// ```
    /// use rutie::{Class, VM, Exception, AnyException, NilClass, Object};
    /// # VM::init();
    ///
    /// let standard_error = Class::from_existing("StandardError");
    /// let custom = Class::new("CustomException", Some(&standard_error));
    ///
    /// let exception = AnyException::new("CustomException", Some("something went wrong"));
    ///
    /// let result = VM::protect(|| {
    ///     VM::raise_ex(AnyException::new("CustomException", Some("something went wrong")));
    ///     NilClass::new().into()
    /// });
    ///
    /// assert!(result.is_err());
    /// assert!(custom.case_equals(&VM::error_pop().unwrap()));
    /// assert_eq!(exception.message(), "something went wrong");
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
    /// ```
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
    ///     # VM::init();
    ///     Class::new("Greeter", None).define(|klass| {
    ///         klass.def_self("greet_rust_with", greet_rust_with);
    ///     });
    ///
    ///     let greeting = VM::eval("Greeter.greet_rust_with { |name| \"Hello, #{name}!\" }").unwrap();
    ///     assert_eq!(greeting.try_convert_to::<RString>().unwrap().to_str(), "Hello, Rust!");
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
    ///
    ///     let sum = VM::eval("Calculator.new.calculate(2, 3)").unwrap();
    ///     assert_eq!(sum.try_convert_to::<Fixnum>(), Ok(Fixnum::new(5)));
    ///
    ///     let product = VM::eval("Calculator.new.calculate(2, 3) { |a, b| a * b }").unwrap();
    ///     assert_eq!(product.try_convert_to::<Fixnum>(), Ok(Fixnum::new(6)));
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
    /// // How `protect_send` uses these three:
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
    ///
    /// ```
    /// use rutie::{AnyException, Exception, Object, VM};
    /// # VM::init();
    ///
    /// let result = VM::protect(|| unsafe { VM::eval_str("raise 'oops'") });
    /// assert!(result.is_err());
    ///
    /// // The exception stays in `$!` until it is taken or cleared.
    /// let error: AnyException = VM::error_info().unwrap();
    /// assert_eq!(error.message(), "oops");
    ///
    /// VM::clear_error_info();
    /// assert!(VM::error_info().is_err());
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
    /// // How `protect_send` uses these three:
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
    ///
    /// ```
    /// use rutie::{AnyException, Exception, Object, VM};
    /// # VM::init();
    ///
    /// let result = VM::protect(|| unsafe { VM::eval_str("raise 'oops'") });
    /// assert!(result.is_err());
    ///
    /// // The exception stays in `$!` until it is taken or cleared.
    /// let error: AnyException = VM::error_info().unwrap();
    /// assert_eq!(error.message(), "oops");
    ///
    /// VM::clear_error_info();
    /// assert!(VM::error_info().is_err());
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
    /// // How `protect_send` uses these three:
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
    ///
    /// ```
    /// use rutie::{AnyException, Exception, Object, VM};
    /// # VM::init();
    ///
    /// let result = VM::protect(|| unsafe { VM::eval_str("raise 'oops'") });
    /// assert!(result.is_err());
    ///
    /// // The exception stays in `$!` until it is taken or cleared.
    /// let error: AnyException = VM::error_info().unwrap();
    /// assert_eq!(error.message(), "oops");
    ///
    /// VM::clear_error_info();
    /// assert!(VM::error_info().is_err());
    /// ```
    pub fn clear_error_info() {
        vm::set_errinfo(NilClass::new().value());
    }

    /// Exit with Ruby VM with status code.
    ///
    /// # Examples
    ///
    /// Outside any `begin`/`rescue` this ends the program with `status`. Inside
    /// [`VM::protect`](#method.protect) it raises `SystemExit`, which the
    /// caller can inspect:
    ///
    /// ```
    /// use rutie::{Class, Fixnum, NilClass, Object, VM};
    /// # VM::init();
    ///
    /// let result = VM::protect(|| {
    ///     VM::exit(3);
    ///     NilClass::new().into()
    /// });
    ///
    /// assert!(result.is_err());
    /// let exit = VM::error_pop().unwrap();
    /// assert!(Class::system_exit().case_equals(&exit));
    /// assert_eq!(unsafe { exit.send("status", &[]) }.try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
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
    /// // This ends the doctest process right here, with a success status,
    /// // which is why nothing can be asserted after it.
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
    ///
    /// Ruby has no public C API for installing signal handlers, so this
    /// (`Signal.trap`) is the supported way to handle signals; a handler can
    /// be a Rust closure wrapped with [`Proc::new`](struct.Proc.html#method.new).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// // Windows only has INT, ILL, ABRT, FPE, SEGV, TERM and EXIT.
    /// let signal = if cfg!(windows) { "TERM" } else { "USR2" };
    ///
    /// let previous = VM::trap(&[RString::new_utf8(signal).into(), RString::new_utf8("IGNORE").into()])
    ///     .unwrap();
    ///
    /// // Restoring the previous handler returns the one just installed.
    /// let installed = VM::trap(&[RString::new_utf8(signal).into(), previous]).unwrap();
    /// assert_eq!(installed.try_convert_to::<RString>().unwrap().to_str(), "IGNORE");
    ///
    /// // Unknown signals are errors.
    /// assert!(VM::trap(&[RString::new_utf8("NOT_A_SIGNAL").into(), RString::new_utf8("IGNORE").into()]).is_err());
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
    /// Before 0.10 this method called `func` immediately; that behaviour is
    /// still available as [`VM::call_protected`](#method.call_protected).
    ///
    /// For code that must run after Ruby is gone (when the VM itself is
    /// freed, after every `at_exit` handler and finalizer), use
    /// [`VM::at_vm_exit`](#method.at_vm_exit) (`ruby_vm_at_exit`) instead.
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
    /// This is what `VM::at_exit` did before 0.10. To run code when the VM
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

    /// Starts the Ruby VM like [`VM::init`](#method.init) (`ruby_setup`),
    /// but returns the error state instead of exiting the process when Ruby
    /// fails to boot.
    ///
    /// Like `VM::init`, it does nothing when the VM is already running, so
    /// calling either of them again is harmless (on the thread that started
    /// the VM). Neither may be called after [`VM::cleanup`](#method.cleanup).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    ///
    /// assert_eq!(VM::try_init(), Ok(()));
    /// assert!(VM::is_initialized());
    ///
    /// // Starting it again is a no-op.
    /// VM::init();
    /// assert_eq!(VM::try_init(), Ok(()));
    /// ```
    pub fn try_init() -> Result<(), i32> {
        match vm::setup() {
            0 => Ok(()),
            state => Err(state),
        }
    }

    /// Starts the Ruby VM and sets `ARGV` to `arguments`
    /// ([`VM::init`](#method.init) followed by
    /// [`VM::set_argv`](#method.set_argv)).
    ///
    /// # Panics
    ///
    /// Panics if an argument contains a NUL byte.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Object, RString, VM};
    ///
    /// VM::init_with_args(&["--verbose", "input.txt"]);
    ///
    /// let argv = unsafe { VM::eval_str("ARGV") }.try_convert_to::<Array>().unwrap();
    ///
    /// assert_eq!(argv.length(), 2);
    /// assert_eq!(argv.at(1).try_convert_to::<RString>().unwrap().to_str(), "input.txt");
    /// ```
    pub fn init_with_args(arguments: &[&str]) {
        vm::init();
        vm::set_argv(arguments);
    }

    /// Replaces the contents of `ARGV` with `arguments` (`ruby_set_argv`).
    /// Ruby copies them into frozen strings.
    ///
    /// # Panics
    ///
    /// Panics if an argument contains a NUL byte.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Object, VM};
    /// # VM::init();
    ///
    /// VM::set_argv(&["a", "b", "c"]);
    ///
    /// let argv = unsafe { VM::eval_str("ARGV") }.try_convert_to::<Array>().unwrap();
    /// assert_eq!(argv.length(), 3);
    /// assert!(argv.at(0).is_frozen());
    ///
    /// VM::set_argv(&[]);
    /// assert_eq!(unsafe { VM::eval_str("ARGV.size") }.try_convert_to::<rutie::Fixnum>().unwrap().to_i64(), 0);
    /// ```
    pub fn set_argv(arguments: &[&str]) {
        vm::set_argv(arguments)
    }

    /// Sets the script name, `$0` and `$PROGRAM_NAME` (`ruby_script`).
    /// [`VM::init`](#method.init) sets it to the program's name.
    ///
    /// # Panics
    ///
    /// Panics if `name` contains a NUL byte.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// VM::set_script_name("my_tool");
    ///
    /// let name = unsafe { VM::eval_str("$0") }.try_convert_to::<RString>().unwrap();
    /// assert_eq!(name.to_str(), "my_tool");
    ///
    /// // Ruby code may assign it too.
    /// VM::eval("$0 = 'renamed'").unwrap();
    /// ```
    pub fn set_script_name(name: &str) {
        vm::set_script_name(name)
    }

    /// Runs the Ruby script at `path` as the main program, with `arguments`
    /// in `ARGV`: `$0` is set to the script's path as `load` resolves it
    /// (`rb_find_file`, which expands it and, on Windows, uses `/`
    /// separators), so `__FILE__ == $0` holds, and the file is loaded at the
    /// top level (`rb_load_protect`).
    ///
    /// Returns the exception the script raised or failed with: a
    /// `SystemExit` for `exit`, a `LoadError` for a missing file, a
    /// `SyntaxError`. It can be called more than once; each run replaces
    /// `$0` and `ARGV`. Unlike the `ruby` command it does not define `DATA`
    /// (what follows `__END__`) or process command line options.
    ///
    /// Before 0.11 this processed the command line with `ruby_options` and
    /// worked once per process; `VM::init` now does that itself on Ruby 3
    /// (to load the Ruby-defined parts of the core library).
    ///
    /// # Panics
    ///
    /// Panics if `path` or an argument contains a NUL byte.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Exception, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let path = std::env::temp_dir().join(format!("rutie_run_file_{}.rb", std::process::id()));
    /// std::fs::write(&path, "$answer = ARGV.map(&:to_i).sum if __FILE__ == $0").unwrap();
    /// let path = path.to_str().unwrap();
    ///
    /// VM::run_file(path, &["40", "2"]).unwrap();
    ///
    /// let answer = VM::global_get("$answer").try_convert_to::<Fixnum>().unwrap();
    /// assert_eq!(answer.to_i64(), 42);
    ///
    /// // The script's exception is returned.
    /// std::fs::write(path, "exit 3").unwrap();
    /// let error = VM::run_file(path, &[]).unwrap_err();
    /// assert!(error.is_kind_of(&Class::system_exit()));
    ///
    /// std::fs::remove_file(path).unwrap();
    /// ```
    pub fn run_file(path: &str, arguments: &[&str]) -> Result<(), AnyException> {
        // The same string `load` gives the script as `__FILE__`. A missing
        // file keeps `path`; `load` then fails with a `LoadError`.
        let script = Self::find_file(path)
            .map(|found| found.to_string())
            .unwrap_or_else(|| path.to_string());
        vm::set_script_name(&script);
        vm::set_argv(arguments);

        Self::load(path, false)
    }

    /// Returns whether the Ruby VM has been started in this process, by
    /// [`VM::init`](#method.init) or, for an extension, by the `ruby`
    /// process that loaded it. It stays `true` after
    /// [`VM::cleanup`](#method.cleanup).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    ///
    /// assert!(!VM::is_initialized());
    ///
    /// VM::init();
    ///
    /// assert!(VM::is_initialized());
    /// ```
    pub fn is_initialized() -> bool {
        vm::is_initialized()
    }

    /// Returns whether the current native thread is a Ruby thread
    /// (`ruby_native_thread_p`): the thread that started the VM, or one
    /// created by Ruby. Other threads must not call Ruby APIs. Always
    /// `false` before the VM starts.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    ///
    /// assert!(!VM::is_ruby_thread());
    ///
    /// VM::init();
    ///
    /// assert!(VM::is_ruby_thread());
    /// assert!(!std::thread::spawn(VM::is_ruby_thread).join().unwrap());
    /// ```
    pub fn is_ruby_thread() -> bool {
        vm::is_ruby_thread()
    }

    /// Returns whether the machine stack of the current Ruby thread is close
    /// to Ruby's limit (`ruby_stack_check`), past which Ruby raises
    /// `SystemStackError`. Deeply recursive Rust code that calls back into
    /// Ruby can check it and stop early.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    /// # VM::init();
    ///
    /// assert!(!VM::is_stack_near_limit());
    /// ```
    pub fn is_stack_near_limit() -> bool {
        vm::is_stack_near_limit()
    }

    /// Returns how much of the current Ruby thread's machine stack is in use,
    /// in `VALUE`-sized words (`ruby_stack_length`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    /// # VM::init();
    ///
    /// fn deeper(depth: u32) -> usize {
    ///     let padding = [depth as u8; 4096];
    ///
    ///     if depth == 0 {
    ///         VM::stack_length() + (padding[0] as usize)
    ///     } else {
    ///         deeper(depth - 1) + (padding[1] as usize)
    ///     }
    /// }
    ///
    /// let shallow = VM::stack_length();
    ///
    /// assert!(shallow > 0);
    /// assert!(deeper(4) > shallow);
    /// ```
    pub fn stack_length() -> usize {
        vm::stack_length()
    }

    /// Returns up to `limit` frames of the current thread's Ruby stack,
    /// skipping the `start` innermost ones (`rb_profile_frames`), for
    /// sampling profilers. The innermost frame, a method written in Rust
    /// calling this, is frame 0.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Fixnum, Object, RString, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn stack_size() -> Fixnum {
    ///         let frames = VM::profile_frames(0, 100);
    ///         assert_eq!(frames[0].method_name().unwrap().to_str(), "stack_size");
    ///
    ///         // Skipping frames.
    ///         let caller = &VM::profile_frames(1, 1)[0];
    ///         assert_eq!(caller.method_name().unwrap().to_str(), "nested");
    ///
    ///         Fixnum::new(frames.len() as i64)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("stack_size", stack_size);
    ///     });
    ///
    ///     // `stack_size`, `nested` and the top level (and, before Ruby 3.2, a frame for the
    ///     // embedding program).
    ///     let size = VM::eval("def nested = stack_size; nested").unwrap();
    ///     let size = size.try_convert_to::<Fixnum>().unwrap().to_i64();
    ///     assert!(size == 3 || size == 4);
    ///
    ///     assert!(VM::profile_frames(0, 0).is_empty());
    /// }
    /// ```
    pub fn profile_frames(start: usize, limit: usize) -> Vec<ProfileFrame> {
        ProfileFrame::from_frames(debug::profile_frames(start, limit))
    }

    /// Registers `func` to be called while the VM is being freed at the end
    /// of [`VM::cleanup`](#method.cleanup) (`ruby_vm_at_exit`), after every
    /// [`VM::at_exit`](#method.at_exit) handler and finalizer has run.
    ///
    /// Unlike `VM::at_exit`, Ruby can no longer be used when `func` runs, so
    /// it takes a plain function (no captured state) for releasing
    /// resources that live outside Ruby. It receives the VM being freed. A
    /// panic in `func` aborts the process.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{types::VmPointer, VM};
    /// use std::sync::atomic::{AtomicBool, Ordering};
    ///
    /// static FREED: AtomicBool = AtomicBool::new(false);
    ///
    /// extern "C" fn on_vm_exit(vm: VmPointer) {
    ///     assert!(!vm.is_null());
    ///     FREED.store(true, Ordering::SeqCst);
    /// }
    ///
    /// VM::init();
    /// VM::at_vm_exit(on_vm_exit);
    ///
    /// unsafe { VM::cleanup() };
    ///
    /// assert!(FREED.load(Ordering::SeqCst));
    /// ```
    pub fn at_vm_exit(func: extern "C" fn(VmPointer)) {
        vm::at_vm_exit(func)
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
    /// Ruby 3 needs it to tell keywords from a trailing positional `Hash`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    /// # VM::init();
    ///
    /// // Not inside a method called with keywords.
    /// assert!(!VM::is_keyword_given());
    /// ```
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
    /// The OS error is the one Rust's `std::io::Error::last_os_error` sees; on
    /// Windows that is `GetLastError`, mapped to an `errno` the way Ruby maps
    /// it (`ERROR_PATH_NOT_FOUND` raises `Errno::ENOENT`).
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
        let code = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);

        exception::syserr_fail(exception::os_error_to_errno(code), message)
    }

    /// Returns the calling thread's C `errno` (`rb_errno`): the error number
    /// left by the last failed system call.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    /// # VM::init();
    ///
    /// VM::set_errno(2);
    ///
    /// assert_eq!(VM::errno(), 2);
    /// ```
    pub fn errno() -> i32 {
        exception::errno()
    }

    /// Returns `true` if Ruby frees all of its memory when the VM shuts
    /// down (`ruby_free_at_exit_p`), which the `RUBY_FREE_AT_EXIT`
    /// environment variable turns on for memory checkers such as Valgrind.
    /// Ruby 3.4+.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    /// # VM::init();
    ///
    /// let expected = std::env::var_os("RUBY_FREE_AT_EXIT").map_or(false, |value| value == "1");
    ///
    /// assert_eq!(VM::free_at_exit(), expected);
    /// ```
    pub fn free_at_exit() -> bool {
        vm::free_at_exit()
    }

    /// Sets the calling thread's C `errno` (`rb_errno_set`), for example
    /// before [`VM::sys_fail`](#method.sys_fail)-like code that reads it.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    /// # VM::init();
    ///
    /// VM::set_errno(0);
    ///
    /// assert_eq!(VM::errno(), 0);
    /// ```
    pub fn set_errno(errno: i32) {
        exception::set_errno(errno)
    }

    /// Emits a warning of `category` (`rb_category_warn`), printed like
    /// [`VM::warn`](#method.warn) unless warnings are disabled (`$VERBOSE`
    /// is `nil`) or the category is (`Warning[:deprecated] = false`).
    /// `Warning.warn` receives the category as its `category:` keyword.
    ///
    /// `message` is never interpreted as a format string. It is cut at its
    /// first NUL byte, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM, WarningCategory};
    /// # VM::init();
    ///
    /// VM::eval("$warnings = []
    ///           def Warning.warn(message, category: nil); $warnings << \"#{category}: #{message}\"; end
    ///           Warning[:deprecated] = true").unwrap();
    ///
    /// VM::warn_category(WarningCategory::Deprecated, "old_api is deprecated");
    ///
    /// let warnings = VM::eval("$warnings.join").unwrap().try_convert_to::<RString>().unwrap();
    ///
    /// assert!(warnings.to_str().starts_with("deprecated: "));
    /// assert!(warnings.to_str().ends_with("warning: old_api is deprecated\n"));
    /// ```
    pub fn warn_category(category: WarningCategory, message: &str) {
        exception::category_warn(category as i32, message);
    }

    /// Emits a warning of `category` only in verbose mode (`$VERBOSE` is
    /// `true`) and when the category is enabled (`rb_category_warning`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM, WarningCategory};
    /// # VM::init();
    ///
    /// VM::eval("$warnings = []; def Warning.warn(message, category: nil); $warnings << message; end").unwrap();
    ///
    /// VM::eval("$VERBOSE = false").unwrap();
    /// VM::warning_category(WarningCategory::Experimental, "quiet");
    ///
    /// VM::eval("$VERBOSE = true").unwrap();
    /// VM::warning_category(WarningCategory::Experimental, "loud");
    ///
    /// let warnings = VM::eval("$warnings.join").unwrap().try_convert_to::<RString>().unwrap();
    ///
    /// assert!(!warnings.to_str().contains("quiet"));
    /// assert!(warnings.to_str().ends_with("warning: loud\n"));
    /// ```
    pub fn warning_category(category: WarningCategory, message: &str) {
        exception::category_warning(category as i32, message);
    }

    /// Emits a warning located at `file:line`, as Ruby does for its parser's
    /// warnings (`rb_compile_warn`), unless warnings are disabled
    /// (`$VERBOSE` is `nil`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// VM::eval("$warnings = []; def Warning.warn(message, category: nil); $warnings << message; end").unwrap();
    ///
    /// VM::compile_warn("template.erb", 12, "unused variable");
    ///
    /// let warnings = VM::eval("$warnings.join").unwrap().try_convert_to::<RString>().unwrap();
    ///
    /// assert_eq!(warnings.to_str(), "template.erb:12: warning: unused variable\n");
    /// ```
    pub fn compile_warn(file: &str, line: i32, message: &str) {
        exception::compile_warn(file, line, message);
    }

    /// Like [`VM::compile_warn`](#method.compile_warn), but only in verbose
    /// mode (`$VERBOSE` is `true`) (`rb_compile_warning`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// VM::eval("$warnings = []; def Warning.warn(message, category: nil); $warnings << message; end").unwrap();
    ///
    /// VM::eval("$VERBOSE = false").unwrap();
    /// VM::compile_warning("a.rb", 1, "quiet");
    ///
    /// VM::eval("$VERBOSE = true").unwrap();
    /// VM::compile_warning("a.rb", 2, "loud");
    ///
    /// let warnings = VM::eval("$warnings.join").unwrap().try_convert_to::<RString>().unwrap();
    ///
    /// assert_eq!(warnings.to_str(), "a.rb:2: warning: loud\n");
    /// ```
    pub fn compile_warning(file: &str, line: i32, message: &str) {
        exception::compile_warning(file, line, message);
    }

    /// Like [`VM::compile_warn`](#method.compile_warn), for a warning of
    /// `category`, which must be enabled (`rb_category_compile_warn`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM, WarningCategory};
    /// # VM::init();
    ///
    /// VM::eval("$warnings = []
    ///           def Warning.warn(message, category: nil); $warnings << \"#{category}: #{message}\"; end").unwrap();
    ///
    /// VM::compile_warn_category(WarningCategory::Experimental, "query.rb", 3, "pattern matching");
    ///
    /// let warnings = VM::eval("$warnings.join").unwrap().try_convert_to::<RString>().unwrap();
    ///
    /// assert_eq!(warnings.to_str(), "experimental: query.rb:3: warning: pattern matching\n");
    /// ```
    pub fn compile_warn_category(category: WarningCategory, file: &str, line: i32, message: &str) {
        exception::category_compile_warn(category as i32, file, line, message);
    }

    /// Emits a warning made of `message` and the description of the current
    /// OS error (`errno`), in verbose mode (`$VERBOSE` is `true`), like
    /// [`VM::sys_fail`](#method.sys_fail) without raising
    /// (`rb_sys_warning`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// VM::eval("$warnings = []; def Warning.warn(message, category: nil); $warnings << message; end").unwrap();
    /// VM::eval("$VERBOSE = true").unwrap();
    ///
    /// VM::sys_warning("closing the log");
    ///
    /// let warnings = VM::eval("$warnings.join").unwrap().try_convert_to::<RString>().unwrap();
    ///
    /// assert!(warnings.to_str().contains("closing the log"));
    /// ```
    pub fn sys_warning(message: &str) {
        exception::sys_warning(message);
    }

    /// Raises `fatal`, the exception Ruby uses for unrecoverable errors
    /// (`rb_fatal`): `rescue` cannot catch it, so it ends the program unless
    /// Rust code catches it with [`VM::protect`](#method.protect).
    ///
    /// `message` is never interpreted as a format string. It is cut at its
    /// first NUL byte, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Exception, Object, VM};
    /// # VM::init();
    ///
    /// let result = VM::protect(|| VM::raise_fatal("state is corrupt"));
    ///
    /// assert!(result.is_err());
    ///
    /// let error = VM::error_pop().unwrap();
    ///
    /// assert_eq!(error.class().name().unwrap().to_str(), "fatal");
    /// assert_eq!(error.message(), "state is corrupt");
    /// ```
    pub fn raise_fatal(message: &str) -> ! {
        exception::fatal(message)
    }

    /// Raises the `SystemCallError` subclass for the OS error number
    /// `errno` (such as `Errno::ENOENT`), with `message` added to its
    /// message (`rb_syserr_fail_str`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Exception, Object, VM};
    /// # VM::init();
    ///
    /// let enoent = VM::eval("Errno::ENOENT::Errno").unwrap().try_convert_to::<rutie::Fixnum>().unwrap();
    ///
    /// let result = VM::protect(|| VM::raise_syserr(enoent.to_i64() as i32, "config.toml"));
    ///
    /// assert!(result.is_err());
    ///
    /// let error = VM::error_pop().unwrap();
    ///
    /// assert!(Class::from_existing("Errno").get_nested_class("ENOENT").case_equals(&error));
    /// assert_eq!(error.message(), "No such file or directory - config.toml");
    /// ```
    pub fn raise_syserr(errno: i32, message: &str) -> ! {
        exception::syserr_fail_str(errno, message)
    }

    /// Like [`VM::raise_syserr`](#method.raise_syserr), with the exception
    /// extended with `module` so callers can rescue errors of one library
    /// by that module (`rb_mod_syserr_fail_str`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Module, Object, VM};
    /// # VM::init();
    ///
    /// let storage_error = Module::new("StorageError");
    /// let eacces = VM::eval("Errno::EACCES::Errno").unwrap().try_convert_to::<rutie::Fixnum>().unwrap();
    ///
    /// let result = VM::protect(|| {
    ///     VM::raise_syserr_with_module(&storage_error, eacces.to_i64() as i32, "/var/db")
    /// });
    ///
    /// assert!(result.is_err());
    ///
    /// let error = VM::error_pop().unwrap();
    ///
    /// assert!(storage_error.case_equals(&error));
    /// assert!(Class::from_existing("Errno").get_nested_class("EACCES").case_equals(&error));
    /// ```
    pub fn raise_syserr_with_module(module: &Module, errno: i32, message: &str) -> ! {
        exception::mod_syserr_fail_str(module.value(), errno, message)
    }

    /// Raises the `SystemCallError` for `errno` extended with
    /// `IO::WaitReadable` or `IO::WaitWritable` (`IO::EAGAINWaitReadable`
    /// for `EAGAIN`, ...), as non-blocking IO methods do when they would
    /// block (`rb_readwrite_syserr_fail`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, IoWait, Object, VM};
    /// # VM::init();
    ///
    /// let eagain = VM::eval("Errno::EAGAIN::Errno").unwrap().try_convert_to::<rutie::Fixnum>().unwrap();
    ///
    /// let result = VM::protect(|| {
    ///     VM::raise_wait_syserr(IoWait::Readable, eagain.to_i64() as i32, "read would block")
    /// });
    ///
    /// assert!(result.is_err());
    ///
    /// let error = VM::error_pop().unwrap();
    ///
    /// assert!(Class::from_existing("IO").get_nested_class("EAGAINWaitReadable").case_equals(&error));
    /// ```
    pub fn raise_wait_syserr(wait: IoWait, errno: i32, message: &str) -> ! {
        exception::readwrite_syserr_fail(wait as i32, errno, message)
    }

    /// Raises `LoadError` with `message` (`rb_loaderror`), and with `path`
    /// as its `path` when given (`rb_loaderror_with_path`).
    ///
    /// `message` is never interpreted as a format string. It is cut at its
    /// first NUL byte, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Exception, Object, RString, VM};
    /// # VM::init();
    ///
    /// let result = VM::protect(|| VM::raise_load_error("cannot load plugin", Some("plugins/x.so")));
    ///
    /// assert!(result.is_err());
    ///
    /// let error = VM::error_pop().unwrap();
    /// let path = unsafe { error.send("path", &[]) };
    ///
    /// assert!(Class::from_existing("LoadError").case_equals(&error));
    /// assert_eq!(error.message(), "cannot load plugin");
    /// assert_eq!(path.try_convert_to::<RString>().unwrap().to_str(), "plugins/x.so");
    /// ```
    pub fn raise_load_error(message: &str, path: Option<&str>) -> ! {
        match path {
            Some(path) => exception::loaderror_with_path(path, message),
            None => exception::loaderror(message),
        }
    }

    /// Raises `NameError` with `message`, for the name `name`, which becomes
    /// the exception's `name` (`rb_name_error_str`).
    ///
    /// `message` is never interpreted as a format string. It is cut at its
    /// first NUL byte, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Exception, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let result = VM::protect(|| VM::raise_name_error("colour", "unknown setting colour"));
    ///
    /// assert!(result.is_err());
    ///
    /// let error = VM::error_pop().unwrap();
    /// let name = unsafe { error.send("name", &[]) };
    ///
    /// assert!(Class::from_existing("NameError").case_equals(&error));
    /// assert_eq!(error.message(), "unknown setting colour");
    /// assert_eq!(name.try_convert_to::<Symbol>(), Ok(Symbol::new("colour")));
    /// ```
    pub fn raise_name_error(name: &str, message: &str) -> ! {
        exception::name_error(name, message)
    }

    /// Raises `FrozenError` with the message "can't modify frozen `what`"
    /// (`rb_error_frozen`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Exception, Object, VM};
    /// # VM::init();
    ///
    /// let result = VM::protect(|| VM::raise_frozen_error("Config"));
    ///
    /// assert!(result.is_err());
    ///
    /// let error = VM::error_pop().unwrap();
    ///
    /// assert!(Class::from_existing("FrozenError").case_equals(&error));
    /// assert_eq!(error.message(), "can't modify frozen Config");
    /// ```
    pub fn raise_frozen_error(what: &str) -> ! {
        exception::error_frozen(what)
    }

    /// Raises `ArgumentError` with the message "invalid value for
    /// `type_name`: `value`", as `Integer("abc")` does (`rb_invalid_str`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Exception, Object, VM};
    /// # VM::init();
    ///
    /// let result = VM::protect(|| VM::raise_invalid_value("tomorrow", "Date"));
    ///
    /// assert!(result.is_err());
    ///
    /// let error = VM::error_pop().unwrap();
    ///
    /// assert!(Class::argument_error().case_equals(&error));
    /// assert_eq!(error.message(), "invalid value for Date: \"tomorrow\"");
    /// ```
    pub fn raise_invalid_value(value: &str, type_name: &str) -> ! {
        exception::invalid_str(value, type_name)
    }

    /// Raises `TypeError` for `object` not being of the type `expected`,
    /// with Ruby's usual message (`rb_unexpected_type`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Exception, Fixnum, Object, VM};
    /// use rutie::types::ValueType;
    /// # VM::init();
    ///
    /// let result = VM::protect(|| VM::raise_unexpected_type(&Fixnum::new(1), ValueType::RString));
    ///
    /// assert!(result.is_err());
    ///
    /// let error = VM::error_pop().unwrap();
    ///
    /// assert!(Class::from_existing("TypeError").case_equals(&error));
    /// assert_eq!(error.message(), "wrong argument type Integer (expected String)");
    /// ```
    pub fn raise_unexpected_type<T: Object>(object: &T, expected: ValueType) -> ! {
        exception::unexpected_type(object.value(), expected)
    }

    /// Builds the exception `raise(*arguments)` would raise, without
    /// raising it (`rb_make_exception`): a message alone makes a
    /// `RuntimeError`, a class (with an optional message) an instance of it.
    ///
    /// Returns `None` for no arguments, and the `TypeError` or
    /// `ArgumentError` `raise` would raise for invalid ones.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Exception, Object, RString, VM};
    /// # VM::init();
    ///
    /// let error = VM::make_exception(&[RString::new_utf8("boom").into()]).unwrap().unwrap();
    ///
    /// assert!(Class::from_existing("RuntimeError").case_equals(&error));
    /// assert_eq!(error.message(), "boom");
    ///
    /// let error = VM::make_exception(&[
    ///     Class::argument_error().into(),
    ///     RString::new_utf8("bad").into(),
    /// ])
    /// .unwrap()
    /// .unwrap();
    ///
    /// assert!(Class::argument_error().case_equals(&error));
    /// assert!(VM::make_exception(&[]).unwrap().is_none());
    /// assert!(VM::make_exception(&[Class::from_existing("String").into()]).is_err());
    /// ```
    pub fn make_exception(arguments: &[AnyObject]) -> Result<Option<AnyException>, AnyException> {
        let arguments = util::arguments_to_values(arguments);

        vm::protect_value(|| vm::make_exception(&arguments))
            .map(|exception| {
                if exception.is_nil() {
                    None
                } else {
                    Some(AnyException::from(exception))
                }
            })
            .map_err(AnyException::from)
    }

    /// Returns the names of all global variables as an `Array` of `Symbol`s
    /// (Ruby's `global_variables`, `rb_f_global_variables`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Symbol, VM};
    /// # VM::init();
    ///
    /// VM::eval("$rutie_setting = 1").unwrap();
    ///
    /// let names = VM::global_variables();
    ///
    /// assert!(names.into_iter().any(|name| name.try_convert_to::<Symbol>() == Ok(Symbol::new("$rutie_setting"))));
    /// ```
    pub fn global_variables() -> Array {
        Array::from(variable::global_variables())
    }

    /// Makes the global variable `new_name` an alias of `old_name` (both
    /// with their `$`), like Ruby's `alias $new $old`
    /// (`rb_alias_variable`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("$rutie_old = 1").unwrap();
    /// VM::alias_global_variable("$rutie_new", "$rutie_old").unwrap();
    /// VM::eval("$rutie_new = 2").unwrap();
    ///
    /// let old = VM::eval("$rutie_old").unwrap();
    ///
    /// assert_eq!(old.try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// ```
    pub fn alias_global_variable(new_name: &str, old_name: &str) -> Result<(), AnyException> {
        vm::protect_value(|| {
            variable::alias_global_variable(new_name, old_name);

            NilClass::new().value()
        })
        .map(|_| ())
        .map_err(AnyException::from)
    }

    /// Calls `handler` with the new value each time the global variable
    /// `name` (with its `$`) is assigned (Ruby's `trace_var`,
    /// `rb_f_trace_var`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, Proc, VM};
    /// # VM::init();
    ///
    /// let handler = VM::eval("$rutie_seen = []; proc { |value| $rutie_seen << value }")
    ///     .unwrap()
    ///     .try_convert_to::<Proc>()
    ///     .unwrap();
    ///
    /// VM::trace_global_variable("$rutie_level", &handler).unwrap();
    /// VM::eval("$rutie_level = 3").unwrap();
    ///
    /// let seen = VM::eval("$rutie_seen").unwrap().try_convert_to::<Array>().unwrap();
    ///
    /// assert_eq!(seen.at(0).try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
    /// ```
    pub fn trace_global_variable(name: &str, handler: &Proc) -> Result<(), AnyException> {
        let handler = handler.value();

        vm::protect_value(|| {
            variable::trace_var(name, handler);

            NilClass::new().value()
        })
        .map(|_| ())
        .map_err(AnyException::from)
    }

    /// Removes every handler added with
    /// [`VM::trace_global_variable`](#method.trace_global_variable) for the
    /// global variable `name`, returning them (Ruby's `untrace_var`,
    /// `rb_f_untrace_var`), or the `NameError` for an undefined variable.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Proc, VM};
    /// # VM::init();
    ///
    /// let handler = VM::eval("proc { |value| raise 'traced' }").unwrap().try_convert_to::<Proc>().unwrap();
    ///
    /// VM::trace_global_variable("$rutie_mode", &handler).unwrap();
    ///
    /// let removed = VM::untrace_global_variable("$rutie_mode").unwrap();
    ///
    /// assert_eq!(removed.length(), 1);
    /// assert!(VM::eval("$rutie_mode = 1").is_ok());
    /// assert!(VM::untrace_global_variable("$rutie_never_defined").is_err());
    /// ```
    pub fn untrace_global_variable(name: &str) -> Result<Array, AnyException> {
        vm::protect_value(|| variable::untrace_var(name))
            .map(|removed| {
                if removed.is_nil() {
                    Array::new()
                } else {
                    Array::from(removed)
                }
            })
            .map_err(AnyException::from)
    }

    /// Returns the current Ruby backtrace as an `Array` of `String`s
    /// (Ruby's `caller(0)`, `rb_make_backtrace`), empty outside Ruby code.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Array, Class, Object, VM};
    ///
    /// class!(Tracer);
    ///
    /// methods!(
    ///     Tracer,
    ///     rtself,
    ///
    ///     fn tracer_trace() -> Array {
    ///         VM::backtrace()
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::new("Tracer", None).define(|klass| {
    ///         klass.def_self("trace", tracer_trace);
    ///     });
    ///
    ///     let lines = VM::eval("def outer; Tracer.trace; end; outer").unwrap();
    ///     let lines = lines.try_convert_to::<Array>().unwrap();
    ///
    ///     assert!(lines.length() >= 2);
    ///     assert!(lines.at(1).inspect_object().to_str().contains("outer"));
    /// }
    /// ```
    pub fn backtrace() -> Array {
        Array::from(vm::make_backtrace())
    }

    /// Prints the current Ruby backtrace to `$stderr` (`rb_backtrace`), one
    /// `from` line per frame.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    /// # VM::init();
    ///
    /// // Prints nothing outside Ruby code.
    /// VM::print_backtrace();
    /// ```
    pub fn print_backtrace() {
        vm::print_backtrace();
    }

    /// Returns the file and line of the Ruby code running, like
    /// `__FILE__` and `__LINE__` (`rb_sourcefile`, `rb_sourceline`), or
    /// `None` when there is none. Outside any Ruby code, Ruby reports the
    /// script name and line `0`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Fixnum, Object, VM};
    ///
    /// class!(Locator);
    ///
    /// methods!(
    ///     Locator,
    ///     rtself,
    ///
    ///     fn locator_line() -> Fixnum {
    ///         let (_file, line) = VM::source_location().unwrap();
    ///
    ///         Fixnum::new(line as i64)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::new("Locator", None).define(|klass| {
    ///         klass.def_self("line", locator_line);
    ///     });
    ///
    ///     let line = VM::eval("\n\nLocator.line").unwrap();
    ///
    ///     assert_eq!(line.try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));
    /// }
    /// ```
    pub fn source_location() -> Option<(String, i32)> {
        vm::source_location()
    }

    /// Returns the original name of the method running (Ruby's
    /// `__method__`, `rb_frame_this_func`), or `None` outside a method. For
    /// a method called through an alias, this is the name it was defined
    /// with; see [`VM::current_callee_name`](#method.current_callee_name).
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Class, Object, Symbol, VM};
    ///
    /// class!(Named);
    ///
    /// methods!(
    ///     Named,
    ///     rtself,
    ///
    ///     fn named_who() -> Symbol {
    ///         VM::current_method_name().unwrap()
    ///     }
    ///
    ///     fn named_called_as() -> Symbol {
    ///         VM::current_callee_name().unwrap()
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::new("Named", None).define(|klass| {
    ///         klass.def("who", named_who);
    ///         klass.def("called_as", named_called_as);
    ///         klass.define_alias("alias_of_who", "who");
    ///         klass.define_alias("alias_of_called_as", "called_as");
    ///     });
    ///
    ///     let who = VM::eval("Named.new.alias_of_who").unwrap();
    ///     let called_as = VM::eval("Named.new.alias_of_called_as").unwrap();
    ///
    ///     assert_eq!(who.try_convert_to::<Symbol>(), Ok(Symbol::new("who")));
    ///     assert_eq!(called_as.try_convert_to::<Symbol>(), Ok(Symbol::new("alias_of_called_as")));
    /// }
    /// ```
    pub fn current_method_name() -> Option<Symbol> {
        vm::frame_this_func().map(Symbol::from)
    }

    /// Returns the name the method running was called by, which is the
    /// alias for a method called through one (Ruby's `__callee__`,
    /// `rb_frame_callee`), or `None` outside a method.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    /// # VM::init();
    ///
    /// // Not inside a method.
    /// assert!(VM::current_callee_name().is_none());
    /// ```
    pub fn current_callee_name() -> Option<Symbol> {
        vm::frame_callee().map(Symbol::from)
    }

    /// Returns the original name of the method running and the class or
    /// module that defines it (`rb_frame_method_id_and_class`), or `None`
    /// outside a method.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{AnyObject, Array, Class, Object, VM};
    ///
    /// class!(Owner);
    ///
    /// methods!(
    ///     Owner,
    ///     rtself,
    ///
    ///     fn owner_where() -> Array {
    ///         let (name, owner) = VM::current_method().unwrap();
    ///
    ///         Array::new().push(name).push(owner)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::new("Owner", None).define(|klass| {
    ///         klass.def("where", owner_where);
    ///     });
    ///
    ///     let found = VM::eval("class SubOwner < Owner; end; SubOwner.new.where.inspect").unwrap();
    ///
    ///     assert_eq!(found.try_convert_to::<rutie::RString>().unwrap().to_str(), "[:where, Owner]");
    /// }
    /// ```
    pub fn current_method() -> Option<(Symbol, AnyObject)> {
        vm::frame_method_id_and_class()
            .map(|(name, owner)| (Symbol::from(name), AnyObject::from(owner)))
    }

    /// Returns `self` of the Ruby code or method running
    /// (`rb_current_receiver`): inside a method defined with `methods!`, the
    /// object the method was called on; at the top level, `main`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, VM};
    /// # VM::init();
    ///
    /// let main = VM::current_receiver().unwrap();
    ///
    /// assert_eq!(main.as_string().to_str(), "main");
    /// ```
    pub fn current_receiver() -> Result<AnyObject, AnyException> {
        vm::protect_value(vm::current_receiver)
            .map(AnyObject::from)
            .map_err(AnyException::from)
    }

    /// Evaluates `code` with its own anonymous module as the place where
    /// methods and constants it defines go, the way `load(file, true)` runs
    /// a file (`rb_eval_string_wrap`), returning the result or the
    /// exception raised.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Exception, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let result = VM::eval_wrapped("LIMIT = 5; LIMIT * 2").unwrap();
    ///
    /// assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(10)));
    ///
    /// // The constant stayed in the wrapping module.
    /// assert!(VM::eval("defined?(LIMIT)").unwrap().is_nil());
    ///
    /// assert_eq!(VM::eval_wrapped("raise 'no'").unwrap_err().message(), "no");
    /// ```
    pub fn eval_wrapped(code: &str) -> Result<AnyObject, AnyException> {
        vm::eval_string_wrap(code)
            .map(AnyObject::from)
            .map_err(AnyException::from)
    }

    /// Returns `ARGV`, the command line arguments of the Ruby program
    /// (`rb_get_argv`). See [`VM::set_argv`](#method.set_argv).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// VM::set_argv(&["--verbose", "input.txt"]);
    ///
    /// let argv = VM::argv();
    ///
    /// assert_eq!(argv.length(), 2);
    /// assert_eq!(argv.at(1).try_convert_to::<RString>().unwrap().to_str(), "input.txt");
    /// ```
    pub fn argv() -> Array {
        Array::from(vm::argv())
    }

    /// Defines a global function: a private method of `Kernel` callable
    /// from anywhere, like `puts` (`rb_define_global_function`).
    ///
    /// Use `methods!` macro to define a `callback`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Fixnum, Object, VM};
    ///
    /// class!(Kernel);
    ///
    /// methods!(
    ///     Kernel,
    ///     rtself,
    ///
    ///     fn double(number: Fixnum) -> Fixnum {
    ///         Fixnum::new(number.unwrap().to_i64() * 2)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     VM::define_global_function("double", double);
    ///
    ///     let result = VM::eval("class Anywhere; def run; double(21); end; end; Anywhere.new.run").unwrap();
    ///
    ///     assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));
    /// }
    /// ```
    pub fn define_global_function<I: Object, O: Object>(name: &str, callback: Callback<I, O>) {
        class::define_global_function(name, callback);
    }

    /// Yields `values` and `keywords` (as keyword arguments) to the block
    /// of the method running (`rb_yield_values_kw`), like Ruby's
    /// `yield(*values, **keywords)`, and returns the block's result.
    ///
    /// Like [`VM::yield_values`](#method.yield_values), an exception (or a
    /// `break`) in the block is not caught.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{AnyObject, Class, Fixnum, Hash, Object, Symbol, VM};
    ///
    /// class!(Emitter);
    ///
    /// methods!(
    ///     Emitter,
    ///     rtself,
    ///
    ///     fn emitter_emit() -> AnyObject {
    ///         let mut keywords = Hash::new();
    ///         keywords.store(Symbol::new("scale"), Fixnum::new(10));
    ///
    ///         VM::yield_values_with_keywords(&[Fixnum::new(4).into()], keywords)
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::new("Emitter", None).define(|klass| {
    ///         klass.def("emit", emitter_emit);
    ///     });
    ///
    ///     let result = VM::eval("Emitter.new.emit { |value, scale: 1| value * scale }").unwrap();
    ///
    ///     assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(40)));
    /// }
    /// ```
    pub fn yield_values_with_keywords(values: &[AnyObject], keywords: Hash) -> AnyObject {
        let mut values = util::arguments_to_values(values);
        values.push(keywords.value());

        AnyObject::from(vm::yield_values_with_keywords(&values))
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
    /// Keywords follow Ruby 3's rules (`rb_scan_args_kw` with
    /// `RB_SCAN_ARGS_PASS_CALLED_KEYWORDS`): the `:` part is filled only when
    /// the method being run was called with keywords, and a trailing `Hash`
    /// is otherwise a positional argument. To take a trailing `Hash` as the
    /// keywords regardless, as Ruby 2 did, use
    /// [`VM::scan_args_with_keywords`](#method.scan_args_with_keywords).
    ///
    /// Returns an `ArgumentError`, without raising it, when the format is
    /// invalid or the arguments do not fit it.
    ///
    /// This is meant for methods that take a variable number of arguments,
    /// defined with a plain function (see the example and `rutie_callback!`) since
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
    /// // `extern "C"` (`extern "C-unwind"` on Windows), see `rutie_callback!`.
    /// rutie_callback! {
    ///     pub fn greet(argc: Argc, argv: *const AnyObject, _rtself: AnyObject) -> RString {
    ///         let arguments = rutie::util::parse_arguments(argc, argv);
    ///
    ///         let args = VM::scan_args(&arguments, "11*");
    ///
    ///         if let Err(ref error) = args {
    ///             VM::raise_message(error.class(), &error.message());
    ///         }
    ///
    ///         // We can safely unwrap here
    ///         let args = args.unwrap();
    ///
    ///         let name = args.required[0].try_convert_to::<RString>().unwrap();
    ///         let greeting = args.optional[0]
    ///             .as_ref()
    ///             .map(|greeting| greeting.try_convert_to::<RString>().unwrap().to_string())
    ///             .unwrap_or_else(|| "Hello".to_string());
    ///         let rest = args.splat.unwrap().length();
    ///
    ///         RString::new_utf8(&format!("{} {} (+{})", greeting, name.to_str(), rest))
    ///     }
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
    /// use rutie::{AnyObject, Class, Exception, Fixnum, Object, Symbol, VM};
    /// use rutie::types::Argc;
    ///
    /// // def describe(count, **options, &block)
    /// rutie::rutie_callback! {
    ///     pub fn describe(argc: Argc, argv: *const AnyObject, _rtself: AnyObject) -> Fixnum {
    ///         let arguments = rutie::util::parse_arguments(argc, argv);
    ///         let args = match VM::scan_args(&arguments, "1:&") {
    ///             Ok(args) => args,
    ///             Err(error) => VM::raise_message(error.class(), &error.message()),
    ///         };
    ///
    ///         let verbose = args
    ///             .keywords
    ///             .map(|options| options.at(&Symbol::new("verbose")))
    ///             .map(|value| value.try_convert_to::<Fixnum>().unwrap().to_i64())
    ///             .unwrap_or(0);
    ///         let block = if args.block.is_some() { 100 } else { 0 };
    ///
    ///         Fixnum::new(args.required.len() as i64 + verbose * 10 + block)
    ///     }
    /// }
    ///
    /// fn main() {
    ///     # VM::init();
    ///     Class::from_existing("Object").define(|klass| {
    ///         klass.def("describe", describe);
    ///     });
    ///
    ///     let describe = |code| VM::eval(code).unwrap().try_convert_to::<Fixnum>().unwrap().to_i64();
    ///
    ///     assert_eq!(describe("describe(1)"), 1);
    ///     assert_eq!(describe("describe(1, verbose: 2)"), 21);
    ///     assert_eq!(describe("describe(1, verbose: 2) {}"), 121);
    ///
    ///     // Without keywords at the call site a `Hash` is positional.
    ///     let error = VM::eval("describe(1, { verbose: 2 })").unwrap_err();
    ///     assert_eq!(error.message(), "wrong number of arguments (given 2, expected 1)");
    ///
    ///     let error = VM::scan_args(&[], "bogus").unwrap_err();
    ///     assert_eq!(error.message(), "bad scan arg format: bogus");
    /// }
    /// ```
    pub fn scan_args(arguments: &[AnyObject], format: &str) -> Result<ScannedArgs, AnyException> {
        Self::scan_args_kw(arguments, format, class::SCAN_ARGS_PASS_CALLED_KEYWORDS)
    }

    /// Like [`VM::scan_args`](#method.scan_args), but a trailing `Hash`
    /// argument is taken as the keywords (the `:` part of `format`) however
    /// the method was called (`rb_scan_args_kw` with
    /// `RB_SCAN_ARGS_LAST_HASH_KEYWORDS`). That is how Ruby 2 treated it, and
    /// what an explicit list of arguments usually means.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{AnyObject, Fixnum, Hash, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let mut options = Hash::new();
    /// options.store(Symbol::new("mode"), Symbol::new("fast"));
    /// let arguments = [Fixnum::new(1).to_any_object(), options.to_any_object()];
    ///
    /// let args = VM::scan_args_with_keywords(&arguments, "1:").unwrap();
    /// assert_eq!(args.keywords.unwrap().at(&Symbol::new("mode")), Symbol::new("fast").into());
    ///
    /// // `scan_args` follows Ruby 3: this call had no keywords, so the hash
    /// // is a second positional argument.
    /// assert!(VM::scan_args(&arguments, "1:").is_err());
    /// ```
    pub fn scan_args_with_keywords(
        arguments: &[AnyObject],
        format: &str,
    ) -> Result<ScannedArgs, AnyException> {
        Self::scan_args_kw(arguments, format, class::SCAN_ARGS_LAST_HASH_KEYWORDS)
    }

    fn scan_args_kw(
        arguments: &[AnyObject],
        format: &str,
        kw_flag: i32,
    ) -> Result<ScannedArgs, AnyException> {
        let spec = ScanArgsFormat::parse(format)
            .map_err(|message| AnyException::new("ArgumentError", Some(&message)))?;

        let arguments = util::arguments_to_values(arguments);
        let format = util::str_to_cstring(format);
        let mut out = [NilClass::new().value(); class::SCAN_ARGS_MAX_VARIABLES];
        let mut positional = 0;

        vm::protect_value(|| {
            positional = class::scan_args(&arguments, &format, &mut out, kw_flag);

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
    /// # VM::init_loadpath();
    /// # VM::require("stringio");
    /// // Capture `$stdout` to check what was printed.
    /// VM::eval("$stdout = StringIO.new").unwrap();
    ///
    /// VM::p(&Fixnum::new(42)); // prints "42"
    ///
    /// let printed = VM::eval("out = $stdout.string; $stdout = STDOUT; out").unwrap();
    /// assert_eq!(printed.try_convert_to::<rutie::RString>().unwrap().to_str(), "42\n");
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
    /// let path = std::env::temp_dir().join(format!("rutie_vm_load_example_{}.rb", std::process::id()));
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
    /// with the same name. Ruby keeps the name's memory, so each call leaks
    /// one small string for the life of the process.
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
    /// let dir = std::env::temp_dir().join(format!("rutie_load_path_example_{}", std::process::id()));
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

    /// Looks up the C symbol `symbol` (such as a function) in the native
    /// extension loaded for `feature` (`rb_ext_resolve_symbol`), so one
    /// extension can call functions another exports. Returns its address, or
    /// `None` if the feature is not loaded, is not a native extension, or
    /// does not define the symbol.
    ///
    /// Casting the address to the right function type, and calling it, is up
    /// to the caller (and `unsafe`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    /// # VM::init();
    /// # VM::init_loadpath();
    ///
    /// assert!(VM::ext_resolve_symbol("etc", "Init_etc").is_none());
    ///
    /// VM::require("etc");
    ///
    /// assert!(VM::ext_resolve_symbol("etc", "Init_etc").is_some());
    /// assert!(VM::ext_resolve_symbol("etc", "rutie_no_such_symbol").is_none());
    /// ```
    pub fn ext_resolve_symbol(feature: &str, symbol: &str) -> Option<*mut c_void> {
        let (feature, symbol) = match (CString::new(feature), CString::new(symbol)) {
            (Ok(feature), Ok(symbol)) => (feature, symbol),
            _ => return None,
        };
        let address = vm::ext_resolve_symbol(&feature, &symbol);

        if address.is_null() {
            None
        } else {
            Some(address)
        }
    }

    /// Invalidates Ruby's inline caches of constant lookups for the constant
    /// name `name` (`rb_clear_constant_cache_for_id`), so code using a
    /// constant of that name looks it up again. Ruby does this itself when a
    /// constant is set or removed; an extension only needs it after changing
    /// what a constant lookup finds by other means.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// VM::eval("LIMIT = 5; def limit = LIMIT").unwrap();
    /// VM::eval("limit").unwrap();
    ///
    /// VM::clear_constant_cache_for("LIMIT");
    ///
    /// assert_eq!(VM::eval("limit").unwrap().try_convert_to::<Fixnum>(), Ok(Fixnum::new(5)));
    /// ```
    pub fn clear_constant_cache_for(name: &str) {
        vm::clear_constant_cache_for(name)
    }

    /// Returns `$_`, the last line read by `gets` in the calling Ruby frame
    /// (`rb_lastline_get`); `nil` when unset.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// VM::set_last_line(&RString::new_utf8("a line\n"));
    ///
    /// assert_eq!(VM::last_line().try_convert_to::<RString>().unwrap().to_str(), "a line\n");
    /// ```
    pub fn last_line() -> AnyObject {
        AnyObject::from(symbol::last_line())
    }

    /// Sets `$_` for the calling Ruby frame (`rb_lastline_set`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{NilClass, Object, RString, VM};
    /// # VM::init();
    ///
    /// VM::set_last_line(&RString::new_utf8("x"));
    /// VM::set_last_line(&NilClass::new());
    ///
    /// assert!(VM::last_line().is_nil());
    /// ```
    pub fn set_last_line<T: Object>(value: &T) {
        symbol::set_last_line(value.value())
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
    ///             VM::raise_message(error.class(), &error.message());
    ///         }
    ///         if let Err(ref error) = b {
    ///             VM::raise_message(error.class(), &error.message());
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
    ///             VM::raise_message(error.class(), &error.message());
    ///         }
    ///         if let Err(ref error) = b {
    ///             VM::raise_message(error.class(), &error.message());
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

    /// Calls the superclass's version of the method running with
    /// `arguments` and `keywords` as keyword arguments, like Ruby's
    /// `super(*arguments, **keywords)` (`rb_call_super_kw`).
    ///
    /// # Safety
    ///
    /// Like [`VM::call_super`](#method.call_super), an exception raised by
    /// the superclass method is not caught.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{AnyObject, Class, Fixnum, Hash, Object, Symbol, VM};
    ///
    /// class!(Rounder);
    ///
    /// methods!(
    ///     Rounder,
    ///     rtself,
    ///
    ///     fn rounder_round(number: Fixnum) -> AnyObject {
    ///         let mut keywords = Hash::new();
    ///         keywords.store(Symbol::new("digits"), Fixnum::new(2));
    ///
    ///         unsafe { VM::call_super_with_keywords(&[number.unwrap().into()], keywords) }
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     VM::eval("class BaseRounder; def round(number, digits: 0); digits; end; end").unwrap();
    ///
    ///     Class::new("Rounder", Some(&Class::from_existing("BaseRounder"))).define(|klass| {
    ///         klass.def("round", rounder_round);
    ///     });
    ///
    ///     let digits = VM::eval("Rounder.new.round(7)").unwrap();
    ///
    ///     assert_eq!(digits.try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// }
    /// ```
    pub unsafe fn call_super_with_keywords(arguments: &[AnyObject], keywords: Hash) -> AnyObject {
        let mut arguments = util::arguments_to_values(arguments);
        arguments.push(keywords.value());

        AnyObject::from(vm::call_super_with_keywords(&arguments))
    }

    /// Reads the next line like Ruby's `Kernel#gets` (`rb_gets`): from the
    /// files named in `ARGV`, or `$stdin` when there are none (`ARGF`).
    /// Returns `None` at the end, and the line is also stored in `$_`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// let path = std::env::temp_dir().join(format!("rutie_gets_example_{}.txt", std::process::id()));
    /// std::fs::write(&path, "first\nsecond\n").unwrap();
    ///
    /// // Read the file through ARGF.
    /// VM::global_set("$rutie_path", RString::new_utf8(path.to_str().unwrap()));
    /// VM::eval("ARGV.replace([$rutie_path])").unwrap();
    ///
    /// assert_eq!(VM::gets().unwrap().unwrap().to_str(), "first\n");
    /// assert_eq!(VM::eval("$_").unwrap().try_convert_to::<RString>().unwrap().to_str(), "first\n");
    /// assert_eq!(VM::gets().unwrap().unwrap().to_str(), "second\n");
    /// # std::fs::remove_file(path).unwrap();
    /// ```
    pub fn gets() -> Result<Option<RString>, AnyException> {
        vm::protect_value(io::kernel_gets)
            .map(|line| {
                if line.is_nil() {
                    None
                } else {
                    Some(RString::from(line))
                }
            })
            .map_err(AnyException::from)
    }

    /// Raises `EOFError` with Ruby's usual message (`rb_eof_error`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Class, Exception, Object, VM};
    /// # VM::init();
    ///
    /// let result = VM::protect(|| VM::raise_eof_error());
    ///
    /// assert!(result.is_err());
    ///
    /// let error = VM::error_pop().unwrap();
    ///
    /// assert!(Class::from_existing("EOFError").case_equals(&error));
    /// assert_eq!(error.message(), "end of file reached");
    /// ```
    pub fn raise_eof_error() -> ! {
        io::eof_error()
    }

    /// Writes `message` to `$stderr` as it is, as Ruby does for its own
    /// error reports (`rb_write_error2`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, RString, VM};
    /// # VM::init();
    ///
    /// VM::eval("require 'stringio'; $saved_stderr = $stderr; $stderr = StringIO.new").unwrap();
    ///
    /// VM::write_error("something failed\n");
    ///
    /// let written = VM::eval("s = $stderr.string; $stderr = $saved_stderr; s").unwrap();
    /// assert_eq!(written.try_convert_to::<RString>().unwrap().to_str(), "something failed\n");
    /// ```
    pub fn write_error(message: &str) {
        io::write_error(message);
    }

    /// Prints Ruby's version, as `ruby -v` does, to the process's standard
    /// output (not `$stdout`) (`ruby_show_version`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    /// # VM::init();
    ///
    /// // Prints something like "ruby 4.0.7 (2026-...) +PRISM [x86_64-linux]".
    /// VM::show_version();
    /// ```
    pub fn show_version() {
        vm::show_version();
    }

    /// Prints Ruby's copyright notice, as `ruby --copyright` does, to the
    /// process's standard output (`ruby_show_copyright`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::VM;
    /// # VM::init();
    ///
    /// // Prints "ruby - Copyright (C) 1993-... Yukihiro Matsumoto".
    /// VM::show_copyright();
    /// ```
    pub fn show_copyright() {
        vm::show_copyright();
    }

    /// Looks for `name` with each of `extensions` (such as `".rb"`) in
    /// `$LOAD_PATH`, as `require` does (`rb_find_file_ext`). Returns the
    /// full path and the index of the extension that was found, `None` when
    /// nothing was, or the error for an invalid name.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RString, VM};
    /// # VM::init();
    ///
    /// let directory = std::env::temp_dir().join(format!("rutie_find_ext_example_{}", std::process::id()));
    /// std::fs::create_dir_all(&directory).unwrap();
    /// std::fs::write(directory.join("rutie_feature.rb"), "").unwrap();
    /// VM::add_load_path(directory.to_str().unwrap());
    ///
    /// let (path, index) = VM::find_file_ext("rutie_feature", &[".so", ".rb"]).unwrap().unwrap();
    ///
    /// assert!(path.to_str().ends_with("rutie_feature.rb"));
    /// assert_eq!(index, 1);
    /// assert_eq!(VM::find_file_ext("rutie_feature", &[".so"]).unwrap(), None);
    /// assert!(VM::find_file_ext("nul\0", &[".rb"]).is_err());
    /// # std::fs::remove_dir_all(directory).unwrap();
    /// ```
    pub fn find_file_ext(
        name: &str,
        extensions: &[&str],
    ) -> Result<Option<(RString, usize)>, AnyException> {
        let extensions = extensions
            .iter()
            .map(|extension| std::ffi::CString::new(*extension))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| {
                AnyException::new("ArgumentError", Some("extension contains a NUL byte"))
            })?;
        let name = RString::new_utf8(name);
        let mut found = None;

        vm::protect_value(|| {
            found = io::find_file_ext(name.value(), &extensions);

            NilClass::new().value()
        })
        .map(|_| found.map(|(path, index)| (RString::from(path), index)))
        .map_err(AnyException::from)
    }

    /// Calls `func` with `false`, unless the same `object` is already being
    /// processed by a `VM::exec_recursive` call further up the stack (of
    /// the current thread): then `func` gets `true`, to stop the recursion
    /// (`rb_exec_recursive`). This is how Ruby's own `inspect`, `hash` and
    /// `==` deal with structures that contain themselves.
    ///
    /// Returns what `func` returns, or the exception it raised.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{AnyObject, Array, Object, RString, VM};
    /// # VM::init();
    ///
    /// fn describe(object: &AnyObject) -> String {
    ///     let array = match object.try_convert_to::<Array>() {
    ///         Ok(array) => array,
    ///         Err(_) => return object.inspect_object().to_string(),
    ///     };
    ///
    ///     let text = VM::exec_recursive(&array, |recursive| {
    ///         if recursive {
    ///             return RString::new_utf8("[...]");
    ///         }
    ///
    ///         let items: Vec<String> = (0..array.length() as i64).map(|i| describe(&array.at(i))).collect();
    ///
    ///         RString::new_utf8(&format!("[{}]", items.join(", ")))
    ///     });
    ///
    ///     text.unwrap().try_convert_to::<RString>().unwrap().to_string()
    /// }
    ///
    /// let nested = VM::eval("a = [1, [2]]; a << a; a").unwrap();
    ///
    /// assert_eq!(describe(&nested), "[1, [2], [...]]");
    /// ```
    pub fn exec_recursive<T, F, R>(object: &T, mut func: F) -> Result<AnyObject, AnyException>
    where
        T: Object,
        F: FnMut(bool) -> R,
        R: Object,
    {
        let object = object.value();

        vm::protect_exception(|| {
            vm::exec_recursive(object, None, false, |recursive| func(recursive).value())
        })
        .map(AnyObject::from)
        .map_err(AnyException::from)
    }

    /// Like [`VM::exec_recursive`](#method.exec_recursive), but recursion
    /// is detected on the pair of `object` and `paired`, as when comparing
    /// two structures with each other (`rb_exec_recursive_paired`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Boolean, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let (a, b) = (Fixnum::new(1), Fixnum::new(2));
    ///
    /// let result = VM::exec_recursive_paired(&a, &b, |outer| {
    ///     assert!(!outer);
    ///
    ///     // The same pair again is a recursion; another pair is not.
    ///     let same = VM::exec_recursive_paired(&a, &b, |recursive| Boolean::new(recursive)).unwrap();
    ///     let other = VM::exec_recursive_paired(&a, &a, |recursive| Boolean::new(recursive)).unwrap();
    ///
    ///     Boolean::new(same.value().is_true() && !other.value().is_true())
    /// });
    ///
    /// assert!(result.unwrap().value().is_true());
    /// ```
    pub fn exec_recursive_paired<T, P, F, R>(
        object: &T,
        paired: &P,
        mut func: F,
    ) -> Result<AnyObject, AnyException>
    where
        T: Object,
        P: Object,
        F: FnMut(bool) -> R,
        R: Object,
    {
        let (object, paired) = (object.value(), paired.value());

        vm::protect_exception(|| {
            vm::exec_recursive(object, Some(paired), false, |recursive| {
                func(recursive).value()
            })
        })
        .map(AnyObject::from)
        .map_err(AnyException::from)
    }

    /// Like [`VM::exec_recursive`](#method.exec_recursive), but when the
    /// recursion is found the whole computation starts over: the nested
    /// calls are abandoned (with a Ruby `throw`) and the outermost call
    /// runs `func` again with `true` (`rb_exec_recursive_outer`). Ruby's
    /// `Array#hash` uses it so that a recursive array has one hash.
    ///
    /// Abandoning the nested calls does not run destructors of the values
    /// they own (they leak), as when Ruby raises through Rust code.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, VM};
    /// use std::cell::Cell;
    /// # VM::init();
    ///
    /// let object = Fixnum::new(7);
    /// let calls = Cell::new(Vec::new());
    ///
    /// let result = VM::exec_recursive_outer(&object, |recursive| {
    ///     let mut seen = calls.take();
    ///     seen.push(recursive);
    ///     calls.set(seen);
    ///
    ///     if recursive {
    ///         return Fixnum::new(0);
    ///     }
    ///
    ///     // Found the recursion: this never returns.
    ///     VM::exec_recursive_outer(&object, |_| Fixnum::new(1)).unwrap();
    ///     unreachable!();
    /// });
    ///
    /// assert_eq!(result.unwrap().try_convert_to::<Fixnum>(), Ok(Fixnum::new(0)));
    /// assert_eq!(calls.take(), [false, true]);
    /// ```
    pub fn exec_recursive_outer<T, F, R>(object: &T, mut func: F) -> Result<AnyObject, AnyException>
    where
        T: Object,
        F: FnMut(bool) -> R,
        R: Object,
    {
        let object = object.value();

        vm::protect_exception(|| {
            vm::exec_recursive(object, None, true, |recursive| func(recursive).value())
        })
        .map(AnyObject::from)
        .map_err(AnyException::from)
    }

    /// [`VM::exec_recursive_outer`](#method.exec_recursive_outer) for a
    /// pair of objects (`rb_exec_recursive_paired_outer`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let (a, b) = (Symbol::new("a"), Symbol::new("b"));
    ///
    /// let result = VM::exec_recursive_paired_outer(&a, &b, |recursive| {
    ///     if !recursive {
    ///         VM::exec_recursive_paired_outer(&a, &b, |_| Fixnum::new(1)).unwrap();
    ///     }
    ///
    ///     Fixnum::new(2)
    /// });
    ///
    /// assert_eq!(result.unwrap().try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// ```
    pub fn exec_recursive_paired_outer<T, P, F, R>(
        object: &T,
        paired: &P,
        mut func: F,
    ) -> Result<AnyObject, AnyException>
    where
        T: Object,
        P: Object,
        F: FnMut(bool) -> R,
        R: Object,
    {
        let (object, paired) = (object.value(), paired.value());

        vm::protect_exception(|| {
            vm::exec_recursive(object, Some(paired), true, |recursive| {
                func(recursive).value()
            })
        })
        .map(AnyObject::from)
        .map_err(AnyException::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        types::ValueType, AnyObject, Array, Boolean, Class, Exception, Fixnum, Hash, IoWait,
        Module, NilClass, Object, Proc, RString, Symbol, WarningCategory, VM,
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
    fn test_raise_message_and_raise_format() {
        crate::on_ruby_thread(|| {
            let result = VM::protect(|| {
                VM::raise_message(Class::from_existing("RuntimeError"), "100%s %d %n done");
            });

            assert!(result.is_err());
            assert_eq!(VM::error_pop().unwrap().message(), "100%s %d %n done");

            // `raise` keeps C's printf semantics: `%%` is a literal `%`.
            let result = VM::protect(|| {
                VM::raise(Class::from_existing("RuntimeError"), "100%% done");

                NilClass::new().into()
            });

            assert!(result.is_err());
            assert_eq!(VM::error_pop().unwrap().message(), "100% done");
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

            let with_keywords = [arguments[0].clone(), keywords.to_any_object()];

            // Ruby 3: not called with keywords, so the hash is positional.
            assert!(VM::scan_args(&with_keywords, "1:").is_err());

            let args = VM::scan_args_with_keywords(&with_keywords, "1:").unwrap();
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
            rutie_callback! {
                fn rutie_scan_block(
                    argc: crate::types::Argc,
                    argv: *const AnyObject,
                    _rtself: AnyObject,
                ) -> AnyObject {
                    let arguments = crate::util::parse_arguments(argc, argv);
                    let args = VM::scan_args(&arguments, "1&");

                    if let Err(ref error) = args {
                        VM::raise_message(error.class(), &error.message());
                    }

                    let args = args.unwrap();

                    match args.block {
                        Some(block) => block.call(&args.required),
                        None => Symbol::new("no_block").into(),
                    }
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

    #[test]
    fn test_is_keyword_given() {
        crate::on_ruby_thread(|| {
            rutie_callback! {
                fn rutie_keyword_given(
                    _argc: crate::types::Argc,
                    _argv: *const AnyObject,
                    _rtself: AnyObject,
                ) -> crate::Boolean {
                    crate::Boolean::new(VM::is_keyword_given())
                }
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

            let message = VM::get_kwargs(Some(&keywords), &["a", "z"], &[], true)
                .unwrap_err()
                .message();
            assert_eq!(message, "missing keyword: :z");

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
            let dir = std::env::temp_dir()
                .join(format!("rutie_vm_load_unit_test_{}", std::process::id()));
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

    #[test]
    fn test_lifecycle_queries() {
        crate::on_ruby_thread(|| {
            // The test VM is already running, so these are no-ops.
            VM::init();
            assert_eq!(VM::try_init(), Ok(()));

            assert!(VM::is_initialized());
            assert!(VM::is_ruby_thread());
            assert!(!std::thread::spawn(VM::is_ruby_thread).join().unwrap());

            assert!(!VM::is_stack_near_limit());
            assert!(VM::stack_length() > 0);
        });
    }

    #[test]
    fn test_argv_and_script_name() {
        crate::on_ruby_thread(|| {
            VM::set_argv(&["one", "two"]);

            let argv = VM::eval("ARGV").unwrap().try_convert_to::<Array>().unwrap();
            assert_eq!(argv.length(), 2);
            assert_eq!(
                argv.at(0).try_convert_to::<RString>().unwrap().to_str(),
                "one"
            );
            assert!(argv.at(1).is_frozen());

            VM::set_argv(&[]);
            assert_eq!(VM::eval("ARGV.empty?").unwrap().value().is_true(), true);

            VM::set_script_name("rutie_unit_test");
            let name = VM::eval("$PROGRAM_NAME")
                .unwrap()
                .try_convert_to::<RString>()
                .unwrap();
            assert_eq!(name.to_str(), "rutie_unit_test");

            // `VM::init` processed a command line, so `$0` can be assigned.
            VM::eval("$0 = 'renamed'").unwrap();
            let name = VM::eval("$0").unwrap().try_convert_to::<RString>().unwrap();
            assert_eq!(name.to_str(), "renamed");
        });
    }

    crate::class!(RutieVmBlocks);

    crate::methods!(
        RutieVmBlocks,
        rtself,
        fn rutie_vm_block_given() -> crate::Boolean {
            crate::Boolean::new(VM::is_block_given())
        },
        fn rutie_vm_yield_one(value: Fixnum) -> AnyObject {
            VM::yield_object(value.unwrap())
        },
        fn rutie_vm_yield_splat(values: Array) -> AnyObject {
            VM::yield_splat(values.unwrap())
        },
        fn rutie_vm_block_proc() -> crate::Proc {
            VM::block_proc()
        },
        fn rutie_vm_need_block() -> NilClass {
            VM::need_block();
            NilClass::new()
        },
        fn rutie_vm_super_greet(name: RString) -> RString {
            let from_parent = unsafe { VM::call_super(&[name.unwrap().into()]) };
            let text = from_parent.try_convert_to::<RString>().unwrap().to_string();
            RString::new_utf8(&format!("{}!", text))
        }
    );

    #[test]
    fn test_blocks_yield_and_super() {
        crate::on_ruby_thread(|| {
            VM::eval("class RutieVmBlocksParent; def greet(name); \"hi #{name}\"; end; end")
                .unwrap();
            let parent = Class::from_existing("RutieVmBlocksParent");
            let mut class = Class::new("RutieVmBlocks", Some(&parent));
            class.define_method("given?", rutie_vm_block_given);
            class.define_method("yield_one", rutie_vm_yield_one);
            class.define_method("yield_splat", rutie_vm_yield_splat);
            class.define_method("capture", rutie_vm_block_proc);
            class.define_method("need_block", rutie_vm_need_block);
            class.define_method("greet", rutie_vm_super_greet);

            let eval = |code: &str| VM::eval(code).unwrap();

            assert!(eval("RutieVmBlocks.new.given? { }").value().is_true());
            assert!(eval("RutieVmBlocks.new.given?").value().is_false());

            let doubled = eval("RutieVmBlocks.new.yield_one(21) { |x| x * 2 }");
            assert_eq!(doubled.try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));

            let summed = eval("RutieVmBlocks.new.yield_splat([1, 2, 3]) { |a, b, c| a + b + c }");
            assert_eq!(summed.try_convert_to::<Fixnum>(), Ok(Fixnum::new(6)));

            let called = eval("RutieVmBlocks.new.capture { |x| x + 1 }.call(1)");
            assert_eq!(called.try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));

            let error = VM::eval("RutieVmBlocks.new.need_block").unwrap_err();
            assert!(Class::from_existing("LocalJumpError").case_equals(&error));
            assert!(VM::eval("RutieVmBlocks.new.need_block { }").is_ok());

            let greeting = eval("RutieVmBlocks.new.greet('bob')");
            assert_eq!(
                greeting.try_convert_to::<RString>().unwrap().to_str(),
                "hi bob!"
            );

            // `iter_break` leaves the iterating method, which returns nil.
            let array: Array = (1..=5).map(|n| Fixnum::new(n).to_any_object()).collect();
            let seen = Cell::new(0);
            let result = array
                .protect_send_with_block("each", &[], |values| {
                    seen.set(seen.get() + 1);
                    if values[0].try_convert_to::<Fixnum>().unwrap().to_i64() == 2 {
                        unsafe { VM::iter_break() }
                    }
                    NilClass::new().into()
                })
                .unwrap();
            assert!(result.is_nil());
            assert_eq!(seen.get(), 2);
        });
    }

    #[test]
    fn test_errors_exit_and_process_state() {
        crate::on_ruby_thread(|| {
            // `protect` leaves `$!` set until it is cleared or popped.
            assert!(VM::protect(|| unsafe { VM::eval_str("raise 'kept'") }).is_err());
            assert_eq!(VM::error_info().unwrap().message(), "kept");
            VM::clear_error_info();
            assert!(VM::error_info().is_err());

            let result = VM::protect(|| VM::raise_interrupt());
            assert!(result.is_err());
            assert!(Class::interrupt().case_equals(&VM::error_pop().unwrap()));

            let result = VM::protect(|| {
                let _ = std::fs::File::open("/rutie/does/not/exist");
                VM::sys_fail("opening");
            });
            assert!(result.is_err());
            let error = VM::error_pop().unwrap();
            assert!(Class::from_existing("SystemCallError").case_equals(&error));
            assert!(error.message().contains("opening"));

            // Inside `protect`, `exit` and `abort` raise SystemExit.
            let result = VM::protect(|| {
                VM::exit(3);
                NilClass::new().into()
            });
            assert!(result.is_err());
            let exit = VM::error_pop().unwrap();
            assert!(Class::system_exit().case_equals(&exit));
            let status = unsafe { exit.send("status", &[]) };
            assert_eq!(status.try_convert_to::<Fixnum>(), Ok(Fixnum::new(3)));

            let result = VM::protect(|| {
                unsafe { VM::abort(&[RString::new_utf8("rutie abort test (expected)").into()]) };
                NilClass::new().into()
            });
            assert!(result.is_err());
            let abort = VM::error_pop().unwrap();
            assert_eq!(abort.message(), "rutie abort test (expected)");

            // Windows has no SIGUSR2.
            let signal = if cfg!(windows) { "TERM" } else { "USR2" };
            let previous = VM::trap(&[
                RString::new_utf8(signal).into(),
                RString::new_utf8("IGNORE").into(),
            ])
            .unwrap();
            assert!(VM::trap(&[RString::new_utf8(signal).into(), previous]).is_ok());

            VM::init_with_args(&["from", "init"]);
            let argv = VM::eval("ARGV.join(' ')").unwrap();
            assert_eq!(
                argv.try_convert_to::<RString>().unwrap().to_str(),
                "from init"
            );
            VM::set_argv(&[]);

            // `p` returns nothing; it prints the inspected object.
            VM::p(&Symbol::new("rutie_vm_p_test"));
        });
    }

    #[test]
    fn test_provide_keeps_feature_name() {
        crate::on_ruby_thread(|| {
            // A name in a heap buffer that is reused right after the call.
            let name = format!("rutie_provide_regression_{}.so", std::process::id());
            VM::provide(&name);

            // Churn the Rust heap so a freed buffer would be overwritten.
            let noise: Vec<String> = (0..2000).map(|i| "x".repeat(i % 64 + 1)).collect();
            drop(noise);
            crate::GC::start();

            let features = VM::global_get("$LOADED_FEATURES")
                .try_convert_to::<Array>()
                .unwrap();
            assert!(features.includes(&RString::new_utf8(&name)));
            assert!(VM::is_provided(&name));
        });
    }

    crate::methods!(
        Fixnum,
        rtself,
        fn rutie_ractor_default() -> Fixnum {
            Fixnum::new(rtself.to_i64() + 1)
        },
        fn rutie_ractor_opt_in() -> Fixnum {
            Fixnum::new(rtself.to_i64() + 2)
        }
    );

    #[test]
    fn test_ractor_safety() {
        crate::on_ruby_thread(|| {
            Class::from_existing("Integer").define(|klass| {
                // `VM::init` leaves methods Ractor-unsafe.
                klass.def("rutie_ractor_default", rutie_ractor_default);

                unsafe { VM::ext_ractor_safe(true) };
                klass.def("rutie_ractor_opt_in", rutie_ractor_opt_in);
                VM::ext_ractor_unsafe();
            });

            let in_ractor = |method: &str| {
                let code = format!(
                    "Warning[:experimental] = false
                     Ractor.new {{ begin; 1.{}.to_s; rescue => e; e.class.name; end }}.value",
                    method
                );
                RString::from(VM::eval(&code).unwrap().value()).to_string()
            };

            assert_eq!(in_ractor("rutie_ractor_default"), "Ractor::UnsafeError");
            assert_eq!(in_ractor("rutie_ractor_opt_in"), "3");

            let main = VM::eval("1.rutie_ractor_default").unwrap();
            assert_eq!(main.try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
        });
    }

    #[test]
    fn test_run_file() {
        crate::on_ruby_thread(|| {
            let dir = std::env::temp_dir().join(format!("rutie_run_file_{}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            let script = dir.join("script.rb");
            std::fs::write(&script, "$rutie_run_file = [$0, ARGV.dup, __FILE__]").unwrap();
            let path = script.to_str().unwrap();

            let original_name = RString::from(VM::eval("$0").unwrap().value()).to_string();

            // `$0` is the path as `load` resolves it (`/` separators on
            // Windows), the same string as the script's `__FILE__`.
            VM::run_file(path, &["a", "b"]).unwrap();
            let seen = Array::from(VM::eval("$rutie_run_file").unwrap().value());
            let program = RString::from(seen.at(0).value()).to_string();
            assert_eq!(program, RString::from(seen.at(2).value()).to_string());
            assert!(program.ends_with("script.rb"));
            assert_eq!(Array::from(seen.at(1).value()).length(), 2);

            // Not once per process: a second run works.
            VM::run_file(path, &[]).unwrap();
            let seen = Array::from(VM::eval("$rutie_run_file").unwrap().value());
            assert_eq!(Array::from(seen.at(1).value()).length(), 0);

            // Errors are returned; `exit` is a `SystemExit`.
            std::fs::write(&script, "raise ArgumentError, 'bad'").unwrap();
            let error = VM::run_file(path, &[]).unwrap_err();
            assert!(Class::from_existing("ArgumentError").case_equals(&error));

            std::fs::write(&script, "exit 4").unwrap();
            let error = VM::run_file(path, &[]).unwrap_err();
            assert!(Class::from_existing("SystemExit").case_equals(&error));

            assert!(VM::run_file(dir.join("missing.rb").to_str().unwrap(), &[]).is_err());

            VM::set_script_name(&original_name);
            VM::set_argv(&[]);
            std::fs::remove_dir_all(&dir).unwrap();
        });
    }

    #[test]
    fn test_errno_and_ext_resolve_symbol() {
        crate::on_ruby_thread(|| {
            VM::set_errno(13);
            assert_eq!(VM::errno(), 13);

            // The same `errno` Rust's standard library reads.
            #[cfg(unix)]
            assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(13));

            VM::set_errno(0);
            assert_eq!(VM::errno(), 0);

            assert!(VM::ext_resolve_symbol("rutie_no_such_feature", "Init_x").is_none());
            assert!(VM::ext_resolve_symbol("et\0c", "Init_etc").is_none());
            assert!(VM::ext_resolve_symbol("etc", "Init\0etc").is_none());
        });
    }

    #[test]
    fn test_clear_constant_cache_for() {
        crate::on_ruby_thread(|| {
            VM::eval("RUTIE_CACHED = 1; def rutie_cached = RUTIE_CACHED; rutie_cached").unwrap();

            VM::clear_constant_cache_for("RUTIE_CACHED");
            VM::clear_constant_cache_for("RUTIE_NEVER_DEFINED_CONSTANT");

            let value = VM::eval("rutie_cached").unwrap();
            assert_eq!(value.try_convert_to::<Fixnum>(), Ok(Fixnum::new(1)));
        });
    }

    #[test]
    fn test_category_and_compile_warnings() {
        crate::on_ruby_thread(|| {
            VM::eval(
                "$rutie_cat_warnings = []
                 $rutie_cat_verbose = $VERBOSE
                 $rutie_cat_deprecated = Warning[:deprecated]
                 $rutie_cat_experimental = Warning[:experimental]
                 Warning.singleton_class.send(:alias_method, :rutie_cat_original_warn, :warn)
                 def Warning.warn(message, category: nil)
                   $rutie_cat_warnings << \"#{category.inspect} #{message}\"
                 end",
            )
            .unwrap();

            let take = || {
                VM::eval("$rutie_cat_warnings.join.tap { $rutie_cat_warnings.clear }")
                    .unwrap()
                    .try_convert_to::<RString>()
                    .unwrap()
                    .to_string()
            };

            // Other tests turn `:experimental` off, for Ractor.
            VM::eval(
                "$VERBOSE = false; Warning[:deprecated] = false; Warning[:experimental] = true",
            )
            .unwrap();
            VM::warn_category(WarningCategory::Deprecated, "disabled category");
            VM::warning_category(WarningCategory::Experimental, "not verbose");
            VM::compile_warning("f.rb", 1, "not verbose");
            assert_eq!(take(), "");

            // Outside Ruby code, Ruby puts the script name before `warning:`.
            VM::warn_category(WarningCategory::Experimental, "%s 100%\0 cut");
            let warning = take();
            assert!(warning.starts_with(":experimental "), "{}", warning);
            assert!(warning.ends_with(" warning: %s 100%\n"), "{}", warning);

            VM::compile_warn("f.rb", 7, "%d");
            assert_eq!(take(), "nil f.rb:7: warning: %d\n");

            VM::eval("Warning[:deprecated] = true; $VERBOSE = true").unwrap();
            VM::warn_category(WarningCategory::Deprecated, "now on");
            VM::warning_category(WarningCategory::Deprecated, "verbose");
            VM::compile_warning("g.rb", 2, "verbose");
            VM::compile_warn_category(WarningCategory::Deprecated, "h.rb", 3, "both");
            let warnings = take();
            let lines: Vec<&str> = warnings.lines().collect();
            assert_eq!(lines.len(), 4, "{}", warnings);
            assert!(lines[0].starts_with(":deprecated ") && lines[0].ends_with(" warning: now on"));
            assert!(
                lines[1].starts_with(":deprecated ") && lines[1].ends_with(" warning: verbose")
            );
            assert_eq!(lines[2], "nil g.rb:2: warning: verbose");
            assert_eq!(lines[3], ":deprecated h.rb:3: warning: both");

            VM::eval("$VERBOSE = nil").unwrap();
            VM::warn_category(WarningCategory::Deprecated, "silenced");
            VM::compile_warn("f.rb", 1, "silenced");
            VM::sys_warning("silenced");
            assert_eq!(take(), "");

            VM::eval(
                "Warning.singleton_class.send(:alias_method, :warn, :rutie_cat_original_warn)
                 Warning[:deprecated] = $rutie_cat_deprecated
                 Warning[:experimental] = $rutie_cat_experimental
                 $VERBOSE = $rutie_cat_verbose",
            )
            .unwrap();
        });
    }

    #[test]
    fn test_more_raising_helpers() {
        crate::on_ruby_thread(|| {
            let errno = |name: &str| {
                VM::eval(&format!("Errno::{}::Errno", name))
                    .unwrap()
                    .try_convert_to::<Fixnum>()
                    .unwrap()
                    .to_i64() as i32
            };
            let raised = |result: Result<AnyObject, i32>| {
                assert!(result.is_err());
                VM::error_pop().unwrap()
            };

            let error = raised(VM::protect(|| VM::raise_fatal("%s\0hidden")));
            assert_eq!(
                error.class(),
                Class::from_existing("Object").get_nested_class("fatal")
            );
            assert_eq!(error.message(), "%s");
            // `rescue` cannot catch `fatal`.
            assert!(VM::eval("begin; raise Exception; rescue Exception; :caught; end").is_ok());

            let error = raised(VM::protect(|| VM::raise_syserr(errno("EPIPE"), "%d pipe")));
            assert!(Class::from_existing("Errno")
                .get_nested_class("EPIPE")
                .case_equals(&error));
            assert!(error.message().ends_with(" - %d pipe"));

            let marker = Module::new("RutieSyserrMarker");
            let error = raised(VM::protect(|| {
                VM::raise_syserr_with_module(&marker, errno("ENOENT"), "missing")
            }));
            assert!(marker.case_equals(&error));

            let error = raised(VM::protect(|| {
                VM::raise_wait_syserr(IoWait::Writable, errno("EAGAIN"), "full")
            }));
            assert!(Class::from_existing("IO")
                .get_nested_class("EAGAINWaitWritable")
                .case_equals(&error));
            let error = raised(VM::protect(|| {
                VM::raise_wait_syserr(IoWait::Readable, errno("EINTR"), "other")
            }));
            assert!(Module::from_existing("IO")
                .get_nested_module("WaitReadable")
                .case_equals(&error));

            let error = raised(VM::protect(|| VM::raise_load_error("%s no", None)));
            assert!(Class::from_existing("LoadError").case_equals(&error));
            assert_eq!(error.message(), "%s no");
            assert!(unsafe { error.send("path", &[]) }.is_nil());

            let error = raised(VM::protect(|| VM::raise_name_error("@@x", "bad %p")));
            assert_eq!(error.message(), "bad %p");
            assert_eq!(
                unsafe { error.send("name", &[]) }.try_convert_to::<Symbol>(),
                Ok(Symbol::new("@@x"))
            );

            let error = raised(VM::protect(|| VM::raise_frozen_error("%s")));
            assert_eq!(error.message(), "can't modify frozen %s");

            let error = raised(VM::protect(|| VM::raise_invalid_value("1\02", "Integer")));
            assert_eq!(error.message(), "invalid value for Integer: \"1\"");

            let error = raised(VM::protect(|| {
                VM::raise_unexpected_type(&NilClass::new(), ValueType::Array)
            }));
            assert_eq!(error.message(), "wrong argument type nil (expected Array)");

            let error = VM::make_exception(&[
                Class::from_existing("KeyError").into(),
                RString::new_utf8("k").into(),
            ])
            .unwrap()
            .unwrap();
            assert!(Class::from_existing("KeyError").case_equals(&error));
            assert!(VM::make_exception(&[Fixnum::new(1).into()]).is_err());
        });
    }

    #[test]
    fn test_global_variable_tracing_and_listing() {
        crate::on_ruby_thread(|| {
            VM::eval("$rutie_gv_source = :one").unwrap();
            VM::alias_global_variable("$rutie_gv_alias", "$rutie_gv_source").unwrap();
            assert_eq!(
                VM::eval("$rutie_gv_alias")
                    .unwrap()
                    .try_convert_to::<Symbol>(),
                Ok(Symbol::new("one"))
            );

            let names = VM::global_variables();
            for name in ["$rutie_gv_source", "$rutie_gv_alias", "$stdout"] {
                assert!(names
                    .dup()
                    .into_iter()
                    .any(|n| n.try_convert_to::<Symbol>() == Ok(Symbol::new(name))));
            }

            let handler = Proc::new(|values| {
                VM::eval("$rutie_gv_traced += 1").unwrap();

                values[0].clone()
            });
            VM::eval("$rutie_gv_traced = 0").unwrap();
            VM::trace_global_variable("$rutie_gv_source", &handler).unwrap();
            VM::trace_global_variable("$rutie_gv_source", &handler).unwrap();
            VM::eval("$rutie_gv_alias = :two").unwrap();
            assert_eq!(
                VM::eval("$rutie_gv_traced")
                    .unwrap()
                    .try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(2))
            );

            assert_eq!(
                VM::untrace_global_variable("$rutie_gv_source")
                    .unwrap()
                    .length(),
                2
            );
            assert_eq!(
                VM::untrace_global_variable("$rutie_gv_source")
                    .unwrap()
                    .length(),
                0
            );
            VM::eval("$rutie_gv_source = :three").unwrap();
            assert_eq!(
                VM::eval("$rutie_gv_traced")
                    .unwrap()
                    .try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(2))
            );
        });
    }

    crate::class!(RutieFrameProbe);

    crate::methods!(
        RutieFrameProbe,
        rtself,
        fn rutie_frame_info() -> Array {
            let (name, owner) = VM::current_method().unwrap();
            let (file, line) = VM::source_location().unwrap();
            let receiver = VM::current_receiver().unwrap();

            Array::new()
                .push(name)
                .push(owner)
                .push(VM::current_method_name().unwrap())
                .push(VM::current_callee_name().unwrap())
                .push(RString::new_utf8(&file))
                .push(Fixnum::new(line as i64))
                .push(Boolean::new(receiver.equals(&rtself)))
                .push(Fixnum::new(VM::backtrace().length() as i64))
        },
        fn rutie_frame_emit() -> AnyObject {
            let keywords = VM::eval("{ factor: 3 }")
                .unwrap()
                .try_convert_to::<Hash>()
                .unwrap();

            VM::yield_values_with_keywords(
                &[Fixnum::new(2).into(), Fixnum::new(5).into()],
                keywords,
            )
        },
        fn rutie_frame_super(value: Fixnum) -> AnyObject {
            let keywords = VM::eval("{ extra: 10 }")
                .unwrap()
                .try_convert_to::<Hash>()
                .unwrap();

            unsafe { VM::call_super_with_keywords(&[value.unwrap().into()], keywords) }
        }
    );

    #[test]
    fn test_frame_information_yield_and_super_with_keywords() {
        crate::on_ruby_thread(|| {
            VM::eval("class RutieFrameBase; def sum(a, extra: 0); a + extra; end; end").unwrap();

            Class::new(
                "RutieFrameProbe",
                Some(&Class::from_existing("RutieFrameBase")),
            )
            .define(|klass| {
                klass.def("info", rutie_frame_info);
                klass.define_alias("info_alias", "info");
                klass.def("emit", rutie_frame_emit);
                klass.def("sum", rutie_frame_super);
            });

            let info = VM::eval("\nRutieFrameProbe.new.info_alias.inspect")
                .unwrap()
                .try_convert_to::<RString>()
                .unwrap()
                .to_string();
            assert!(
                info.starts_with(
                    "[:info, RutieFrameProbe, :info, :info_alias, \"eval\", 2, true, "
                ),
                "{}",
                info
            );

            let emitted =
                VM::eval("RutieFrameProbe.new.emit { |a, b, factor:| (a + b) * factor }").unwrap();
            assert_eq!(emitted.try_convert_to::<Fixnum>(), Ok(Fixnum::new(21)));

            let sum = VM::eval("RutieFrameProbe.new.sum(1)").unwrap();
            assert_eq!(sum.try_convert_to::<Fixnum>(), Ok(Fixnum::new(11)));

            assert!(VM::current_method().is_none());
            assert!(VM::current_method_name().is_none());
        });
    }

    crate::class!(RutieGlobalFunctions);

    crate::methods!(
        RutieGlobalFunctions,
        rtself,
        fn rutie_global_triple(number: Fixnum) -> Fixnum {
            Fixnum::new(number.unwrap().to_i64() * 3)
        }
    );

    #[test]
    fn test_eval_wrapped_argv_and_global_functions() {
        crate::on_ruby_thread(|| {
            let result =
                VM::eval_wrapped("def rutie_wrapped_helper; 1; end; RUTIE_WRAPPED = 2; self")
                    .unwrap();
            assert_eq!(result.as_string().to_str(), "main");
            assert!(VM::eval("defined?(RUTIE_WRAPPED)").unwrap().is_nil());
            assert!(VM::eval_wrapped("RUTIE_WRAPPED").is_err());
            let error = VM::eval_wrapped("raise ArgumentError, 'wrapped'").unwrap_err();
            assert!(Class::argument_error().case_equals(&error));
            assert!(VM::error_info().is_err());

            assert!(VM::argv().equals(&VM::eval("ARGV").unwrap()));

            VM::define_global_function("rutie_global_triple", rutie_global_triple);
            let result = VM::eval("rutie_global_triple(5)").unwrap();
            assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(15)));
            assert!(VM::eval("Object.new.rutie_global_triple(1)").is_err());
            assert!(VM::eval("Kernel.rutie_global_triple(1)").is_ok());

            assert!(VM::backtrace().length() == 0);
        });
    }

    #[test]
    fn test_exec_recursive_family() {
        crate::on_ruby_thread(|| {
            let array = VM::eval("a = [1]; a << a; a").unwrap();

            // Nested calls on the same object see the recursion.
            let flags = std::cell::RefCell::new(Vec::new());
            let result = VM::exec_recursive(&array, |outer| {
                flags.borrow_mut().push(outer);
                VM::exec_recursive(&array, |inner| {
                    flags.borrow_mut().push(inner);
                    NilClass::new()
                })
                .unwrap();
                Fixnum::new(1)
            });
            assert_eq!(
                result.unwrap().try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(1))
            );
            assert_eq!(*flags.borrow(), [false, true]);

            // Exceptions and panics in the closure are returned.
            let error = VM::exec_recursive(&array, |_| -> NilClass {
                VM::raise(Class::from_existing("KeyError"), "inside");
                unreachable!()
            })
            .unwrap_err();
            assert!(Class::from_existing("KeyError").case_equals(&error));
            let error = VM::exec_recursive(&array, |_| -> NilClass { panic!("recursive panic") })
                .unwrap_err();
            assert!(error.message().contains("recursive panic"));

            // The recursion state is cleaned up after an exception.
            let fresh =
                VM::exec_recursive(&array, |recursive| crate::Boolean::new(recursive)).unwrap();
            assert!(!fresh.value().is_true());

            let (a, b) = (Fixnum::new(1), Fixnum::new(2));
            let nested = VM::exec_recursive_paired(&a, &b, |_| {
                let same = VM::exec_recursive_paired(&a, &b, |r| crate::Boolean::new(r)).unwrap();
                let swapped =
                    VM::exec_recursive_paired(&b, &a, |r| crate::Boolean::new(r)).unwrap();
                fixnums(&[
                    same.value().is_true() as i64,
                    swapped.value().is_true() as i64,
                ])
            });
            assert_eq!(nested.unwrap().inspect_object().to_string(), "[1, 0]");

            let runs = Cell::new(0);
            let result = VM::exec_recursive_outer(&a, |recursive| {
                runs.set(runs.get() + 1);
                if !recursive {
                    VM::exec_recursive_outer(&a, |_| Fixnum::new(-1)).unwrap();
                }
                Fixnum::new(if recursive { 10 } else { 20 })
            });
            assert_eq!(
                result.unwrap().try_convert_to::<Fixnum>(),
                Ok(Fixnum::new(10))
            );
            assert_eq!(runs.get(), 2);

            // Without recursion the outer variants are plain calls.
            let result =
                VM::exec_recursive_paired_outer(&a, &b, |recursive| crate::Boolean::new(recursive));
            assert!(!result.unwrap().value().is_true());
        });
    }

    #[test]
    fn test_io_helpers() {
        crate::on_ruby_thread(|| {
            let result = VM::protect(|| VM::raise_eof_error());
            assert!(result.is_err());
            assert!(Class::from_existing("EOFError").case_equals(&VM::error_pop().unwrap()));

            eval_raising("require 'stringio'; $rutie_saved = $stderr; $stderr = StringIO.new");
            VM::write_error("with\0nul");
            let written = eval_raising("s = $stderr.string; $stderr = $rutie_saved; s");
            assert_eq!(
                RString::from(written.value()).to_bytes_unchecked(),
                b"with\0nul"
            );

            let dir =
                std::env::temp_dir().join(format!("rutie_vm_find_ext_{}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("rutie_vm_ext.rb"), "").unwrap();
            VM::add_load_path(dir.to_str().unwrap());
            let (path, index) = VM::find_file_ext("rutie_vm_ext", &[".rb"])
                .unwrap()
                .unwrap();
            assert!(path.to_str().ends_with("rutie_vm_ext.rb"));
            assert_eq!(index, 0);
            assert_eq!(
                VM::find_file_ext("rutie_vm_missing", &[".rb", ".so"]).unwrap(),
                None
            );
            assert!(VM::find_file_ext("x", &["nul\0"]).is_err());
            assert_eq!(VM::find_file_ext("rutie_vm_ext", &[]).unwrap(), None);
            std::fs::remove_dir_all(&dir).unwrap();
        });
    }
}
