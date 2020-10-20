use darling::ast::{Data, Style};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::collections::HashSet;
use syn::visit_mut::VisitMut;
use syn::{visit_mut, Error, Lifetime, Type};

use crate::args::{self, InterfaceField, InterfaceFieldArgument, RenameRuleExt, RenameTarget};
use crate::output_type::OutputType;
use crate::utils::{generate_default, get_crate_name, get_rustdoc, GeneratorResult};

pub fn generate(interface_args: &args::Interface) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(interface_args.internal);
    let ident = &interface_args.ident;
    let generics = &interface_args.generics;
    let s = match &interface_args.data {
        Data::Enum(s) => s,
        _ => {
            return Err(
                Error::new_spanned(ident, "Interface can only be applied to an enum.").into(),
            )
        }
    };
    let extends = interface_args.extends;
    let mut enum_names = Vec::new();
    let mut enum_items = HashSet::new();
    let mut type_into_impls = Vec::new();
    let gql_typename = interface_args
        .name
        .clone()
        .unwrap_or_else(|| RenameTarget::Type.rename(ident.to_string()));

    let desc = get_rustdoc(&interface_args.attrs)?
        .map(|s| quote! { ::std::option::Option::Some(#s) })
        .unwrap_or_else(|| quote! {::std::option::Option::None});

    let mut registry_types = Vec::new();
    let mut possible_types = Vec::new();
    let mut get_introspection_typename = Vec::new();
    let mut collect_all_fields = Vec::new();

    for variant in s {
        let enum_name = &variant.ident;
        let ty = match variant.fields.style {
            Style::Tuple if variant.fields.fields.len() == 1 => &variant.fields.fields[0],
            Style::Tuple => {
                return Err(Error::new_spanned(
                    enum_name,
                    "Only single value variants are supported",
                )
                .into())
            }
            Style::Unit => {
                return Err(
                    Error::new_spanned(enum_name, "Empty variants are not supported").into(),
                )
            }
            Style::Struct => {
                return Err(Error::new_spanned(
                    enum_name,
                    "Variants with named fields are not supported",
                )
                .into())
            }
        };

        if let Type::Path(p) = ty {
            // This validates that the field type wasn't already used
            if !enum_items.insert(p) {
                return Err(
                    Error::new_spanned(ty, "This type already used in another variant").into(),
                );
            }

            struct RemoveLifetime;
            impl VisitMut for RemoveLifetime {
                fn visit_lifetime_mut(&mut self, i: &mut Lifetime) {
                    i.ident = Ident::new("_", Span::call_site());
                    visit_mut::visit_lifetime_mut(self, i);
                }
            }

            let mut assert_ty = p.clone();
            RemoveLifetime.visit_type_path_mut(&mut assert_ty);

            type_into_impls.push(quote! {
                #crate_name::static_assertions::assert_impl_one!(#assert_ty: #crate_name::ObjectType);

                #[allow(clippy::all, clippy::pedantic)]
                impl #generics ::std::convert::From<#p> for #ident #generics {
                    fn from(obj: #p) -> Self {
                        #ident::#enum_name(obj)
                    }
                }
            });
            enum_names.push(enum_name);

            registry_types.push(quote! {
                <#p as #crate_name::Type>::create_type_info(registry);
                registry.add_implements(&<#p as #crate_name::Type>::type_name(), #gql_typename);
            });

            possible_types.push(quote! {
                possible_types.insert(<#p as #crate_name::Type>::type_name().into_owned());
            });

            get_introspection_typename.push(quote! {
                #ident::#enum_name(obj) => <#p as #crate_name::Type>::type_name()
            });

            collect_all_fields.push(quote! {
                #ident::#enum_name(obj) => obj.collect_all_fields(ctx, fields)
            });
        } else {
            return Err(Error::new_spanned(ty, "Invalid type").into());
        }
    }

    let mut methods = Vec::new();
    let mut schema_fields = Vec::new();
    let mut resolvers = Vec::new();

    if interface_args.fields.is_empty() {
        return Err(Error::new_spanned(
            &ident,
            "An GraphQL Interface type must define one or more fields.",
        )
        .into());
    }

    for InterfaceField {
        name,
        method,
        desc,
        ty,
        args,
        deprecation,
        external,
        provides,
        requires,
    } in &interface_args.fields
    {
        let (name, method_name) = if let Some(method) = method {
            (name.to_string(), Ident::new(method, Span::call_site()))
        } else {
            let method_name = Ident::new(&name, Span::call_site());
            (
                interface_args
                    .rename_fields
                    .rename(name, RenameTarget::Field),
                method_name,
            )
        };
        let ty = match syn::parse_str::<syn::Type>(&ty.value()) {
            Ok(ty) => ty,
            Err(_) => return Err(Error::new_spanned(&ty, "Expect type").into()),
        };
        let mut calls = Vec::new();
        let mut use_params = Vec::new();
        let mut decl_params = Vec::new();
        let mut get_params = Vec::new();
        let mut schema_args = Vec::new();
        let requires = match &requires {
            Some(requires) => quote! { ::std::option::Option::Some(#requires) },
            None => quote! { ::std::option::Option::None },
        };
        let provides = match &provides {
            Some(provides) => quote! { ::std::option::Option::Some(#provides) },
            None => quote! { ::std::option::Option::None },
        };

        decl_params.push(quote! { ctx: &'ctx #crate_name::Context<'ctx> });
        use_params.push(quote! { ctx });

        for InterfaceFieldArgument {
            name,
            desc,
            ty,
            default,
            default_with,
        } in args
        {
            let ident = Ident::new(name, Span::call_site());
            let name = interface_args
                .rename_args
                .rename(name, RenameTarget::Argument);
            let ty = match syn::parse_str::<syn::Type>(&ty.value()) {
                Ok(ty) => ty,
                Err(_) => return Err(Error::new_spanned(&ty, "Expect type").into()),
            };
            decl_params.push(quote! { #ident: #ty });
            use_params.push(quote! { #ident });

            let default = generate_default(&default, &default_with)?;
            let get_default = match &default {
                Some(default) => quote! { ::std::option::Option::Some(|| -> #ty { #default }) },
                None => quote! { ::std::option::Option::None },
            };
            get_params.push(quote! {
                let #ident: #ty = ctx.param_value(#name, #get_default)?;
            });

            let desc = desc
                .as_ref()
                .map(|s| quote! {::std::option::Option::Some(#s)})
                .unwrap_or_else(|| quote! {::std::option::Option::None});
            let schema_default = default
                .as_ref()
                .map(|value| {
                    quote! {
                        ::std::option::Option::Some(::std::string::ToString::to_string(
                            &<#ty as #crate_name::InputValueType>::to_value(&#value)
                        ))
                    }
                })
                .unwrap_or_else(|| quote! {::std::option::Option::None});
            schema_args.push(quote! {
                args.insert(#name, #crate_name::registry::MetaInputValue {
                    name: #name,
                    description: #desc,
                    ty: <#ty as #crate_name::Type>::create_type_info(registry),
                    default_value: #schema_default,
                    validator: ::std::option::Option::None,
                });
            });
        }

        for enum_name in &enum_names {
            calls.push(quote! {
                #ident::#enum_name(obj) => obj.#method_name(#(#use_params),*).await
            });
        }

        let desc = desc
            .as_ref()
            .map(|s| quote! {::std::option::Option::Some(#s)})
            .unwrap_or_else(|| quote! {::std::option::Option::None});
        let deprecation = deprecation
            .as_ref()
            .map(|s| quote! {::std::option::Option::Some(#s)})
            .unwrap_or_else(|| quote! {::std::option::Option::None});

        let oty = OutputType::parse(&ty)?;
        let ty = match oty {
            OutputType::Value(ty) => ty,
            OutputType::Result(_, ty) => ty,
        };
        let schema_ty = oty.value_type();

        methods.push(quote! {
            #[inline]
            async fn #method_name<'ctx>(&self, #(#decl_params),*) -> #crate_name::Result<#ty> {
                match self {
                    #(#calls,)*
                }
            }
        });

        schema_fields.push(quote! {
            fields.insert(::std::string::ToString::to_string(#name), #crate_name::registry::MetaField {
                name: ::std::string::ToString::to_string(#name),
                description: #desc,
                args: {
                    let mut args = #crate_name::indexmap::IndexMap::new();
                    #(#schema_args)*
                    args
                },
                ty: <#schema_ty as #crate_name::Type>::create_type_info(registry),
                deprecation: #deprecation,
                cache_control: ::std::default::Default::default(),
                external: #external,
                provides: #provides,
                requires: #requires,
            });
        });

        let resolve_obj = quote! {
            self.#method_name(#(#use_params),*)
                .await
                .map_err(|err| err.into_server_error().at(ctx.item.pos))?
        };

        resolvers.push(quote! {
            if ctx.item.node.name.node == #name {
                #(#get_params)*
                let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                return #crate_name::OutputValueType::resolve(&#resolve_obj, &ctx_obj, ctx.item).await.map(::std::option::Option::Some);
            }
        });
    }

    let introspection_type_name = if get_introspection_typename.is_empty() {
        quote! { ::std::unreachable!() }
    } else {
        quote! {
            match self {
            #(#get_introspection_typename),*
            }
        }
    };

    let expanded = quote! {
        #(#type_into_impls)*

        #[allow(clippy::all, clippy::pedantic)]
        impl #generics #ident #generics {
            #(#methods)*
        }

        #[allow(clippy::all, clippy::pedantic)]
        impl #generics #crate_name::Type for #ident #generics {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn introspection_type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                #introspection_type_name
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                registry.create_type::<Self, _>(|registry| {
                    #(#registry_types)*

                    #crate_name::registry::MetaType::Interface {
                        name: ::std::string::ToString::to_string(#gql_typename),
                        description: #desc,
                        fields: {
                            let mut fields = #crate_name::indexmap::IndexMap::new();
                            #(#schema_fields)*
                            fields
                        },
                        possible_types: {
                            let mut possible_types = #crate_name::indexmap::IndexSet::new();
                            #(#possible_types)*
                            possible_types
                        },
                        extends: #extends,
                        keys: ::std::option::Option::None,
                    }
                })
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::resolver_utils::ContainerType for #ident #generics {
            async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                #(#resolvers)*
                ::std::result::Result::Ok(::std::option::Option::None)
            }

            fn collect_all_fields<'__life>(&'__life self, ctx: &#crate_name::ContextSelectionSet<'__life>, fields: &mut #crate_name::resolver_utils::Fields<'__life>) -> #crate_name::ServerResult<()> {
                match self {
                    #(#collect_all_fields),*
                }
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::OutputValueType for #ident #generics {
            async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::Value> {
                #crate_name::resolver_utils::resolve_container(ctx, self).await
            }
        }

        impl #generics #crate_name::InterfaceType for #ident #generics {}
    };
    Ok(expanded.into())
}
