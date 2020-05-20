use crate::args;
use crate::utils::{check_reserved_name, get_crate_name, get_rustdoc};
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::{Data, DeriveInput, Error, Fields, Result, Type};

pub fn generate(union_args: &args::Interface, input: &DeriveInput) -> Result<TokenStream> {
    let crate_name = get_crate_name(union_args.internal);
    let ident = &input.ident;
    let generics = &input.generics;
    let s = match &input.data {
        Data::Enum(s) => s,
        _ => {
            return Err(Error::new_spanned(
                input,
                "Unions can only be applied to an enum.",
            ))
        }
    };
    let mut enum_names = Vec::new();
    let mut enum_items = HashSet::new();
    let mut type_into_impls = Vec::new();
    let gql_typename = union_args.name.clone().unwrap_or_else(|| ident.to_string());
    check_reserved_name(&gql_typename, union_args.internal)?;

    let desc = union_args
        .desc
        .clone()
        .or_else(|| get_rustdoc(&input.attrs).ok().flatten())
        .map(|s| quote! { Some(#s) })
        .unwrap_or_else(|| quote! {None});

    let mut registry_types = Vec::new();
    let mut possible_types = Vec::new();
    let mut collect_inline_fields = Vec::new();
    let mut get_introspection_typename = Vec::new();

    for variant in s.variants.iter() {
        let enum_name = &variant.ident;
        let field = match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => fields.unnamed.first().unwrap(),
            Fields::Unnamed(_) => {
                return Err(Error::new_spanned(
                    variant,
                    "Only single value variants are supported",
                ))
            }
            Fields::Unit => {
                return Err(Error::new_spanned(
                    variant,
                    "Empty variants are not supported",
                ))
            }
            Fields::Named(_) => {
                return Err(Error::new_spanned(
                    variant,
                    "Variants with named fields are not supported",
                ))
            }
        };
        if let Type::Path(p) = &field.ty {
            // This validates that the field type wasn't already used
            if !enum_items.insert(p) {
                return Err(Error::new_spanned(
                    field,
                    "This type already used in another variant",
                ));
            }

            enum_names.push(enum_name);
            type_into_impls.push(quote! {
                impl #generics From<#p> for #ident #generics {
                    fn from(obj: #p) -> Self {
                        #ident::#enum_name(obj)
                    }
                }
            });
            registry_types.push(quote! {
                <#p as #crate_name::Type>::create_type_info(registry);
            });
            possible_types.push(quote! {
                possible_types.insert(<#p as #crate_name::Type>::type_name().to_string());
            });
            collect_inline_fields.push(quote! {
                if let #ident::#enum_name(obj) = self {
                    return obj.collect_inline_fields(name, ctx, futures);
                }
            });
            get_introspection_typename.push(quote! {
                #ident::#enum_name(obj) => <#p as #crate_name::Type>::type_name()
            })
        } else {
            return Err(Error::new_spanned(field, "Invalid type"));
        }
    }

    let expanded = quote! {
        #input

        #(#type_into_impls)*

        impl #generics #crate_name::Type for #ident #generics {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn introspection_type_name(&self) -> std::borrow::Cow<'static, str> {
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

        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::ObjectType for #ident #generics {
            async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::Result<#crate_name::serde_json::Value> {
                Err(#crate_name::QueryError::FieldNotFound {
                    field_name: ctx.name.to_string(),
                    object: #gql_typename.to_string(),
                }.into_error(ctx.position()))
            }

            fn collect_inline_fields<'a>(
                &'a self,
                name: &str,
                ctx: &#crate_name::ContextSelectionSet<'a>,
                futures: &mut Vec<#crate_name::BoxFieldFuture<'a>>,
            ) -> #crate_name::Result<()> {
                #(#collect_inline_fields)*
                Ok(())
            }
        }

        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::OutputValueType for #ident #generics {
            async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::query::Field>) -> #crate_name::Result<#crate_name::serde_json::Value> {
                #crate_name::do_resolve(ctx, self).await
            }
        }
    };
    Ok(expanded.into())
}
