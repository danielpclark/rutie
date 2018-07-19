#![allow(unused_imports,dead_code)]
#[macro_use]
extern crate lazy_static;

mod binding;
mod class;
pub mod rubysys;

#[macro_use]
pub mod dsl;

pub mod typed_data;
pub mod types;
pub mod util;

pub use class::any_exception::AnyException;
pub use class::any_object::AnyObject;
pub use class::array::Array;
pub use class::boolean::Boolean;
pub use class::class::Class;
pub use class::encoding::Encoding;
pub use class::fixnum::Fixnum;
pub use class::float::Float;
pub use class::gc::GC;
pub use class::hash::Hash;
pub use class::integer::Integer;
pub use class::nil_class::NilClass;
pub use class::module::Module;
pub use class::rproc::Proc;
pub use class::string::RString;
pub use class::symbol::Symbol;
pub use class::thread::Thread;
pub use class::vm::VM;

pub use class::traits::encoding_support::EncodingSupport;
pub use class::traits::exception::Exception;
pub use class::traits::object::Object;
pub use class::traits::verified_object::VerifiedObject;
pub use class::traits::try_convert::TryConvert;

#[test]
fn it_works() {}
