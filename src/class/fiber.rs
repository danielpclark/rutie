use std::{convert::From, time::Duration};

use crate::{
    binding::{thread, vm},
    rubysys::exception::rb_eException,
    types::Value,
    util, AnyException, AnyObject, Class, Hash, Object, Thread, VerifiedObject,
};

#[cfg(ruby_gte_3_2)]
use crate::NilClass;

// Fibers are created, resumed and yielded through `vm::fiber_call`, which
// gives Ruby the tag it needs to switch fibers and returns an exception as
// `Err` (see there for why that is `rb_rescue2`, or `rb_protect` on Rubies
// whose fibers copy the machine stack).
fn rescue<F>(func: F) -> Result<AnyObject, AnyException>
where
    F: FnOnce() -> Value,
{
    vm::fiber_call(func)
        .map(AnyObject::from)
        .map_err(AnyException::from)
}

// `nil` (no scheduler) as `None`.
fn scheduler_from(value: Value) -> Option<AnyObject> {
    if value.is_nil() {
        None
    } else {
        Some(AnyObject::from(value))
    }
}

// The arguments of a `_kw` call: `arguments`, then `keywords`.
fn with_keywords(arguments: &[AnyObject], keywords: Hash) -> Vec<Value> {
    let mut arguments = util::arguments_to_values(arguments);
    arguments.push(keywords.value());

    arguments
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

    /// Like [`resume`](#method.resume), with `keywords` passed as keyword
    /// arguments (`rb_fiber_resume_kw`), like Ruby's
    /// `fiber.resume(*arguments, **keywords)`.
    ///
    /// A fiber created with [`Fiber::new`](#method.new) gets the keywords as
    /// a `Hash` after the other arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{AnyObject, Fiber, Fixnum, Hash, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let fiber = Fiber::new(|arguments: &[AnyObject]| {
    ///     let base = arguments[0].try_convert_to::<Fixnum>().unwrap().to_i64();
    ///     let options = arguments[1].try_convert_to::<Hash>().unwrap();
    ///     let step = options.at(&Symbol::new("step")).try_convert_to::<Fixnum>().unwrap().to_i64();
    ///
    ///     Fixnum::new(base + step).into()
    /// });
    ///
    /// let mut keywords = Hash::new();
    /// keywords.store(Symbol::new("step"), Fixnum::new(2));
    ///
    /// let result = fiber.resume_with_keywords(&[Fixnum::new(40).into()], keywords).unwrap();
    /// assert_eq!(result, Fixnum::new(42).into());
    /// ```
    pub fn resume_with_keywords(
        &self,
        arguments: &[AnyObject],
        keywords: Hash,
    ) -> Result<AnyObject, AnyException> {
        let fiber = self.value();
        let arguments = with_keywords(arguments, keywords);

        rescue(|| thread::fiber_resume_kw(fiber, &arguments))
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

    /// Like [`yield_values`](#method.yield_values), with `keywords` passed
    /// as keyword arguments (`rb_fiber_yield_kw`), like Ruby's
    /// `Fiber.yield(*values, **keywords)`.
    ///
    /// The `resume` that receives them gets the keywords as a `Hash` after
    /// the other values (alone, it is the result itself).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fiber, Fixnum, Hash, NilClass, Object, Symbol, VM};
    /// # VM::init();
    ///
    /// let fiber = Fiber::new(|_| {
    ///     let mut keywords = Hash::new();
    ///     keywords.store(Symbol::new("progress"), Fixnum::new(50));
    ///
    ///     Fiber::yield_with_keywords(&[], keywords).unwrap();
    ///
    ///     NilClass::new().into()
    /// });
    ///
    /// let yielded = fiber.resume(&[]).unwrap().try_convert_to::<Hash>().unwrap();
    /// assert_eq!(yielded.at(&Symbol::new("progress")), Fixnum::new(50).into());
    ///
    /// // Outside a fiber there is nothing to yield to.
    /// assert!(Fiber::yield_with_keywords(&[], Hash::new()).is_err());
    /// ```
    pub fn yield_with_keywords(
        values: &[AnyObject],
        keywords: Hash,
    ) -> Result<AnyObject, AnyException> {
        let values = with_keywords(values, keywords);

        rescue(|| thread::fiber_yield_kw(&values))
    }

    /// Switches to the fiber, passing it `arguments` (Ruby's `transfer`,
    /// `rb_fiber_transfer`), and returns what is passed back when some fiber
    /// transfers to this one again, or what the fiber returns when it
    /// finishes (control then goes back to the main fiber).
    ///
    /// Unlike [`resume`](#method.resume), the fiber does not return to the
    /// one that started it: it must transfer somewhere explicitly. A fiber
    /// that was transferred to cannot be resumed or yield, and the other way
    /// around, while it is being resumed; such misuse is returned as a
    /// `FiberError`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fiber, Fixnum, Object, VM};
    /// # VM::init();
    ///
    /// let main = Fiber::current();
    /// let main_value = main.value();
    ///
    /// let worker = Fiber::new(move |arguments| {
    ///     let n = arguments[0].try_convert_to::<Fixnum>().unwrap().to_i64();
    ///
    ///     // Hand a result to the main fiber, and wait to be called again.
    ///     let next = Fiber::from(main_value).transfer(&[Fixnum::new(n * 10).into()]).unwrap();
    ///     let m = next.try_convert_to::<Fixnum>().unwrap().to_i64();
    ///
    ///     Fixnum::new(m + 1).into()
    /// });
    ///
    /// assert_eq!(worker.transfer(&[Fixnum::new(4).into()]).unwrap(), Fixnum::new(40).into());
    /// assert_eq!(worker.transfer(&[Fixnum::new(7).into()]).unwrap(), Fixnum::new(8).into());
    /// assert!(!worker.is_alive());
    /// assert!(worker.transfer(&[]).is_err());
    /// ```
    pub fn transfer(&self, arguments: &[AnyObject]) -> Result<AnyObject, AnyException> {
        let fiber = self.value();
        let arguments = util::arguments_to_values(arguments);

        rescue(|| thread::fiber_transfer(fiber, &arguments))
    }

    /// Like [`transfer`](#method.transfer), with `keywords` passed as
    /// keyword arguments (`rb_fiber_transfer_kw`), like Ruby's
    /// `fiber.transfer(*arguments, **keywords)`.
    ///
    /// A fiber created with [`Fiber::new`](#method.new) gets the keywords as
    /// a `Hash` after the other arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{AnyObject, Fiber, Hash, Object, RString, Symbol, VM};
    /// # VM::init();
    ///
    /// let fiber = Fiber::new(|arguments: &[AnyObject]| {
    ///     let options = arguments[0].try_convert_to::<Hash>().unwrap();
    ///
    ///     options.at(&Symbol::new("name"))
    /// });
    ///
    /// let mut keywords = Hash::new();
    /// keywords.store(Symbol::new("name"), RString::new_utf8("worker"));
    ///
    /// let name = fiber.transfer_with_keywords(&[], keywords).unwrap();
    /// assert_eq!(name.try_convert_to::<RString>().unwrap().to_str(), "worker");
    /// ```
    pub fn transfer_with_keywords(
        &self,
        arguments: &[AnyObject],
        keywords: Hash,
    ) -> Result<AnyObject, AnyException> {
        let fiber = self.value();
        let arguments = with_keywords(arguments, keywords);

        rescue(|| thread::fiber_transfer_kw(fiber, &arguments))
    }

    /// Raises an exception inside the fiber where it last yielded, and
    /// resumes it (Ruby's `raise`, `rb_fiber_raise`). `arguments` are what
    /// Ruby's `raise` takes: an exception, or an exception class and a
    /// message (and a backtrace).
    ///
    /// Returns what the fiber yields or returns next if it rescues the
    /// exception, or else the exception. A fiber that has not started yet
    /// cannot take an exception: that is a `FiberError`.
    ///
    /// In a fiber created with [`Fiber::new`](#method.new), the exception
    /// is returned as `Err` by the [`yield_values`](#method.yield_values)
    /// it was suspended in.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{AnyException, Exception, Fiber, Object, RString, VM};
    /// # VM::init();
    ///
    /// let fiber = Fiber::new(|_| {
    ///     match Fiber::yield_values(&[]) {
    ///         Ok(_) => RString::new_utf8("resumed").into(),
    ///         Err(error) => RString::new_utf8(&format!("rescued: {}", error.message())).into(),
    ///     }
    /// });
    ///
    /// let stop = AnyException::new("RuntimeError", Some("stop"));
    ///
    /// // Not started yet.
    /// assert!(fiber.raise(&[stop.to_any_object()]).is_err());
    ///
    /// fiber.resume(&[]).unwrap();
    /// let outcome = fiber.raise(&[stop.to_any_object()]).unwrap();
    /// assert_eq!(outcome.try_convert_to::<RString>().unwrap().to_str(), "rescued: stop");
    /// assert!(!fiber.is_alive());
    /// ```
    pub fn raise(&self, arguments: &[AnyObject]) -> Result<AnyObject, AnyException> {
        let fiber = self.value();
        let arguments = util::arguments_to_values(arguments);

        rescue(|| thread::fiber_raise(fiber, &arguments))
    }

    /// Returns the Fiber scheduler of the current thread (Ruby's
    /// `Fiber.scheduler`, `rb_fiber_scheduler_get`), or `None` when it has
    /// none.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fiber, NilClass, Object, VM};
    /// # VM::init();
    ///
    /// assert!(Fiber::scheduler().is_none());
    ///
    /// // A scheduler with the hooks Ruby requires (doing nothing here).
    /// let scheduler = VM::eval(
    ///     "Class.new do
    ///        def block(blocker, timeout = nil); end
    ///        def unblock(blocker, fiber); end
    ///        def kernel_sleep(duration = nil); end
    ///        def io_wait(io, events, timeout); end
    ///      end.new",
    /// )
    /// .unwrap();
    ///
    /// Fiber::set_scheduler(&scheduler).unwrap();
    /// assert!(Fiber::scheduler().unwrap().equals(&scheduler));
    ///
    /// Fiber::set_scheduler(&NilClass::new()).unwrap();
    /// assert!(Fiber::scheduler().is_none());
    /// ```
    pub fn scheduler() -> Option<AnyObject> {
        scheduler_from(thread::fiber_scheduler_get())
    }

    /// Sets the Fiber scheduler of the current thread (Ruby's
    /// `Fiber.set_scheduler`, `rb_fiber_scheduler_set`); `nil` removes it.
    /// The previous scheduler is closed first (its `close` method is
    /// called), which runs the fibers it still has.
    ///
    /// Returns the `ArgumentError` when `scheduler` lacks one of the hooks
    /// Ruby requires (`block`, `unblock`, `kernel_sleep` and `io_wait`), or
    /// what closing the previous scheduler raised.
    ///
    /// Once set, non-blocking fibers hand their blocking operations (such as
    /// `sleep` and IO waits) to the scheduler. See
    /// [`Fiber::scheduler`](#method.scheduler) for another example.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Exception, Fiber, NilClass, Object, VM};
    /// # VM::init();
    ///
    /// let scheduler = VM::eval(
    ///     "Class.new do
    ///        attr_reader :closed
    ///        def block(blocker, timeout = nil); end
    ///        def unblock(blocker, fiber); end
    ///        def kernel_sleep(duration = nil); end
    ///        def io_wait(io, events, timeout); end
    ///        def close; @closed = true; end
    ///      end.new",
    /// )
    /// .unwrap();
    ///
    /// Fiber::set_scheduler(&scheduler).unwrap();
    /// Fiber::set_scheduler(&NilClass::new()).unwrap();
    /// assert!(scheduler.protect_send("closed", &[]).unwrap().is_true());
    ///
    /// let incomplete = VM::eval("Object.new").unwrap();
    /// let error = Fiber::set_scheduler(&incomplete).unwrap_err();
    /// assert_eq!(error.class().name().unwrap().to_str(), "ArgumentError");
    /// assert!(Fiber::scheduler().is_none());
    /// ```
    pub fn set_scheduler<T: Object>(scheduler: &T) -> Result<(), AnyException> {
        let scheduler = scheduler.value();

        vm::protect_value(|| thread::fiber_scheduler_set(scheduler))
            .map(|_| ())
            .map_err(AnyException::from)
    }

    /// Returns the scheduler of the current thread if the current fiber is
    /// non-blocking, so that blocking operations should go through it, and
    /// `None` otherwise (Ruby's `Fiber.current_scheduler`,
    /// `rb_fiber_scheduler_current`).
    ///
    /// The main fiber is blocking. Fibers created with
    /// [`Fiber::new`](#method.new) are non-blocking from Ruby 3.2 (blocking
    /// on Ruby 3.1), and those started by `Fiber.schedule` always are.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use] extern crate rutie;
    ///
    /// use rutie::{Boolean, Fiber, NilClass, Object, VM};
    ///
    /// methods!(
    ///     rutie::AnyObject,
    ///     _rtself,
    ///     fn has_current_scheduler() -> Boolean {
    ///         Boolean::new(Fiber::current_scheduler().is_some())
    ///     }
    /// );
    ///
    /// fn main() {
    ///     # VM::init();
    ///     rutie::Class::from_existing("Object").define(|klass| {
    ///         klass.def_private("has_current_scheduler", has_current_scheduler);
    ///     });
    ///
    ///     let scheduler = VM::eval(
    ///         "Class.new do
    ///            def block(blocker, timeout = nil); end
    ///            def unblock(blocker, fiber); end
    ///            def kernel_sleep(duration = nil); end
    ///            def io_wait(io, events, timeout); end
    ///            def fiber(&block) = Fiber.new(blocking: false, &block).tap(&:resume)
    ///          end.new",
    ///     )
    ///     .unwrap();
    ///     Fiber::set_scheduler(&scheduler).unwrap();
    ///
    ///     // The main fiber is blocking.
    ///     assert!(Fiber::current_scheduler().is_none());
    ///
    ///     // A fiber started by `Fiber.schedule` is not.
    ///     let scheduled = VM::eval("result = nil; Fiber.schedule { result = has_current_scheduler }; result").unwrap();
    ///     assert!(scheduled.is_true());
    ///
    ///     Fiber::set_scheduler(&NilClass::new()).unwrap();
    /// }
    /// ```
    pub fn current_scheduler() -> Option<AnyObject> {
        scheduler_from(thread::fiber_scheduler_current())
    }

    /// Like [`Fiber::current_scheduler`](#method.current_scheduler), for
    /// `thread`: its scheduler if its current fiber is non-blocking
    /// (`rb_fiber_scheduler_current_for_thread`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fiber, NilClass, Thread, VM};
    /// # VM::init();
    ///
    /// let scheduler = VM::eval(
    ///     "Class.new do
    ///        def block(blocker, timeout = nil); end
    ///        def unblock(blocker, fiber); end
    ///        def kernel_sleep(duration = nil); end
    ///        def io_wait(io, events, timeout); end
    ///      end.new",
    /// )
    /// .unwrap();
    /// Fiber::set_scheduler(&scheduler).unwrap();
    ///
    /// // The main thread runs its blocking main fiber.
    /// assert!(Fiber::scheduler().is_some());
    /// assert!(Fiber::current_scheduler_for_thread(&Thread::current()).is_none());
    ///
    /// Fiber::set_scheduler(&NilClass::new()).unwrap();
    /// assert!(Fiber::current_scheduler_for_thread(&Thread::current()).is_none());
    /// ```
    pub fn current_scheduler_for_thread(thread: &Thread) -> Option<AnyObject> {
        scheduler_from(thread::fiber_scheduler_current_for_thread(thread.value()))
    }

    /// Converts `timeout` to what scheduler hooks take
    /// (`rb_fiber_scheduler_make_timeout`): a `Float` of seconds, or `nil`
    /// for no timeout.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use rutie::{Fiber, Float, Object, VM};
    /// # VM::init();
    ///
    /// let timeout = Fiber::make_scheduler_timeout(Some(Duration::from_millis(1500)));
    /// assert_eq!(timeout.try_convert_to::<Float>().unwrap().to_f64(), 1.5);
    ///
    /// assert!(Fiber::make_scheduler_timeout(None).is_nil());
    /// ```
    pub fn make_scheduler_timeout(timeout: Option<Duration>) -> AnyObject {
        AnyObject::from(thread::fiber_scheduler_make_timeout(timeout))
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
        thread::is_fiber(object.value())
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

    // Walking the stack from inside a fiber crashed on Ruby 3.3+ on arm64
    // (see `end_fiber_frame_chain` in src/binding/thread.rs). A panic in a
    // fiber does this walk when `RUST_BACKTRACE` is set; `force_capture`
    // always does.
    #[test]
    fn test_backtrace_inside_fiber() {
        use crate::Boolean;
        use std::backtrace::{Backtrace, BacktraceStatus};

        crate::on_ruby_thread(|| {
            let fiber = Fiber::new(|_| {
                let captured = Backtrace::force_capture().status() == BacktraceStatus::Captured;

                Boolean::new(captured).into()
            });

            assert_eq!(fiber.resume(&[]).unwrap(), Boolean::new(true).into());
            assert!(!fiber.is_alive());
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

    #[test]
    fn test_transfer_raise_and_keywords() {
        use crate::{AnyObject, Hash, RString, Symbol};

        crate::on_ruby_thread(|| {
            // Ping-pong between the main fiber and a worker with `transfer`.
            let main = Fiber::current().value();
            let worker = Fiber::new(move |arguments| {
                let mut n = arguments[0].try_convert_to::<Fixnum>().unwrap().to_i64();

                while n < 3 {
                    let next = Fiber::from(main)
                        .transfer(&[Fixnum::new(n).into()])
                        .unwrap();
                    n = next.try_convert_to::<Fixnum>().unwrap().to_i64() + 1;
                }

                Fixnum::new(-n).into()
            });
            assert_eq!(
                worker.transfer(&[Fixnum::new(0).into()]).unwrap(),
                Fixnum::new(0).into()
            );
            assert_eq!(
                worker.transfer(&[Fixnum::new(1).into()]).unwrap(),
                Fixnum::new(2).into()
            );
            assert_eq!(
                worker.transfer(&[Fixnum::new(2).into()]).unwrap(),
                Fixnum::new(-3).into()
            );
            assert!(!worker.is_alive());

            // A transferred fiber cannot be resumed.
            let transferred = Fiber::new(move |_| {
                Fiber::from(main).transfer(&[]).unwrap();

                crate::NilClass::new().into()
            });
            transferred.transfer(&[]).unwrap();
            let error = transferred.resume(&[]).unwrap_err();
            assert_eq!(error.class().name().unwrap().to_str(), "FiberError");
            transferred.transfer(&[]).unwrap();
            assert!(!transferred.is_alive());

            // `raise` with an exception class and a message.
            let rescuing = Fiber::new(|_| match Fiber::yield_values(&[]) {
                Ok(_) => RString::new_utf8("resumed").into(),
                Err(error) => RString::new_utf8(&error.message()).into(),
            });
            let argument_error = crate::Class::from_existing("ArgumentError");
            assert!(rescuing.raise(&[argument_error.to_any_object()]).is_err());
            rescuing.resume(&[]).unwrap();
            let outcome = rescuing
                .raise(&[
                    argument_error.to_any_object(),
                    RString::new_utf8("bad input").into(),
                ])
                .unwrap();
            assert_eq!(
                outcome.try_convert_to::<RString>().unwrap().to_str(),
                "bad input"
            );

            // An exception the fiber does not rescue comes back.
            let failing = Fiber::new(|_| {
                Fiber::yield_values(&[]).unwrap();

                crate::NilClass::new().into()
            });
            failing.resume(&[]).unwrap();
            let error = failing
                .raise(&[AnyException::new("IOError", Some("closed")).to_any_object()])
                .unwrap_err();
            assert!(error.message().contains("closed"));
            assert!(!failing.is_alive());

            // Keywords arrive as a trailing `Hash`.
            let echo = Fiber::new(|arguments: &[AnyObject]| {
                let mut keywords = Hash::new();
                keywords.store(Symbol::new("count"), Fixnum::new(arguments.len() as i64));

                Fiber::yield_with_keywords(&[arguments[0].clone()], keywords).unwrap()
            });
            let mut keywords = Hash::new();
            keywords.store(Symbol::new("verbose"), crate::Boolean::new(true));
            let yielded = echo
                .resume_with_keywords(&[Fixnum::new(1).into()], keywords)
                .unwrap()
                .try_convert_to::<crate::Array>()
                .unwrap();
            assert_eq!(yielded.at(0), Fixnum::new(1).into());
            let yielded_keywords = yielded.at(1).try_convert_to::<Hash>().unwrap();
            assert_eq!(
                yielded_keywords.at(&Symbol::new("count")),
                Fixnum::new(2).into()
            );
            let done = echo.resume(&[Fixnum::new(5).into()]).unwrap();
            assert_eq!(done, Fixnum::new(5).into());

            let mut keywords = Hash::new();
            keywords.store(Symbol::new("id"), Fixnum::new(9));
            let keyword_fiber = Fiber::new(|arguments: &[AnyObject]| arguments[0].clone());
            let received = keyword_fiber
                .transfer_with_keywords(&[], keywords)
                .unwrap()
                .try_convert_to::<Hash>()
                .unwrap();
            assert_eq!(received.at(&Symbol::new("id")), Fixnum::new(9).into());

            // `rb_obj_is_fiber` backs the type check.
            assert!(Fiber::current()
                .to_any_object()
                .try_convert_to::<Fiber>()
                .is_ok());
            assert!(Fixnum::new(1)
                .to_any_object()
                .try_convert_to::<Fiber>()
                .is_err());
            assert!(VM::eval("Fiber")
                .unwrap()
                .try_convert_to::<Fiber>()
                .is_err());
        });
    }

    #[test]
    fn test_scheduler() {
        use crate::{Array, Float, NilClass, Symbol, Thread};
        use std::time::Duration;

        crate::on_ruby_thread(|| {
            assert!(Fiber::scheduler().is_none());
            assert!(Fiber::current_scheduler().is_none());

            let scheduler = VM::eval(
                "Class.new do
                   attr_reader :log
                   def initialize; @log = []; end
                   def block(blocker, timeout = nil); @log << :block; end
                   def unblock(blocker, fiber); @log << :unblock; end
                   def kernel_sleep(duration = nil); @log << [:kernel_sleep, duration]; end
                   def io_wait(io, events, timeout); @log << :io_wait; end
                   def fiber(&block); @log << :fiber; Fiber.new(blocking: false, &block).tap(&:resume); end
                   def close; @log << :close; end
                 end.new",
            )
            .unwrap();

            Fiber::set_scheduler(&scheduler).unwrap();
            assert!(Fiber::scheduler().unwrap().equals(&scheduler));
            // The main fiber is blocking.
            assert!(Fiber::current_scheduler().is_none());
            assert!(Fiber::current_scheduler_for_thread(&Thread::current()).is_none());

            // Inside a non-blocking fiber, the scheduler is current.
            let current = VM::eval(
                "found = nil
                 Fiber.schedule { found = [Fiber.current_scheduler, sleep(0.25)] }
                 found",
            )
            .unwrap()
            .try_convert_to::<Array>()
            .unwrap();
            assert!(current.at(0).equals(&scheduler));

            // The previous scheduler is closed when replaced.
            Fiber::set_scheduler(&NilClass::new()).unwrap();
            assert!(Fiber::scheduler().is_none());

            let log = scheduler.protect_send("log", &[]).unwrap();
            let log = log.try_convert_to::<Array>().unwrap();
            assert_eq!(log.at(0), Symbol::new("fiber").into());
            let sleep = log.at(1).try_convert_to::<Array>().unwrap();
            assert_eq!(sleep.at(0), Symbol::new("kernel_sleep").into());
            assert_eq!(
                sleep.at(1).try_convert_to::<Float>().unwrap().to_f64(),
                0.25
            );
            assert_eq!(log.at(2), Symbol::new("close").into());

            // A scheduler without the required hooks is refused.
            let error = Fiber::set_scheduler(&VM::eval("Object.new").unwrap()).unwrap_err();
            assert_eq!(error.class().name().unwrap().to_str(), "ArgumentError");
            assert!(Fiber::scheduler().is_none());

            assert!(Fiber::make_scheduler_timeout(None).is_nil());
            let timeout = Fiber::make_scheduler_timeout(Some(Duration::new(2, 500_000_000)));
            assert_eq!(timeout.try_convert_to::<Float>().unwrap().to_f64(), 2.5);
            let huge = Fiber::make_scheduler_timeout(Some(Duration::from_secs(u64::MAX)));
            assert!(huge.try_convert_to::<Float>().unwrap().to_f64() > 1e9);
        });
    }

    // The raw hooks in `rubysys::scheduler` call the scheduler's methods
    // with the arguments they were given.
    #[test]
    fn test_raw_scheduler_hooks() {
        use crate::rubysys::scheduler as raw;
        use crate::{types::c_void, Array, RString, Symbol};

        crate::on_ruby_thread(|| {
            let scheduler = VM::eval(
                "Class.new do
                   attr_reader :log
                   def initialize; @log = []; end
                   %i[block unblock kernel_sleep io_wait process_wait io_read io_write io_pread
                      io_pwrite io_close address_resolve io_select close].each do |name|
                     define_method(name) { |*args| @log << [name, *args.map { _1.is_a?(IO::Buffer) ? :buffer : _1 }]; name }
                   end
                   def fiber(*args, **kw, &block); @log << [:fiber, *args, kw]; :fiber; end
                 end.new",
            )
            .unwrap();
            let s = scheduler.value();
            let io = VM::eval("$stdout").unwrap().value();
            let nil = crate::NilClass::new().value();
            let int = |n: i64| Fixnum::new(n).value();
            let mut memory = [0u8; 16];
            let memory_ptr = memory.as_mut_ptr() as *mut c_void;

            let returned = unsafe {
                vec![
                    raw::rb_fiber_scheduler_close(s),
                    raw::rb_fiber_scheduler_kernel_sleep(s, int(1)),
                    raw::rb_fiber_scheduler_kernel_sleepv(s, 1, [int(2)].as_mut_ptr()),
                    raw::rb_fiber_scheduler_process_wait(s, 123, 4),
                    raw::rb_fiber_scheduler_block(s, int(5), int(6)),
                    raw::rb_fiber_scheduler_unblock(s, int(7), int(8)),
                    raw::rb_fiber_scheduler_io_wait(s, io, int(1), int(9)),
                    raw::rb_fiber_scheduler_io_wait_readable(s, io),
                    raw::rb_fiber_scheduler_io_wait_writable(s, io),
                    #[cfg(not(ruby_gte_3_2))]
                    raw::rb_fiber_scheduler_io_read(s, io, nil, 10),
                    #[cfg(ruby_gte_3_2)]
                    raw::rb_fiber_scheduler_io_read(s, io, nil, 10, 11),
                    #[cfg(not(ruby_gte_3_2))]
                    raw::rb_fiber_scheduler_io_write(s, io, nil, 12),
                    #[cfg(ruby_gte_3_2)]
                    raw::rb_fiber_scheduler_io_write(s, io, nil, 12, 13),
                    #[cfg(not(ruby_gte_3_2))]
                    raw::rb_fiber_scheduler_io_pread(s, io, nil, 14, 1 << 40),
                    #[cfg(ruby_gte_3_2)]
                    raw::rb_fiber_scheduler_io_pread(s, io, 1 << 40, nil, 14, 15),
                    #[cfg(not(ruby_gte_3_2))]
                    raw::rb_fiber_scheduler_io_pwrite(s, io, nil, 16, 1 << 41),
                    #[cfg(ruby_gte_3_2)]
                    raw::rb_fiber_scheduler_io_pwrite(s, io, 1 << 41, nil, 16, 17),
                    raw::rb_fiber_scheduler_io_close(s, io),
                    raw::rb_fiber_scheduler_address_resolve(
                        s,
                        RString::new_utf8("localhost").value(),
                    ),
                ]
            };
            assert!(returned.iter().all(|value| !value.is_nil()));

            let log = scheduler.protect_send("log", &[]).unwrap();
            let entries: Vec<String> = log
                .try_convert_to::<Array>()
                .unwrap()
                .into_iter()
                .map(|entry| {
                    entry
                        .protect_send("inspect", &[])
                        .unwrap()
                        .try_convert_to::<RString>()
                        .unwrap()
                        .to_string()
                })
                .collect();

            let mut expected = vec![
                "[:close]",
                "[:kernel_sleep, 1]",
                "[:kernel_sleep, 2]",
                "[:process_wait, 123, 4]",
                "[:block, 5, 6]",
                "[:unblock, 7, 8]",
                "[:io_wait, #<IO:<STDOUT>>, 1, 9]",
                "[:io_wait, #<IO:<STDOUT>>, 1, nil]",
                "[:io_wait, #<IO:<STDOUT>>, 4, nil]",
            ];
            #[cfg(not(ruby_gte_3_2))]
            expected.extend([
                "[:io_read, #<IO:<STDOUT>>, nil, 10]",
                "[:io_write, #<IO:<STDOUT>>, nil, 12]",
                "[:io_pread, #<IO:<STDOUT>>, nil, 14, 1099511627776]",
                "[:io_pwrite, #<IO:<STDOUT>>, nil, 16, 2199023255552]",
            ]);
            #[cfg(ruby_gte_3_2)]
            expected.extend([
                "[:io_read, #<IO:<STDOUT>>, nil, 10, 11]",
                "[:io_write, #<IO:<STDOUT>>, nil, 12, 13]",
                "[:io_pread, #<IO:<STDOUT>>, nil, 1099511627776, 14, 15]",
                "[:io_pwrite, #<IO:<STDOUT>>, nil, 2199023255552, 16, 17]",
            ]);
            expected.extend([
                "[:io_close, #<IO:<STDOUT>>]",
                "[:address_resolve, \"localhost\"]",
            ]);
            assert_eq!(entries, expected);

            // The memory variants wrap the memory in an `IO::Buffer`.
            let log_size = |scheduler: &crate::AnyObject| {
                scheduler
                    .protect_send("log", &[])
                    .unwrap()
                    .try_convert_to::<Array>()
                    .unwrap()
                    .length()
            };
            let before = log_size(&scheduler);
            unsafe {
                raw::rb_fiber_scheduler_io_read_memory(s, io, memory_ptr, 16, 4);
                raw::rb_fiber_scheduler_io_write_memory(s, io, memory_ptr as *const c_void, 16, 4);
                #[cfg(ruby_gte_3_3)]
                raw::rb_fiber_scheduler_io_pread_memory(s, io, 99, memory_ptr, 16, 4);
                #[cfg(ruby_gte_3_3)]
                raw::rb_fiber_scheduler_io_pwrite_memory(
                    s,
                    io,
                    99,
                    memory_ptr as *const c_void,
                    16,
                    4,
                );
            }
            let log = scheduler
                .protect_send("log", &[])
                .unwrap()
                .try_convert_to::<Array>()
                .unwrap();
            let name = |i: usize| log.at(i as i64).try_convert_to::<Array>().unwrap().at(0);
            assert_eq!(name(before), Symbol::new("io_read").into());
            assert_eq!(name(before + 1), Symbol::new("io_write").into());
            #[cfg(ruby_gte_3_3)]
            {
                assert_eq!(name(before + 2), Symbol::new("io_pread").into());
                assert_eq!(name(before + 3), Symbol::new("io_pwrite").into());
            }

            #[cfg(ruby_gte_3_2)]
            unsafe {
                let select = raw::rb_fiber_scheduler_io_select(s, int(1), int(2), int(3), int(4));
                assert_eq!(select, Symbol::new("io_select").value());
                let selectv = raw::rb_fiber_scheduler_io_selectv(
                    s,
                    4,
                    [int(5), int(6), int(7), int(8)].as_mut_ptr(),
                );
                assert_eq!(selectv, Symbol::new("io_select").value());

                let mut keywords = crate::Hash::new();
                keywords.store(Symbol::new("blocking"), crate::Boolean::new(false));
                let fiber = raw::rb_fiber_scheduler_fiber(s, 1, [keywords.value()].as_mut_ptr(), 1);
                assert_eq!(fiber, Symbol::new("fiber").value());

                let log = scheduler
                    .protect_send("log", &[])
                    .unwrap()
                    .try_convert_to::<Array>()
                    .unwrap();
                let last = |back: i64| {
                    log.at(log.length() as i64 - back)
                        .protect_send("inspect", &[])
                        .unwrap()
                        .try_convert_to::<RString>()
                        .unwrap()
                        .to_string()
                };
                assert_eq!(last(3), "[:io_select, 1, 2, 3, 4]");
                assert_eq!(last(2), "[:io_select, 5, 6, 7, 8]");
                assert_eq!(last(1), "[:fiber, {:blocking=>false}]");
            }
        });
    }
}
