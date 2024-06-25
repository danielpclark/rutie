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
    any_exception::AnyException, any_object::AnyObject, array::Array, binding::Binding,
    boolean::Boolean, class::Class, encoding::Encoding, enumerator::Enumerator, fixnum::Fixnum,
    float::Float, gc::GC, hash::Hash, integer::Integer, marshal::Marshal, module::Module,
    nil_class::NilClass, rproc::Proc, string::RString, symbol::Symbol, thread::Thread, vm::VM,
};

pub use crate::class::traits::{
    encoding_support::EncodingSupport, exception::Exception, object::Object,
    try_convert::TryConvert, verified_object::VerifiedObject,
};

pub use crate::helpers::codepoint_iterator::CodepointIterator;

use std::sync::{Arc, RwLock};

#[cfg(test)]
lazy_static! {
    pub static ref LOCK_FOR_TEST: RwLock<i32> = RwLock::new(0);
}

#[cfg(test)]
mod current_ruby {
    use super::{Object, RString, VM, *};
    use std::process::Command;

    #[test]
    fn is_linked_ruby() {
        let _guard = LOCK_FOR_TEST.write().unwrap();
        VM::init();

        let rv = RString::from(VM::eval("RUBY_VERSION").unwrap().value()).to_string();
        let output = Command::new("ruby")
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
    }
}
