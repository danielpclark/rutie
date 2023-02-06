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

pub use class::any_exception::AnyException;
pub use class::any_object::AnyObject;
pub use class::array::Array;
pub use class::binding::Binding;
pub use class::boolean::Boolean;
pub use class::class::Class;
pub use class::encoding::Encoding;
pub use class::enumerator::Enumerator;
pub use class::fixnum::Fixnum;
pub use class::float::Float;
pub use class::gc::GC;
pub use class::hash::Hash;
pub use class::integer::Integer;
pub use class::module::Module;
pub use class::nil_class::NilClass;
pub use class::rproc::Proc;
pub use class::string::RString;
pub use class::symbol::Symbol;
pub use class::thread::Thread;
pub use class::vm::VM;

pub use class::traits::encoding_support::EncodingSupport;
pub use class::traits::exception::Exception;
pub use class::traits::object::Object;
pub use class::traits::try_convert::TryConvert;
pub use class::traits::verified_object::VerifiedObject;

pub use helpers::codepoint_iterator::CodepointIterator;

use std::sync::{Arc, RwLock};

#[cfg(test)]
lazy_static! {
    pub static ref LOCK_FOR_TEST: RwLock<i32> = RwLock::new(0);
}

#[cfg(test)]
mod current_ruby {
    use super::*;
    use super::{Object, RString, VM};
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
