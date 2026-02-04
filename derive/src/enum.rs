use darling::ast::Data;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, ext::IdentExt};

use crate::{
    args::{self, RenameRuleExt, RenameTarget, TypeDirectiveLocation},
    utils::{
        GeneratorResult, gen_boxed_trait, gen_deprecation, gen_directive_calls, get_crate_path,
        get_rustdoc, visible_fn,
    },
};

pub fn generate(enum_args: &args::Enum) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_path(&enum_args.crate_path, enum_args.internal);
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
    let gql_typename_string = if !enum_args.name_type {
        let name = enum_args
            .name
            .clone()
            .unwrap_or_else(|| RenameTarget::Type.rename(ident.to_string()));
        quote!(::std::string::ToString::to_string(#name))
    } else {
        quote!(::std::string::ToString::to_string(&#gql_typename))
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

    let directives = gen_directive_calls(
        &crate_name,
        &enum_args.directives,
        TypeDirectiveLocation::Enum,
    );
    let desc = get_rustdoc(&enum_args.attrs)?;
    let has_desc = desc.is_some();

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
        let directives = gen_directive_calls(
            &crate_name,
            &variant.directives,
            TypeDirectiveLocation::EnumValue,
        );
        let item_deprecation = gen_deprecation(&variant.deprecation, &crate_name);
        let item_desc = get_rustdoc(&variant.attrs)?;

        enum_items.push(item_ident);
        enum_names.push(gql_item_name.clone());
        items.push(quote! {
            #crate_name::resolver_utils::EnumItem {
                name: #gql_item_name,
                value: #ident::#item_ident,
            }
        });

        let visible = visible_fn(&variant.visible);
        let has_desc = item_desc.is_some();
        let has_deprecation = !matches!(variant.deprecation, args::Deprecation::NoDeprecated);
        let has_visible = !matches!(variant.visible, None | Some(args::Visible::None));
        let has_tags = !variant.tags.is_empty();
        let has_directives = !variant.directives.is_empty();

        let mut value_sets = Vec::new();
        if has_desc {
            let desc = item_desc.as_ref().expect("checked desc");
            value_sets.push(quote! {
                value.description = ::std::option::Option::Some(::std::string::ToString::to_string(#desc));
            });
        }
        if has_deprecation {
            value_sets.push(quote!(value.deprecation = #item_deprecation;));
        }
        if has_visible {
            value_sets.push(quote!(value.visible = #visible;));
        }
        if inaccessible {
            value_sets.push(quote!(value.inaccessible = true;));
        }
        if has_tags {
            value_sets.push(quote!(value.tags = ::std::vec![ #(#tags),* ];));
        }
        if has_directives {
            value_sets.push(quote!(value.directive_invocations = ::std::vec![ #(#directives),* ];));
        }

        schema_enum_items.push(quote! {
            {
                let mut value = #crate_name::registry::MetaEnumValue::new(
                    ::std::string::ToString::to_string(#gql_item_name),
                );
                #(#value_sets)*
                enum_items.insert(::std::string::ToString::to_string(#gql_item_name), value);
            }
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
    let has_visible = !matches!(enum_args.visible, None | Some(args::Visible::None));
    let has_tags = !enum_args.tags.is_empty();
    let has_directives = !enum_args.directives.is_empty();
    let has_requires_scopes = !enum_args.requires_scopes.is_empty();
    let mut enum_builder_calls = Vec::new();
    if has_desc {
        let desc = desc.as_ref().expect("checked desc");
        enum_builder_calls.push(quote! {
            .description(::std::option::Option::Some(::std::string::ToString::to_string(#desc)))
        });
    }
    if has_visible {
        enum_builder_calls.push(quote!(.visible(#visible)));
    }
    if inaccessible {
        enum_builder_calls.push(quote!(.inaccessible(true)));
    }
    if has_tags {
        enum_builder_calls.push(quote!(.tags(::std::vec![ #(#tags),* ])));
    }
    enum_builder_calls.push(quote!(.rust_typename(::std::any::type_name::<Self>())));
    if has_directives {
        enum_builder_calls.push(quote!(.directive_invocations(::std::vec![ #(#directives),* ])));
    }
    if has_requires_scopes {
        enum_builder_calls.push(quote!(.requires_scopes(::std::vec![ #(#requires_scopes),* ])));
    }
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
                    #crate_name::registry::EnumBuilder::new(
                        #gql_typename_string,
                        {
                            let mut enum_items = #crate_name::indexmap::IndexMap::new();
                            #(#schema_enum_items)*
                            enum_items
                        },
                    )
                    #(#enum_builder_calls)*
                    .build()
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

        #boxed_trait
        impl #crate_name::OutputType for #ident {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                Self::__type_name()
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                Self::__create_type_info(registry)
            }

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
