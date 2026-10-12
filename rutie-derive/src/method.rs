//! The model of one Ruby method and the code that implements it: a
//! callback converting Ruby's arguments, the call into the user's Rust, the
//! definition on a class or module, and its RBS signature.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    spanned::Spanned, visit_mut::VisitMut, Error, FnArg, GenericArgument, Ident, Pat,
    PathArguments, Result, ReturnType, Signature, Type, TypePath,
};

use crate::attrs::{Errors, RubyOptions};

/// The most arguments Ruby passes to a C method as separate parameters.
const MAX_FIXED_ARITY: usize = 15;

#[derive(Clone, Copy, PartialEq)]
pub enum Receiver {
    None,
    Shared,
    Mutable,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Access {
    // `&T`
    Ref,
    // `&mut T`
    Mut,
    // `T`
    Owned,
}

pub enum ParamKind {
    Positional { optional: bool },
    Splat,
    Keyword { optional: bool },
    Block { optional: bool },
}

pub struct Param {
    pub name: String,
    // The type converted from Ruby: `T` for `&T`, `Option<&T>` and `Option<T>`.
    pub ty: Type,
    pub access: Access,
    pub kind: ParamKind,
    pub span: Span,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Definition {
    Public,
    Private,
    Protected,
    Singleton,
    ModuleFunction,
    // `initialize`, from a constructor.
    Initialize,
}

pub struct Method {
    pub ruby_name: String,
    pub wrapper: Ident,
    pub receiver: Receiver,
    pub params: Vec<Param>,
    // What to call: a path, or a closure taking the receiver first.
    pub call: TokenStream,
    pub definition: Definition,
    pub without_gvl: bool,
    // The `#[rubyclass]` struct, for receivers and constructors.
    pub self_ty: Option<Type>,
    pub output: Option<Type>,
    pub span: Span,
}

/// Replaces `Self` with the struct's type, since generated callbacks are not
/// inside the `impl` block.
pub struct ReplaceSelf<'a>(pub &'a Type);

impl VisitMut for ReplaceSelf<'_> {
    fn visit_type_mut(&mut self, ty: &mut Type) {
        if let Type::Path(TypePath { qself: None, path }) = ty {
            if path.is_ident("Self") {
                *ty = self.0.clone();
                return;
            }
        }

        syn::visit_mut::visit_type_mut(self, ty);
    }
}

pub fn check_signature(signature: &Signature) -> Result<()> {
    let problem = if signature.constness.is_some() {
        Some("cannot be `const`")
    } else if signature.asyncness.is_some() {
        Some("cannot be `async`")
    } else if signature.unsafety.is_some() {
        Some("cannot be `unsafe`")
    } else if signature.abi.is_some() {
        Some("cannot have an ABI")
    } else if !signature.generics.params.is_empty() {
        Some("cannot be generic")
    } else if signature.variadic.is_some() {
        Some("cannot be variadic")
    } else {
        None
    };

    match problem {
        Some(problem) => Err(Error::new(
            signature.span(),
            format!(
                "a method for Ruby {} (use `#[ruby(skip)]` to leave it out)",
                problem
            ),
        )),
        None => Ok(()),
    }
}

/// Reads a method's receiver, parameters (taking their `#[ruby(...)]`
/// attributes off) and output type.
pub fn read_signature(
    signature: &mut Signature,
    self_ty: Option<&Type>,
) -> Result<(Receiver, Vec<Param>, Option<Type>)> {
    check_signature(signature)?;

    let mut receiver = Receiver::None;
    let mut params = Vec::new();
    let mut errors = Errors::default();

    for (index, input) in signature.inputs.iter_mut().enumerate() {
        match input {
            FnArg::Receiver(self_receiver) => {
                receiver = match (&self_receiver.reference, &self_receiver.mutability) {
                    (Some(_), None) => Receiver::Shared,
                    (Some(_), Some(_)) => Receiver::Mutable,
                    (None, _) => {
                        errors.push(Error::new(
                            self_receiver.span(),
                            "Ruby methods take `&self` or `&mut self`: the struct stays in the Ruby object",
                        ));
                        continue;
                    }
                };
            }
            FnArg::Typed(typed) => {
                let options = match RubyOptions::take(&mut typed.attrs) {
                    Ok(options) => options,
                    Err(error) => {
                        errors.push(error);
                        continue;
                    }
                };

                if let Err(error) =
                    options.only(&["keyword", "splat", "block", "name"], "a parameter")
                {
                    errors.push(error);
                    continue;
                }

                let name = match (&options.name, &*typed.pat) {
                    (Some(name), _) => name.value(),
                    (None, Pat::Ident(ident)) => {
                        ident.ident.to_string().trim_start_matches("r#").to_string()
                    }
                    (None, _) => format!("arg{}", index),
                };
                let mut ty = (*typed.ty).clone();

                if let Some(self_ty) = self_ty {
                    ReplaceSelf(self_ty).visit_type_mut(&mut ty);
                }

                let span = typed.span();
                let param = if options.splat {
                    Param {
                        name,
                        ty: owned(ty, "a `#[ruby(splat)]` parameter is `Vec<T>` or `Array`")?,
                        access: Access::Owned,
                        kind: ParamKind::Splat,
                        span,
                    }
                } else if options.block {
                    let (ty, optional) = match option_inner(&ty) {
                        Some(inner) => (inner.clone(), true),
                        None => (ty, false),
                    };

                    Param {
                        name,
                        ty: owned(
                            ty,
                            "a `#[ruby(block)]` parameter is `Proc` or `Option<Proc>`",
                        )?,
                        access: Access::Owned,
                        kind: ParamKind::Block { optional },
                        span,
                    }
                } else {
                    let (ty, optional) = match option_inner(&ty) {
                        Some(inner) => (inner.clone(), true),
                        None => (ty, false),
                    };
                    let (ty, access) = strip_reference(ty)?;

                    if optional && access == Access::Mut {
                        errors.push(Error::new(
                            span,
                            "`Option<&mut T>` parameters are not supported",
                        ));
                        continue;
                    }

                    let kind = if options.keyword {
                        ParamKind::Keyword { optional }
                    } else {
                        ParamKind::Positional { optional }
                    };

                    Param {
                        name,
                        ty,
                        access,
                        kind,
                        span,
                    }
                };

                params.push(param);
            }
        }
    }

    errors.finish()?;
    check_order(&params)?;

    let output = match &signature.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => {
            if let Type::ImplTrait(_) = **ty {
                return Err(Error::new(
                    ty.span(),
                    "`impl Trait` can't be returned to Ruby",
                ));
            }

            let mut ty = (**ty).clone();

            if let Some(self_ty) = self_ty {
                ReplaceSelf(self_ty).visit_type_mut(&mut ty);
            }

            Some(ty)
        }
    };

    Ok((receiver, params, output))
}

// Ruby's order: required, optional, splat; keywords and the block anywhere.
fn check_order(params: &[Param]) -> Result<()> {
    let mut seen_optional = false;
    let mut seen_splat = false;
    let mut seen_block = false;

    for param in params {
        match param.kind {
            ParamKind::Positional { optional: false } if seen_optional || seen_splat => {
                return Err(Error::new(
                    param.span,
                    "required parameters come before `Option` and `#[ruby(splat)]` ones",
                ));
            }
            ParamKind::Positional { optional: true } if seen_splat => {
                return Err(Error::new(
                    param.span,
                    "`Option` parameters come before the `#[ruby(splat)]` one",
                ));
            }
            ParamKind::Positional { optional: true } => seen_optional = true,
            ParamKind::Splat if seen_splat => {
                return Err(Error::new(
                    param.span,
                    "only one `#[ruby(splat)]` parameter",
                ));
            }
            ParamKind::Splat => seen_splat = true,
            ParamKind::Block { .. } if seen_block => {
                return Err(Error::new(
                    param.span,
                    "only one `#[ruby(block)]` parameter",
                ));
            }
            ParamKind::Block { .. } => seen_block = true,
            _ => {}
        }
    }

    Ok(())
}

fn owned(ty: Type, message: &str) -> Result<Type> {
    match ty {
        Type::Reference(_) => Err(Error::new(ty.span(), message)),
        ty => Ok(ty),
    }
}

fn strip_reference(ty: Type) -> Result<(Type, Access)> {
    match ty {
        Type::Reference(reference) => {
            let access = if reference.mutability.is_some() {
                Access::Mut
            } else {
                Access::Ref
            };

            Ok((*reference.elem, access))
        }
        Type::ImplTrait(_) => Err(Error::new(ty.span(), "parameters can't be `impl Trait`")),
        ty => Ok((ty, Access::Owned)),
    }
}

pub fn option_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(TypePath { qself: None, path }) = ty else {
        return None;
    };
    let segment = path.segments.last()?;

    if segment.ident != "Option" {
        return None;
    }

    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };

    match arguments.args.first() {
        Some(GenericArgument::Type(inner)) if arguments.args.len() == 1 => Some(inner),
        _ => None,
    }
}

fn vec_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(TypePath { qself: None, path }) = ty else {
        return None;
    };
    let segment = path.segments.last()?;
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };

    match arguments.args.first() {
        Some(GenericArgument::Type(inner)) if segment.ident == "Vec" => Some(inner),
        _ => None,
    }
}

pub fn private() -> TokenStream {
    quote!(::rutie::derive::__private)
}

impl Method {
    fn fixed_arity(&self) -> Option<usize> {
        let mut count = 0;

        for param in &self.params {
            match param.kind {
                ParamKind::Positional { optional: false } => count += 1,
                ParamKind::Block { .. } => {}
                _ => return None,
            }
        }

        (count <= MAX_FIXED_ARITY).then_some(count)
    }

    fn has_keywords(&self) -> bool {
        self.params
            .iter()
            .any(|param| matches!(param.kind, ParamKind::Keyword { .. }))
    }

    /// The callback function item, named `self.wrapper`. `variadic` forces
    /// the `(argc, argv, self)` form (for `#[rubyfunction]`'s `def` callback).
    pub fn callback(&self, variadic: bool, public: bool) -> TokenStream {
        let wrapper = &self.wrapper;
        let fixed = if variadic { None } else { self.fixed_arity() };
        let visibility = if public { quote!(pub) } else { quote!() };
        let body = self.body(fixed);

        let signature = match fixed {
            Some(count) => {
                let names: Vec<Ident> = (0..count)
                    .map(|index| format_ident!("ruby_argument_{}", index))
                    .collect();

                quote! {
                    #visibility fn #wrapper(receiver: ::rutie::AnyObject #(, #names: ::rutie::AnyObject)*) -> ::rutie::AnyObject
                }
            }
            None => quote! {
                #visibility fn #wrapper(
                    argc: ::rutie::types::Argc,
                    argv: *const ::rutie::AnyObject,
                    receiver: ::rutie::AnyObject,
                ) -> ::rutie::AnyObject
            },
        };

        let span = self.span;

        quote_spanned! {span=>
            ::rutie::rutie_callback! {
                #[allow(unused_variables, unused_mut, clippy::all)]
                #signature {
                    #body
                }
            }
        }
    }

    /// The statement defining the method on `target`.
    pub fn define(&self, target: &TokenStream) -> TokenStream {
        let private = private();
        let wrapper = &self.wrapper;
        let ruby_name = &self.ruby_name;
        let arity = match self.fixed_arity() {
            Some(count) => count as i32,
            None => -1,
        };
        let how = match self.definition {
            Definition::Public => quote!(Public),
            Definition::Private | Definition::Initialize => quote!(Private),
            Definition::Protected => quote!(Protected),
            Definition::Singleton => quote!(Singleton),
            Definition::ModuleFunction => quote!(ModuleFunction),
        };

        quote! {
            #private::define(
                &#target,
                #ruby_name,
                #wrapper as *const ::rutie::types::c_void,
                #arity,
                #private::Definition::#how,
            );
        }
    }

    // The callback's body: everything runs inside `invoke`, so it is dropped
    // before an exception is raised.
    fn body(&self, fixed: Option<usize>) -> TokenStream {
        let private = private();
        let mut prelude = Vec::new();
        let mut holds = Vec::new();
        let mut arguments = Vec::new();
        let mut asserts = Vec::new();

        match fixed {
            Some(count) => {
                let names = (0..count).map(|index| format_ident!("ruby_argument_{}", index));

                prelude.push(quote! {
                    let arguments: [::rutie::AnyObject; #count] = [#(#names),*];
                });
            }
            None => {
                prelude.push(quote! {
                    let arguments = #private::arguments(argc, argv);
                });

                if self.has_keywords() {
                    prelude.push(quote! {
                        let (arguments, mut keywords) = #private::Keywords::split(arguments);
                        let mut missing_keywords = #private::Vec::new();
                    });
                }

                let required = self
                    .params
                    .iter()
                    .filter(|param| matches!(param.kind, ParamKind::Positional { optional: false }))
                    .count();
                let optional = self
                    .params
                    .iter()
                    .filter(|param| matches!(param.kind, ParamKind::Positional { optional: true }))
                    .count();
                let maximum = if self
                    .params
                    .iter()
                    .any(|param| matches!(param.kind, ParamKind::Splat))
                {
                    quote!(#private::Option::None)
                } else {
                    quote!(#private::Option::Some(#optional))
                };

                prelude.push(quote! {
                    #private::check_arity(arguments.len(), #required, #maximum)?;
                });
            }
        }

        // The receiver.
        let self_ty = self.self_ty.as_ref();
        let receiver = match (self.receiver, self_ty) {
            (Receiver::Shared, Some(self_ty)) => {
                holds.push(quote! {
                    let this = <#self_ty as ::rutie::derive::RubyClass>::borrow_from(&receiver)?;
                });
                Some(quote!(&*this))
            }
            (Receiver::Mutable, Some(self_ty)) => {
                holds.push(quote! {
                    let mut this = <#self_ty as ::rutie::derive::RubyClass>::borrow_mut_from(&receiver)?;
                });
                Some(quote!(&mut *this))
            }
            _ => None,
        };

        if self.without_gvl {
            if let (Receiver::Shared, Some(self_ty)) = (self.receiver, self_ty) {
                asserts.push(quote!(#private::assert_sync::<#self_ty>();));
            }

            if let Some(output) = &self.output {
                asserts.push(quote!(#private::assert_without_gvl::<#output>();));
            }
        }

        let mut position = 0usize;

        for (index, param) in self.params.iter().enumerate() {
            let holder = format_ident!("argument_{}", index);
            let ty = &param.ty;
            let span = param.span;

            if self.without_gvl {
                asserts.push(quote_spanned!(span=> #private::assert_without_gvl::<#ty>();));
            }

            // Spanned at the type, so an unsupported type's error points at it.
            let ty_span = ty.span();
            let hold = |value: TokenStream| match param.access {
                Access::Mut => {
                    quote_spanned!(ty_span=> <#ty as ::rutie::derive::FromArgumentMut>::hold_mut(#value))
                }
                _ => quote_spanned!(ty_span=> <#ty as ::rutie::derive::FromArgument>::hold(#value)),
            };

            match param.kind {
                ParamKind::Positional { optional: false } => {
                    let value = hold(quote!(::std::clone::Clone::clone(&arguments[#position])));
                    holds.push(quote_spanned!(span=> let mut #holder = #value?;));
                    position += 1;
                }
                ParamKind::Positional { optional: true } => {
                    let value = hold(quote!(::std::clone::Clone::clone(argument)));
                    holds.push(quote_spanned! {span=>
                        let mut #holder = match arguments.get(#position) {
                            #private::Option::Some(argument) if !::rutie::Object::is_nil(argument) => #private::Option::Some(#value?),
                            _ => #private::Option::None,
                        };
                    });
                    position += 1;
                }
                ParamKind::Splat => {
                    holds.push(quote_spanned! {span=>
                        let #holder = <#ty as ::rutie::derive::FromSplat>::from_splat(arguments.get(#position..).unwrap_or(&[]))?;
                    });
                }
                ParamKind::Keyword { optional } => {
                    let name = &param.name;
                    let take = if optional {
                        quote!(keywords.optional::<#ty>(#name)?)
                    } else {
                        quote!(keywords.required::<#ty>(#name, &mut missing_keywords)?)
                    };

                    holds.push(quote_spanned!(span=> let mut #holder = #take;));
                }
                ParamKind::Block { optional } => {
                    let take = if optional {
                        quote!(#private::block())
                    } else {
                        quote!(#private::require_block(#private::block())?)
                    };

                    holds.push(quote_spanned!(span=> let #holder = #take;));
                }
            }

            arguments.push(match (&param.kind, param.access) {
                (ParamKind::Positional { optional: true }, Access::Ref)
                | (ParamKind::Keyword { optional: true }, Access::Ref) => {
                    quote_spanned!(ty_span=> #holder.as_ref().map(<#ty as ::rutie::derive::FromArgument>::borrow))
                }
                (ParamKind::Positional { optional: true }, _) | (ParamKind::Keyword { optional: true }, _) => quote!(#holder),
                (_, Access::Ref) => quote_spanned!(ty_span=> <#ty as ::rutie::derive::FromArgument>::borrow(&#holder)),
                (_, Access::Mut) => quote_spanned!(ty_span=> <#ty as ::rutie::derive::FromArgumentMut>::borrow_mut(&mut #holder)),
                (_, Access::Owned) => quote!(#holder),
            });
        }

        if self.has_keywords() {
            holds.push(quote!(keywords.finish(missing_keywords)?;));

            // Required keywords are all there once `finish` succeeds.
            for (index, param) in self.params.iter().enumerate() {
                if let ParamKind::Keyword { optional: false } = param.kind {
                    let holder = format_ident!("argument_{}", index);

                    holds.push(quote! {
                        let mut #holder = match #holder {
                            #private::Option::Some(value) => value,
                            #private::Option::None => unreachable!("checked by Keywords::finish"),
                        };
                    });
                }
            }
        }

        let call = &self.call;
        let call_arguments = receiver.into_iter().chain(arguments);
        let invoke = if self.without_gvl {
            quote!(#private::call_without_gvl(|| (#call)(#(#call_arguments),*))?)
        } else {
            quote!(#private::call(|| (#call)(#(#call_arguments),*))?)
        };

        let finish = match (self.definition, self_ty) {
            (Definition::Initialize, Some(self_ty)) => quote! {
                let value = <_ as ::rutie::derive::ConstructorOutput<#self_ty>>::into_value(output)?;
                <#self_ty as ::rutie::derive::RubyClass>::initialize_object(&receiver, value)?;

                #private::Result::Ok(::rutie::Object::to_any_object(&::rutie::NilClass::new()))
            },
            _ => {
                let span = self
                    .output
                    .as_ref()
                    .map_or(self.span, |output| output.span());
                let convert = quote_spanned!(span=> ::rutie::derive::MethodOutput::into_output(output, &output_receiver));

                quote! {
                    #private::Result::Ok(#convert?)
                }
            }
        };

        quote! {
            let output_receiver = ::std::clone::Clone::clone(&receiver);

            #private::invoke(move || {
                #(#asserts)*
                #(#prelude)*
                #(#holds)*

                let output = #invoke;

                #finish
            })
        }
    }

    /// An expression pushing the method's RBS declaration onto `out`.
    pub fn rbs(&self) -> TokenStream {
        let private = private();
        let mut parameters = Vec::new();
        let mut block = quote!(#private::Option::None);

        for param in &self.params {
            let name = &param.name;
            let ty = &param.ty;
            let probe = quote!((&#private::RbsProbe::<#ty>::new()).rbs());

            match param.kind {
                ParamKind::Positional { optional } => {
                    let prefix = if optional { "?" } else { "" };
                    parameters.push(quote!(format!("{}{} {}", #prefix, #probe, #name)));
                }
                ParamKind::Splat => {
                    let element = match vec_inner(ty) {
                        Some(element) => quote!((&#private::RbsProbe::<#element>::new()).rbs()),
                        None => quote!(#private::String::from("untyped")),
                    };

                    parameters.push(quote!(format!("*{} {}", #element, #name)));
                }
                ParamKind::Keyword { optional } => {
                    let prefix = if optional { "?" } else { "" };
                    parameters.push(quote!(format!("{}{}: {}", #prefix, #name, #probe)));
                }
                ParamKind::Block { optional } => {
                    let required = !optional;
                    block = quote!(#private::Option::Some(#required));
                }
            }
        }

        let output = match (&self.output, self.definition) {
            (_, Definition::Initialize) => quote!(#private::String::from("void")),
            (None, _) => quote!(#private::String::from("nil")),
            (Some(output), _) => quote!((&#private::RbsProbe::<#output>::new()).rbs()),
        };
        let name = &self.ruby_name;
        let singleton = matches!(
            self.definition,
            Definition::Singleton | Definition::ModuleFunction
        );

        quote! {
            {
                #[allow(unused_imports)]
                use #private::{ProbeRbs as _, ProbeRbsFallback as _};

                out.push(#private::method(#name, #singleton, &[#(#parameters),*], #block, &#output));
            }
        }
    }
}

/// The default Ruby name of a Rust method or function.
pub fn ruby_name(ident: &Ident, options: &RubyOptions) -> String {
    options
        .name
        .as_ref()
        .map(|name| name.value())
        .unwrap_or_else(|| ident.to_string().trim_start_matches("r#").to_string())
}

pub fn wrapper_name(ident: &Ident) -> Ident {
    format_ident!("__rutie_{}", ident.to_string().trim_start_matches("r#"))
}
