//! `#[rubymethods]`, `#[rubymodule]`, `#[rubyfunction]` and `#[rubyerror]`.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::Parser, spanned::Spanned, Data, DeriveInput, Error, Fields, ImplItem, Item, ItemFn,
    ItemImpl, ItemMod, LitInt, LitStr, Result,
};

use crate::{
    attrs::{Errors, RubyOptions},
    method::{private, read_signature, ruby_name, wrapper_name, Definition, Method, Receiver},
};

const METHOD_OPTIONS: &[&str] = &[
    "name",
    "skip",
    "constructor",
    "private",
    "protected",
    "without_gvl",
];

pub fn rubymethods(attribute: TokenStream, mut item: ItemImpl) -> Result<TokenStream> {
    let mut part: usize = 0;

    syn::meta::parser(|meta| {
        if meta.path.is_ident("part") {
            let value: LitInt = meta.value()?.parse()?;
            part = value.base10_parse()?;

            if part > 7 {
                return Err(Error::new(value.span(), "`part` is 0 to 7"));
            }

            Ok(())
        } else {
            Err(meta.error("expected `part = N` (1 to 7, for a struct's second to eighth block)"))
        }
    })
    .parse2(attribute)?;

    if let Some((_, trait_path, _)) = &item.trait_ {
        return Err(Error::new(
            trait_path.span(),
            "`#[rubymethods]` goes on an inherent `impl` block, not a trait implementation",
        ));
    }

    if !item.generics.params.is_empty() {
        return Err(Error::new(
            item.generics.span(),
            "`#[rubymethods]` blocks can't be generic",
        ));
    }

    let self_ty = (*item.self_ty).clone();
    let mut methods = Vec::new();
    let mut errors = Errors::default();

    for impl_item in item.items.iter_mut() {
        let ImplItem::Fn(function) = impl_item else {
            continue;
        };

        let Some(options) = errors.check(RubyOptions::take(&mut function.attrs)) else {
            continue;
        };

        if errors
            .check(options.only(METHOD_OPTIONS, "a method"))
            .is_none()
            || options.skip
        {
            continue;
        }

        let Some((receiver, params, output)) =
            errors.check(read_signature(&mut function.sig, Some(&self_ty)))
        else {
            continue;
        };

        let ident = &function.sig.ident;
        let constructor = options.constructor || (ident == "new" && receiver == Receiver::None);

        if constructor && receiver != Receiver::None {
            errors.push(Error::new(
                function.sig.span(),
                "a constructor has no `self`",
            ));
            continue;
        }

        let definition = if constructor {
            Definition::Initialize
        } else if receiver == Receiver::None {
            Definition::Singleton
        } else if options.private {
            Definition::Private
        } else if options.protected {
            Definition::Protected
        } else {
            Definition::Public
        };

        methods.push(Method {
            ruby_name: if constructor {
                "initialize".to_string()
            } else {
                ruby_name(ident, &options)
            },
            wrapper: wrapper_name(ident),
            receiver,
            params,
            call: quote!(<#self_ty>::#ident),
            definition,
            without_gvl: options.without_gvl,
            self_ty: Some(self_ty.clone()),
            output,
            span: ident.span(),
        });
    }

    errors.finish()?;

    let private = private();
    let target = quote!(*class);
    let callbacks: Vec<TokenStream> = methods
        .iter()
        .map(|method| method.callback(false, false))
        .collect();
    let defines: Vec<TokenStream> = methods
        .iter()
        .map(|method| method.define(&target))
        .collect();
    let rbs: Vec<TokenStream> = methods.iter().map(Method::rbs).collect();

    Ok(quote! {
        #item

        impl ::rutie::derive::RubyMethods<#part> for #self_ty {
            fn define_methods(class: &mut ::rutie::Class) {
                #(#callbacks)*
                #(#defines)*
            }

            fn rbs_methods(out: &mut #private::Vec<#private::String>) {
                #(#rbs)*
            }
        }
    })
}

fn camel_case(snake: &str) -> String {
    snake
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();

            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect()
}

fn has_attribute(attributes: &[syn::Attribute], name: &str) -> bool {
    attributes.iter().any(|attribute| {
        attribute
            .path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == name)
    })
}

pub fn rubymodule(attribute: TokenStream, mut item: ItemMod) -> Result<TokenStream> {
    let mut path: Option<LitStr> = None;

    syn::meta::parser(|meta| {
        if meta.path.is_ident("path") {
            path = Some(meta.value()?.parse()?);
            Ok(())
        } else {
            Err(meta.error("expected `path = \"…\"`"))
        }
    })
    .parse2(attribute)?;

    let path = path
        .unwrap_or_else(|| LitStr::new(&camel_case(&item.ident.to_string()), item.ident.span()));
    let Some((_, content)) = &mut item.content else {
        return Err(Error::new(
            item.span(),
            "`#[rubymodule]` needs an inline module: `mod name { … }`",
        ));
    };

    let mut methods = Vec::new();
    let mut constants = Vec::new();
    let mut nested = Vec::new();
    let mut errors = Errors::default();

    for content_item in content.iter_mut() {
        match content_item {
            Item::Fn(function) => {
                let Some(options) = errors.check(RubyOptions::take(&mut function.attrs)) else {
                    continue;
                };

                if errors
                    .check(options.only(
                        &["name", "skip", "module_function", "without_gvl"],
                        "a module function",
                    ))
                    .is_none()
                    || options.skip
                {
                    continue;
                }

                let Some((_, params, output)) =
                    errors.check(read_signature(&mut function.sig, None))
                else {
                    continue;
                };
                let ident = &function.sig.ident;

                methods.push(Method {
                    ruby_name: ruby_name(ident, &options),
                    wrapper: wrapper_name(ident),
                    receiver: Receiver::None,
                    params,
                    call: quote!(#ident),
                    definition: if options.module_function {
                        Definition::ModuleFunction
                    } else {
                        Definition::Singleton
                    },
                    without_gvl: options.without_gvl,
                    self_ty: None,
                    output,
                    span: ident.span(),
                });
            }
            Item::Const(constant) => {
                let Some(options) = errors.check(RubyOptions::take(&mut constant.attrs)) else {
                    continue;
                };

                if errors
                    .check(options.only(&["constant", "name"], "a constant"))
                    .is_none()
                    || !options.constant
                {
                    continue;
                }

                let ident = &constant.ident;
                let name = ruby_name(ident, &options);
                let ty = &constant.ty;

                constants.push((name, ident.clone(), (**ty).clone()));
            }
            Item::Struct(structure) if has_attribute(&structure.attrs, "rubyclass") => {
                nested.push(structure.ident.clone());
            }
            Item::Mod(module) if has_attribute(&module.attrs, "rubymodule") => {
                nested.push(module.ident.clone());
            }
            Item::Enum(enumeration) if has_attribute(&enumeration.attrs, "rubyerror") => {
                nested.push(enumeration.ident.clone());
            }
            Item::Struct(structure) if has_attribute(&structure.attrs, "rubyerror") => {
                nested.push(structure.ident.clone());
            }
            _ => {}
        }
    }

    errors.finish()?;

    let private = private();
    let target = quote!(module);
    let callbacks: Vec<TokenStream> = methods
        .iter()
        .map(|method| method.callback(false, false))
        .collect();
    let defines: Vec<TokenStream> = methods
        .iter()
        .map(|method| method.define(&target))
        .collect();
    let rbs: Vec<TokenStream> = methods.iter().map(Method::rbs).collect();
    let constant_sets = constants
        .iter()
        .map(|(name, ident, _)| quote!(#private::constant(&mut module, #name, #ident);));
    let constant_rbs = constants.iter().map(|(name, _, ty)| {
        quote! {
            {
                #[allow(unused_imports)]
                use #private::{ProbeRbs as _, ProbeRbsFallback as _};

                out.push(format!("{}: {}", #name, (&#private::RbsProbe::<#ty>::new()).rbs()));
            }
        }
    });

    let generated: Vec<Item> = vec![
        syn::parse_quote! {
            /// Defines the Ruby module: its functions, constants, and the
            /// `#[rubyclass]`, `#[rubymodule]` and `#[rubyerror]` items in it.
            pub fn define_ruby() -> ::rutie::Module {
                let mut module = #private::define_module(#path);

                #(#callbacks)*
                #(#defines)*
                #(#constant_sets)*
                #(#nested::define_ruby();)*

                module
            }
        },
        syn::parse_quote! {
            /// The module's RBS declaration, followed by those of the items
            /// in it.
            pub fn rbs() -> #private::String {
                let mut out: #private::Vec<#private::String> = #private::Vec::new();

                #(#constant_rbs)*
                #(#rbs)*

                let mut declaration = #private::declaration(#path, false, #private::Option::None, &out);
                #(declaration.push_str(&#nested::rbs());)*

                declaration
            }
        },
    ];

    content.extend(generated);

    Ok(quote!(#item))
}

pub fn rubyfunction(attribute: TokenStream, mut item: ItemFn) -> Result<TokenStream> {
    if !attribute.is_empty() {
        return Err(Error::new(
            Span::call_site(),
            "`#[rubyfunction]` takes no arguments",
        ));
    }

    let options = RubyOptions::take(&mut item.attrs)?;
    options.only(&["without_gvl"], "a `#[rubyfunction]`")?;

    let (_, params, output) = read_signature(&mut item.sig, None)?;
    let ident = item.sig.ident.clone();
    let visibility = &item.vis;
    let method = Method {
        ruby_name: ident.to_string(),
        wrapper: syn::Ident::new("ruby", ident.span()),
        receiver: Receiver::None,
        params,
        call: quote!(super::#ident),
        definition: Definition::Singleton,
        without_gvl: options.without_gvl,
        self_ty: None,
        output,
        span: ident.span(),
    };
    let callback = method.callback(true, true);

    Ok(quote! {
        #item

        /// The Ruby callback for the function, `ruby`, for `def`/`def_self`.
        #[allow(non_snake_case)]
        #visibility mod #ident {
            #[allow(unused_imports)]
            use super::*;

            #callback
        }
    })
}

pub fn rubyerror(attribute: TokenStream, mut input: DeriveInput) -> Result<TokenStream> {
    let mut path: Option<LitStr> = None;
    let mut superclass: Option<LitStr> = None;

    syn::meta::parser(|meta| {
        if meta.path.is_ident("path") {
            path = Some(meta.value()?.parse()?);
        } else if meta.path.is_ident("superclass") {
            superclass = Some(meta.value()?.parse()?);
        } else {
            return Err(meta.error("expected `path = \"…\"` or `superclass = \"…\"`"));
        }

        Ok(())
    })
    .parse2(attribute)?;

    let name = input.ident.clone();
    let path = path
        .unwrap_or_else(|| LitStr::new(&name.to_string(), name.span()))
        .value();
    let superclass = superclass
        .map(|superclass| superclass.value())
        .unwrap_or_else(|| "StandardError".to_string());
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let private = private();

    let mut arms = Vec::new();
    let mut defines = Vec::new();
    let mut rbs = Vec::new();

    if let Data::Enum(data) = &mut input.data {
        for variant in data.variants.iter_mut() {
            let options = RubyOptions::take(&mut variant.attrs)?;
            options.only(&["path"], "a `#[rubyerror]` variant")?;

            let Some(variant_path) = options.path else {
                continue;
            };
            let variant_path = variant_path.value();
            let variant_name = &variant.ident;
            let pattern = match variant.fields {
                Fields::Named(_) => quote!(Self::#variant_name { .. }),
                Fields::Unnamed(_) => quote!(Self::#variant_name(..)),
                Fields::Unit => quote!(Self::#variant_name),
            };

            arms.push(quote! {
                #pattern => {
                    #private::exception_class(#path, #superclass);
                    #private::exception_class(#variant_path, #path)
                }
            });
            defines.push(quote!(#private::exception_class(#variant_path, #path);));
            rbs.push(quote!(out.push_str(&#private::declaration(#variant_path, true, #private::Option::Some(#path), &[]));));
        }
    }

    let item_tokens = match &input.data {
        Data::Struct(_) | Data::Enum(_) => quote!(#input),
        Data::Union(union) => {
            return Err(Error::new(
                union.union_token.span,
                "`#[rubyerror]` doesn't support unions",
            ))
        }
    };

    let class = if arms.is_empty() {
        quote!(#private::exception_class(#path, #superclass))
    } else {
        quote! {
            match self {
                #(#arms)*
                _ => #private::exception_class(#path, #superclass),
            }
        }
    };

    Ok(quote! {
        #item_tokens

        impl #impl_generics ::rutie::derive::IntoException for #name #type_generics #where_clause {
            #[allow(unreachable_patterns)]
            fn into_exception(self) -> ::rutie::AnyException {
                let class = #class;

                ::rutie::AnyException::from_class(&class, &::std::string::ToString::to_string(&self))
            }
        }

        impl #impl_generics #name #type_generics #where_clause {
            /// Defines the Ruby exception classes.
            pub fn define_ruby() -> ::rutie::Class {
                let class = #private::exception_class(#path, #superclass);
                #(#defines)*

                class
            }

            /// The exception classes' RBS declarations.
            pub fn rbs() -> #private::String {
                let mut out = #private::declaration(#path, true, #private::Option::Some(#superclass), &[]);
                #(#rbs)*

                out
            }
        }
    })
}
