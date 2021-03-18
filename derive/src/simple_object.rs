use darling::ast::Data;
use proc_macro::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::Error;

use crate::args::{self, RenameRuleExt, RenameTarget};
use crate::utils::{
    gen_deprecation, generate_guards, get_crate_name, get_rustdoc, visible_fn, GeneratorResult,
};

pub fn generate(object_args: &args::SimpleObject) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let ident = &object_args.ident;
    let (impl_generics, ty_generics, where_clause) = object_args.generics.split_for_impl();
    let extends = object_args.extends;
    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| RenameTarget::Type.rename(ident.to_string()));

    let desc = get_rustdoc(&object_args.attrs)?
        .map(|s| quote! { ::std::option::Option::Some(#s) })
        .unwrap_or_else(|| quote! {::std::option::Option::None});

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
            .map(|s| quote! {::std::option::Option::Some(#s)})
            .unwrap_or_else(|| quote! {::std::option::Option::None});
        let field_deprecation = gen_deprecation(&field.deprecation, &crate_name);
        let external = field.external;
        let requires = match &field.requires {
            Some(requires) => quote! { ::std::option::Option::Some(#requires) },
            None => quote! { ::std::option::Option::None },
        };
        let provides = match &field.provides {
            Some(provides) => quote! { ::std::option::Option::Some(#provides) },
            None => quote! { ::std::option::Option::None },
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

        let visible = visible_fn(&field.visible);

        schema_fields.push(quote! {
            fields.insert(::std::borrow::ToOwned::to_owned(#field_name), #crate_name::registry::MetaField {
                name: ::std::borrow::ToOwned::to_owned(#field_name),
                description: #field_desc,
                args: ::std::default::Default::default(),
                ty: <#ty as #crate_name::Type>::create_type_info(registry),
                deprecation: #field_deprecation,
                cache_control: #cache_control,
                external: #external,
                provides: #provides,
                requires: #requires,
                visible: #visible,
                compute_complexity: ::std::option::Option::None,
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
                     ::std::result::Result::Ok(&self.#ident)
                 }
            }
        } else {
            quote! {
                #[inline]
                #[allow(missing_docs)]
                #vis async fn #ident(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::Result<#ty> {
                    ::std::result::Result::Ok(::std::clone::Clone::clone(&self.#ident))
                }
            }
        });

        resolvers.push(quote! {
            if ctx.item.node.name.node == #field_name {
                #guard
                let res = self.#ident(ctx).await.map_err(|err| err.into_server_error().at(ctx.item.pos))?;
                let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                return #crate_name::OutputType::resolve(&res, &ctx_obj, ctx.item).await.map(::std::option::Option::Some);
            }
        });
    }

    if !object_args.dummy && resolvers.is_empty() {
        return Err(Error::new_spanned(
            &ident,
            "A GraphQL Object type must define one or more fields.",
        )
        .into());
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

    let visible = visible_fn(&object_args.visible);

    let mut concat_complex_fields = quote!();
    let mut complex_resolver = quote!();

    if object_args.complex {
        concat_complex_fields = quote! {
            fields.extend(<Self as #crate_name::ComplexObject>::fields(registry));
        };
        complex_resolver = quote! {
            if let Some(value) = <Self as #crate_name::ComplexObject>::resolve_field(self, ctx).await? {
                return Ok(Some(value));
            }
        };
    }

    let expanded = if object_args.concretes.is_empty() {
        quote! {
            #[allow(clippy::all, clippy::pedantic)]
            impl #impl_generics #ident #ty_generics #where_clause {
                #(#getters)*
            }

            #[allow(clippy::all, clippy::pedantic)]
            impl #impl_generics #crate_name::Type for #ident #ty_generics #where_clause {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed(#gql_typename)
                }

                fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                    registry.create_type::<Self, _>(|registry| #crate_name::registry::MetaType::Object {
                        name: ::std::borrow::ToOwned::to_owned(#gql_typename),
                        description: #desc,
                        fields: {
                            let mut fields = #crate_name::indexmap::IndexMap::new();
                            #(#schema_fields)*
                            #concat_complex_fields
                            fields
                        },
                        cache_control: #cache_control,
                        extends: #extends,
                        keys: ::std::option::Option::None,
                        visible: #visible,
                    })
                }
            }

            #[allow(clippy::all, clippy::pedantic)]
            #[#crate_name::async_trait::async_trait]

            impl #impl_generics #crate_name::resolver_utils::ContainerType for #ident #ty_generics #where_clause {
                async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                    #(#resolvers)*
                    #complex_resolver
                    ::std::result::Result::Ok(::std::option::Option::None)
                }
            }

            #[allow(clippy::all, clippy::pedantic)]
            #[#crate_name::async_trait::async_trait]
            impl #impl_generics #crate_name::OutputType for #ident #ty_generics #where_clause {
                async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::Value> {
                    #crate_name::resolver_utils::resolve_container(ctx, self).await
                }
            }

            impl #impl_generics #crate_name::ObjectType for #ident #ty_generics #where_clause {}
        }
    } else {
        let mut code = Vec::new();

        code.push(quote! {
            impl #impl_generics #ident #ty_generics #where_clause {
                #(#getters)*

                fn __internal_create_type_info(registry: &mut #crate_name::registry::Registry, name: &str) -> ::std::string::String where Self: #crate_name::OutputType {
                    registry.create_type::<Self, _>(|registry| #crate_name::registry::MetaType::Object {
                        name: ::std::borrow::ToOwned::to_owned(name),
                        description: #desc,
                        fields: {
                            let mut fields = #crate_name::indexmap::IndexMap::new();
                            #(#schema_fields)*
                            fields
                        },
                        cache_control: #cache_control,
                        extends: #extends,
                        keys: ::std::option::Option::None,
                        visible: #visible,
                    })
                }

                async fn __internal_resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> where Self: #crate_name::ContainerType {
                    #(#resolvers)*
                    ::std::result::Result::Ok(::std::option::Option::None)
                }
            }
        });

        for concrete in &object_args.concretes {
            let gql_typename = &concrete.name;
            let params = &concrete.params.0;
            let concrete_type = quote! { #ident<#(#params),*> };

            let expanded = quote! {
                #[allow(clippy::all, clippy::pedantic)]
                impl #crate_name::Type for #concrete_type {
                    fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                        ::std::borrow::Cow::Borrowed(#gql_typename)
                    }

                    fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                        Self::__internal_create_type_info(registry, #gql_typename)
                    }
                }

                #[allow(clippy::all, clippy::pedantic)]
                #[#crate_name::async_trait::async_trait]
                impl #crate_name::resolver_utils::ContainerType for #concrete_type {
                    async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                        self.__internal_resolve_field(ctx).await
                    }
                }

                #[allow(clippy::all, clippy::pedantic)]
                #[#crate_name::async_trait::async_trait]
                impl #crate_name::OutputType for #concrete_type {
                    async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::Value> {
                        #crate_name::resolver_utils::resolve_container(ctx, self).await
                    }
                }

                impl #crate_name::ObjectType for #concrete_type {}
            };
            code.push(expanded);
        }

        quote!(#(#code)*)
    };

    Ok(expanded.into())
}
