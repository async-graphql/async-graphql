use crate::args;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Result};

pub fn generate(enum_args: &args::Enum, input: &DeriveInput) -> Result<TokenStream> {
    let attrs = &input.attrs;
    let vis = &input.vis;
    let ident = &input.ident;
    let e = match &input.data {
        Data::Enum(e) => e,
        _ => return Err(Error::new_spanned(input, "It should be a enum")),
    };

    let gql_typename = enum_args.name.clone().unwrap_or_else(|| ident.to_string());

    let mut enum_items = Vec::new();
    let mut items = Vec::new();
    for variant in &e.variants {
        if !variant.fields.is_empty() {
            return Err(Error::new_spanned(
                &variant,
                format!(
                    "Invalid enum variant {}.\nGraphQL enums may only contain unit variants.",
                    variant.ident
                ),
            ));
        }

        let item_ident = &variant.ident;
        let mut item_args = args::EnumItem::parse(&variant.attrs)?;
        let gql_item_name = item_args
            .name
            .take()
            .unwrap_or_else(|| variant.ident.to_string());
        enum_items.push(variant);
        let desc = match item_args.desc.take() {
            Some(desc) => quote! { Some(#desc) },
            None => quote! { None },
        };
        items.push(quote! {
            async_graphql::GQLEnumItem {
                name: #gql_item_name,
                desc: #desc,
                value: #ident::#item_ident,
            }
        });
    }

    let expanded = quote! {
        #(#attrs)*
        #[derive(Copy, Clone, Eq, PartialEq)]
        #vis enum #ident {
            #(#enum_items),*
        }

        impl async_graphql::GQLEnum for #ident {
            fn items() -> &'static [async_graphql::GQLEnumItem<#ident>] {
                &[#(#items),*]
            }
        }

        impl async_graphql::GQLType for #ident {
            fn type_name() -> String {
                #gql_typename.to_string()
            }
        }

        impl async_graphql::GQLInputValue for #ident {
            fn parse(value: async_graphql::Value) -> Result<Self> {
                Self::parse_enum(value)
            }

            fn parse_from_json(value: async_graphql::serde_json::Value) -> Result<Self> {
                Self::parse_json_enum(value)
            }
        }

        #[async_graphql::async_trait::async_trait]
        impl async_graphql::GQLOutputValue for #ident {
            async fn resolve(self, _: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
                self.resolve_enum()
            }
        }
    };
    Ok(expanded.into())
}
