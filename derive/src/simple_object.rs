use std::str::FromStr;

use darling::ast::Data;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, Ident, LifetimeParam, Path, Type, ext::IdentExt, visit::Visit};

use crate::{
    args::{
        self, RenameRuleExt, RenameTarget, Resolvability, SimpleObjectField, TypeDirectiveLocation,
    },
    utils::{
        GeneratorResult, gen_boxed_trait, gen_deprecation, gen_directive_calls, generate_guards,
        get_crate_name, get_rustdoc, parse_complexity_expr, visible_fn,
    },
};

#[derive(Debug)]
struct DerivedFieldMetadata {
    ident: Ident,
    into: Type,
    owned: Option<bool>,
    with: Option<Path>,
}

struct SimpleObjectFieldGenerator<'a> {
    field: &'a SimpleObjectField,
    derived: Option<DerivedFieldMetadata>,
}

pub fn generate(object_args: &args::SimpleObject) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let boxed_trait = gen_boxed_trait(&crate_name);
    let ident = &object_args.ident;
    let (impl_generics, ty_generics, where_clause) = object_args.generics.split_for_impl();
    let extends = object_args.extends;
    let shareable = object_args.shareable;
    let inaccessible = object_args.inaccessible;
    let interface_object = object_args.interface_object;
    let resolvable = matches!(object_args.resolvability, Resolvability::Resolvable);
    let tags = object_args
        .tags
        .iter()
        .map(|tag| quote!(::std::string::ToString::to_string(#tag)))
        .collect::<Vec<_>>();
    let requires_scopes = object_args
        .requires_scopes
        .iter()
        .map(|scopes| quote!(::std::string::ToString::to_string(#scopes)))
        .collect::<Vec<_>>();
    let object_directives =
        gen_directive_calls(&object_args.directives, TypeDirectiveLocation::Object);
    let gql_typename = if !object_args.name_type {
        object_args
            .name
            .as_ref()
            .map(|name| quote!(::std::borrow::Cow::Borrowed(#name)))
            .unwrap_or_else(|| {
                let name = RenameTarget::Type.rename(ident.to_string());
                quote!(::std::borrow::Cow::Borrowed(#name))
            })
    } else {
        quote!(<Self as #crate_name::TypeName>::type_name())
    };

    let desc = get_rustdoc(&object_args.attrs)?
        .map(|s| quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#s)) })
        .unwrap_or_else(|| quote! {::std::option::Option::None});

    let s = match &object_args.data {
        Data::Struct(e) => e,
        _ => {
            return Err(Error::new_spanned(
                ident,
                "SimpleObject can only be applied to an struct.",
            )
            .into());
        }
    };
    let mut getters = Vec::new();
    let mut resolvers = Vec::new();
    let mut schema_fields = Vec::new();

    let mut processed_fields: Vec<SimpleObjectFieldGenerator> = vec![];

    // Before processing the fields, we generate the derived fields
    for field in &s.fields {
        processed_fields.push(SimpleObjectFieldGenerator {
            field,
            derived: None,
        });

        for derived in &field.derived {
            if derived.name.is_some() && derived.into.is_some() {
                let name = derived.name.clone().unwrap();
                let into = match syn::parse2::<Type>(
                    proc_macro2::TokenStream::from_str(&derived.into.clone().unwrap()).unwrap(),
                ) {
                    Ok(e) => e,
                    _ => {
                        return Err(Error::new_spanned(
                            &name,
                            "derived into must be a valid type.",
                        )
                        .into());
                    }
                };

                let derived = DerivedFieldMetadata {
                    ident: name,
                    into,
                    owned: derived.owned,
                    with: derived.with.clone(),
                };

                processed_fields.push(SimpleObjectFieldGenerator {
                    field,
                    derived: Some(derived),
                })
            }
        }
    }

    for SimpleObjectFieldGenerator { field, derived } in &processed_fields {
        if (field.skip || field.skip_output) && derived.is_none() {
            continue;
        }

        let base_ident = match &field.ident {
            Some(ident) => ident,
            None => return Err(Error::new_spanned(ident, "All fields must be named.").into()),
        };

        let ident = if let Some(derived) = derived {
            &derived.ident
        } else {
            base_ident
        };

        let field_name = field.name.clone().unwrap_or_else(|| {
            object_args
                .rename_fields
                .rename(ident.unraw().to_string(), RenameTarget::Field)
        });
        let field_desc = get_rustdoc(&field.attrs)?
            .map(|s| quote! {::std::option::Option::Some(::std::string::ToString::to_string(#s))})
            .unwrap_or_else(|| quote! {::std::option::Option::None});
        let field_deprecation = gen_deprecation(&field.deprecation, &crate_name);
        let external = field.external;
        let shareable = field.shareable;
        let inaccessible = field.inaccessible;
        let tags = field
            .tags
            .iter()
            .map(|tag| quote!(::std::string::ToString::to_string(#tag)))
            .collect::<Vec<_>>();
        let requires_scopes = field
            .requires_scopes
            .iter()
            .map(|scopes| quote!(::std::string::ToString::to_string(#scopes)))
            .collect::<Vec<_>>();
        let override_from = match &field.override_from {
            Some(from) => {
                quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#from)) }
            }
            None => quote! { ::std::option::Option::None },
        };
        let requires = match &field.requires {
            Some(requires) => {
                quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#requires)) }
            }
            None => quote! { ::std::option::Option::None },
        };
        let provides = match &field.provides {
            Some(provides) => {
                quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#provides)) }
            }
            None => quote! { ::std::option::Option::None },
        };
        let vis = &field.vis;

        let ty = if let Some(derived) = derived {
            &derived.into
        } else {
            &field.ty
        };

        let owned = if let Some(derived) = derived {
            derived.owned.unwrap_or(field.owned)
        } else {
            field.owned
        };

        let cache_control = {
            let public = field.cache_control.is_public();
            let max_age = if field.cache_control.no_cache {
                -1
            } else {
                field.cache_control.max_age as i32
            };
            quote! {
            #crate_name::CacheControl {
                        public: #public,
                        max_age: #max_age,
                    }
                }
        };

        let visible = visible_fn(&field.visible);
        let directives =
            gen_directive_calls(&field.directives, TypeDirectiveLocation::FieldDefinition);

        let complexity = if let Some(complexity) = &field.complexity {
            let (_, expr) = parse_complexity_expr(complexity.clone())?;
            quote! {
                ::std::option::Option::Some(|__ctx, __variables_definition, __field, child_complexity| {
                    ::std::result::Result::Ok(#expr)
                })
            }
        } else {
            quote! { ::std::option::Option::None }
        };

        if !field.flatten {
            schema_fields.push(quote! {
                fields.insert(::std::borrow::ToOwned::to_owned(#field_name), #crate_name::registry::MetaField {
                    name: ::std::borrow::ToOwned::to_owned(#field_name),
                    description: #field_desc,
                    args: ::std::default::Default::default(),
                    ty: <#ty as #crate_name::OutputTypeMarker>::create_type_info(registry),
                    deprecation: #field_deprecation,
                    cache_control: #cache_control,
                    external: #external,
                    provides: #provides,
                    requires: #requires,
                    shareable: #shareable,
                    inaccessible: #inaccessible,
                    tags: ::std::vec![ #(#tags),* ],
                    override_from: #override_from,
                    visible: #visible,
                    compute_complexity: #complexity,
                    directive_invocations: ::std::vec![ #(#directives),* ],
                    requires_scopes: ::std::vec![ #(#requires_scopes),* ],
                });
            });
        } else {
            schema_fields.push(quote! {
                <#ty as #crate_name::OutputTypeMarker>::create_type_info(registry);
                if let #crate_name::registry::MetaType::Object { fields: obj_fields, .. } =
                    registry.create_fake_output_type::<#ty>() {
                    fields.extend(obj_fields);
                }
            });
        }

        let guard_map_err = quote! {
            .map_err(|err| err.into_server_error(ctx.item.pos))
        };
        let guard = match field.guard.as_ref().or(object_args.guard.as_ref()) {
            Some(code) => Some(generate_guards(&crate_name, code, guard_map_err)?),
            None => None,
        };

        let with_function = derived.as_ref().and_then(|x| x.with.as_ref());

        let mut block = match !owned {
            true => quote! {
                &self.#base_ident
            },
            false => quote! {
                ::std::clone::Clone::clone(&self.#base_ident)
            },
        };

        block = match (derived, with_function) {
            (Some(_), Some(with)) => quote! {
                #with(#block)
            },
            (Some(_), None) => quote! {
                ::std::convert::Into::into(#block)
            },
            (_, _) => block,
        };

        let ty = match !owned {
            true => quote! { &#ty },
            false => quote! { #ty },
        };

        if !field.flatten {
            getters.push(quote! {
                 #[inline]
                 #[allow(missing_docs)]
                 #vis async fn #ident(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::Result<#ty> {
                     ::std::result::Result::Ok(#block)
                 }
            });

            resolvers.push(quote! {
                if ctx.item.node.name.node == #field_name {
                    let f = async move {
                        #guard
                        self.#ident(ctx).await.map_err(|err| err.into_server_error(ctx.item.pos))
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return #crate_name::OutputType::resolve(&obj, &ctx_obj, ctx.item).await.map(::std::option::Option::Some);
                }
            });
        } else {
            resolvers.push(quote! {
                if let ::std::option::Option::Some(value) = #crate_name::ContainerType::resolve_field(&self.#ident, ctx).await? {
                    return ::std::result::Result::Ok(std::option::Option::Some(value));
                }
            });
        }
    }

    if !object_args.fake && resolvers.is_empty() {
        return Err(Error::new_spanned(
            ident,
            "A GraphQL Object type must define one or more fields.",
        )
        .into());
    }

    let cache_control = {
        let public = object_args.cache_control.is_public();
        let max_age = if object_args.cache_control.no_cache {
            -1
        } else {
            object_args.cache_control.max_age as i32
        };
        quote! {
            #crate_name::CacheControl {
                public: #public,
                max_age: #max_age,
            }
        }
    };

    let keys = match &object_args.resolvability {
        Resolvability::Resolvable => quote!(::std::option::Option::None),
        Resolvability::Unresolvable { key: Some(key) } => quote!(::std::option::Option::Some(
            ::std::vec![ ::std::string::ToString::to_string(#key)]
        )),
        Resolvability::Unresolvable { key: None } => {
            let keys = processed_fields
                .iter()
                .filter(|g| !g.field.skip && !g.field.skip_output)
                .map(|generator| {
                    let ident = if let Some(derived) = &generator.derived {
                        &derived.ident
                    } else {
                        generator.field.ident.as_ref().unwrap()
                    };
                    generator.field.name.clone().unwrap_or_else(|| {
                        object_args
                            .rename_fields
                            .rename(ident.unraw().to_string(), RenameTarget::Field)
                    })
                })
                .reduce(|mut keys, key| {
                    keys.push(' ');
                    keys.push_str(&key);
                    keys
                })
                .unwrap();

            quote!(::std::option::Option::Some(
                ::std::vec![ ::std::string::ToString::to_string(#keys) ]
            ))
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

    let resolve_container = if object_args.serial {
        if cfg!(feature = "boxed-trait") {
            quote! { #crate_name::resolver_utils::resolve_container_serial(ctx, &self as &dyn #crate_name::resolver_utils::ContainerType, &self).await }
        } else {
            quote! { #crate_name::resolver_utils::resolve_container_serial(ctx, self).await }
        }
    } else {
        if cfg!(feature = "boxed-trait") {
            quote! { #crate_name::resolver_utils::resolve_container(ctx, &self as &dyn #crate_name::resolver_utils::ContainerType, &self).await }
        } else {
            quote! { #crate_name::resolver_utils::resolve_container(ctx, self).await }
        }
    };

    let expanded = if object_args.concretes.is_empty() {
        quote! {
            #[allow(clippy::all, clippy::pedantic)]
            impl #impl_generics #ident #ty_generics #where_clause {
                #(#getters)*
            }

            #[allow(clippy::all, clippy::pedantic)]
            #boxed_trait
            impl #impl_generics #crate_name::resolver_utils::ContainerType for #ident #ty_generics #where_clause {
                async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                    #(#resolvers)*
                    #complex_resolver
                    ::std::result::Result::Ok(::std::option::Option::None)
                }
            }

            #[allow(clippy::all, clippy::pedantic)]
            impl #impl_generics #crate_name::OutputTypeMarker for #ident #ty_generics #where_clause {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    #gql_typename
                }

                fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                    registry.create_output_type::<Self, _>(#crate_name::registry::MetaTypeId::Object, |registry| #crate_name::registry::MetaType::Object {
                        name: ::std::borrow::Cow::into_owned(#gql_typename),
                        description: #desc,
                        fields: {
                            let mut fields = #crate_name::indexmap::IndexMap::new();
                            #(#schema_fields)*
                            #concat_complex_fields
                            fields
                        },
                        cache_control: #cache_control,
                        extends: #extends,
                        shareable: #shareable,
                        resolvable: #resolvable,
                        inaccessible: #inaccessible,
                        interface_object: #interface_object,
                        tags: ::std::vec![ #(#tags),* ],
                        keys: #keys,
                        visible: #visible,
                        is_subscription: false,
                        rust_typename: ::std::option::Option::Some(::std::any::type_name::<Self>()),
                        directive_invocations: ::std::vec![ #(#object_directives),* ],
                        requires_scopes: ::std::vec![ #(#requires_scopes),* ],
                    })
                }
            }

            #[allow(clippy::all, clippy::pedantic)]
            #boxed_trait
            impl #impl_generics #crate_name::OutputType for #ident #ty_generics #where_clause {

                async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::Value> {
                    #resolve_container
                }
            }

            impl #impl_generics #crate_name::ObjectType for #ident #ty_generics #where_clause {}
        }
    } else {
        let mut code = Vec::new();

        #[derive(Default)]
        struct GetLifetimes<'a> {
            lifetimes: Vec<&'a LifetimeParam>,
        }

        impl<'a> Visit<'a> for GetLifetimes<'a> {
            fn visit_lifetime_param(&mut self, i: &'a LifetimeParam) {
                self.lifetimes.push(i);
            }
        }

        let mut visitor = GetLifetimes::default();
        visitor.visit_generics(&object_args.generics);
        let lifetimes = visitor.lifetimes;

        let type_lifetimes = if !lifetimes.is_empty() {
            Some(quote!(#(#lifetimes,)*))
        } else {
            None
        };

        code.push(quote! {
            impl #impl_generics #ident #ty_generics #where_clause {
                #(#getters)*

                fn __internal_create_type_info_simple_object(
                    registry: &mut #crate_name::registry::Registry,
                    name: &str,
                    complex_fields: #crate_name::indexmap::IndexMap<::std::string::String, #crate_name::registry::MetaField>,
                ) -> ::std::string::String where Self: #crate_name::OutputType {
                    registry.create_output_type::<Self, _>(#crate_name::registry::MetaTypeId::Object, |registry| #crate_name::registry::MetaType::Object {
                        name: ::std::borrow::ToOwned::to_owned(name),
                        description: #desc,
                        fields: {
                            let mut fields = #crate_name::indexmap::IndexMap::new();
                            #(#schema_fields)*
                            ::std::iter::Extend::extend(&mut fields, complex_fields.clone());
                            fields
                        },
                        cache_control: #cache_control,
                        extends: #extends,
                        shareable: #shareable,
                        resolvable: #resolvable,
                        inaccessible: #inaccessible,
                        interface_object: #interface_object,
                        tags: ::std::vec![ #(#tags),* ],
                        keys: ::std::option::Option::None,
                        visible: #visible,
                        is_subscription: false,
                        rust_typename: ::std::option::Option::Some(::std::any::type_name::<Self>()),
                        directive_invocations: ::std::vec![ #(#object_directives),* ],
                        requires_scopes: ::std::vec![ #(#requires_scopes),* ],
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
            let concrete_type = quote! { #ident<#type_lifetimes #(#params),*> };

            let def_bounds = if !lifetimes.is_empty() || !concrete.bounds.0.is_empty() {
                let bounds = lifetimes
                    .iter()
                    .map(|l| quote!(#l))
                    .chain(concrete.bounds.0.iter().map(|b| quote!(#b)));
                Some(quote!(<#(#bounds),*>))
            } else {
                None
            };

            let expanded = quote! {
                #[allow(clippy::all, clippy::pedantic)]
                #boxed_trait
                impl #def_bounds #crate_name::resolver_utils::ContainerType for #concrete_type {
                    async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                        #complex_resolver
                        self.__internal_resolve_field(ctx).await
                    }
                }

                #[allow(clippy::all, clippy::pedantic)]
                impl #def_bounds #crate_name::OutputTypeMarker for #concrete_type {
                    fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                        ::std::borrow::Cow::Borrowed(#gql_typename)
                    }

                    fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                        let mut fields = #crate_name::indexmap::IndexMap::new();
                        #concat_complex_fields
                        Self::__internal_create_type_info_simple_object(registry, #gql_typename, fields)
                    }
                }

                #[allow(clippy::all, clippy::pedantic)]
                #boxed_trait
                impl #def_bounds #crate_name::OutputType for #concrete_type {

                    async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::Value> {
                        #resolve_container
                    }
                }

                impl #def_bounds #crate_name::ObjectType for #concrete_type {}
            };
            code.push(expanded);
        }

        quote!(#(#code)*)
    };

    Ok(expanded.into())
}
