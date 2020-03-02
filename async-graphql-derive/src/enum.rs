use crate::args;
use crate::utils::get_crate_name;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Result};

pub fn generate(enum_args: &args::Enum, input: &DeriveInput) -> Result<TokenStream> {
    let crate_name = get_crate_name(enum_args.internal);
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
        enum_items.push(&variant.ident);
        let desc = match item_args.desc.take() {
            Some(desc) => quote! { Some(#desc) },
            None => quote! { None },
        };
        items.push(quote! {
            #crate_name::GQLEnumItem {
                name: #gql_item_name,
                desc: #desc,
                value: #ident::#item_ident,
            }
        });
    }

    let expanded = quote! {
        #(#attrs)*
        #[derive(Copy, Clone, Eq, PartialEq, Debug)]
        #vis enum #ident {
            #(#enum_items),*
        }

        impl #crate_name::GQLEnum for #ident {
            fn items() -> &'static [#crate_name::GQLEnumItem<#ident>] {
                &[#(#items),*]
            }
        }

        impl #crate_name::GQLType for #ident {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#gql_typename)
            }
        }

        impl #crate_name::GQLInputValue for #ident {
            fn parse(value: #crate_name::Value) -> #crate_name::Result<Self> {
                #crate_name::GQLEnum::parse_enum(value)
            }

            fn parse_from_json(value: #crate_name::serde_json::Value) -> #crate_name::Result<Self> {
                #crate_name::GQLEnum::parse_json_enum(value)
            }
        }

        #[#crate_name::async_trait::async_trait]
        impl #crate_name::GQLOutputValue for #ident {
            async fn resolve(&self, _: &#crate_name::ContextSelectionSet<'_>) -> #crate_name::Result<serde_json::Value> {
                #crate_name::GQLEnum::resolve_enum(self)
            }
        }
    };
    Ok(expanded.into())
}
