use crate::args;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Error, Ident, Result};

pub fn generate(object_args: &args::Object, input: &DeriveInput) -> Result<TokenStream> {
    let attrs = &input.attrs;
    let vis = &input.vis;
    let ident = &input.ident;
    let s = match &input.data {
        Data::Struct(s) => s,
        _ => return Err(Error::new_spanned(input, "It should be a struct.")),
    };

    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| ident.to_string());
    let trait_ident = Ident::new(&format!("{}Fields", ident.to_string()), Span::call_site());
    let mut new_fields = Vec::new();
    let mut trait_fns = Vec::new();
    let mut resolvers = Vec::new();

    for field in &s.fields {
        if let Some(field_args) = args::Field::parse(&field.attrs)? {
            // is field
            let vis = &field.vis;
            let ty = &field.ty;
            let ident = field.ident.as_ref().unwrap();
            let field_name = ident.to_string();

            if field_args.is_attr {
                new_fields.push(quote! { #vis #ident: #ty });
            }

            let mut decl_params = Vec::new();
            let mut get_params = Vec::new();
            let mut use_params = Vec::new();

            for arg in &field_args.arguments {
                let name = Ident::new(&arg.name, Span::call_site());
                let ty = &arg.ty;
                let name_str = name.to_string();

                decl_params.push(quote! { #name: #ty });
                get_params.push(quote! {
                    let #name: #ty = ctx_field.param_value(#name_str)?;
                });
                use_params.push(quote! { #name });
            }

            trait_fns.push(quote! {
                async fn #ident(&self, ctx: &async_graphql::ContextField<'_>, #(#decl_params),*) -> async_graphql::Result<#ty>;
            });

            resolvers.push(quote! {
                if field.name.as_str() == #field_name {
                    #(#get_params)*
                    let obj = #trait_ident::#ident(&self, &ctx_field, #(#use_params),*).await.
                        map_err(|err| err.with_position(field.position))?;
                    let ctx_obj = ctx_field.with_item(&field.selection_set);
                    let value = obj.resolve(&ctx_obj).await.
                        map_err(|err| err.with_position(field.position))?;
                    let name = field.alias.clone().unwrap_or_else(|| field.name.clone());
                    result.insert(name, value.into());
                    continue;
                }
            });
        } else {
            new_fields.push(quote! { #field });
        }
    }

    let expanded = quote! {
        #(#attrs)*
        #vis struct #ident {
            #(#new_fields),*
        }

        #[async_graphql::async_trait::async_trait]
        #vis trait #trait_ident {
            #(#trait_fns)*
        }

        impl async_graphql::GQLType for #ident {
            fn type_name() -> String {
                #gql_typename.to_string()
            }
        }

        #[async_graphql::async_trait::async_trait]
        impl async_graphql::GQLOutputValue for #ident {
            async fn resolve(self, ctx: &async_graphql::ContextSelectionSet<'_>) -> async_graphql::Result<async_graphql::serde_json::Value> {
                use async_graphql::ErrorWithPosition;

                if ctx.items.is_empty() {
                    async_graphql::anyhow::bail!(async_graphql::QueryError::MustHaveSubFields {
                        object: #gql_typename,
                    }.with_position(ctx.span.0));
                }

                let mut result = async_graphql::serde_json::Map::<String, async_graphql::serde_json::Value>::new();
                for selection in &ctx.items {
                    match selection {
                        async_graphql::graphql_parser::query::Selection::Field(field) => {
                            let ctx_field = ctx.with_item(field);
                            if field.name.as_str() == "__typename" {
                                let name = field.alias.clone().unwrap_or_else(|| field.name.clone());
                                result.insert(name, #gql_typename.into());
                                continue;
                            }
                            #(#resolvers)*
                            async_graphql::anyhow::bail!(async_graphql::QueryError::FieldNotFound {
                                field_name: field.name.clone(),
                                object: #gql_typename,
                            }.with_position(field.position));
                        }
                        _ => {}
                    }
                }

                Ok(async_graphql::serde_json::Value::Object(result))
            }
        }

        impl async_graphql::GQLObject for #ident {}
    };
    Ok(expanded.into())
}
