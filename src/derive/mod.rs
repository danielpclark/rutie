//! Ruby classes, modules and exceptions from ordinary Rust: the
//! `#[rubyclass]`, `#[rubymethods]`, `#[rubymodule]`, `#[rubyfunction]` and
//! `#[rubyerror]` attributes, and `#[derive(Mark)]` (the `derive` feature).
//!
//! ```
//! use rutie::{rubyclass, rubymethods, Class, Object, RString, VM};
//!
//! #[rubyclass(path = "Stats::Counter")]
//! #[derive(Clone)]
//! pub struct Counter {
//!     #[ruby(get)]
//!     count: i64,
//!     step: i64,
//! }
//!
//! #[rubymethods]
//! impl Counter {
//!     // `new` becomes `initialize`: `Stats::Counter.new(40, step: 2)`.
//!     pub fn new(start: Option<i64>, #[ruby(keyword)] step: Option<i64>) -> Self {
//!         Counter { count: start.unwrap_or(0), step: step.unwrap_or(1) }
//!     }
//!
//!     // `&mut self` is the struct inside the Ruby object.
//!     pub fn tick(&mut self) -> Result<i64, String> {
//!         self.count = self.count.checked_add(self.step).ok_or("counter overflow")?;
//!         Ok(self.count)
//!     }
//!
//!     #[ruby(name = "to_s")]
//!     pub fn describe(&self) -> String {
//!         format!("#<Counter {}>", self.count)
//!     }
//! }
//!
//! fn main() {
//!     # VM::init();
//!     Counter::define_ruby();
//!
//!     let result = VM::eval("c = Stats::Counter.new(40, step: 2); c.tick; c.dup.to_s").unwrap();
//!     assert_eq!(result.try_convert_to::<RString>().unwrap().to_str(), "#<Counter 42>");
//!
//!     // A wrong argument is a `TypeError`; `Err` is raised after the
//!     // method's Rust values are dropped.
//!     let error = VM::eval("Stats::Counter.new('40')").unwrap_err();
//!     assert!(Class::from_existing("TypeError").case_equals(&error));
//! }
//! ```
//!
//! # `#[rubyclass]`
//!
//! `#[rubyclass(path = "A::B")]` on a struct stores it in the Ruby objects of
//! class `A::B` (the path defaults to the struct's name). Its generated
//! `define_ruby()` defines the class, any modules in its path that don't
//! exist yet, its allocator, the field accessors, `initialize_copy`, and the
//! methods of its `#[rubymethods]` blocks.
//!
//! - **Borrows are checked.** The struct sits behind a borrow flag: if Ruby
//!   code called from a method reaches the same object again, a conflicting
//!   borrow (`&mut self` while borrowed, or `&self` while borrowed mutably)
//!   raises a `RuntimeError` instead of creating aliasing references.
//! - **Ruby objects in fields are kept alive.** Each field whose type
//!   implements [`Mark`] (Ruby types, and `Option`, `Vec`, `HashMap`, tuples,
//!   … of them, and `#[derive(Mark)]` types) is marked for the GC and updated
//!   after `GC.compact`. `#[ruby(no_mark)]` skips a field;
//!   `mark = path::to::fn` replaces all of it with a `fn(&Self)` that marks
//!   with [`GC::mark`](../struct.GC.html#method.mark) (objects then never
//!   move).
//! - **Ruby's object model.** `Class#new` allocates an uninitialized object
//!   and calls `initialize`, which is the struct's constructor (a method
//!   without a receiver named `new`, or marked `#[ruby(constructor)]`), so
//!   Ruby subclasses can define `initialize` and call `super`. Using an
//!   uninitialized object raises `TypeError`. `dup` and `clone` copy the
//!   struct when it is `Clone` and raise `TypeError` otherwise.
//!   `allocator = false` leaves the class without an allocator: objects then
//!   come only from Rust.
//! - **Inheritance.** `extends = Parent` makes the class a subclass of
//!   another `#[rubyclass]`; the struct holds the parent's struct in a field
//!   marked `#[ruby(base)]`, and the parent's methods work on the child's
//!   objects.
//! - **Accessors.** `#[ruby(get)]`, `#[ruby(set)]` (or both) on a field define
//!   `name` and `name=`; `#[ruby(get, name = "label")]` renames them. The
//!   getter returns a clone of the field.
//! - `size = path::to::fn` reports memory to `ObjectSpace.memsize_of`.
//!
//! # `#[rubymethods]`
//!
//! On an `impl` block of a `#[rubyclass]` struct, which stays plain Rust:
//!
//! - `&self` and `&mut self` methods are instance methods; methods without a
//!   receiver are singleton methods (`def self.name`), except the
//!   constructor, which is `initialize`.
//! - Positional parameters are `&T` or `T` (required) and `Option<&T>` or
//!   `Option<T>` (optional, trailing; missing or `nil` is `None`), for Ruby
//!   types (`&RString`, `Integer`, `&AnyObject`, …), Rust types (`i64`, `f64`,
//!   `bool`, `String`, `&str`, `&[u8]`, [`Bytes`], `Vec<T>`, `HashMap<K, V>`,
//!   …) and `#[rubyclass]` structs (`&T` or `&mut T`). A wrong count raises
//!   `ArgumentError`, a wrong type `TypeError`, and an integer out of range
//!   `RangeError`.
//! - `#[ruby(splat)] rest: Vec<T>` (or `Array`) takes the remaining
//!   arguments; `#[ruby(keyword)] name: T` is a keyword argument (optional
//!   when `T` is an `Option`); `#[ruby(block)] block: Proc` (or
//!   `Option<Proc>`) is the block.
//! - The return type is anything [`IntoRubyObject`] (Ruby and Rust types,
//!   `()`, `Option`, `Vec`, `HashMap`, tuples, `#[rubyclass]` structs), or a
//!   `Result` of one whose `Err` is [`IntoException`] (`AnyException`,
//!   `String`, `std::io::Error`, `Box<dyn Error>`, `#[rubyerror]` types).
//! - `#[ruby(name = "zero?")]` names the Ruby method; `#[ruby(skip)]` leaves
//!   a method out; `#[ruby(private)]` and `#[ruby(protected)]` set its
//!   visibility.
//! - `#[ruby(without_gvl)]` runs the method with Ruby's GVL released, so
//!   other Ruby threads run meanwhile. Its parameters and result must be
//!   [`WithoutGvl`] (plain Rust data), a `&self` receiver's struct must be
//!   `Sync`, and it must not use Ruby.
//! - A struct can have up to eight `#[rubymethods]` blocks:
//!   `#[rubymethods(part = 1)]` … `(part = 7)` for the ones after the first.
//!
//! Every generated method converts its arguments, takes its borrows, then
//! calls your method under `rb_protect`: a Ruby exception raised by Ruby code
//! your method calls is caught there, the borrows and converted arguments
//! are released, and the exception continues. Your own Rust values are only
//! dropped if you return `Err` instead (#159). A panic is raised as a
//! `RuntimeError`.
//!
//! Methods with only required positional parameters (and at most fifteen)
//! are defined with that arity, so Ruby checks the count and passes the
//! arguments without an array.
//!
//! # `#[rubymodule]`, `#[rubyfunction]`
//!
//! `#[rubymodule(path = "Greeter")]` on an inline `mod` defines its functions
//! as the module's singleton methods (`#[ruby(module_function)]` for a
//! module function), its `#[ruby(constant)]` constants as constants, and the
//! `#[rubyclass]` structs inside it, all from the module's generated
//! `define_ruby()`. `#[rubyfunction]` on a single function keeps it, and adds
//! a module of the same name holding `ruby`, the callback for
//! `def`/`def_self`.
//!
//! # `#[rubyerror]`
//!
//! `#[rubyerror(path = "Stats::Error", superclass = "StandardError")]` on an
//! error type that implements `Display` maps it to that Ruby exception
//! class; `#[ruby(path = "Stats::OverflowError")]` on an enum variant gives
//! the variant a subclass. Return it in `Err` from any method.
//!
//! # RBS
//!
//! `T::rbs()` on a `#[rubyclass]`, `#[rubymodule]` or `#[rubyerror]` type is
//! its RBS declaration, for writing `sig/*.rbs` files.

mod call;
mod cell;
mod class;
mod convert;
mod mark;
mod rbs;

pub use self::{
    call::RubyMethods,
    cell::{BorrowError, Borrowed, BorrowedMut},
    class::{ParentLink, RubyClass, RubyClassDataType},
    convert::{
        Bytes, ConstructorOutput, FromArgument, FromArgumentMut, FromSplat, IntoException,
        IntoRubyObject, MethodOutput, WithoutGvl,
    },
    mark::{Mark, Marker},
    rbs::RbsType,
};

/// What generated code uses; not a stable API.
#[doc(hidden)]
pub mod __private {
    pub use super::{
        call::{
            arguments, block, call, call_without_gvl, check_arity, constant, define,
            initialize_copy, invoke, require_block, wrap, CloneProbe, DefinePart,
            DefinePartFallback, Definition, Failure, Keywords, Part, ProbeClone,
            ProbeCloneFallback,
        },
        class::{define_class_at, define_module, exception_class, is_borrowed},
        convert::{assert_sync, assert_without_gvl},
        mark::{CompactField, CompactFieldFallback, MarkField, MarkFieldFallback},
        rbs::{declaration, method, ProbeRbs, ProbeRbsFallback, RbsProbe},
    };
    pub use std::{option::Option, result::Result, string::String, sync::OnceLock, vec::Vec};
}
