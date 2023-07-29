use proc_macro::TokenStream;
use quote::quote;
use syn::{ext::IdentExt, FnArg, ItemFn, Pat};

use crate::{
    args,
    args::{Argument, RenameRuleExt, RenameTarget},
    utils::{
        generate_default, get_crate_name, get_rustdoc, parse_graphql_attrs, remove_graphql_attrs,
        visible_fn, GeneratorResult,
    },
};

pub fn generate(
    directive_args: &args::TypeDirective,
    item_fn: &mut ItemFn,
) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(directive_args.internal);
    let ident = &item_fn.sig.ident;
    let vis = &item_fn.vis;
    let directive_name = if !directive_args.name_type {
        let name = directive_args
            .name
            .clone()
            .unwrap_or_else(|| item_fn.sig.ident.to_string());
        quote!(::std::borrow::Cow::Borrowed(#name))
    } else {
        quote!(<Self as #crate_name::TypeName>::type_name())
    };
    let desc = get_rustdoc(&item_fn.attrs)?
        .map(|s| quote!(::std::option::Option::Some(::std::string::ToString::to_string(#s))))
        .unwrap_or_else(|| quote!(::std::option::Option::None));
    let visible = visible_fn(&directive_args.visible);
    let repeatable = directive_args.repeatable;

    let composable = match directive_args.composable.as_ref() {
        Some(url) => quote!(::std::option::Option::Some(::std::string::ToString::to_string(#url))),
        None => quote!(::std::option::Option::None),
    };

    let mut schema_args = Vec::new();
    let mut input_args = Vec::new();
    let mut directive_input_args = Vec::new();

    for arg in item_fn.sig.inputs.iter_mut() {
        let mut arg_info = None;

        if let FnArg::Typed(pat) = arg {
            if let Pat::Ident(ident) = &*pat.pat {
                arg_info = Some((ident.clone(), pat.ty.clone(), pat.attrs.clone()));
                remove_graphql_attrs(&mut pat.attrs);
            }
        }

        let (arg_ident, arg_ty, arg_attrs) = match arg_info {
            Some(info) => info,
            None => {
                return Err(syn::Error::new_spanned(arg, "Invalid argument type.").into());
            }
        };

        let Argument {
            name,
            desc,
            default,
            default_with,
            visible,
            secret,
            ..
        } = parse_graphql_attrs::<args::Argument>(&arg_attrs)?.unwrap_or_default();

        let name = name.clone().unwrap_or_else(|| {
            directive_args
                .rename_args
                .rename(arg_ident.ident.unraw().to_string(), RenameTarget::Argument)
        });
        let desc = desc
            .as_ref()
            .map(|s| quote! {::std::option::Option::Some(::std::string::ToString::to_string(#s))})
            .unwrap_or_else(|| quote! {::std::option::Option::None});
        let default = generate_default(&default, &default_with)?;
        let schema_default = default
            .as_ref()
            .map(|value| {
                quote! {
                    ::std::option::Option::Some(::std::string::ToString::to_string(
                        &<#arg_ty as #crate_name::InputType>::to_value(&#value)
                    ))
                }
            })
            .unwrap_or_else(|| quote! {::std::option::Option::None});
        let visible = visible_fn(&visible);

        schema_args.push(quote! {
            args.insert(::std::borrow::ToOwned::to_owned(#name), #crate_name::registry::MetaInputValue {
                name: ::std::string::ToString::to_string(#name),
                description: #desc,
                ty: <#arg_ty as #crate_name::InputType>::create_type_info(registry),
                default_value: #schema_default,
                visible: #visible,
                inaccessible: false,
                tags: ::std::default::Default::default(),
                is_secret: #secret,
            });
        });

        input_args.push(quote! { #arg });

        directive_input_args.push(quote! {
            if let Some(val) = #crate_name::InputType::as_raw_value(&#arg_ident) {
                args.insert(::std::string::ToString::to_string(#name), #crate_name::ScalarType::to_value(val));
            };
        });
    }

    let locations = directive_args
        .locations
        .iter()
        .map(|loc| {
            let loc = quote::format_ident!("{}", loc.to_string());
            quote!(#crate_name::registry::__DirectiveLocation::#loc)
        })
        .collect::<Vec<_>>();

    if locations.is_empty() {
        return Err(syn::Error::new(
            ident.span(),
            "At least one location is required for the directive.",
        )
        .into());
    }

    let location_traits = directive_args
        .locations
        .iter()
        .map(|loc| loc.location_trait_identifier())
        .collect::<Vec<_>>();

    let expanded = quote! {
        #[allow(non_camel_case_types)]
        #vis struct #ident;

        impl #crate_name::TypeDirective for #ident {
            fn name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                #directive_name
            }

            fn register(&self, registry: &mut #crate_name::registry::Registry) {
                let meta = #crate_name::registry::MetaDirective {
                    name: ::std::borrow::Cow::into_owned(#directive_name),
                    description: #desc,
                    locations: vec![#(#locations),*],
                    args: {
                        #[allow(unused_mut)]
                        let mut args = #crate_name::indexmap::IndexMap::new();
                        #(#schema_args)*
                        args
                    },
                    is_repeatable: #repeatable,
                    visible: #visible,
                    composable: #composable,
                };
                registry.add_directive(meta);
            }

        }

       #(impl #crate_name::registry::location_traits::#location_traits for #ident {})*

        impl #ident {
            pub fn apply(#(#input_args),*) -> #crate_name::registry::MetaDirectiveInvocation {
                let directive = ::std::borrow::Cow::into_owned(#directive_name);
                let mut args = #crate_name::indexmap::IndexMap::new();
                #(#directive_input_args)*;
                #crate_name::registry::MetaDirectiveInvocation {
                    name: directive,
                    args,
                }
            }
        }
    };

    Ok(expanded.into())
}
