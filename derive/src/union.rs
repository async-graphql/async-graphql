use core::panic;
use std::collections::HashSet;

use darling::ast::{Data, Style};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, LifetimeParam, Type, visit::Visit, visit_mut::VisitMut};

use crate::{
    args::{self, RenameTarget},
    utils::{
        GeneratorResult, RemoveLifetime, gen_boxed_trait, get_crate_name, get_rustdoc, visible_fn,
    },
};

pub fn generate(union_args: &args::Union) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(union_args.internal);
    let boxed_trait = gen_boxed_trait(&crate_name);
    let ident = &union_args.ident;
    let type_params = union_args.generics.type_params().collect::<Vec<_>>();
    let (impl_generics, ty_generics, where_clause) = union_args.generics.split_for_impl();
    let s = match &union_args.data {
        Data::Enum(s) => s,
        _ => return Err(Error::new_spanned(ident, "Union can only be applied to an enum.").into()),
    };
    let mut enum_names = Vec::new();
    let mut enum_items = HashSet::new();
    let mut type_into_impls = Vec::new();
    let gql_typename = if !union_args.name_type {
        let name = union_args
            .name
            .clone()
            .unwrap_or_else(|| RenameTarget::Type.rename(ident.to_string()));
        quote!(::std::borrow::Cow::Borrowed(#name))
    } else {
        quote!(<Self as #crate_name::TypeName>::type_name())
    };

    let inaccessible = union_args.inaccessible;
    let tags = union_args
        .tags
        .iter()
        .map(|tag| quote!(::std::string::ToString::to_string(#tag)))
        .collect::<Vec<_>>();
    let desc = get_rustdoc(&union_args.attrs)?
        .map(|s| quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#s)) })
        .unwrap_or_else(|| quote! {::std::option::Option::None});

    let mut lazy_types = Vec::new();

    #[derive(Clone)]
    struct LazyType {
        ty: syn::Type,
        enum_name: syn::Ident,
        flatten: bool,
    }

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

        let mut ty = ty;
        while let Type::Group(group) = ty {
            ty = &*group.elem;
        }

        if matches!(ty, Type::Path(_) | Type::Macro(_)) {
            // This validates that the field type wasn't already used
            if !enum_items.insert(ty) {
                return Err(
                    Error::new_spanned(ty, "This type is already used in another variant").into(),
                );
            }

            enum_names.push(enum_name);

            let mut assert_ty = ty.clone();
            RemoveLifetime.visit_type_mut(&mut assert_ty);

            if !variant.flatten {
                type_into_impls.push(quote! {
                    #crate_name::static_assertions_next::assert_impl!(for(#(#type_params),*) #assert_ty: #crate_name::ObjectType);

                    #[allow(clippy::all, clippy::pedantic)]
                    impl #impl_generics ::std::convert::From<#ty> for #ident #ty_generics #where_clause {
                        fn from(obj: #ty) -> Self {
                            #ident::#enum_name(obj)
                        }
                    }
                });
            } else {
                type_into_impls.push(quote! {
                    #crate_name::static_assertions_next::assert_impl!(for(#(#type_params),*) #assert_ty: #crate_name::UnionType);

                    #[allow(clippy::all, clippy::pedantic)]
                    impl #impl_generics ::std::convert::From<#ty> for #ident #ty_generics #where_clause {
                        fn from(obj: #ty) -> Self {
                            #ident::#enum_name(obj)
                        }
                    }
                });
            }

            lazy_types.push(LazyType {
                ty: ty.clone(),
                enum_name: enum_name.clone(),
                flatten: variant.flatten,
            });

            collect_all_fields.push(quote! {
                #ident::#enum_name(obj) => obj.collect_all_fields(ctx, fields)
            });
        } else {
            return Err(Error::new_spanned(ty, "Invalid type").into());
        }
    }

    if lazy_types.is_empty() {
        return Err(Error::new_spanned(
            ident,
            "A GraphQL Union type must include one or more unique member types.",
        )
        .into());
    }

    let visible = visible_fn(&union_args.visible);

    let get_introspection_typename = |lazy_types: Vec<LazyType>| {
        lazy_types.into_iter().map(|lazy| {
            let ty = lazy.ty;
            let enum_name = &lazy.enum_name;
            if !lazy.flatten {
                quote! {
                    #ident::#enum_name(obj) => <#ty as #crate_name::OutputTypeMarker>::type_name()
                }
            } else {
                quote! {
                    #ident::#enum_name(obj) => <#ty as #crate_name::OutputTypeMarker>::introspection_type_name(obj)
                }
            }
        })
    };

    let registry_types = |lazy_types: Vec<LazyType>| {
        lazy_types.into_iter().filter_map(|lazy| {
            let ty = lazy.ty;
            if !lazy.flatten {
                Some(quote! {
                    <#ty as #crate_name::OutputTypeMarker>::create_type_info(registry);
                })
            } else {
                None
            }
        })
    };

    let possible_types = |lazy_types: Vec<LazyType>| {
        lazy_types.into_iter().map(|lazy| {
        let ty = lazy.ty;
        if !lazy.flatten {
            quote! {
                possible_types.insert(<#ty as #crate_name::OutputTypeMarker>::type_name().into_owned());
            }
        } else {
            quote! {
                if let #crate_name::registry::MetaType::Union { possible_types: possible_types2, .. } =
                    registry.create_fake_output_type::<#ty>() {
                    possible_types.extend(possible_types2);
                }
            }
        }
    })
    };

    let expanded = if union_args.concretes.is_empty() {
        let get_introspection_typename = get_introspection_typename(lazy_types.clone());
        let registry_types = registry_types(lazy_types.clone());
        let possible_types = possible_types(lazy_types.clone());

        quote! {
            #(#type_into_impls)*

            #[allow(clippy::all, clippy::pedantic)]
            #boxed_trait
            impl #impl_generics #crate_name::resolver_utils::ContainerType for #ident #ty_generics #where_clause {
                async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                    ::std::result::Result::Ok(::std::option::Option::None)
                }

                fn collect_all_fields<'__life>(&'__life self, ctx: &#crate_name::ContextSelectionSet<'__life>, fields: &mut #crate_name::resolver_utils::Fields<'__life>) -> #crate_name::ServerResult<()> {
                    match self {
                        #(#collect_all_fields),*
                    }
                }
            }

            #[allow(clippy::all, clippy::pedantic)]
            impl #impl_generics #crate_name::OutputTypeMarker for #ident #ty_generics #where_clause {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    #gql_typename
                }

                fn introspection_type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    match self {
                        #(#get_introspection_typename),*
                    }
                }


                fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                    registry.create_output_type::<Self, _>(#crate_name::registry::MetaTypeId::Union, |registry| {
                        #(#registry_types)*

                        #crate_name::registry::MetaType::Union {
                            name: ::std::borrow::Cow::into_owned(#gql_typename),
                            description: #desc,
                            possible_types: {
                                let mut possible_types = #crate_name::indexmap::IndexSet::new();
                                #(#possible_types)*
                                possible_types
                            },
                            visible: #visible,
                            inaccessible: #inaccessible,
                            tags: ::std::vec![ #(#tags),* ],
                            rust_typename: ::std::option::Option::Some(::std::any::type_name::<Self>()),
                            directive_invocations: ::std::vec::Vec::new(),
                        }
                    })
                }
            }
            
            #[allow(clippy::all, clippy::pedantic)]
            #boxed_trait
            impl #impl_generics #crate_name::OutputType for #ident #ty_generics #where_clause {
                #[cfg(feature = "boxed-trait")]
                async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::Value> {
                    #crate_name::resolver_utils::resolve_container(ctx, self as &dyn #crate_name::ContainerType, self).await
                }

                #[cfg(not(feature = "boxed-trait"))]
                async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::Value> {
                    #crate_name::resolver_utils::resolve_container(ctx, self).await
                }
            }

            impl #impl_generics #crate_name::UnionType for #ident #ty_generics #where_clause {}
        }
    } else {
        let mut code = Vec::new();

        #[derive(Default)]
        struct GetLifetimes<'a> {
            lifetimes: Vec<&'a LifetimeParam>,
        }

        impl<'a> Visit<'a> for GetLifetimes<'a> {
            fn visit_lifetime_param(&mut self, i: &'a LifetimeParam) {
                self.lifetimes.push(i);
            }
        }

        let mut visitor = GetLifetimes::default();
        visitor.visit_generics(&union_args.generics);
        let lifetimes = visitor.lifetimes;

        let type_lifetimes = if !lifetimes.is_empty() {
            Some(quote!(#(#lifetimes,)*))
        } else {
            None
        };

        for concrete in &union_args.concretes {
            let gql_typename = &concrete.name;
            let params = &concrete.params.0;
            let concrete_type = quote! { #ident<#type_lifetimes #(#params),*> };

            let def_bounds = if !lifetimes.is_empty() || !concrete.bounds.0.is_empty() {
                let bounds = lifetimes
                    .iter()
                    .map(|l| quote!(#l))
                    .chain(concrete.bounds.0.iter().map(|b| quote!(#b)));
                Some(quote!(<#(#bounds),*>))
            } else {
                None
            };

            let lazy_types = lazy_types
                .clone()
                .into_iter()
                .map(|mut l| {
                    match &mut l.ty {
                        syn::Type::Path(p) => {
                            let last_segment = p.path.segments.last_mut().unwrap();

                            match last_segment.arguments {
                                syn::PathArguments::None => {
                                    if let Some(idx) = type_params
                                        .iter()
                                        .position(|p| p.ident == last_segment.ident)
                                    {
                                        let param = &params[idx];
                                        l.ty = syn::parse2::<syn::Type>(quote!(#param)).unwrap();
                                    }
                                }
                                syn::PathArguments::AngleBracketed(ref mut inner) => {
                                    for arg in &mut inner.args {
                                        let ty = match arg {
                                            syn::GenericArgument::Type(t) => t,
                                            syn::GenericArgument::AssocType(a) => &mut a.ty,
                                            _ => continue,
                                        };

                                        // Check if the type is a generic parameter which we should
                                        // convert to a concrete type
                                        if let syn::Type::Path(ty_path) = ty {
                                            if let Some(idx) = type_params.iter().position(|p| {
                                                p.ident == ty_path.path.segments[0].ident
                                            }) {
                                                let param = &params[idx];
                                                *ty = syn::parse2::<syn::Type>(quote!(#param))
                                                    .unwrap();
                                            }
                                        }
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }
                        syn::Type::Macro(_) => {
                            panic!("Macro types with generics are not supported yet")
                        }
                        _ => unreachable!(),
                    };

                    l
                })
                .collect::<Vec<_>>();

            let get_introspection_typename = get_introspection_typename(lazy_types.clone());
            let registry_types = registry_types(lazy_types.clone());
            let possible_types = possible_types(lazy_types.clone());

            let expanded = quote! {
                #[allow(clippy::all, clippy::pedantic)]
                #boxed_trait
                impl #def_bounds #crate_name::resolver_utils::ContainerType for #concrete_type {
                    async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                        ::std::result::Result::Ok(::std::option::Option::None)
                    }

                    fn collect_all_fields<'__life>(&'__life self, ctx: &#crate_name::ContextSelectionSet<'__life>, fields: &mut #crate_name::resolver_utils::Fields<'__life>) -> #crate_name::ServerResult<()> {
                        match self {
                            #(#collect_all_fields),*
                        }
                    }
                }

                #[allow(clippy::all, clippy::pedantic)]
                impl #def_bounds #crate_name::OutputTypeMarker for #concrete_type {
                    fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                        ::std::borrow::Cow::Borrowed(#gql_typename)
                    }

                    fn introspection_type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                        match self {
                            #(#get_introspection_typename),*
                        }
                    }

                    fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                        registry.create_output_type::<Self, _>(#crate_name::registry::MetaTypeId::Union, |registry| {
                            #(#registry_types)*

                            #crate_name::registry::MetaType::Union {
                                name: ::std::borrow::ToOwned::to_owned(#gql_typename),
                                description: #desc,
                                possible_types: {
                                    let mut possible_types = #crate_name::indexmap::IndexSet::new();
                                    #(#possible_types)*
                                    possible_types
                                },
                                visible: #visible,
                                inaccessible: #inaccessible,
                                tags: ::std::vec![ #(#tags),* ],
                                rust_typename: ::std::option::Option::Some(::std::any::type_name::<Self>()),
                                directive_invocations: ::std::vec::Vec::new()
                            }
                        })
                    }
                }

                #[allow(clippy::all, clippy::pedantic)]
                #boxed_trait
                impl #def_bounds #crate_name::OutputType for #concrete_type {
                    #[cfg(feature = "boxed-trait")]
                    async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::Value> {
                        #crate_name::resolver_utils::resolve_container(ctx, &self as &dyn #crate_name::resolver_utils::ContainerType, self).await
                    }

                    #[cfg(not(feature = "boxed-trait"))]
                    async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::Value> {
                        #crate_name::resolver_utils::resolve_container(ctx, self).await
                    }

                }

                impl #def_bounds #crate_name::ObjectType for #concrete_type {}
            };
            code.push(expanded);
        }

        quote!(#(#code)*)
    };

    Ok(expanded.into())
}
