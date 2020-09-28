use crate::args;
use crate::utils::{
    generate_guards, generate_post_guards, get_crate_name, get_rustdoc, GeneratorResult,
};
use darling::ast::Data;
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::Error;

pub fn generate(object_args: &args::SimpleObject) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let ident = &object_args.ident;
    let generics = &object_args.generics;
    let where_clause = &generics.where_clause;
    let extends = object_args.extends;
    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| ident.to_string());

    let desc = get_rustdoc(&object_args.attrs)?
        .map(|s| quote! { Some(#s) })
        .unwrap_or_else(|| quote! {None});

    let s = match &object_args.data {
        Data::Struct(e) => e,
        _ => {
            return Err(Error::new_spanned(
                &ident,
                "SimpleObject can only be applied to an struct.",
            )
            .into())
        }
    };
    let mut getters = Vec::new();
    let mut resolvers = Vec::new();
    let mut schema_fields = Vec::new();

    for field in &s.fields {
        if field.skip {
            continue;
        }
        let ident = match &field.ident {
            Some(ident) => ident,
            None => return Err(Error::new_spanned(&ident, "All fields must be named.").into()),
        };

        let field_name = field
            .name
            .clone()
            .unwrap_or_else(|| ident.unraw().to_string().to_camel_case());
        let field_desc = get_rustdoc(&field.attrs)?
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
        let vis = &field.vis;
        let ty = &field.ty;

        let cache_control = {
            let public = field.cache_control.is_public();
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

        let guard = match &field.guard {
            Some(meta) => generate_guards(&crate_name, &meta)?,
            None => None,
        };
        let guard = guard.map(|guard| quote! { #guard.check(ctx).await.map_err(|err| err.into_error_with_path(ctx.item.pos, ctx.path_node.as_ref()))?; });

        let post_guard = match &field.post_guard {
            Some(meta) => generate_post_guards(&crate_name, &meta)?,
            None => None,
        };
        let post_guard = post_guard.map(|guard| quote! { #guard.check(ctx, &res).await.map_err(|err| err.into_error_with_path(ctx.item.pos, ctx.path_node.as_ref()))?; });

        getters.push(if !field.owned {
            quote! {
                 #[inline]
                 #[allow(missing_docs)]
                 #vis async fn #ident(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::FieldResult<&#ty> {
                     Ok(&self.#ident)
                 }
            }
        } else {
            quote! {
                #[inline]
                #[allow(missing_docs)]
                #vis async fn #ident(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::FieldResult<#ty> {
                    Ok(self.#ident.clone())
                }
            }
        });

        resolvers.push(quote! {
            if ctx.item.node.name.node == #field_name {
                #guard
                let res = self.#ident(ctx).await.map_err(|err| err.into_error_with_path(ctx.item.pos, ctx.path_node.as_ref()))?;
                let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                #post_guard
                return #crate_name::OutputValueType::resolve(&res, &ctx_obj, ctx.item).await;
            }
        });
    }

    let cache_control = {
        let public = object_args.cache_control.is_public();
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
        impl #generics #crate_name::resolver_utils::ObjectType for #ident #generics #where_clause {
            async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::Result<#crate_name::serde_json::Value> {
                #(#resolvers)*
                Err(#crate_name::QueryError::FieldNotFound {
                    field_name: ctx.item.node.name.to_string(),
                    object: #gql_typename.to_string(),
                }.into_error(ctx.item.pos))
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::OutputValueType for #ident #generics #where_clause {
            async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::Result<#crate_name::serde_json::Value> {
                #crate_name::resolver_utils::resolve_object(ctx, self).await
            }
        }

        impl #generics #crate_name::type_mark::TypeMarkObject for #ident #generics #where_clause {}
    };
    Ok(expanded.into())
}
