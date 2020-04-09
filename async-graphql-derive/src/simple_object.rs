use crate::args;
use crate::utils::{check_reserved_name, get_crate_name};
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Result};

pub fn generate(object_args: &args::Object, input: &mut DeriveInput) -> Result<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let ident = &input.ident;
    let generics = &input.generics;
    let extends = object_args.extends;
    let s = match &mut input.data {
        Data::Struct(e) => e,
        _ => return Err(Error::new_spanned(input, "It should be a struct")),
    };
    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| ident.to_string());
    check_reserved_name(&gql_typename, object_args.internal)?;

    let desc = object_args
        .desc
        .as_ref()
        .map(|s| quote! { Some(#s) })
        .unwrap_or_else(|| quote! {None});

    let mut resolvers = Vec::new();
    let mut schema_fields = Vec::new();
    let fields = match &mut s.fields {
        Fields::Named(fields) => fields,
        _ => return Err(Error::new_spanned(input, "All fields must be named.")),
    };

    for item in &mut fields.named {
        if let Some(field) = args::Field::parse(&item.attrs)? {
            let field_name = field
                .name
                .clone()
                .unwrap_or_else(|| item.ident.as_ref().unwrap().to_string().to_camel_case());
            let field_desc = field
                .desc
                .as_ref()
                .map(|s| quote! {Some(#s)})
                .unwrap_or_else(|| quote! {None});
            let field_deprecation = field
                .deprecation
                .as_ref()
                .map(|s| quote! {Some(#s)})
                .unwrap_or_else(|| quote! {None});
            let external = field.external;
            let requires = match &field.requires {
                Some(requires) => quote! { Some(#requires) },
                None => quote! { None },
            };
            let provides = match &field.provides {
                Some(provides) => quote! { Some(#provides) },
                None => quote! { None },
            };
            let ty = &item.ty;

            let cache_control = {
                let public = field.cache_control.public;
                let max_age = field.cache_control.max_age;
                quote! {
                    #crate_name::CacheControl {
                        public: #public,
                        max_age: #max_age,
                    }
                }
            };

            schema_fields.push(quote! {
                fields.insert(#field_name.to_string(), #crate_name::registry::Field {
                    name: #field_name.to_string(),
                    description: #field_desc,
                    args: Default::default(),
                    ty: <#ty as #crate_name::Type>::create_type_info(registry),
                    deprecation: #field_deprecation,
                    cache_control: #cache_control,
                    external: #external,
                    provides: #provides,
                    requires: #requires,
                });
            });

            let ident = &item.ident;
            resolvers.push(quote! {
                if field.name.as_str() == #field_name {
                    let ctx_obj = ctx.with_selection_set(&field.selection_set);
                    return #crate_name::OutputValueType::resolve(&self.#ident, &ctx_obj, field.position).await;
                }
            });

            item.attrs.remove(
                item.attrs
                    .iter()
                    .enumerate()
                    .find(|(_, a)| a.path.is_ident("field"))
                    .map(|(idx, _)| idx)
                    .unwrap(),
            );
        }
    }

    let cache_control = {
        let public = object_args.cache_control.public;
        let max_age = object_args.cache_control.max_age;
        quote! {
            #crate_name::CacheControl {
                public: #public,
                max_age: #max_age,
            }
        }
    };

    let expanded = quote! {
        #input

        impl #generics #crate_name::Type for #ident {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> String {
                registry.create_type::<Self, _>(|registry| #crate_name::registry::Type::Object {
                    name: #gql_typename.to_string(),
                    description: #desc,
                    fields: {
                        let mut fields = std::collections::HashMap::new();
                        #(#schema_fields)*
                        fields
                    },
                    cache_control: #cache_control,
                    extends: #extends,
                    keys: None,
                })
            }
        }

        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::ObjectType for #ident #generics {
            async fn resolve_field(&self, ctx: &#crate_name::Context<'_>, field: &#crate_name::graphql_parser::query::Field) -> #crate_name::Result<#crate_name::serde_json::Value> {
                #(#resolvers)*

                Err(#crate_name::QueryError::FieldNotFound {
                    field_name: field.name.clone(),
                    object: #gql_typename.to_string(),
                }.into_error(field.position))
            }
        }

        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::OutputValueType for #ident #generics {
            async fn resolve(value: &Self, ctx: &#crate_name::ContextSelectionSet<'_>, _pos: #crate_name::Pos) -> #crate_name::Result<#crate_name::serde_json::Value> {
                #crate_name::do_resolve(ctx, value).await
            }
        }
    };
    Ok(expanded.into())
}
