use std::collections::HashSet;

use darling::ast::{Data, Style};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Error, Type, visit_mut::VisitMut};

use crate::{
    args::{
        self, InterfaceField, InterfaceFieldArgument, RenameRuleExt, RenameTarget,
        TypeDirectiveLocation,
    },
    output_type::OutputType,
    utils::{
        GeneratorResult, RemoveLifetime, gen_boxed_trait, gen_deprecation, gen_directive_calls,
        generate_default, get_crate_name, get_rustdoc, visible_fn,
    },
};

pub fn generate(interface_args: &args::Interface) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(interface_args.internal);
    let boxed_trait = gen_boxed_trait(&crate_name);
    let ident = &interface_args.ident;
    let type_params = interface_args.generics.type_params().collect::<Vec<_>>();
    let (impl_generics, ty_generics, where_clause) = interface_args.generics.split_for_impl();
    let s = match &interface_args.data {
        Data::Enum(s) => s,
        _ => {
            return Err(
                Error::new_spanned(ident, "Interface can only be applied to an enum.").into(),
            );
        }
    };
    let extends = interface_args.extends;
    let mut enum_names = Vec::new();
    let mut enum_items = HashSet::new();
    let mut type_into_impls = Vec::new();
    let inaccessible = interface_args.inaccessible;
    let tags = interface_args
        .tags
        .iter()
        .map(|tag| quote!(::std::string::ToString::to_string(#tag)))
        .collect::<Vec<_>>();
    let requires_scopes = interface_args
        .requires_scopes
        .iter()
        .map(|scopes| quote!(::std::string::ToString::to_string(#scopes)))
        .collect::<Vec<_>>();
    let directives =
        gen_directive_calls(&interface_args.directives, TypeDirectiveLocation::Interface);
    let gql_typename = if !interface_args.name_type {
        let name = interface_args
            .name
            .clone()
            .unwrap_or_else(|| RenameTarget::Type.rename(ident.to_string()));
        quote!(::std::borrow::Cow::Borrowed(#name))
    } else {
        quote!(<Self as #crate_name::TypeName>::type_name())
    };

    let desc = get_rustdoc(&interface_args.attrs)?
        .map(|s| quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#s)) })
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
                .into());
            }
            Style::Unit => {
                return Err(
                    Error::new_spanned(enum_name, "Empty variants are not supported").into(),
                );
            }
            Style::Struct => {
                return Err(Error::new_spanned(
                    enum_name,
                    "Variants with named fields are not supported",
                )
                .into());
            }
        };

        if let Type::Path(p) = ty {
            // This validates that the field type wasn't already used
            if !enum_items.insert(p) {
                return Err(
                    Error::new_spanned(ty, "This type already used in another variant").into(),
                );
            }

            let mut assert_ty = p.clone();
            RemoveLifetime.visit_type_path_mut(&mut assert_ty);

            type_into_impls.push(quote! {
                #crate_name::static_assertions_next::assert_impl!(for(#(#type_params),*) #assert_ty: (#crate_name::ObjectType) | (#crate_name::InterfaceType));

                #[allow(clippy::all, clippy::pedantic)]
                impl #impl_generics ::std::convert::From<#p> for #ident #ty_generics #where_clause {
                    fn from(obj: #p) -> Self {
                        #ident::#enum_name(obj)
                    }
                }
            });
            enum_names.push(enum_name);

            registry_types.push(quote! {
                <#p as #crate_name::OutputTypeMarker>::create_type_info(registry);
                registry.add_implements(&<#p as #crate_name::OutputTypeMarker>::type_name(), ::std::convert::AsRef::as_ref(&#gql_typename));
            });

            possible_types.push(quote! {
                possible_types.insert(<#p as #crate_name::OutputTypeMarker>::type_name().into_owned());
            });

            get_introspection_typename.push(quote! {
                #ident::#enum_name(obj) => <#p as #crate_name::OutputTypeMarker>::type_name()
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
            ident,
            "A GraphQL Interface type must define one or more fields.",
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
        visible,
        shareable,
        inaccessible,
        tags,
        override_from,
        directives,
        requires_scopes,
    } in &interface_args.fields
    {
        let (name, method_name) = if let Some(method) = method {
            (name.to_string(), Ident::new_raw(method, Span::call_site()))
        } else {
            let method_name = Ident::new_raw(name, Span::call_site());
            (
                interface_args
                    .rename_fields
                    .rename(name.as_ref(), RenameTarget::Field),
                method_name,
            )
        };
        let mut calls = Vec::new();
        let mut use_params = Vec::new();
        let mut decl_params = Vec::new();
        let mut get_params = Vec::new();
        let mut schema_args = Vec::new();
        let requires = match &requires {
            Some(requires) => {
                quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#requires)) }
            }
            None => quote! { ::std::option::Option::None },
        };
        let provides = match &provides {
            Some(provides) => {
                quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#provides)) }
            }
            None => quote! { ::std::option::Option::None },
        };
        let override_from = match &override_from {
            Some(from) => {
                quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#from)) }
            }
            None => quote! { ::std::option::Option::None },
        };

        decl_params.push(quote! { ctx: &'ctx #crate_name::Context<'ctx> });
        use_params.push(quote! { ctx });

        for (
            i,
            InterfaceFieldArgument {
                name,
                desc,
                ty,
                default,
                default_with,
                visible,
                inaccessible,
                tags,
                secret,
                directives,
                deprecation,
            },
        ) in args.iter().enumerate()
        {
            let ident = Ident::new(&format!("arg{}", i), Span::call_site());
            let name = interface_args
                .rename_args
                .rename(name, RenameTarget::Argument);
            decl_params.push(quote! { #ident: #ty });
            use_params.push(quote! { #ident });

            let default = generate_default(default, default_with)?;
            let get_default = match &default {
                Some(default) => quote! { ::std::option::Option::Some(|| -> #ty { #default }) },
                None => quote! { ::std::option::Option::None },
            };
            get_params.push(quote! {
                let (_, #ident) = ctx.param_value::<#ty>(#name, #get_default)?;
            });

            let desc = desc
                .as_ref()
                .map(|s| quote! {::std::option::Option::Some(::std::string::ToString::to_string(#s))})
                .unwrap_or_else(|| quote! {::std::option::Option::None});
            let schema_default = default
                .as_ref()
                .map(|value| {
                    quote! {
                        ::std::option::Option::Some(::std::string::ToString::to_string(
                            &<#ty as #crate_name::InputType>::to_value(&#value)
                        ))
                    }
                })
                .unwrap_or_else(|| quote! {::std::option::Option::None});
            let visible = visible_fn(visible);
            let tags = tags
                .iter()
                .map(|tag| quote!(::std::string::ToString::to_string(#tag)))
                .collect::<Vec<_>>();
            let directives =
                gen_directive_calls(directives, TypeDirectiveLocation::ArgumentDefinition);
            let deprecation = gen_deprecation(deprecation, &crate_name);

            schema_args.push(quote! {
                    args.insert(::std::borrow::ToOwned::to_owned(#name), #crate_name::registry::MetaInputValue {
                        name: ::std::string::ToString::to_string(#name),
                        description: #desc,
                        ty: <#ty as #crate_name::InputType>::create_type_info(registry),
                        deprecation: #deprecation,
                        default_value: #schema_default,
                        visible: #visible,
                        inaccessible: #inaccessible,
                        tags: ::std::vec![ #(#tags),* ],
                        is_secret: #secret,
                        directive_invocations: ::std::vec![ #(#directives),* ],
                    });
                });
        }

        for enum_name in &enum_names {
            calls.push(quote! {
                #ident::#enum_name(obj) => obj.#method_name(#(#use_params),*)
                    .await.map_err(|err| ::std::convert::Into::<#crate_name::Error>::into(err))
                    .map(::std::convert::Into::into)
            });
        }

        let desc = desc
            .as_ref()
            .map(|s| quote! {::std::option::Option::Some(::std::string::ToString::to_string(#s))})
            .unwrap_or_else(|| quote! {::std::option::Option::None});
        let deprecation = gen_deprecation(deprecation, &crate_name);

        let oty = OutputType::parse(ty)?;
        let ty = match oty {
            OutputType::Value(ty) => ty,
            OutputType::Result(ty) => ty,
        };
        let schema_ty = oty.value_type();

        methods.push(quote! {
            #[inline]
            pub async fn #method_name<'ctx>(&self, #(#decl_params),*) -> #crate_name::Result<#ty> {
                match self {
                    #(#calls,)*
                }
            }
        });

        let visible = visible_fn(visible);
        let tags = tags
            .iter()
            .map(|tag| quote!(::std::string::ToString::to_string(#tag)))
            .collect::<Vec<_>>();
        let requires_scopes = requires_scopes
            .iter()
            .map(|scopes| quote!(::std::string::ToString::to_string(#scopes)))
            .collect::<Vec<_>>();
        let directives = gen_directive_calls(directives, TypeDirectiveLocation::FieldDefinition);

        schema_fields.push(quote! {
            fields.insert(::std::string::ToString::to_string(#name), #crate_name::registry::MetaField {
                name: ::std::string::ToString::to_string(#name),
                description: #desc,
                args: {
                    let mut args = #crate_name::indexmap::IndexMap::new();
                    #(#schema_args)*
                    args
                },
                ty: <#schema_ty as #crate_name::OutputTypeMarker>::create_type_info(registry),
                deprecation: #deprecation,
                cache_control: ::std::default::Default::default(),
                external: #external,
                provides: #provides,
                requires: #requires,
                shareable: #shareable,
                inaccessible: #inaccessible,
                tags: ::std::vec![ #(#tags),* ],
                override_from: #override_from,
                visible: #visible,
                compute_complexity: ::std::option::Option::None,
                directive_invocations: ::std::vec![ #(#directives),* ],
                requires_scopes: ::std::vec![ #(#requires_scopes),* ],
            });
        });

        let resolve_obj = quote! {
            self.#method_name(#(#use_params),*)
                .await
                .map_err(|err| ::std::convert::Into::<#crate_name::Error>::into(err).into_server_error(ctx.item.pos))?
        };

        resolvers.push(quote! {
            if ctx.item.node.name.node == #name {
                #(#get_params)*
                let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                return #crate_name::OutputType::resolve(&#resolve_obj, &ctx_obj, ctx.item).await.map(::std::option::Option::Some);
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

    let visible = visible_fn(&interface_args.visible);
    let expanded = quote! {
        #(#type_into_impls)*

        #[allow(clippy::all, clippy::pedantic)]
        impl #impl_generics #ident #ty_generics #where_clause {
            #(#methods)*
        }

        #[allow(clippy::all, clippy::pedantic)]
        #boxed_trait
        impl #impl_generics #crate_name::resolver_utils::ContainerType for #ident #ty_generics #where_clause {
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

         impl #impl_generics #crate_name::OutputTypeMarker for #ident #ty_generics #where_clause {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                #gql_typename
            }

            fn introspection_type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                #introspection_type_name
            }
             fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                registry.create_output_type::<Self, _>(#crate_name::registry::MetaTypeId::Interface, |registry| {
                    #(#registry_types)*

                    #crate_name::registry::MetaType::Interface {
                        name: ::std::borrow::Cow::into_owned(#gql_typename),
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
        #boxed_trait
        impl #impl_generics #crate_name::OutputType for #ident #ty_generics #where_clause {
            #[cfg(feature = "boxed-trait")]
            async fn resolve(
                &self,
                ctx: &#crate_name::ContextSelectionSet<'_>,
                _field: &#crate_name::Positioned<#crate_name::parser::types::Field>,
            ) -> #crate_name::ServerResult<#crate_name::Value> {
                #crate_name::resolver_utils::resolve_container(ctx, self, self).await
            }

            #[cfg(not(feature = "boxed-trait"))]
            async fn resolve(
                &self,
                ctx: &#crate_name::ContextSelectionSet<'_>,
                _field: &#crate_name::Positioned<#crate_name::parser::types::Field>,
            ) -> #crate_name::ServerResult<#crate_name::Value> {
                #crate_name::resolver_utils::resolve_container(ctx, self).await
            }

        }

        impl #impl_generics #crate_name::InterfaceType for #ident #ty_generics #where_clause {}
    };
    Ok(expanded.into())
}
