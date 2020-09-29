use crate::args;
use crate::utils::{get_crate_name, get_rustdoc, GeneratorResult};
use darling::ast::{Data, Style};
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::{Error, Type};

pub fn generate(union_args: &args::Union) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(union_args.internal);
    let ident = &union_args.ident;
    let generics = &union_args.generics;
    let s = match &union_args.data {
        Data::Enum(s) => s,
        _ => {
            return Err(Error::new_spanned(&ident, "Union can only be applied to an enum.").into())
        }
    };
    let mut enum_names = Vec::new();
    let mut enum_items = HashSet::new();
    let mut type_into_impls = Vec::new();
    let gql_typename = union_args.name.clone().unwrap_or_else(|| ident.to_string());

    let desc = get_rustdoc(&union_args.attrs)?
        .map(|s| quote! { Some(#s) })
        .unwrap_or_else(|| quote! {None});

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

        if let Type::Path(p) = &ty {
            // This validates that the field type wasn't already used
            if !enum_items.insert(p) {
                return Err(
                    Error::new_spanned(&ty, "This type already used in another variant").into(),
                );
            }

            enum_names.push(enum_name);

            if !variant.flatten {
                type_into_impls.push(quote! {
                    #crate_name::static_assertions::assert_impl_one!(#p: #crate_name::ObjectType);

                    #[allow(clippy::all, clippy::pedantic)]
                    impl #generics ::std::convert::From<#p> for #ident #generics {
                        fn from(obj: #p) -> Self {
                            #ident::#enum_name(obj)
                        }
                    }
                });
            } else {
                type_into_impls.push(quote! {
                    #crate_name::static_assertions::assert_impl_one!(#p: #crate_name::UnionType);

                    #[allow(clippy::all, clippy::pedantic)]
                    impl #generics ::std::convert::From<#p> for #ident #generics {
                        fn from(obj: #p) -> Self {
                            #ident::#enum_name(obj)
                        }
                    }
                });
            }

            registry_types.push(quote! {
                <#p as #crate_name::Type>::create_type_info(registry);
            });

            if !variant.flatten {
                possible_types.push(quote! {
                    possible_types.insert(<#p as #crate_name::Type>::type_name().to_string());
                });
            } else {
                possible_types.push(quote! {
                    if let Some(#crate_name::registry::MetaType::Union { possible_types: possible_types2, .. }) =
                        registry.types.remove(&*<#p as #crate_name::Type>::type_name()) {
                        possible_types.extend(possible_types2);
                    }
                });
            }

            if !variant.flatten {
                get_introspection_typename.push(quote! {
                    #ident::#enum_name(obj) => <#p as #crate_name::Type>::type_name()
                });
            } else {
                get_introspection_typename.push(quote! {
                    #ident::#enum_name(obj) => <#p as #crate_name::Type>::introspection_type_name(obj)
                });
            }

            collect_all_fields.push(quote! {
                #ident::#enum_name(obj) => obj.collect_all_fields(ctx, fields)
            });
        } else {
            return Err(Error::new_spanned(ty, "Invalid type").into());
        }
    }

    let expanded = quote! {
        #(#type_into_impls)*

        #[allow(clippy::all, clippy::pedantic)]
        impl #generics #crate_name::Type for #ident #generics {
            fn type_name() -> ::std::borrow::Cow<'static, str> {
               ::std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn introspection_type_name(&self) -> ::std::borrow::Cow<'static, str> {
                match self {
                    #(#get_introspection_typename),*
                }
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> String {
                registry.create_type::<Self, _>(|registry| {
                    #(#registry_types)*

                    #crate_name::registry::MetaType::Union {
                        name: #gql_typename.to_string(),
                        description: #desc,
                        possible_types: {
                            let mut possible_types = #crate_name::indexmap::IndexSet::new();
                            #(#possible_types)*
                            possible_types
                        }
                    }
                })
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::resolver_utils::ContainerType for #ident #generics {
            async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::Result<#crate_name::serde_json::Value> {
                Err(#crate_name::QueryError::FieldNotFound {
                    field_name: ctx.item.node.name.to_string(),
                    object: #gql_typename.to_string(),
                }.into_error(ctx.item.pos))
            }

            fn collect_all_fields<'a>(&'a self, ctx: &#crate_name::ContextSelectionSet<'a>, fields: &mut #crate_name::resolver_utils::Fields<'a>) -> #crate_name::Result<()> {
                match self {
                    #(#collect_all_fields),*
                }
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::OutputValueType for #ident #generics {
            async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::Result<#crate_name::serde_json::Value> {
                #crate_name::resolver_utils::resolve_container(ctx, self).await
            }
        }

        impl #generics #crate_name::UnionType for #ident #generics {}
    };
    Ok(expanded.into())
}
