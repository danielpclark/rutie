use std::convert::From;

use crate::{
    binding::{thread, vm},
    rubysys::exception::rb_eException,
    types::Value,
    util, AnyException, AnyObject, Class, Object, VerifiedObject,
};

#[cfg(ruby_gte_3_2)]
use crate::{Hash, NilClass};

// Resuming or yielding a fiber under an `rb_protect` other than the one it
// was created under raises "fiber called across stack rewinding barrier",
// so exceptions are rescued with `rb_rescue2` instead. Its tag also
// satisfies Ruby's requirement that fibers are only used while the thread
// is "running" (has an active tag).
fn rescue<F>(func: F) -> Result<AnyObject, AnyException>
where
    F: FnOnce() -> Value,
{
    let mut error = None;

    let result = vm::rescue(
        func,
        |exception| {
            error = Some(exception);

            exception
        },
        &[unsafe { rb_eException }],
    );

    match error {
        Some(exception) => Err(AnyException::from(exception)),
        None => Ok(AnyObject::from(result)),
    }
}

/// Ruby's `Fiber`, a coroutine.
#[derive(Debug)]
#[repr(C)]
pub struct Fiber {
    value: Value,
}

impl Fiber {
    /// Creates a fiber that runs the Rust closure `func` when first resumed
    /// (`rb_fiber_new`). The closure gets the arguments of the first
    /// `resume`; its result is what that fiber's last `resume` returns.
    ///
    /// The closure is kept until Ruby garbage collects the fiber. A panic in
    /// it is raised as a Ruby `RuntimeError`.
    ///
    /// Ruby's `eval` (`VM::eval`) refuses to run directly on top of a fiber
    /// ("Can't eval on top of Fiber or Thread"); call Ruby methods from the
    /// body instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fiber, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let counter = Fiber::new(|arguments| {
    ///     let mut n = arguments[0].try_convert_to::<Fixnum>().unwrap().to_i64();
    ///
    ///     loop {
    ///         n += 1;
    ///         let _ = Fiber::yield_values(&[Fixnum::new(n).into()]);
    ///
    ///         if n == 3 {
    ///             return Fixnum::new(-1).into();
    ///         }
    ///     }
    /// });
    ///
    /// let next = |fiber: &Fiber| fiber.resume(&[Fixnum::new(0).into()]).unwrap().try_convert_to::<Fixnum>().unwrap().to_i64();
    ///
    /// assert_eq!(next(&counter), 1);
    /// assert_eq!(next(&counter), 2);
    /// assert_eq!(next(&counter), 3);
    /// assert_eq!(next(&counter), -1);
    /// assert!(!counter.is_alive());
    /// assert!(counter.resume(&[]).is_err());
    /// ```
    pub fn new<F>(mut func: F) -> Self
    where
        F: FnMut(&[AnyObject]) -> AnyObject + 'static,
    {
        let closure = move |arguments: &[Value]| {
            // `AnyObject` is a `#[repr(C)]` wrapper around a single `Value`.
            let arguments = unsafe {
                std::slice::from_raw_parts(arguments.as_ptr() as *const AnyObject, arguments.len())
            };

            func(arguments).value()
        };

        // Ruby only creates fibers while the thread has an active tag
        // ("not running thread" otherwise), which a program embedding Ruby
        // does not have between calls; `rb_rescue2` provides one.
        let fiber = rescue(|| thread::fiber_new(closure)).expect("could not create a Fiber");

        Fiber::from(fiber.value())
    }

    /// Like [`Fiber::new`](#method.new), but the fiber starts with its own
    /// fiber storage (`Fiber[key]`) instead of a copy of the current fiber's
    /// (`rb_fiber_new_storage`, Ruby 3.2+).
    ///
    /// `None` starts with empty storage. `Some(hash)` starts with a copy of
    /// `hash`, which must have only `Symbol` keys and not be frozen; otherwise
    /// the `TypeError` or `FrozenError` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{AnyObject, Class, Exception, Fiber, Fixnum, Hash, NilClass, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let read_storage = |_: &[AnyObject]| {
    ///     // `Fiber[:request_id]`
    ///     let fiber_class = Class::from_existing("Fiber");
    ///     let value = fiber_class.protect_send("[]", &[Symbol::new("request_id").into()]).unwrap();
    ///     if value.is_nil() { Fixnum::new(0).into() } else { value }
    /// };
    ///
    /// let mut storage = Hash::new();
    /// storage.store(Symbol::new("request_id"), Fixnum::new(7));
    ///
    /// let fiber = Fiber::with_storage(Some(&storage), read_storage).unwrap();
    /// assert_eq!(fiber.resume(&[]).unwrap(), Fixnum::new(7).into());
    ///
    /// let empty = Fiber::with_storage(None, read_storage).unwrap();
    /// assert_eq!(empty.resume(&[]).unwrap(), Fixnum::new(0).into());
    ///
    /// let mut bad = Hash::new();
    /// bad.store(Fixnum::new(1), NilClass::new());
    /// let error = Fiber::with_storage(Some(&bad), read_storage).unwrap_err();
    /// assert_eq!(error.class().name().unwrap().to_str(), "TypeError");
    /// ```
    #[cfg(ruby_gte_3_2)]
    pub fn with_storage<F>(storage: Option<&Hash>, mut func: F) -> Result<Self, AnyException>
    where
        F: FnMut(&[AnyObject]) -> AnyObject + 'static,
    {
        let closure = move |arguments: &[Value]| {
            // `AnyObject` is a `#[repr(C)]` wrapper around a single `Value`.
            let arguments = unsafe {
                std::slice::from_raw_parts(arguments.as_ptr() as *const AnyObject, arguments.len())
            };

            func(arguments).value()
        };
        let storage = match storage {
            Some(hash) => hash.value(),
            None => NilClass::new().value(),
        };

        rescue(|| thread::fiber_new_storage(closure, storage))
            .map(|fiber| Fiber::from(fiber.value()))
    }

    /// Starts or continues the fiber, passing `arguments` (Ruby's `resume`,
    /// `rb_fiber_resume`). Returns what the fiber yields or finally returns,
    /// or the exception it raised (including the `FiberError` for resuming
    /// a finished fiber).
    ///
    /// Ruby only resumes a fiber under the protect tag it was created under,
    /// so a fiber created inside `VM::eval` or `VM::protect` returns a
    /// `FiberError` when resumed from Rust; create it with `Fiber::new`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{AnyObject, Fiber, Object, RString, Symbol, VM};
    /// # VM::init();
    ///
    /// let fiber = Fiber::new(|arguments: &[AnyObject]| {
    ///     let text = arguments[0].try_convert_to::<RString>().unwrap().to_string();
    ///     let doubled = RString::new_utf8(&text.repeat(2));
    ///
    ///     Fiber::yield_values(&[doubled.into()]).unwrap();
    ///
    ///     Symbol::new("done").into()
    /// });
    ///
    /// let doubled = fiber.resume(&[RString::new_utf8("ab").into()]).unwrap();
    /// assert_eq!(doubled.try_convert_to::<RString>().unwrap().to_str(), "abab");
    ///
    /// let done = fiber.resume(&[]).unwrap();
    /// assert_eq!(done.try_convert_to::<Symbol>().unwrap().to_str(), "done");
    ///
    /// assert!(fiber.resume(&[]).is_err());
    /// ```
    pub fn resume(&self, arguments: &[AnyObject]) -> Result<AnyObject, AnyException> {
        let fiber = self.value();
        let arguments = util::arguments_to_values(arguments);

        rescue(|| thread::fiber_resume(fiber, &arguments))
    }

    /// Switches back to the fiber that resumed the current one, passing it
    /// `values` (Ruby's `Fiber.yield`, `rb_fiber_yield`), and returns what
    /// the next `resume` passes in. Returns the `FiberError` when called
    /// outside a fiber.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fiber, VM};
    /// # VM::init();
    ///
    /// // The main program is not inside a fiber.
    /// assert!(Fiber::yield_values(&[]).is_err());
    /// ```
    pub fn yield_values(values: &[AnyObject]) -> Result<AnyObject, AnyException> {
        let values = util::arguments_to_values(values);

        rescue(|| thread::fiber_yield(&values))
    }

    /// Returns the fiber running now (Ruby's `Fiber.current`,
    /// `rb_fiber_current`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fiber, VM};
    /// # VM::init();
    ///
    /// assert!(Fiber::current().is_alive());
    /// ```
    pub fn current() -> Self {
        Fiber::from(thread::fiber_current())
    }

    /// Returns `true` until the fiber finishes (Ruby's `alive?`,
    /// `rb_fiber_alive_p`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fiber, NilClass, VM};
    /// # VM::init();
    ///
    /// let fiber = Fiber::new(|_| NilClass::new().into());
    ///
    /// assert!(fiber.is_alive());
    /// fiber.resume(&[]).unwrap();
    /// assert!(!fiber.is_alive());
    /// ```
    pub fn is_alive(&self) -> bool {
        thread::fiber_is_alive(self.value())
    }
}

impl From<Value> for Fiber {
    fn from(value: Value) -> Self {
        Fiber { value }
    }
}

impl Into<Value> for Fiber {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Fiber {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for Fiber {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Fiber {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        Class::from_existing("Fiber").case_equals(object)
    }

    fn error_message() -> &'static str {
        "Error converting to Fiber"
    }
}

impl PartialEq for Fiber {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{AnyException, Exception, Fiber, Fixnum, Object, GC, VM};

    #[test]
    fn test_fiber() {
        crate::on_ruby_thread(|| {
            let generator = Fiber::new(|_| {
                for i in 1..=3 {
                    Fiber::yield_values(&[Fixnum::new(i).into()]).unwrap();
                }

                crate::NilClass::new().into()
            });
            GC::start();

            let mut seen = Vec::new();
            while generator.is_alive() {
                let value = generator.resume(&[]).unwrap();

                if !value.is_nil() {
                    seen.push(value.try_convert_to::<Fixnum>().unwrap().to_i64());
                }
            }
            assert_eq!(seen, vec![1, 2, 3]);

            // Values passed to resume come back out of yield.
            let echo = Fiber::new(|first| {
                let second = Fiber::yield_values(&[first[0].clone()]).unwrap();

                second
            });
            let a = echo.resume(&[Fixnum::new(10).into()]).unwrap();
            let b = echo.resume(&[Fixnum::new(20).into()]).unwrap();
            assert_eq!(a.try_convert_to::<Fixnum>(), Ok(Fixnum::new(10)));
            assert_eq!(b.try_convert_to::<Fixnum>(), Ok(Fixnum::new(20)));

            // `eval` refuses to run directly on top of a fiber, so raise instead.
            let failing = Fiber::new(|_| {
                VM::raise_ex(AnyException::new("RuntimeError", Some("in fiber")));

                crate::NilClass::new().into()
            });
            assert_eq!(failing.resume(&[]).unwrap_err().message(), "in fiber");
            assert!(!failing.is_alive());

            let panicking = Fiber::new(|_| panic!("fiber panic"));
            assert_eq!(
                panicking.resume(&[]).unwrap_err().message(),
                "Rust panic: fiber panic"
            );

            assert!(VM::eval("Fiber.new {}")
                .unwrap()
                .try_convert_to::<Fiber>()
                .is_ok());
        });
    }

    #[cfg(ruby_gte_3_2)]
    #[test]
    fn test_with_storage() {
        use crate::{AnyObject, Class, Hash, NilClass, Symbol};

        crate::on_ruby_thread(|| {
            let storage_value = |_: &[AnyObject]| {
                Class::from_existing("Fiber")
                    .protect_send("[]", &[Symbol::new("key").into()])
                    .unwrap()
            };

            let mut storage = Hash::new();
            storage.store(Symbol::new("key"), Fixnum::new(5));

            let fiber = Fiber::with_storage(Some(&storage), storage_value).unwrap();
            assert_eq!(fiber.resume(&[]).unwrap(), Fixnum::new(5).into());

            // The fiber got a copy.
            storage.store(Symbol::new("key"), Fixnum::new(6));
            let fiber = Fiber::with_storage(None, storage_value).unwrap();
            assert!(fiber.resume(&[]).unwrap().is_nil());

            let mut frozen = Hash::new();
            frozen.freeze();
            let error = Fiber::with_storage(Some(&frozen), storage_value).unwrap_err();
            assert_eq!(error.class().name().unwrap().to_str(), "FrozenError");

            let mut bad = Hash::new();
            bad.store(Fixnum::new(1), NilClass::new());
            assert!(Fiber::with_storage(Some(&bad), storage_value).is_err());
        });
    }
}
