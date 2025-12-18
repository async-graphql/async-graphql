use darling::ast::Data;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, ext::IdentExt};

use crate::{
    args::{self, RenameRuleExt, RenameTarget, TypeDirectiveLocation},
    utils::{
        GeneratorResult, gen_boxed_trait, gen_deprecation, gen_directive_calls, get_crate_name,
        get_rustdoc, visible_fn,
    },
};

pub fn generate(enum_args: &args::Enum) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(enum_args.internal);
    let boxed_trait = gen_boxed_trait(&crate_name);
    let ident = &enum_args.ident;
    let e = match &enum_args.data {
        Data::Enum(e) => e,
        _ => return Err(Error::new_spanned(ident, "Enum can only be applied to an enum.").into()),
    };

    let gql_typename = if !enum_args.name_type {
        let name = enum_args
            .name
            .clone()
            .unwrap_or_else(|| RenameTarget::Type.rename(ident.to_string()));
        quote!(::std::borrow::Cow::Borrowed(#name))
    } else {
        quote!(<Self as #crate_name::TypeName>::type_name())
    };

    let inaccessible = enum_args.inaccessible;
    let tags = enum_args
        .tags
        .iter()
        .map(|tag| quote!(::std::string::ToString::to_string(#tag)))
        .collect::<Vec<_>>();
    let requires_scopes = enum_args
        .requires_scopes
        .iter()
        .map(|scopes| quote!(::std::string::ToString::to_string(#scopes)))
        .collect::<Vec<_>>();
    let directives = gen_directive_calls(&enum_args.directives, TypeDirectiveLocation::Enum);
    let desc = get_rustdoc(&enum_args.attrs)?
        .map(|s| quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#s)) })
        .unwrap_or_else(|| quote! {::std::option::Option::None});

    let mut enum_items = Vec::new();
    let mut enum_names = Vec::new();
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
        let inaccessible = variant.inaccessible;
        let tags = variant
            .tags
            .iter()
            .map(|tag| quote!(::std::string::ToString::to_string(#tag)))
            .collect::<Vec<_>>();
        let directives = gen_directive_calls(&variant.directives, TypeDirectiveLocation::EnumValue);
        let item_deprecation = gen_deprecation(&variant.deprecation, &crate_name);
        let item_desc = get_rustdoc(&variant.attrs)?
            .map(|s| quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#s)) })
            .unwrap_or_else(|| quote! {::std::option::Option::None});

        enum_items.push(item_ident);
        enum_names.push(gql_item_name.clone());
        items.push(quote! {
            #crate_name::resolver_utils::EnumItem {
                name: #gql_item_name,
                value: #ident::#item_ident,
            }
        });

        let visible = visible_fn(&variant.visible);
        schema_enum_items.push(quote! {
            enum_items.insert(::std::string::ToString::to_string(#gql_item_name), #crate_name::registry::MetaEnumValue {
                name: ::std::string::ToString::to_string(#gql_item_name),
                description: #item_desc,
                deprecation: #item_deprecation,
                visible: #visible,
                inaccessible: #inaccessible,
                tags: ::std::vec![ #(#tags),* ],
                directive_invocations: ::std::vec![ #(#directives),* ]
            });
        });
    }

    let remote_conversion = if let Some(remote_ty) = &enum_args.remote {
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

    if schema_enum_items.is_empty() {
        return Err(Error::new_spanned(
            ident,
            "A GraphQL Enum type must define one or more unique enum values.",
        )
        .into());
    }

    let display = if enum_args.display {
        let items = enum_items.iter().zip(&enum_names).map(|(item, name)| {
            quote! {
                #ident::#item => #name,
            }
        });
        Some(quote! {
            impl ::std::fmt::Display for #ident {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    f.write_str(match self {
                        #(#items)*
                    })
                }
            }
        })
    } else {
        None
    };

    let visible = visible_fn(&enum_args.visible);
    let expanded = quote! {
        #[allow(clippy::all, clippy::pedantic)]
        impl #crate_name::resolver_utils::EnumType for #ident {
            fn items() -> &'static [#crate_name::resolver_utils::EnumItem<#ident>] {
                &[#(#items),*]
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        impl #ident {
            fn __type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                #gql_typename
            }

            fn __create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                registry.create_input_type::<Self, _>(#crate_name::registry::MetaTypeId::Enum, |registry| {
                    #crate_name::registry::MetaType::Enum {
                        name: ::std::borrow::Cow::into_owned(#gql_typename),
                        description: #desc,
                        enum_values: {
                            let mut enum_items = #crate_name::indexmap::IndexMap::new();
                            #(#schema_enum_items)*
                            enum_items
                        },
                        visible: #visible,
                        inaccessible: #inaccessible,
                        tags: ::std::vec![ #(#tags),* ],
                        rust_typename: ::std::option::Option::Some(::std::any::type_name::<Self>()),
                        directive_invocations: ::std::vec![ #(#directives),* ],
                        requires_scopes: ::std::vec![ #(#requires_scopes),* ],
                    }
                })
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        impl #crate_name::InputType for #ident {
            type RawValueType = Self;

            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                Self::__type_name()
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                Self::__create_type_info(registry)
            }

            fn parse(value: ::std::option::Option<#crate_name::Value>) -> #crate_name::InputValueResult<Self> {
                #crate_name::resolver_utils::parse_enum(value.unwrap_or_default())
            }

            fn to_value(&self) -> #crate_name::Value {
                #crate_name::resolver_utils::enum_value(*self)
            }

            fn as_raw_value(&self) -> ::std::option::Option<&Self::RawValueType> {
                ::std::option::Option::Some(self)
            }
        }

         impl #crate_name::OutputTypeMarker for #ident {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                Self::__type_name()
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                Self::__create_type_info(registry)
            }
         }

        #boxed_trait
        impl #crate_name::OutputType for #ident {
            async fn resolve(&self, _: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::Value> {
                ::std::result::Result::Ok(#crate_name::resolver_utils::enum_value(*self))
            }
        }

        impl ::std::convert::From<#ident> for #crate_name::Value {
            fn from(value: #ident) -> #crate_name::Value {
                #crate_name::resolver_utils::enum_value(value)
            }
        }

        #remote_conversion
        #display
    };
    Ok(expanded.into())
}
