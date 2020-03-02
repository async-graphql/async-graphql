use crate::args;
use crate::utils::get_crate_name;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Error, Ident, Result};

pub fn generate(object_args: &args::Object, input: &DeriveInput) -> Result<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
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
    let mut obj_attrs = Vec::new();
    let mut obj_fields = Vec::new();
    let mut trait_fns = Vec::new();
    let mut resolvers = Vec::new();
    let mut all_is_simple_attr = true;

    for field in &s.fields {
        if let Some(field_args) = args::Field::parse(&field.attrs)? {
            // is field
            let vis = &field.vis;
            let ty = &field.ty;
            let ident = field.ident.as_ref().unwrap();
            let field_name = ident.to_string();

            obj_fields.push(field);
            if field_args.is_attr {
                let ty = field_args.attr_type.as_ref().unwrap_or(ty);
                obj_attrs.push(quote! { #vis #ident: #ty });
                if !field_args.arguments.is_empty() || field_args.attr_type.is_some() {
                    all_is_simple_attr = false;
                }
            } else {
                all_is_simple_attr = false;
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
                async fn #ident(&self, ctx: &#crate_name::ContextField<'_>, #(#decl_params),*) -> #crate_name::Result<#ty>;
            });

            resolvers.push(quote! {
                if field.name.as_str() == #field_name {
                    #(#get_params)*
                    let obj = #trait_ident::#ident(self, &ctx_field, #(#use_params),*).await.
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
            obj_attrs.push(quote! { #field });
        }
    }

    let mut impl_fields = quote! {};
    if object_args.auto_impl && all_is_simple_attr {
        let mut impl_fns = Vec::new();
        for field in obj_fields {
            let ident = &field.ident;
            let ty = &field.ty;
            impl_fns.push(quote! {
                async fn #ident(&self, _: &#crate_name::ContextField<'_>) -> #crate_name::Result<#ty> {
                    Ok(self.#ident.clone())
                }
            });
        }
        impl_fields = quote! {
            #[#crate_name::async_trait::async_trait]
            impl #trait_ident for #ident {
                #(#impl_fns)*
            }
        };
    }

    let expanded = quote! {
        #(#attrs)*
        #vis struct #ident {
            #(#obj_attrs),*
        }

        #[#crate_name::async_trait::async_trait]
        #vis trait #trait_ident {
            #(#trait_fns)*
        }

        impl #crate_name::GQLType for #ident {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#gql_typename)
            }
        }

        #[#crate_name::async_trait::async_trait]
        impl #crate_name::GQLOutputValue for #ident {
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
                            if field.name.as_str() == "__typename" {
                                let name = field.alias.clone().unwrap_or_else(|| field.name.clone());
                                result.insert(name, #gql_typename.into());
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

        impl #crate_name::GQLObject for #ident {}

        #impl_fields
    };
    Ok(expanded.into())
}
