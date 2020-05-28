use crate::args;
use crate::utils::{check_reserved_name, get_crate_name, get_rustdoc};
use inflector::Inflector;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Data, DeriveInput, Error, Meta, NestedMeta, Result};

const DERIVES: &[&str] = &["Copy", "Clone", "Eq", "PartialEq"];

pub fn generate(enum_args: &args::Enum, input: &DeriveInput) -> Result<TokenStream> {
    let crate_name = get_crate_name(enum_args.internal);
    let attrs = &input.attrs;
    let vis = &input.vis;
    let ident = &input.ident;
    let e = match &input.data {
        Data::Enum(e) => e,
        _ => return Err(Error::new_spanned(input, "It should be a enum")),
    };

    let mut new_attrs = Vec::new();
    let mut has_derive = false;
    for attr in attrs {
        match attr.parse_meta()? {
            Meta::List(ls) if ls.path.is_ident("derive") => {
                let mut items = ls
                    .nested
                    .iter()
                    .map(|item| quote! { #item })
                    .collect::<Vec<_>>();

                for name in DERIVES {
                    let mut found = false;
                    for nested_meta in &ls.nested {
                        if let NestedMeta::Meta(meta) = nested_meta {
                            if meta.path().is_ident(name) {
                                found = true;
                                break;
                            }
                        }
                    }
                    if !found {
                        let name_ident = Ident::new(name, Span::call_site());
                        items.push(quote! { #name_ident })
                    }
                }

                new_attrs.push(quote! { #[derive(#(#items),*)] });
                has_derive = true;
            }
            _ => new_attrs.push(quote! { #attr }),
        }
    }

    if !has_derive {
        let mut items = Vec::new();
        for name in DERIVES {
            let name_ident = Ident::new(name, Span::call_site());
            items.push(quote! { #name_ident });
        }
        new_attrs.push(quote! { #[derive(#(#items),*)] });
    }

    let gql_typename = enum_args.name.clone().unwrap_or_else(|| ident.to_string());
    check_reserved_name(&gql_typename, enum_args.internal)?;

    let desc = enum_args
        .desc
        .clone()
        .or_else(|| get_rustdoc(&input.attrs).ok().flatten())
        .map(|s| quote! { Some(#s) })
        .unwrap_or_else(|| quote! {None});

    let mut enum_items = Vec::new();
    let mut items = Vec::new();
    let mut schema_enum_items = Vec::new();

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
        let item_attrs = variant
            .attrs
            .iter()
            .filter(|attr| !attr.path.is_ident("item"))
            .collect::<Vec<_>>();
        let mut item_args = args::EnumItem::parse(&variant.attrs)?;
        let gql_item_name = item_args
            .name
            .take()
            .unwrap_or_else(|| variant.ident.to_string().to_screaming_snake_case());
        let item_deprecation = item_args
            .deprecation
            .as_ref()
            .map(|s| quote! { Some(#s) })
            .unwrap_or_else(|| quote! {None});
        let item_desc = item_args
            .desc
            .as_ref()
            .map(|s| quote! { Some(#s) })
            .unwrap_or_else(|| quote! {None});
        enum_items.push(quote! { #(#item_attrs)* #item_ident});
        items.push(quote! {
            #crate_name::EnumItem {
                name: #gql_item_name,
                value: #ident::#item_ident,
            }
        });
        schema_enum_items.push(quote! {
            enum_items.insert(#gql_item_name, #crate_name::registry::MetaEnumValue {
                name: #gql_item_name,
                description: #item_desc,
                deprecation: #item_deprecation,
            });
        });
    }

    let expanded = quote! {
        #(#new_attrs)*
        #vis enum #ident {
            #(#enum_items),*
        }

        impl #crate_name::EnumType for #ident {
            fn items() -> &'static [#crate_name::EnumItem<#ident>] {
                &[#(#items),*]
            }
        }

        impl #crate_name::Type for #ident {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> String {
                registry.create_type::<Self, _>(|registry| {
                    #crate_name::registry::MetaType::Enum {
                        name: #gql_typename.to_string(),
                        description: #desc,
                        enum_values: {
                            let mut enum_items = #crate_name::indexmap::IndexMap::new();
                            #(#schema_enum_items)*
                            enum_items
                        },
                    }
                })
            }
        }

        impl #crate_name::InputValueType for #ident {
            fn parse(value: Option<#crate_name::Value>) -> #crate_name::InputValueResult<Self> {
                #crate_name::EnumType::parse_enum(value.unwrap_or_default())
            }

            fn to_value(&self) -> #crate_name::Value {
                #crate_name::EnumType::to_value(self)
            }
        }

        #[#crate_name::async_trait::async_trait]
        impl #crate_name::OutputValueType for #ident {
            async fn resolve(&self, _: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::query::Field>) -> #crate_name::Result<#crate_name::serde_json::Value> {
                Ok(#crate_name::EnumType::to_value(self).into())
            }
        }
    };
    Ok(expanded.into())
}
