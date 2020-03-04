use crate::args;
use crate::utils::{build_value_repr, get_crate_name};
use inflector::Inflector;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Error, Ident, Result};

pub fn generate(object_args: &args::Object, input: &DeriveInput) -> Result<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let vis = &input.vis;
    let ident = &input.ident;
    let generics = &input.generics;
    match &input.data {
        Data::Struct(_) => {}
        _ => return Err(Error::new_spanned(input, "It should be a struct.")),
    };

    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| ident.to_string());
    let desc = object_args
        .desc
        .as_ref()
        .map(|s| quote! {Some(#s)})
        .unwrap_or_else(|| quote! {None});
    let trait_ident = Ident::new(&format!("{}Fields", ident.to_string()), Span::call_site());
    let mut trait_fns = Vec::new();
    let mut resolvers = Vec::new();
    let mut schema_fields = Vec::new();

    for field in &object_args.fields {
        let ty = &field.ty;
        let field_name = &field.name;
        let desc = field
            .desc
            .as_ref()
            .map(|s| quote! {Some(#s)})
            .unwrap_or_else(|| quote! {None});
        let deprecation = field
            .deprecation
            .as_ref()
            .map(|s| quote! { Some(#s) })
            .unwrap_or_else(|| quote! {None});

        let mut decl_params = Vec::new();
        let mut get_params = Vec::new();
        let mut use_params = Vec::new();
        let mut schema_args = Vec::new();

        for arg in &field.arguments {
            let name = Ident::new(&arg.name, Span::call_site());
            let ty = &arg.ty;
            let name_str = name.to_string();
            let snake_case_name = Ident::new(&name.to_string().to_snake_case(), ident.span());
            let desc = arg
                .desc
                .as_ref()
                .map(|s| quote! { Some(#s) })
                .unwrap_or_else(|| quote! {None});
            let schema_default = arg
                .default
                .as_ref()
                .map(|v| {
                    let s = v.to_string();
                    quote! {Some(#s)}
                })
                .unwrap_or_else(|| quote! {None});

            decl_params.push(quote! { #snake_case_name: #ty });

            let default = match &arg.default {
                Some(default) => {
                    let repr = build_value_repr(&crate_name, &default);
                    quote! {Some(|| #repr)}
                }
                None => quote! { None },
            };
            get_params.push(quote! {
                let #snake_case_name: #ty = ctx_field.param_value(#name_str, #default)?;
            });

            use_params.push(quote! { #snake_case_name });

            schema_args.push(quote! {
                #crate_name::registry::InputValue {
                    name: #name_str,
                    description: #desc,
                    ty: <#ty as #crate_name::GQLType>::create_type_info(registry),
                    default_value: #schema_default,
                }
            });
        }

        let resolver = Ident::new(
            &field
                .resolver
                .as_ref()
                .unwrap_or(&field.name.to_snake_case()),
            Span::call_site(),
        );
        if field.is_owned {
            trait_fns.push(quote! {
                    async fn #resolver(&self, ctx: &#crate_name::Context<'_>, #(#decl_params),*) -> #crate_name::Result<#ty>;
                });
        } else {
            trait_fns.push(quote! {
                    async fn #resolver<'a>(&'a self, ctx: &#crate_name::Context<'_>, #(#decl_params),*) -> #crate_name::Result<&'a #ty>;
                });
        }

        resolvers.push(quote! {
            if field.name.as_str() == #field_name {
                #(#get_params)*
                let obj = #trait_ident::#resolver(self, &ctx_field, #(#use_params),*).await.
                    map_err(|err| err.with_position(field.position))?;
                let ctx_obj = ctx_field.with_item(&field.selection_set);
                let value = obj.resolve(&ctx_obj).await.
                    map_err(|err| err.with_position(field.position))?;
                let name = field.alias.clone().unwrap_or_else(|| field.name.clone());
                result.insert(name, value.into());
                continue;
            }
        });

        schema_fields.push(quote! {
            #crate_name::registry::Field {
                name: #field_name,
                description: #desc,
                args: vec![#(#schema_args),*],
                ty: <#ty as #crate_name::GQLType>::create_type_info(registry),
                deprecation: #deprecation,
            }
        });
    }

    let expanded = quote! {
        #input

        #[#crate_name::async_trait::async_trait]
        #vis trait #trait_ident {
            #(#trait_fns)*
        }

        impl#generics #crate_name::GQLType for #ident#generics {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> String {
                registry.create_type::<Self, _>(|registry| #crate_name::registry::Type::Object {
                    name: #gql_typename,
                    description: #desc,
                    fields: vec![#(#schema_fields),*]
                })
            }
        }

        #[#crate_name::async_trait::async_trait]
        impl#generics #crate_name::GQLOutputValue for #ident#generics {
            async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>) -> #crate_name::Result<#crate_name::serde_json::Value> {
                use #crate_name::ErrorWithPosition;

                if ctx.items.is_empty() {
                    #crate_name::anyhow::bail!(#crate_name::QueryError::MustHaveSubFields {
                        object: #gql_typename,
                    }.with_position(ctx.span.0));
                }

                let mut result = #crate_name::serde_json::Map::<String, #crate_name::serde_json::Value>::new();
                for selection in &ctx.items {
                    match selection {
                        #crate_name::graphql_parser::query::Selection::Field(field) => {
                            let ctx_field = ctx.with_item(field);
                            if ctx_field.is_skip_this()? {
                                continue;
                            }
                            if field.name.as_str() == "__typename" {
                                let name = field.alias.clone().unwrap_or_else(|| field.name.clone());
                                result.insert(name, #gql_typename.into());
                                continue;
                            }
                            if field.name.as_str() == "__schema" {
                                continue;
                            }
                            #(#resolvers)*
                            #crate_name::anyhow::bail!(#crate_name::QueryError::FieldNotFound {
                                field_name: field.name.clone(),
                                object: #gql_typename,
                            }.with_position(field.position));
                        }
                        _ => {}
                    }
                }

                Ok(#crate_name::serde_json::Value::Object(result))
            }
        }

        impl#generics #crate_name::GQLObject for #ident#generics {}
    };
    Ok(expanded.into())
}
