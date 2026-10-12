//! `#[rubyclass]`.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::Parser, spanned::Spanned, Error, Fields, Index, ItemStruct, LitBool, LitStr, Member,
    Path, Result, Type,
};

use crate::{
    attrs::{Errors, RubyOptions},
    mark::field_marking,
    method::{private, Access, Definition, Method, Param, ParamKind, Receiver},
};

pub fn expand(attribute: TokenStream, mut item: ItemStruct) -> Result<TokenStream> {
    let mut path: Option<LitStr> = None;
    let mut mark: Option<Path> = None;
    let mut size: Option<Path> = None;
    let mut extends: Option<Type> = None;
    let mut allocator: Option<LitBool> = None;

    syn::meta::parser(|meta| {
        if meta.path.is_ident("path") {
            path = Some(meta.value()?.parse()?);
        } else if meta.path.is_ident("mark") {
            mark = Some(meta.value()?.parse()?);
        } else if meta.path.is_ident("size") {
            size = Some(meta.value()?.parse()?);
        } else if meta.path.is_ident("extends") {
            extends = Some(meta.value()?.parse()?);
        } else if meta.path.is_ident("allocator") {
            allocator = Some(meta.value()?.parse()?);
        } else {
            return Err(meta.error(
                "expected `path = \"…\"`, `extends = Parent`, `mark = …`, `size = …` or `allocator = false`",
            ));
        }

        Ok(())
    })
    .parse2(attribute)?;

    if !item.generics.params.is_empty() {
        return Err(Error::new(
            item.generics.span(),
            "`#[rubyclass]` structs can't be generic",
        ));
    }

    let name = item.ident.clone();
    let self_ty: Type = syn::parse_quote!(#name);
    let path = path.unwrap_or_else(|| LitStr::new(&name.to_string(), name.span()));

    if path.value().split("::").any(|part| part.is_empty()) {
        return Err(Error::new(
            path.span(),
            "expected a Ruby class path such as \"A::B\"",
        ));
    }

    let private = private();
    let mut errors = Errors::default();
    let mut members: Vec<(Member, Type, RubyOptions)> = Vec::new();

    match &mut item.fields {
        Fields::Named(fields) => {
            for field in fields.named.iter_mut() {
                if let Some(options) = errors.check(RubyOptions::take(&mut field.attrs)) {
                    let member = Member::Named(field.ident.clone().expect("named field"));
                    members.push((member, field.ty.clone(), options));
                }
            }
        }
        Fields::Unnamed(fields) => {
            for (index, field) in fields.unnamed.iter_mut().enumerate() {
                if let Some(options) = errors.check(RubyOptions::take(&mut field.attrs)) {
                    let member = Member::Unnamed(Index {
                        index: index as u32,
                        span: field.span(),
                    });
                    members.push((member, field.ty.clone(), options));
                }
            }
        }
        Fields::Unit => {}
    }

    for (_, _, options) in &members {
        errors.check(options.only(
            &["get", "set", "name", "no_mark", "base"],
            "a `#[rubyclass]` field",
        ));
    }

    errors.finish()?;

    // `extends = Parent` with the parent's struct in a `#[ruby(base)]` field.
    let base: Vec<&(Member, Type, RubyOptions)> = members
        .iter()
        .filter(|(_, _, options)| options.base)
        .collect();
    let parent = match (&extends, base.as_slice()) {
        (None, []) => quote!(),
        (Some(parent), [(member, ty, _)]) => quote! {
            fn parent() -> #private::Option<::rutie::derive::ParentLink> {
                // The `#[ruby(base)]` field holds the parent's struct.
                fn base(value: &#name) -> &#parent {
                    let field: &#ty = &value.#member;
                    field
                }
                let _ = base;

                #private::Option::Some(::rutie::derive::ParentLink::new::<#parent>(
                    ::core::mem::offset_of!(#name, #member),
                ))
            }
        },
        (Some(parent), []) => {
            return Err(Error::new(
                parent.span(),
                "`extends` needs a field holding the parent's struct, marked `#[ruby(base)]`",
            ))
        }
        (None, [(member, _, _), ..]) => {
            return Err(Error::new(
                member.span(),
                "`#[ruby(base)]` needs `extends = Parent` on `#[rubyclass]`",
            ))
        }
        (Some(_), [_, (member, _, _), ..]) => {
            return Err(Error::new(member.span(), "only one `#[ruby(base)]` field"))
        }
    };

    let allocator = match allocator {
        Some(allocator) if !allocator.value => quote! {
            fn allocator() -> bool {
                false
            }
        },
        _ => quote!(),
    };

    // Marking: the given function (pinning), or each field.
    let (mark_fn, compact_fn) = match &mark {
        Some(mark) => (
            quote! {
                fn mark(&self, _marker: &::rutie::derive::Marker) {
                    #mark(self)
                }
            },
            quote!(),
        ),
        None => {
            let fields: Vec<&Member> = members
                .iter()
                .filter(|(_, _, options)| !options.no_mark)
                .map(|(member, _, _)| member)
                .collect();
            let (mark_body, compact_body) = field_marking(
                &fields
                    .iter()
                    .map(|member| quote!(self.#member))
                    .collect::<Vec<_>>(),
            );

            (
                quote! {
                    fn mark(&self, marker: &::rutie::derive::Marker) {
                        #mark_body
                    }
                },
                quote! {
                    fn compact(&mut self) {
                        #compact_body
                    }
                },
            )
        }
    };

    let size_fn = size.map(|size| {
        quote! {
            fn size(&self) -> usize {
                #size(self)
            }
        }
    });

    // Accessors.
    let mut accessors = Vec::new();
    let mut accessor_rbs = Vec::new();

    for (member, ty, options) in &members {
        if !options.get && !options.set {
            continue;
        }

        let field_name = match (&options.name, member) {
            (Some(name), _) => name.value(),
            (None, Member::Named(ident)) => ident.to_string().trim_start_matches("r#").to_string(),
            (None, Member::Unnamed(index)) => {
                return Err(Error::new(
                    index.span,
                    "name a tuple field's accessor with `#[ruby(get, name = \"…\")]`",
                ))
            }
        };
        let suffix = match member {
            Member::Named(ident) => ident.to_string().trim_start_matches("r#").to_string(),
            Member::Unnamed(index) => index.index.to_string(),
        };

        if options.get {
            accessors.push(Method {
                ruby_name: field_name.clone(),
                wrapper: format_ident!("__rutie_get_{}", suffix),
                receiver: Receiver::Shared,
                params: Vec::new(),
                call: quote!(|this: &#name| -> #ty { ::std::clone::Clone::clone(&this.#member) }),
                definition: Definition::Public,
                without_gvl: false,
                self_ty: Some(self_ty.clone()),
                output: Some(ty.clone()),
                span: member.span(),
            });
        }

        if options.set {
            accessors.push(Method {
                ruby_name: format!("{}=", field_name),
                wrapper: format_ident!("__rutie_set_{}", suffix),
                receiver: Receiver::Mutable,
                params: vec![Param {
                    name: "value".to_string(),
                    ty: ty.clone(),
                    access: Access::Owned,
                    kind: ParamKind::Positional { optional: false },
                    span: member.span(),
                }],
                call: quote!(|this: &mut #name, value: #ty| { this.#member = value; }),
                definition: Definition::Public,
                without_gvl: false,
                self_ty: Some(self_ty.clone()),
                output: None,
                span: member.span(),
            });
        }

        let kind = match (options.get, options.set) {
            (true, true) => "attr_accessor",
            (true, false) => "attr_reader",
            _ => "attr_writer",
        };

        accessor_rbs.push(quote! {
            {
                #[allow(unused_imports)]
                use #private::{ProbeRbs as _, ProbeRbsFallback as _};

                out.push(format!("{} {}: {}", #kind, #field_name, (&#private::RbsProbe::<#ty>::new()).rbs()));
            }
        });
    }

    let target = quote!(class);
    let accessor_callbacks: Vec<TokenStream> = accessors
        .iter()
        .map(|method| method.callback(false, false))
        .collect();
    let accessor_defines: Vec<TokenStream> = accessors
        .iter()
        .map(|method| method.define(&target))
        .collect();
    let parts: Vec<TokenStream> = (0..8usize)
        .map(|part| quote!((&#private::Part::<#name, #part>::new()).define_part(&mut class);))
        .collect();
    let rbs_parts: Vec<TokenStream> = (0..8usize)
        .map(|part| quote!((&#private::Part::<#name, #part>::new()).rbs_part(&mut out);))
        .collect();
    let superclass = match &extends {
        Some(parent) => {
            quote!(#private::Option::Some(<#parent as ::rutie::derive::RubyClass>::PATH))
        }
        None => quote!(#private::Option::None),
    };

    Ok(quote! {
        #item

        impl ::rutie::derive::RubyClass for #name {
            const PATH: &'static str = #path;

            fn data_type() -> &'static ::rutie::derive::RubyClassDataType<Self> {
                static DATA_TYPE: #private::OnceLock<::rutie::derive::RubyClassDataType<#name>> =
                    #private::OnceLock::new();

                ::rutie::derive::RubyClassDataType::get(&DATA_TYPE)
            }

            fn define() -> ::rutie::Class {
                #name::define_ruby()
            }

            #parent
            #allocator
            #mark_fn
            #compact_fn
            #size_fn
        }

        impl ::rutie::derive::Mark for #name {
            fn mark(&self, marker: &::rutie::derive::Marker) {
                <Self as ::rutie::derive::RubyClass>::mark(self, marker)
            }

            fn compact(&mut self) {
                <Self as ::rutie::derive::RubyClass>::compact(self)
            }
        }

        impl ::rutie::derive::IntoRubyObject for #name {
            fn into_ruby_object(
                self,
                receiver: &::rutie::AnyObject,
            ) -> #private::Result<::rutie::AnyObject, ::rutie::AnyException> {
                #private::wrap(self, receiver)
            }
        }

        impl ::rutie::derive::FromArgument for #name {
            type Holder = ::rutie::derive::Borrowed<#name>;

            fn hold(
                argument: ::rutie::AnyObject,
            ) -> #private::Result<Self::Holder, ::rutie::AnyException> {
                <Self as ::rutie::derive::RubyClass>::borrow_from(&argument)
            }

            fn borrow(holder: &Self::Holder) -> &Self {
                holder
            }
        }

        impl ::rutie::derive::FromArgumentMut for #name {
            type HolderMut = ::rutie::derive::BorrowedMut<#name>;

            fn hold_mut(
                argument: ::rutie::AnyObject,
            ) -> #private::Result<Self::HolderMut, ::rutie::AnyException> {
                <Self as ::rutie::derive::RubyClass>::borrow_mut_from(&argument)
            }

            fn borrow_mut(holder: &mut Self::HolderMut) -> &mut Self {
                holder
            }
        }

        impl ::rutie::derive::RbsType for #name {
            fn rbs() -> #private::String {
                #private::String::from(#path)
            }
        }

        impl #name {
            /// Defines the Ruby class: its allocator, field accessors,
            /// `initialize_copy` and the methods of its `#[rubymethods]`
            /// blocks.
            pub fn define_ruby() -> ::rutie::Class {
                #[allow(unused_imports)]
                use #private::{DefinePart as _, DefinePartFallback as _, ProbeClone as _, ProbeCloneFallback as _};

                let mut class = <#name as ::rutie::derive::RubyClass>::define_class();

                ::rutie::rutie_callback! {
                    fn __rutie_initialize_copy(
                        receiver: ::rutie::AnyObject,
                        original: ::rutie::AnyObject,
                    ) -> ::rutie::AnyObject {
                        #[allow(unused_imports)]
                        use #private::{ProbeClone as _, ProbeCloneFallback as _};

                        #private::invoke(move || {
                            #private::initialize_copy::<#name>(
                                &receiver,
                                &original,
                                (&#private::CloneProbe::<#name>::new()).clone_fn(),
                            )
                        })
                    }
                }

                #private::define(
                    &class,
                    "initialize_copy",
                    __rutie_initialize_copy as *const ::rutie::types::c_void,
                    1,
                    #private::Definition::Private,
                );

                #(#accessor_callbacks)*
                #(#accessor_defines)*
                #(#parts)*

                class
            }

            /// The class's RBS declaration.
            pub fn rbs() -> #private::String {
                #[allow(unused_imports)]
                use #private::{DefinePart as _, DefinePartFallback as _};

                let mut out: #private::Vec<#private::String> = #private::Vec::new();

                #(#accessor_rbs)*
                #(#rbs_parts)*

                #private::declaration(#path, true, #superclass, &out)
            }
        }
    })
}
