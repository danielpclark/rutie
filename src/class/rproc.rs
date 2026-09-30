use std::convert::From;

use crate::{
    binding::{rproc, vm},
    types::Value,
    util, AnyException, AnyObject, Boolean, Class, Object, VerifiedObject,
};

/// `Proc` (works with `Lambda` as well)
#[derive(Debug)]
#[repr(C)]
pub struct Proc {
    value: Value,
}

impl Proc {
    /// Creates a Ruby `Proc` that calls the Rust closure `func` with the
    /// values it is called (or yielded) with (`rb_proc_new`).
    ///
    /// The closure is kept until Ruby garbage collects the proc. A panic in
    /// it is raised as a Ruby `RuntimeError`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, Proc, VM};
    /// # VM::init();
    ///
    /// let double = Proc::new(|arguments| {
    ///     let x = arguments[0].try_convert_to::<Fixnum>().unwrap().to_i64();
    ///
    ///     Fixnum::new(x * 2).into()
    /// });
    ///
    /// assert_eq!(double.call(&[Fixnum::new(21).into()]).try_convert_to::<Fixnum>(), Ok(Fixnum::new(42)));
    ///
    /// // It works as a block too: [1, 2, 3].map(&double)
    /// let array: Array = (1..=3).map(|i| Fixnum::new(i).to_any_object()).collect();
    /// let doubled = unsafe { array.send_with_proc("map", &[], &double) };
    ///
    /// assert_eq!(doubled.try_convert_to::<Array>().unwrap().at(2).try_convert_to::<Fixnum>(), Ok(Fixnum::new(6)));
    /// ```
    pub fn new<F>(mut func: F) -> Self
    where
        F: FnMut(&[AnyObject]) -> AnyObject + 'static,
    {
        Proc::from(rproc::new(move |arguments: &[Value]| {
            // `AnyObject` is a `#[repr(C)]` wrapper around a single `Value`.
            let arguments = unsafe {
                std::slice::from_raw_parts(arguments.as_ptr() as *const AnyObject, arguments.len())
            };

            func(arguments).value()
        }))
    }

    /// Returns the number of required arguments, or `-n - 1` when there are
    /// `n` required arguments and optional ones (Ruby's `arity`,
    /// `rb_proc_arity`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Proc, VM};
    /// # VM::init();
    ///
    /// let two = VM::eval("lambda { |a, b| }").unwrap().try_convert_to::<Proc>().unwrap();
    /// let splat = VM::eval("lambda { |a, *rest| }").unwrap().try_convert_to::<Proc>().unwrap();
    ///
    /// assert_eq!(two.arity(), 2);
    /// assert_eq!(splat.arity(), -2);
    /// ```
    pub fn arity(&self) -> i32 {
        rproc::arity(self.value())
    }

    /// Like [`call`](#method.call), but returns the exception raised by the
    /// proc instead of propagating it.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Exception, Object, Proc, VM};
    /// # VM::init();
    ///
    /// let failing = VM::eval("proc { raise 'nope' }").unwrap().try_convert_to::<Proc>().unwrap();
    ///
    /// assert_eq!(failing.protect_call(&[]).unwrap_err().message(), "nope");
    /// ```
    pub fn protect_call(&self, arguments: &[AnyObject]) -> Result<AnyObject, AnyException> {
        let rproc = self.value();
        let arguments = util::arguments_to_values(arguments);

        vm::protect_value(|| rproc::call(rproc, &arguments))
            .map(AnyObject::from)
            .map_err(AnyException::from)
    }

    /// Calls a proc with given arguments
    ///
    /// # Examples
    ///
    /// ```no_run
    /// #[macro_use]
    /// extern crate rutie;
    ///
    /// use rutie::{Class, Object, Proc, RString};
    ///
    /// class!(Greeter);
    ///
    /// methods!(
    ///     Greeter,
    ///     rtself,
    ///
    ///     fn greet_rust_with(greeting_template: Proc) -> RString {
    ///         let name = RString::new_utf8("Rust").to_any_object();
    ///         let rendered_template = greeting_template.unwrap().call(&[name]);
    ///
    ///         rendered_template.try_convert_to::<RString>().unwrap()
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
    ///   def self.greet_rust_with(greeting_template)
    ///     greeting_template.call('Rust')
    ///   end
    /// end
    ///
    /// greeting_template = -> (name) { "Hello, #{name}!" }
    ///
    /// Greeter.greet_rust_with(greeting_template) # => "Hello, Rust!"
    /// ```
    pub fn call(&self, arguments: &[AnyObject]) -> AnyObject {
        let arguments = util::arguments_to_values(arguments);
        let result = rproc::call(self.value(), &arguments);

        AnyObject::from(result)
    }

    /// Check if Proc is a lambda
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Proc, VM, VerifiedObject};
    /// # VM::init();
    ///
    /// let procish = VM::eval("lambda {|a,b| a + b }").unwrap();
    ///
    /// assert!(Proc::is_correct_type(&procish), "not Proc!");
    /// ```
    ///
    /// Ruby:
    ///
    /// ```ruby
    /// procish = lambda {|a,b| a + b }
    ///
    /// procish.lambda? # => true
    /// ```
    pub fn is_lambda(&self) -> bool {
        Boolean::from(unsafe { self.send("lambda?", &[]) }.value()).to_bool()
    }
}

impl From<Value> for Proc {
    fn from(value: Value) -> Self {
        Proc { value }
    }
}

impl Into<Value> for Proc {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Proc {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for Proc {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Proc {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        Class::proc().case_equals(object)
    }

    fn error_message() -> &'static str {
        "Error converting to Proc"
    }
}

impl PartialEq for Proc {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Array, Exception, Fixnum, Object, Proc, GC, VM};
    use std::sync::Arc;

    #[test]
    fn test_proc_from_closure() {
        crate::on_ruby_thread(|| {
            let mut total = 0;
            let accumulate = Proc::new(move |arguments| {
                for argument in arguments {
                    total += argument.try_convert_to::<Fixnum>().unwrap().to_i64();
                }

                Fixnum::new(total).into()
            });
            GC::start();

            accumulate.call(&[Fixnum::new(1).into(), Fixnum::new(2).into()]);
            let result = accumulate.call(&[Fixnum::new(3).into()]);
            assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(6)));
            assert!(!accumulate.is_lambda());

            // Called from Ruby, and as a block.
            VM::global_set("$rutie_test_proc", Proc::from(accumulate.value()));
            let sum = VM::eval("$rutie_test_proc.call(4)").unwrap();
            assert_eq!(sum.try_convert_to::<Fixnum>(), Ok(Fixnum::new(10)));

            let array: Array = (1..=2).map(|i| Fixnum::new(i).to_any_object()).collect();
            unsafe { array.send_with_proc("each", &[], &accumulate) };
            let result = accumulate.call(&[]);
            assert_eq!(result.try_convert_to::<Fixnum>(), Ok(Fixnum::new(13)));
            VM::global_set("$rutie_test_proc", crate::NilClass::new());

            let panicking = Proc::new(|_| panic!("in proc"));
            assert_eq!(
                panicking.protect_call(&[]).unwrap_err().message(),
                "Rust panic: in proc"
            );
        });
    }

    #[test]
    fn test_proc_closure_is_dropped_after_gc() {
        crate::on_ruby_thread(|| {
            let shared = Arc::new(());

            for _ in 0..100 {
                let captured = shared.clone();
                Proc::new(move |_| {
                    let _ = &captured;
                    crate::NilClass::new().into()
                });
            }

            assert_eq!(Arc::strong_count(&shared), 101);

            GC::start();
            GC::start();
            // Let deferred finalizers run.
            VM::eval("10.times { Object.new }").unwrap();

            assert!(
                Arc::strong_count(&shared) < 51,
                "only {} closures freed",
                101 - Arc::strong_count(&shared)
            );
        });
    }

    #[test]
    fn test_proc_arity() {
        crate::on_ruby_thread(|| {
            // Optional arguments count for lambdas, not for procs.
            let optional = VM::eval("lambda { |a, b = 1| }")
                .unwrap()
                .try_convert_to::<Proc>()
                .unwrap();
            assert_eq!(optional.arity(), -2);

            let plain = VM::eval("proc { |a, b = 1| }")
                .unwrap()
                .try_convert_to::<Proc>()
                .unwrap();
            assert_eq!(plain.arity(), 1);

            let lambda = VM::eval("->(a) { a * 2 }")
                .unwrap()
                .try_convert_to::<Proc>()
                .unwrap();
            assert!(lambda.is_lambda());
            assert!(lambda.protect_call(&[]).is_err());
        });
    }
}
