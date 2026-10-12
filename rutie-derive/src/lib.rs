//! The `#[rubyclass]`, `#[rubymethods]`, `#[rubymodule]`, `#[rubyfunction]`
//! and `#[rubyerror]` attributes and `#[derive(Mark)]`.
//!
//! Use them through Rutie's `derive` feature (`rutie::rubyclass`, …); the
//! `rutie::derive` module documents them. The generated code only uses
//! `::rutie` paths, and this crate doesn't depend on Ruby.

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, ItemFn, ItemImpl, ItemMod, ItemStruct};

mod attrs;
mod class;
mod items;
mod mark;
mod method;

fn finish(result: syn::Result<proc_macro2::TokenStream>) -> TokenStream {
    match result {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// Stores a struct in the Ruby objects of a Ruby class:
/// `#[rubyclass(path = "A::B", extends = Parent, mark = …, size = …,
/// allocator = false)]`. See `rutie::derive`.
#[proc_macro_attribute]
pub fn rubyclass(attribute: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemStruct);

    finish(class::expand(attribute.into(), item))
}

/// Defines the methods of an `impl` block of a `#[rubyclass]` struct as Ruby
/// methods: `#[rubymethods]`, or `#[rubymethods(part = N)]` for a struct's
/// further blocks. See `rutie::derive`.
#[proc_macro_attribute]
pub fn rubymethods(attribute: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemImpl);

    finish(items::rubymethods(attribute.into(), item))
}

/// Turns an inline `mod` into a Ruby module: `#[rubymodule(path = "A::B")]`.
/// See `rutie::derive`.
#[proc_macro_attribute]
pub fn rubymodule(attribute: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemMod);

    finish(items::rubymodule(attribute.into(), item))
}

/// Adds `name::ruby`, a Ruby callback for the function, for
/// `def`/`def_self`. See `rutie::derive`.
#[proc_macro_attribute]
pub fn rubyfunction(attribute: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);

    finish(items::rubyfunction(attribute.into(), item))
}

/// Maps an error type to a Ruby exception class:
/// `#[rubyerror(path = "A::Error", superclass = "StandardError")]`, with
/// `#[ruby(path = "A::OtherError")]` on enum variants. See `rutie::derive`.
#[proc_macro_attribute]
pub fn rubyerror(attribute: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);

    finish(items::rubyerror(attribute.into(), item))
}

/// Marks the Ruby objects in a type's fields for the GC. See
/// `rutie::derive::Mark`.
#[proc_macro_derive(Mark, attributes(ruby))]
pub fn derive_mark(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);

    finish(mark::derive(item))
}
