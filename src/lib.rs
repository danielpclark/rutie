#![allow(unused_imports, dead_code)]
#[macro_use]
extern crate lazy_static;

#[cfg(windows)]
/// Defines a function for Ruby to call, or names the type of one, with the
/// right ABI: `extern "C"`, or `extern "C-unwind"` on Windows.
///
/// Use it for functions Rutie passes to Ruby that a Ruby exception (`raise`,
/// `throw`, `break`) can pass through, such as a method body given to
/// [`Object::def`](trait.Object.html#method.def) or an allocator given to
/// [`Class::define_alloc_func`](struct.Class.html#method.define_alloc_func).
/// `methods!` and `unsafe_methods!` already use it.
///
/// On Windows, a Ruby that jumps to its exception handlers with a `longjmp`
/// that unwinds the frames in between (as Ruby 2.5 for Windows did) aborts
/// the process when the unwind leaves an `extern "C"` function (Rust 1.81 and
/// later). RubyInstaller's Ruby 3 builds do not unwind; the ABI is kept so
/// callbacks are safe with any Windows Ruby. `extern "C-unwind"` needs Rust
/// 1.71 or later.
///
/// `rutie_callback! { fn name(...) -> T { ... } }` defines a function (with
/// any attributes, visibility and generics), and
/// `rutie_callback!(type fn(...) -> T)` is its type.
///
/// # Examples
///
/// ```
/// #[macro_use] extern crate rutie;
///
/// use rutie::{AnyObject, Class, Object, VM};
///
/// rutie_callback! {
///     fn refuse(klass: Class) -> AnyObject {
///         VM::raise(Class::from_existing("TypeError"), "allocation refused");
///         klass.into()
///     }
/// }
///
/// fn main() {
///     # VM::init();
///     let allocator: rutie_callback!(type fn(Class) -> AnyObject) = refuse;
///
///     Class::new("Refused", None).define_alloc_func(allocator);
///
///     // The exception passes through `allocate` back to Ruby.
///     assert!(VM::eval("Refused.new").is_err());
/// }
/// ```
#[macro_export]
macro_rules! rutie_callback {
    (type fn $($signature:tt)*) => { extern "C-unwind" fn $($signature)* };
    ($(#[$attribute:meta])* $visibility:vis fn $($function:tt)*) => {
        $(#[$attribute])* $visibility extern "C-unwind" fn $($function)*
    };
}

#[cfg(not(windows))]
/// Defines a function for Ruby to call, or names the type of one, with the
/// right ABI: `extern "C"`, or `extern "C-unwind"` on Windows.
///
/// Use it for functions Rutie passes to Ruby that a Ruby exception (`raise`,
/// `throw`, `break`) can pass through, such as a method body given to
/// [`Object::def`](trait.Object.html#method.def) or an allocator given to
/// [`Class::define_alloc_func`](struct.Class.html#method.define_alloc_func).
/// `methods!` and `unsafe_methods!` already use it.
///
/// On Windows, a Ruby that jumps to its exception handlers with a `longjmp`
/// that unwinds the frames in between (as Ruby 2.5 for Windows did) aborts
/// the process when the unwind leaves an `extern "C"` function (Rust 1.81 and
/// later). RubyInstaller's Ruby 3 builds do not unwind; the ABI is kept so
/// callbacks are safe with any Windows Ruby. `extern "C-unwind"` needs Rust
/// 1.71 or later.
///
/// `rutie_callback! { fn name(...) -> T { ... } }` defines a function (with
/// any attributes, visibility and generics), and
/// `rutie_callback!(type fn(...) -> T)` is its type.
///
/// # Examples
///
/// ```
/// #[macro_use] extern crate rutie;
///
/// use rutie::{AnyObject, Class, Object, VM};
///
/// rutie_callback! {
///     fn refuse(klass: Class) -> AnyObject {
///         VM::raise(Class::from_existing("TypeError"), "allocation refused");
///         klass.into()
///     }
/// }
///
/// fn main() {
///     # VM::init();
///     let allocator: rutie_callback!(type fn(Class) -> AnyObject) = refuse;
///
///     Class::new("Refused", None).define_alloc_func(allocator);
///
///     // The exception passes through `allocate` back to Ruby.
///     assert!(VM::eval("Refused.new").is_err());
/// }
/// ```
#[macro_export]
macro_rules! rutie_callback {
    (type fn $($signature:tt)*) => { extern "C" fn $($signature)* };
    ($(#[$attribute:meta])* $visibility:vis fn $($function:tt)*) => {
        $(#[$attribute])* $visibility extern "C" fn $($function)*
    };
}

mod binding;
mod class;
mod helpers;
pub mod rubysys;

#[macro_use]
pub mod dsl;

pub mod typed_data;
pub mod types;
pub mod util;

// libruby for crates that depend on Rutie. Rutie's own unit tests link it
// differently; see `link_libruby` in build.rs.
#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/link_ruby.rs"));

pub use crate::class::{
    any_exception::AnyException,
    any_object::AnyObject,
    array::Array,
    binding::Binding,
    boolean::Boolean,
    class::Class,
    complex::Complex,
    debug::{DebugInspector, ProfileFrame},
    encoding::Encoding,
    enumerator::{Enumerator, EnumeratorIterator},
    fiber::Fiber,
    fixnum::Fixnum,
    float::Float,
    gc::GC,
    global_variable::GlobalVariable,
    hash::{Hash, HashIterator},
    integer::Integer,
    io::{File, IO},
    marshal::Marshal,
    method::Method,
    module::Module,
    mutex::{Mutex, MutexGuard},
    nil_class::NilClass,
    ractor::{Ractor, RactorLocalKey},
    range::Range,
    rational::Rational,
    regexp::{MatchData, Regexp},
    rproc::Proc,
    rstruct::Struct,
    string::{CodeRange, RString},
    symbol::Symbol,
    thread::Thread,
    time::Time,
    vm::VM,
};

#[cfg(ruby_gte_3_2)]
pub use crate::class::thread::{InternalThreadEvent, InternalThreadEventHook};

#[cfg(ruby_gte_3_3)]
pub use crate::class::{debug::PostponedJob, thread::InternalThreadSpecificKey};

pub use crate::class::traits::{
    encoding_support::EncodingSupport, exception::Exception, object::Object,
    try_convert::TryConvert, verified_object::VerifiedObject,
};

pub use crate::helpers::{
    codepoint_iterator::CodepointIterator,
    scan_args::{KeywordArgs, ScannedArgs},
};

use std::sync::{Arc, RwLock};

#[cfg(test)]
lazy_static! {
    pub static ref LOCK_FOR_TEST: RwLock<i32> = RwLock::new(0);
}

// Ruby is bound to the native thread that starts it: `ruby_init` records
// that thread's stack for the GC to scan and for stack overflow checks. The
// test harness runs every test on its own thread, so unit tests send their
// bodies to one long-lived thread that owns the VM instead of calling
// `VM::init` themselves. A panic in `test` is re-raised in the test thread.
#[cfg(test)]
pub(crate) fn on_ruby_thread<F>(test: F)
where
    F: FnOnce() + Send + 'static,
{
    use std::{
        panic::{self, AssertUnwindSafe},
        sync::{mpsc, Mutex},
        thread,
    };

    type Job = (
        String,
        Box<dyn FnOnce() + Send>,
        mpsc::Sender<thread::Result<()>>,
    );

    lazy_static! {
        static ref RUBY_THREAD: Mutex<mpsc::Sender<Job>> = {
            let (sender, receiver) = mpsc::channel::<Job>();

            thread::Builder::new()
                .name("ruby".to_string())
                .stack_size(16 * 1024 * 1024)
                .spawn(move || {
                    VM::init();

                    let trace = std::env::var_os("RUTIE_TEST_TRACE").is_some();

                    for (name, test, result) in receiver {
                        // Written straight to stderr (not `eprintln!`, which the
                        // harness captures) so a crash still shows which test ran.
                        if trace {
                            use std::io::Write;
                            let _ = writeln!(std::io::stderr(), "[ruby thread] running {}", name);
                        }

                        let outcome = {
                            let _guard = LOCK_FOR_TEST.write().unwrap_or_else(|e| e.into_inner());

                            panic::catch_unwind(AssertUnwindSafe(test))
                        };

                        let _ = result.send(outcome);
                    }
                })
                .expect("failed to start the Ruby test thread");

            Mutex::new(sender)
        };
    }

    let (sender, receiver) = mpsc::channel();

    RUBY_THREAD
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .send((
            thread::current().name().unwrap_or("<unnamed>").to_string(),
            Box::new(test),
            sender,
        ))
        .expect("the Ruby test thread has stopped");

    if let Err(payload) = receiver.recv().expect("the Ruby test thread has stopped") {
        // The Ruby thread's panic message went to the output capture of the
        // test that started the thread, so repeat it here, where the harness
        // shows it with this test's failure.
        let message = payload
            .downcast_ref::<&str>()
            .map(|message| message.to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "(non-string panic payload)".to_string());
        eprintln!("panicked on the Ruby test thread: {}", message);

        panic::resume_unwind(payload);
    }
}

#[cfg(test)]
mod current_ruby {
    use super::{Object, RString, VM, *};
    use std::process::Command;

    #[test]
    fn is_linked_ruby() {
        crate::on_ruby_thread(|| {
            let rv = RString::from(VM::eval("RUBY_VERSION").unwrap().value()).to_string();
            // The same Ruby `build.rs` links against.
            let ruby = std::env::var_os("RUBY").unwrap_or_else(|| "ruby".into());
            let output = Command::new(ruby)
                .arg("-e")
                .arg("printf RUBY_VERSION")
                .output()
                .unwrap()
                .stdout;
            let crv = String::from_utf8_lossy(&output);

            assert_eq!(
                rv, crv,
                "\nCurrent console Ruby is version {} but the \
                       linked Ruby is version {} \
                       Please run `cargo clean` first to remove previously used symbolic link in \
                       the dependency directory.",
                crv, rv
            );
        });
    }

    // `build.rs` sets the version cfg flags from the Ruby it links against;
    // make sure they agree with the Ruby actually running.
    #[test]
    fn cfg_flags_match_linked_ruby() {
        crate::on_ruby_thread(|| {
            let version = RString::from(
                VM::eval("RUBY_VERSION.split('.')[0, 2].join('.')")
                    .unwrap()
                    .value(),
            )
            .to_string();

            let cfg_version = if cfg!(ruby_3_0) {
                "3.0"
            } else if cfg!(ruby_3_1) {
                "3.1"
            } else if cfg!(ruby_3_2) {
                "3.2"
            } else {
                "unsupported"
            };

            assert_eq!(version, cfg_version, "Ruby version cfg flag mismatch");
            assert!(cfg!(ruby_gte_3_0));
            assert_eq!(cfg!(ruby_gte_3_1), version.as_str() >= "3.1");
            assert_eq!(cfg!(ruby_gte_3_2), version.as_str() >= "3.2");
        });
    }

    // Ruby 3.2 with YJIT exports a Rust runtime from libruby; if this binary
    // bound its panic runtime to it, re-raising a caught panic aborted the
    // whole test run (see `link_libruby` in build.rs).
    #[test]
    fn rust_panics_unwind_with_libruby_linked() {
        use std::panic::{self, AssertUnwindSafe};

        let caught = panic::catch_unwind(|| panic!("first")).unwrap_err();
        let resumed = panic::catch_unwind(AssertUnwindSafe(|| panic::resume_unwind(caught)));

        assert_eq!(resumed.unwrap_err().downcast_ref::<&str>(), Some(&"first"));
    }
}
