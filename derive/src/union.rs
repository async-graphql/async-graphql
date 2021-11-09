use darling::ast::{Data, Style};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::collections::HashSet;
use syn::visit_mut::VisitMut;
use syn::{visit_mut, Error, Lifetime, Type};

use crate::args::{self, RenameTarget};
use crate::utils::{get_crate_name, get_rustdoc, visible_fn, GeneratorResult};

pub fn generate(union_args: &args::Union) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(union_args.internal);
    let ident = &union_args.ident;
    let (impl_generics, ty_generics, where_clause) = union_args.generics.split_for_impl();
    let s = match &union_args.data {
        Data::Enum(s) => s,
        _ => {
            return Err(Error::new_spanned(&ident, "Union can only be applied to an enum.").into())
        }
    };
    let mut enum_names = Vec::new();
    let mut enum_items = HashSet::new();
    let mut type_into_impls = Vec::new();
    let gql_typename = union_args
        .name
        .clone()
        .unwrap_or_else(|| RenameTarget::Type.rename(ident.to_string()));

    let desc = get_rustdoc(&union_args.attrs)?
        .map(|s| quote! { ::std::option::Option::Some(#s) })
        .unwrap_or_else(|| quote! {::std::option::Option::None});

    let mut registry_types = Vec::new();
    let mut possible_types = Vec::new();
    let mut union_values = Vec::new();
    let mut get_introspection_typename = Vec::new();
    let mut collect_all_fields = Vec::new();

    for variant in s {
        let enum_name = &variant.ident;
        let union_visible = visible_fn(&variant.visible);
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

        if let Type::Path(p) = &ty {
            // This validates that the field type wasn't already used
            if !enum_items.insert(p) {
                return Err(
                    Error::new_spanned(&ty, "This type already used in another variant").into(),
                );
            }

            enum_names.push(enum_name);
            union_values.push(quote! {
                union_values.insert(
                    <#p as #crate_name::OutputType>::type_name().into_owned(),
                    #crate_name::registry::MetaUnionValue {
                        name: <#p as #crate_name::OutputType>::type_name().into_owned(),
                        visible: #union_visible,
                    }
                );
            });

            struct RemoveLifetime;
            impl VisitMut for RemoveLifetime {
                fn visit_lifetime_mut(&mut self, i: &mut Lifetime) {
                    i.ident = Ident::new("_", Span::call_site());
                    visit_mut::visit_lifetime_mut(self, i);
                }
            }

            let mut assert_ty = p.clone();
            RemoveLifetime.visit_type_path_mut(&mut assert_ty);

            if !variant.flatten {
                type_into_impls.push(quote! {
                    #crate_name::static_assertions::assert_impl_one!(#assert_ty: #crate_name::ObjectType);

                    #[allow(clippy::all, clippy::pedantic)]
                    impl #impl_generics ::std::convert::From<#p> for #ident #ty_generics #where_clause {
                        fn from(obj: #p) -> Self {
                            #ident::#enum_name(obj)
                        }
                    }
                });
            } else {
                type_into_impls.push(quote! {
                    #crate_name::static_assertions::assert_impl_one!(#assert_ty: #crate_name::UnionType);

                    #[allow(clippy::all, clippy::pedantic)]
                    impl #impl_generics ::std::convert::From<#p> for #ident #ty_generics #where_clause {
                        fn from(obj: #p) -> Self {
                            #ident::#enum_name(obj)
                        }
                    }
                });
            }

            if !variant.flatten {
                registry_types.push(quote! {
                    <#p as #crate_name::OutputType>::create_type_info(registry);
                });
                possible_types.push(quote! {
                    possible_types.insert(<#p as #crate_name::OutputType>::type_name().into_owned());
                });
            } else {
                possible_types.push(quote! {
                    if let #crate_name::registry::MetaType::Union { possible_types: possible_types2, .. } =
                        registry.create_fake_output_type::<#p>() {
                        possible_types.extend(possible_types2);
                    }
                });
            }

            if !variant.flatten {
                get_introspection_typename.push(quote! {
                    #ident::#enum_name(obj) => <#p as #crate_name::OutputType>::type_name()
                });
            } else {
                get_introspection_typename.push(quote! {
                    #ident::#enum_name(obj) => <#p as #crate_name::OutputType>::introspection_type_name(obj)
                });
            }

            collect_all_fields.push(quote! {
                #ident::#enum_name(obj) => obj.collect_all_fields(ctx, fields)
            });
        } else {
            return Err(Error::new_spanned(ty, "Invalid type").into());
        }
    }

    if possible_types.is_empty() {
        return Err(Error::new_spanned(
            &ident,
            "A GraphQL Union type must include one or more unique member types.",
        )
        .into());
    }

    let visible = visible_fn(&union_args.visible);
    let expanded = quote! {
        #(#type_into_impls)*

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait]

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
        #[#crate_name::async_trait::async_trait]
        impl #impl_generics #crate_name::OutputType for #ident #ty_generics #where_clause {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
               ::std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn introspection_type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                match self {
                    #(#get_introspection_typename),*
                }
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                registry.create_output_type::<Self, _>(|registry| {
                    #(#registry_types)*

                    #crate_name::registry::MetaType::Union {
                        name: ::std::borrow::ToOwned::to_owned(#gql_typename),
                        description: #desc,
                        possible_types: {
                            let mut possible_types = #crate_name::indexmap::IndexSet::new();
                            #(#possible_types)*
                            possible_types
                        },
                        union_values: {
                            let mut union_values = #crate_name::indexmap::IndexMap::new();
                            #(#union_values)*
                            union_values
                        },
                        visible: #visible,
                        rust_typename: ::std::any::type_name::<Self>(),
                    }
                })
            }

            async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::Value> {
                #crate_name::resolver_utils::resolve_container(ctx, self).await
            }
        }

        impl #impl_generics #crate_name::UnionType for #ident #ty_generics #where_clause {}
    };
    Ok(expanded.into())
}
