//! `#[derive(Mark)]`, and the field-by-field marking `#[rubyclass]` shares.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, Data, DeriveInput, Fields, Result};

use crate::{attrs::RubyOptions, method::private};

/// Statements marking and compacting each of `places` (field expressions).
/// A field whose type implements `Mark` is marked; others are skipped.
pub fn field_marking(places: &[TokenStream]) -> (TokenStream, TokenStream) {
    let private = private();

    (
        quote! {
            #[allow(unused_imports)]
            use #private::{MarkField as _, MarkFieldFallback as _};
            #((&#places).__rutie_mark(marker);)*
        },
        quote! {
            #[allow(unused_imports)]
            use #private::{CompactField as _, CompactFieldFallback as _};
            #((&mut #places).__rutie_compact();)*
        },
    )
}

pub fn derive(mut input: DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;

    // A type parameter must be `Mark` for its fields to be marked.
    for parameter in input.generics.type_params_mut() {
        parameter.bounds.push(parse_quote!(::rutie::derive::Mark));
    }

    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let (mark, compact) = match &input.data {
        Data::Struct(data) => {
            let places = marked_fields(&data.fields)?
                .into_iter()
                .map(|member| quote!(self.#member))
                .collect::<Vec<_>>();
            let (mark, compact) = field_marking(&places);

            (mark, compact)
        }
        Data::Enum(data) => {
            let mut mark_arms = Vec::new();
            let mut compact_arms = Vec::new();

            for variant in &data.variants {
                let variant_name = &variant.ident;
                let members = marked_fields(&variant.fields)?;
                let bindings: Vec<_> = (0..members.len())
                    .map(|index| format_ident!("field_{}", index))
                    .collect();
                let pattern = quote!(Self::#variant_name { #(#members: #bindings,)* .. });
                let places: Vec<TokenStream> =
                    bindings.iter().map(|binding| quote!(*#binding)).collect();
                let (mark, compact) = field_marking(&places);

                mark_arms.push(quote!(#pattern => { #mark }));
                compact_arms.push(quote!(#pattern => { #compact }));
            }

            (
                quote! {
                    match self { #(#mark_arms)* _ => {} }
                },
                quote! {
                    match self { #(#compact_arms)* _ => {} }
                },
            )
        }
        Data::Union(union) => {
            return Err(syn::Error::new(
                union.union_token.span,
                "`#[derive(Mark)]` doesn't support unions",
            ))
        }
    };

    Ok(quote! {
        impl #impl_generics ::rutie::derive::Mark for #name #type_generics #where_clause {
            #[allow(unreachable_patterns)]
            fn mark(&self, marker: &::rutie::derive::Marker) {
                #mark
            }

            #[allow(unreachable_patterns)]
            fn compact(&mut self) {
                #compact
            }
        }
    })
}

// The fields to mark, as members; `#[ruby(no_mark)]` skips one.
fn marked_fields(fields: &Fields) -> Result<Vec<syn::Member>> {
    let mut members = Vec::new();

    for (index, field) in fields.iter().enumerate() {
        let mut attributes = field.attrs.clone();
        let options = RubyOptions::take(&mut attributes)?;
        options.only(&["no_mark"], "a `#[derive(Mark)]` field")?;

        if options.no_mark {
            continue;
        }

        members.push(match &field.ident {
            Some(ident) => syn::Member::Named(ident.clone()),
            None => syn::Member::Unnamed(syn::Index::from(index)),
        });
    }

    Ok(members)
}
