use crate::{
    binding::{ractor, vm},
    rubysys::ractor::RbRactorLocalKey,
    AnyException, AnyObject, Object, IO,
};

/// Helpers for Ruby's Ractors: each Ractor's standard streams, and sharing
/// objects between Ractors.
///
/// See [`RactorLocalKey`](struct.RactorLocalKey.html) for values each Ractor
/// keeps for itself, and [`VM::ext_ractor_safe`](struct.VM.html#method.ext_ractor_safe)
/// for methods other Ractors may call.
pub struct Ractor;

impl Ractor {
    /// Returns the current Ractor's `$stdin` (`rb_ractor_stdin`). Each
    /// Ractor has its own; in the main Ractor it is
    /// [`IO::stdin`](struct.IO.html#method.stdin).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IO, Object, Ractor, VM};
    /// # VM::init();
    ///
    /// assert!(Ractor::stdin().is_equal(&IO::stdin()));
    /// ```
    pub fn stdin() -> IO {
        IO::from(ractor::stdin())
    }

    /// Returns the current Ractor's `$stdout` (`rb_ractor_stdout`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Ractor, VM};
    /// # VM::init();
    ///
    /// assert!(Ractor::stdout().is_equal(&VM::eval("$stdout").unwrap()));
    /// ```
    pub fn stdout() -> IO {
        IO::from(ractor::stdout())
    }

    /// Returns the current Ractor's `$stderr` (`rb_ractor_stderr`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Ractor, VM};
    /// # VM::init();
    ///
    /// assert!(Ractor::stderr().is_equal(&VM::eval("$stderr").unwrap()));
    /// ```
    pub fn stderr() -> IO {
        IO::from(ractor::stderr())
    }

    /// Replaces the current Ractor's `$stdin` (`rb_ractor_stdin_set`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IO, Object, Ractor, RString, VM};
    /// # VM::init();
    ///
    /// let pipe = VM::eval("reader, writer = IO.pipe; writer.puts('typed'); writer.close; reader").unwrap();
    /// let original = Ractor::stdin();
    ///
    /// Ractor::set_stdin(&pipe.try_convert_to::<IO>().unwrap());
    ///
    /// let line = VM::eval("$stdin.gets").unwrap().try_convert_to::<RString>().unwrap();
    ///
    /// Ractor::set_stdin(&original);
    ///
    /// assert_eq!(line.to_str(), "typed\n");
    /// assert!(Ractor::stdin().is_equal(&original));
    /// ```
    pub fn set_stdin(io: &IO) {
        ractor::set_stdin(io.value())
    }

    /// Replaces the current Ractor's `$stdout` (`rb_ractor_stdout_set`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IO, Object, Ractor, RString, VM};
    /// # VM::init();
    ///
    /// let pipe = VM::eval("$pipe = IO.pipe").unwrap();
    /// let writer = VM::eval("$pipe[1]").unwrap().try_convert_to::<IO>().unwrap();
    /// let original = Ractor::stdout();
    ///
    /// Ractor::set_stdout(&writer);
    /// VM::eval("puts 'captured'").unwrap();
    /// Ractor::set_stdout(&original);
    ///
    /// writer.close().unwrap();
    ///
    /// let output = VM::eval("$pipe[0].read").unwrap().try_convert_to::<RString>().unwrap();
    ///
    /// assert_eq!(output.to_str(), "captured\n");
    /// ```
    pub fn set_stdout(io: &IO) {
        ractor::set_stdout(io.value())
    }

    /// Replaces the current Ractor's `$stderr` (`rb_ractor_stderr_set`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{IO, Object, Ractor, RString, VM};
    /// # VM::init();
    ///
    /// VM::eval("$pipe = IO.pipe").unwrap();
    ///
    /// let writer = VM::eval("$pipe[1]").unwrap().try_convert_to::<IO>().unwrap();
    /// let original = Ractor::stderr();
    ///
    /// Ractor::set_stderr(&writer);
    /// VM::eval("warn 'careful'").unwrap();
    /// Ractor::set_stderr(&original);
    ///
    /// writer.close().unwrap();
    ///
    /// let output = VM::eval("$pipe[0].read").unwrap().try_convert_to::<RString>().unwrap();
    ///
    /// assert_eq!(output.to_str(), "careful\n");
    /// ```
    pub fn set_stderr(io: &IO) {
        ractor::set_stderr(io.value())
    }

    /// Returns `true` if `object` may be passed between Ractors without
    /// copying (Ruby's `Ractor.shareable?`, `rb_ractor_shareable_p`):
    /// immediates, and deeply frozen objects.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Fixnum, Object, Ractor, RString, VM};
    /// # VM::init();
    ///
    /// assert!(Ractor::is_shareable(&Fixnum::new(1)));
    /// assert!(!Ractor::is_shareable(&RString::new_utf8("text")));
    ///
    /// let frozen = VM::eval("['text'.freeze].freeze").unwrap();
    ///
    /// assert!(Ractor::is_shareable(&frozen));
    /// assert!(!Ractor::is_shareable(&VM::eval("['text'].freeze").unwrap()));
    /// ```
    pub fn is_shareable<T: Object>(object: &T) -> bool {
        ractor::is_shareable(object.value())
    }

    /// Deeply freezes `object` so any Ractor can use it (Ruby's
    /// `Ractor.make_shareable`, `rb_ractor_make_shareable`), and returns it,
    /// or the `Ractor::Error` for an object that cannot be shared (such as a
    /// `Proc` whose `self` is not shareable).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Array, Object, Ractor, RString, VM};
    /// # VM::init();
    ///
    /// let mut names = Array::new();
    ///
    /// names.push(RString::new_utf8("ruby"));
    ///
    /// let names = Ractor::make_shareable(&names).unwrap();
    ///
    /// assert!(Ractor::is_shareable(&names));
    /// assert!(names.at(0).is_frozen());
    ///
    /// let proc = VM::eval("proc { self }").unwrap();
    ///
    /// assert!(Ractor::make_shareable(&proc).is_err());
    /// ```
    pub fn make_shareable<T: Object>(object: &T) -> Result<T, AnyException> {
        let object = object.value();

        vm::protect_value(|| ractor::make_shareable(object))
            .map(T::from)
            .map_err(AnyException::from)
    }

    /// Returns a deeply frozen, shareable copy of `object`, leaving `object`
    /// itself unchanged (Ruby's `Ractor.make_shareable(object, copy: true)`,
    /// `rb_ractor_make_shareable_copy`), or the error for an object that
    /// cannot be copied or shared.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Ractor, RString, VM};
    /// # VM::init();
    ///
    /// let text = RString::new_utf8("draft");
    /// let copy = Ractor::make_shareable_copy(&text).unwrap();
    ///
    /// assert!(Ractor::is_shareable(&copy));
    /// assert_eq!(copy.to_str(), "draft");
    /// assert!(!text.is_frozen());
    /// ```
    pub fn make_shareable_copy<T: Object>(object: &T) -> Result<T, AnyException> {
        let object = object.value();

        vm::protect_value(|| ractor::make_shareable_copy(object))
            .map(T::from)
            .map_err(AnyException::from)
    }
}

/// A key to Ractor-local storage: each Ractor keeps its own Ruby object
/// under the key (`rb_ractor_local_storage_value_*`), the way thread-local
/// storage keeps one value per thread.
///
/// Ruby never frees a key, so create each one once, for example in a
/// `lazy_static!`, and share it: it is `Send`, `Sync` and `Copy`. Like the
/// rest of Rutie, it must only be used from a Ruby thread. Ruby keeps the
/// stored object alive as long as its Ractor.
///
/// # Examples
///
/// ```
/// #[macro_use] extern crate rutie;
/// #[macro_use] extern crate lazy_static;
///
/// use rutie::{AnyObject, Class, NilClass, Object, RactorLocalKey, RString, VM};
///
/// lazy_static! {
///     static ref LABEL: RactorLocalKey = RactorLocalKey::new();
/// }
///
/// class!(Labels);
///
/// methods!(
///     Labels,
///     _rtself,
///
///     fn label() -> AnyObject {
///         LABEL.get().unwrap_or_else(|| NilClass::new().into())
///     }
///
///     fn set_label(label: RString) -> NilClass {
///         LABEL.set(&label.unwrap());
///
///         NilClass::new()
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     Class::new("Labels", None).define(|klass| {
///         // Ractor-safe: they only touch Ractor-local state.
///         unsafe { VM::ext_ractor_safe(true) };
///         klass.def_self("label", label);
///         klass.def_self("label=", set_label);
///         VM::ext_ractor_unsafe();
///     });
///
///     VM::eval("Labels.label = 'main'").unwrap();
///
///     // A new Ractor starts without a value, and its own does not leak out.
///     let seen = VM::eval(
///         "Warning[:experimental] = false
///          Ractor.new { before = Labels.label; Labels.label = 'other'; [before, Labels.label] }.value.inspect",
///     )
///     .unwrap();
///
///     assert_eq!(seen.try_convert_to::<RString>().unwrap().to_str(), "[nil, \"other\"]");
///     assert_eq!(LABEL.get().unwrap().try_convert_to::<RString>().unwrap().to_str(), "main");
/// }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct RactorLocalKey {
    key: RbRactorLocalKey,
}

// A key is an immutable handle that Ruby shares between Ractors; the values
// live in each Ractor.
unsafe impl Send for RactorLocalKey {}
unsafe impl Sync for RactorLocalKey {}

impl RactorLocalKey {
    /// Creates a key with no value in any Ractor
    /// (`rb_ractor_local_storage_value_newkey`). The key is never freed.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{RactorLocalKey, VM};
    /// # VM::init();
    ///
    /// let key = RactorLocalKey::new();
    ///
    /// assert!(key.get().is_none());
    /// ```
    pub fn new() -> Self {
        RactorLocalKey {
            key: ractor::local_storage_value_newkey(),
        }
    }

    /// Returns the current Ractor's object for this key, or `None` if it has
    /// not set one (`rb_ractor_local_storage_value_lookup`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, NilClass, Object, RactorLocalKey, VM};
    /// # VM::init();
    ///
    /// let key = RactorLocalKey::new();
    ///
    /// assert!(key.get().is_none());
    ///
    /// key.set(&NilClass::new());
    ///
    /// assert!(key.get().unwrap().is_nil());
    /// ```
    pub fn get(&self) -> Option<AnyObject> {
        ractor::local_storage_value_lookup(self.key).map(AnyObject::from)
    }

    /// Stores `value` as the current Ractor's object for this key
    /// (`rb_ractor_local_storage_value_set`), replacing any previous one.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Object, RactorLocalKey, VM};
    /// # VM::init();
    ///
    /// let key = RactorLocalKey::new();
    ///
    /// key.set(&Fixnum::new(1));
    /// key.set(&Fixnum::new(2));
    ///
    /// assert_eq!(key.get().unwrap().try_convert_to::<Fixnum>(), Ok(Fixnum::new(2)));
    /// ```
    pub fn set<T: Object>(&self, value: &T) {
        ractor::local_storage_value_set(self.key, value.value())
    }
}

#[cfg(test)]
mod tests {
    use super::{Ractor, RactorLocalKey};
    use crate::{AnyObject, Array, Class, NilClass, Object, RString, Symbol, IO, VM};

    lazy_static! {
        static ref RUTIE_TEST_KEY: RactorLocalKey = RactorLocalKey::new();
    }

    crate::class!(RutieRactorLocal);

    crate::methods!(
        RutieRactorLocal,
        _rtself,
        fn rutie_ractor_local_get() -> AnyObject {
            RUTIE_TEST_KEY
                .get()
                .unwrap_or_else(|| NilClass::new().into())
        },
        fn rutie_ractor_local_set(value: AnyObject) -> NilClass {
            RUTIE_TEST_KEY.set(&value.unwrap());

            NilClass::new()
        },
        fn rutie_ractor_shareable(value: AnyObject) -> crate::Boolean {
            crate::Boolean::new(Ractor::is_shareable(&value.unwrap()))
        }
    );

    #[test]
    fn test_ractor_local_storage_per_ractor() {
        crate::on_ruby_thread(|| {
            Class::new("RutieRactorLocal", None).define(|klass| {
                unsafe { VM::ext_ractor_safe(true) };
                klass.def_self("get", rutie_ractor_local_get);
                klass.def_self("set", rutie_ractor_local_set);
                klass.def_self("shareable?", rutie_ractor_shareable);
                VM::ext_ractor_unsafe();
            });

            assert!(RUTIE_TEST_KEY.get().is_none());
            RUTIE_TEST_KEY.set(&Symbol::new("main"));

            let results = VM::eval(
                "Warning[:experimental] = false
                 GC.start
                 rs = 3.times.map do |i|
                   Ractor.new(i) do |i|
                     before = RutieRactorLocal.get
                     RutieRactorLocal.set(\"ractor #{i}\")
                     GC.start
                     [before, RutieRactorLocal.get, RutieRactorLocal.shareable?(1), RutieRactorLocal.shareable?([])]
                   end
                 end
                 rs.map(&:value).inspect",
            )
            .unwrap()
            .try_convert_to::<RString>()
            .unwrap();

            assert_eq!(
                results.to_str(),
                "[[nil, \"ractor 0\", true, false], [nil, \"ractor 1\", true, false], \
                 [nil, \"ractor 2\", true, false]]"
            );

            // The main Ractor's value survives a GC (Ruby marks it).
            RUTIE_TEST_KEY.set(&RString::new_utf8("kept"));
            crate::GC::start();
            assert_eq!(
                RUTIE_TEST_KEY
                    .get()
                    .unwrap()
                    .try_convert_to::<RString>()
                    .unwrap()
                    .to_str(),
                "kept"
            );

            let other = RactorLocalKey::new();
            assert!(other.get().is_none());
        });
    }

    #[test]
    fn test_ractor_shareable() {
        crate::on_ruby_thread(|| {
            assert!(Ractor::is_shareable(&NilClass::new()));
            assert!(Ractor::is_shareable(&Symbol::new("sym")));
            assert!(Ractor::is_shareable(&Class::string()));
            assert!(!Ractor::is_shareable(&Array::new()));

            let nested = VM::eval("[{ key: ['value'] }]").unwrap();
            assert!(!Ractor::is_shareable(&nested));

            let copy = Ractor::make_shareable_copy(&nested).unwrap();
            assert!(Ractor::is_shareable(&copy));
            assert!(!copy.is_equal(&nested));
            assert!(!nested.is_frozen());

            let shared = Ractor::make_shareable(&nested).unwrap();
            assert!(shared.is_equal(&nested));
            assert!(Ractor::is_shareable(&nested));

            let error = Ractor::make_shareable(&VM::eval("proc { self }").unwrap()).unwrap_err();
            assert!(Class::from_existing("Ractor")
                .get_nested_class("Error")
                .case_equals(&error));
        });
    }

    #[test]
    fn test_ractor_streams() {
        crate::on_ruby_thread(|| {
            assert!(Ractor::stdin().is_equal(&IO::stdin()));
            assert!(Ractor::stdout().is_equal(&IO::stdout()));
            assert!(Ractor::stderr().is_equal(&IO::stderr()));

            let pipe = VM::eval("IO.pipe")
                .unwrap()
                .try_convert_to::<Array>()
                .unwrap();
            let reader = pipe.at(0).try_convert_to::<IO>().unwrap();
            let writer = pipe.at(1).try_convert_to::<IO>().unwrap();
            let original = Ractor::stdout();

            Ractor::set_stdout(&writer);
            assert!(VM::eval("$stdout").unwrap().is_equal(&writer));
            VM::eval("print 'to pipe'").unwrap();
            Ractor::set_stdout(&original);

            writer.close().unwrap();
            let output = unsafe { reader.send("read", &[]) };
            assert_eq!(
                output.try_convert_to::<RString>().unwrap().to_str(),
                "to pipe"
            );
            assert!(Ractor::stdout().is_equal(&original));
        });
    }
}
