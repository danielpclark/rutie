#![allow(unused_imports, dead_code)]
#[macro_use]
extern crate lazy_static;

mod binding;
mod class;
mod helpers;
pub mod rubysys;

#[macro_use]
pub mod dsl;

pub mod typed_data;
pub mod types;
pub mod util;

pub use crate::class::{
    any_exception::AnyException,
    any_object::AnyObject,
    array::Array,
    binding::Binding,
    boolean::Boolean,
    class::Class,
    complex::Complex,
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

// Ruby 2 is bound to the native thread that starts it: `ruby_init` records
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

            let cfg_version = if cfg!(ruby_2_5) {
                "2.5"
            } else if cfg!(ruby_2_6) {
                "2.6"
            } else if cfg!(ruby_2_7) {
                "2.7"
            } else {
                "unsupported"
            };

            assert_eq!(version, cfg_version, "Ruby version cfg flag mismatch");
            assert_eq!(cfg!(ruby_gte_2_6), version.as_str() >= "2.6");
            assert_eq!(cfg!(ruby_gte_2_7), version.as_str() >= "2.7");
        });
    }
}
