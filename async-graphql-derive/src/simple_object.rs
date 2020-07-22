use crate::args;
use crate::utils::{feature_block, get_crate_name, get_rustdoc};
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::{Data, DeriveInput, Error, Fields, Result};

pub fn generate(object_args: &args::Object, input: &DeriveInput) -> Result<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let ident = &input.ident;
    let generics = &input.generics;
    let where_clause = &generics.where_clause;
    let extends = object_args.extends;
    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| ident.to_string());

    let desc = object_args
        .desc
        .clone()
        .or_else(|| get_rustdoc(&input.attrs).ok().flatten())
        .map(|s| quote! { Some(#s) })
        .unwrap_or_else(|| quote! {None});

    let s = match &input.data {
        Data::Struct(e) => e,
        _ => return Err(Error::new_spanned(input, "It should be a struct")),
    };
    let mut getters = Vec::new();
    let mut resolvers = Vec::new();
    let mut schema_fields = Vec::new();
    let fields = match &s.fields {
        Fields::Named(fields) => Some(fields),
        Fields::Unit => None,
        _ => return Err(Error::new_spanned(input, "All fields must be named.")),
    };

    if let Some(fields) = fields {
        for item in &fields.named {
            if let Some(field) = args::Field::parse(&crate_name, &item.attrs)? {
                let field_name = field.name.clone().unwrap_or_else(|| {
                    item.ident
                        .as_ref()
                        .unwrap()
                        .unraw()
                        .to_string()
                        .to_camel_case()
                });
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
                let vis = &item.vis;
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
                    fields.insert(#field_name.to_string(), #crate_name::registry::MetaField {
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
                let guard = field
                    .guard
                    .map(|guard| quote! { #guard.check(ctx).await.map_err(|err| err.into_error_with_path(ctx.position(), ctx.path_node.as_ref()))?; });
                let post_guard = field
                    .post_guard
                    .map(|guard| quote! { #guard.check(ctx, &res).await.map_err(|err| err.into_error_with_path(ctx.position(), ctx.path_node.as_ref()))?; });

                let features = &field.features;
                getters.push(if !field.owned {
                    let block = feature_block(
                        &crate_name,
                        &features,
                        &field_name,
                        quote! { Ok(&self.#ident) },
                    );
                    quote! {
                         #[inline]
                         #[allow(missing_docs)]
                         #vis async fn #ident(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::FieldResult<&#ty> {
                             #block
                         }
                    }
                } else {
                    let block = feature_block(
                        &crate_name,
                        &features,
                        &field_name,
                        quote! { Ok(self.#ident.clone()) },
                    );
                    quote! {
                        #[inline]
                        #[allow(missing_docs)]
                        #vis async fn #ident(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::FieldResult<#ty> {
                            #block
                        }
                    }
                });

                resolvers.push(quote! {
                    if ctx.name.node == #field_name {
                        #guard
                        let res = self.#ident(ctx).await.map_err(|err| err.into_error_with_path(ctx.position(), ctx.path_node.as_ref()))?;
                        let ctx_obj = ctx.with_selection_set(&ctx.selection_set);
                        #post_guard
                        return #crate_name::OutputValueType::resolve(&res, &ctx_obj, ctx.item).await;
                    }
                });
            }
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
        #[allow(clippy::all, clippy::pedantic)]
        impl #generics #ident #where_clause {
            #(#getters)*
        }

        #[allow(clippy::all, clippy::pedantic)]
        impl #generics #crate_name::Type for #ident #generics #where_clause {
            fn type_name() -> ::std::borrow::Cow<'static, str> {
                ::std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> String {
                registry.create_type::<Self, _>(|registry| #crate_name::registry::MetaType::Object {
                    name: #gql_typename.to_string(),
                    description: #desc,
                    fields: {
                        let mut fields = #crate_name::indexmap::IndexMap::new();
                        #(#schema_fields)*
                        fields
                    },
                    cache_control: #cache_control,
                    extends: #extends,
                    keys: None,
                })
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::ObjectType for #ident #generics #where_clause {
            async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::Result<#crate_name::serde_json::Value> {
                #(#resolvers)*
                Err(#crate_name::QueryError::FieldNotFound {
                    field_name: ctx.name.to_string(),
                    object: #gql_typename.to_string(),
                }.into_error(ctx.position()))
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::OutputValueType for #ident #generics #where_clause {
            async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::query::Field>) -> #crate_name::Result<#crate_name::serde_json::Value> {
                #crate_name::do_resolve(ctx, self).await
            }
        }
    };
    Ok(expanded.into())
}
