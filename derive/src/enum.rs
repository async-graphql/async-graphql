use darling::ast::Data;
use proc_macro::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::Error;

use crate::args::{self, RenameRuleExt, RenameTarget};
use crate::utils::{get_crate_name, get_rustdoc, GeneratorResult};

pub fn generate(enum_args: &args::Enum) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(enum_args.internal);
    let ident = &enum_args.ident;
    let e = match &enum_args.data {
        Data::Enum(e) => e,
        _ => return Err(Error::new_spanned(ident, "Enum can only be applied to an enum.").into()),
    };

    let gql_typename = enum_args
        .name
        .clone()
        .unwrap_or_else(|| RenameTarget::Type.rename(ident.to_string()));

    let desc = get_rustdoc(&enum_args.attrs)?
        .map(|s| quote! { ::std::option::Option::Some(#s) })
        .unwrap_or_else(|| quote! {::std::option::Option::None});

    let mut enum_items = Vec::new();
    let mut items = Vec::new();
    let mut schema_enum_items = Vec::new();

    for variant in e {
        if !variant.fields.is_empty() {
            return Err(Error::new_spanned(
                &variant.ident,
                format!(
                    "Invalid enum variant {}.\nGraphQL enums may only contain unit variants.",
                    variant.ident
                ),
            )
            .into());
        }

        let item_ident = &variant.ident;
        let gql_item_name = variant.name.clone().unwrap_or_else(|| {
            enum_args
                .rename_items
                .rename(variant.ident.unraw().to_string(), RenameTarget::EnumItem)
        });
        let item_deprecation = variant
            .deprecation
            .as_ref()
            .map(|s| quote! { ::std::option::Option::Some(#s) })
            .unwrap_or_else(|| quote! {::std::option::Option::None});
        let item_desc = get_rustdoc(&variant.attrs)?
            .map(|s| quote! { ::std::option::Option::Some(#s) })
            .unwrap_or_else(|| quote! {::std::option::Option::None});

        enum_items.push(item_ident);
        items.push(quote! {
            #crate_name::resolver_utils::EnumItem {
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

    let remote_conversion = if let Some(remote) = &enum_args.remote {
        let remote_ty = if let Ok(ty) = syn::parse_str::<syn::Type>(remote) {
            ty
        } else {
            return Err(
                Error::new_spanned(remote, format!("Invalid remote type: '{}'", remote)).into(),
            );
        };

        let local_to_remote_items = enum_items.iter().map(|item| {
            quote! {
                #ident::#item => #remote_ty::#item,
            }
        });
        let remote_to_local_items = enum_items.iter().map(|item| {
            quote! {
                #remote_ty::#item => #ident::#item,
            }
        });
        Some(quote! {
            impl ::std::convert::From<#ident> for #remote_ty {
                fn from(value: #ident) -> Self {
                    match value {
                        #(#local_to_remote_items)*
                    }
                }
            }

            impl ::std::convert::From<#remote_ty> for #ident {
                fn from(value: #remote_ty) -> Self {
                    match value {
                        #(#remote_to_local_items)*
                    }
                }
            }
        })
    } else {
        None
    };

    let expanded = quote! {
        #[allow(clippy::all, clippy::pedantic)]
        impl #crate_name::resolver_utils::EnumType for #ident {
            fn items() -> &'static [#crate_name::resolver_utils::EnumItem<#ident>] {
                &[#(#items),*]
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        impl #crate_name::Type for #ident {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
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

        #[allow(clippy::all, clippy::pedantic)]
        impl #crate_name::InputValueType for #ident {
            fn parse(value: ::std::option::Option<#crate_name::Value>) -> #crate_name::InputValueResult<Self> {
                #crate_name::resolver_utils::parse_enum(value.unwrap_or_default())
            }

            fn to_value(&self) -> #crate_name::Value {
                #crate_name::resolver_utils::enum_value(*self)
            }
        }

        #[#crate_name::async_trait::async_trait]
        impl #crate_name::OutputValueType for #ident {
            async fn resolve(&self, _: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::Value> {
                ::std::result::Result::Ok(#crate_name::resolver_utils::enum_value(*self))
            }
        }

        #remote_conversion
    };
    Ok(expanded.into())
}
