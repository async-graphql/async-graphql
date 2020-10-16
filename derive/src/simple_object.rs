use darling::ast::Data;
use proc_macro::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::Error;

use crate::args::{self, RenameRuleExt, RenameTarget};
use crate::utils::{generate_guards, get_crate_name, get_rustdoc, GeneratorResult};

pub fn generate(object_args: &args::SimpleObject) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let ident = &object_args.ident;
    let generics = &object_args.generics;
    let where_clause = &generics.where_clause;
    let extends = object_args.extends;
    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| RenameTarget::Type.rename(ident.to_string()));

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

        let field_name = field.name.clone().unwrap_or_else(|| {
            object_args
                .rename_fields
                .rename(ident.unraw().to_string(), RenameTarget::Field)
        });
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
        let guard = guard.map(|guard| quote! { #guard.check(ctx).await.map_err(|err| err.into_server_error().at(ctx.item.pos))?; });

        getters.push(if !field.owned {
            quote! {
                 #[inline]
                 #[allow(missing_docs)]
                 #vis async fn #ident(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::Result<&#ty> {
                     Ok(&self.#ident)
                 }
            }
        } else {
            quote! {
                #[inline]
                #[allow(missing_docs)]
                #vis async fn #ident(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::Result<#ty> {
                    Ok(self.#ident.clone())
                }
            }
        });

        resolvers.push(quote! {
            if ctx.item.node.name.node == #field_name {
                #guard
                let res = self.#ident(ctx).await.map_err(|err| err.into_server_error().at(ctx.item.pos))?;
                let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                return #crate_name::OutputValueType::resolve(&res, &ctx_obj, ctx.item).await.map(::std::option::Option::Some);
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
        impl #generics #ident #generics #where_clause {
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

        impl #generics #crate_name::resolver_utils::ContainerType for #ident #generics #where_clause {
            async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                #(#resolvers)*
                Ok(None)
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::OutputValueType for #ident #generics #where_clause {

            async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::Value> {
                #crate_name::resolver_utils::resolve_container(ctx, self).await
            }
        }

        impl #generics #crate_name::ObjectType for #ident #generics #where_clause {}
    };
    Ok(expanded.into())
}
