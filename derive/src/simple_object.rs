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
        get_crate_path, get_rustdoc, parse_complexity_expr, visible_fn,
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
    let crate_name = get_crate_path(&object_args.crate_path, object_args.internal);
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

    let object_directives = gen_directive_calls(
        &crate_name,
        &object_args.directives,
        TypeDirectiveLocation::Object,
    );
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
    let gql_typename_string = if !object_args.name_type {
        let name = object_args
            .name
            .as_ref()
            .cloned()
            .unwrap_or_else(|| RenameTarget::Type.rename(ident.to_string()));
        quote!(::std::string::ToString::to_string(#name))
    } else {
        quote!(::std::string::ToString::to_string(&#gql_typename))
    };

    let desc_value = get_rustdoc(&object_args.attrs)?;
    let has_desc = desc_value.is_some();
    let desc = desc_value
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
        let field_desc_value = get_rustdoc(&field.attrs)?;
        let has_field_desc = field_desc_value.is_some();
        let field_desc = field_desc_value
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

        let has_cache_control = field.cache_control.no_cache
            || field.cache_control.max_age != 0
            || !field.cache_control.is_public();
        let has_deprecation = !matches!(field.deprecation, args::Deprecation::NoDeprecated);
        let has_external = external;
        let has_shareable = shareable;
        let has_inaccessible = inaccessible;
        let has_requires = field.requires.is_some();
        let has_provides = field.provides.is_some();
        let has_override_from = field.override_from.is_some();
        let has_visible = !matches!(field.visible, None | Some(args::Visible::None));
        let has_tags = !field.tags.is_empty();
        let has_complexity = field.complexity.is_some();
        let has_directives = !field.directives.is_empty();
        let has_requires_scopes = !field.requires_scopes.is_empty();

        let visible = visible_fn(&field.visible);
        let directives = gen_directive_calls(
            &crate_name,
            &field.directives,
            TypeDirectiveLocation::FieldDefinition,
        );

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
            let mut field_sets = Vec::new();
            if has_field_desc {
                field_sets.push(quote!(field.description = #field_desc;));
            }
            if has_deprecation {
                field_sets.push(quote!(field.deprecation = #field_deprecation;));
            }
            if has_cache_control {
                field_sets.push(quote!(field.cache_control = #cache_control;));
            }
            if has_external {
                field_sets.push(quote!(field.external = true;));
            }
            if has_provides {
                field_sets.push(quote!(field.provides = #provides;));
            }
            if has_requires {
                field_sets.push(quote!(field.requires = #requires;));
            }
            if has_shareable {
                field_sets.push(quote!(field.shareable = true;));
            }
            if has_inaccessible {
                field_sets.push(quote!(field.inaccessible = true;));
            }
            if has_tags {
                field_sets.push(quote!(field.tags = ::std::vec![ #(#tags),* ];));
            }
            if has_override_from {
                field_sets.push(quote!(field.override_from = #override_from;));
            }
            if has_visible {
                field_sets.push(quote!(field.visible = #visible;));
            }
            if has_complexity {
                field_sets.push(quote!(field.compute_complexity = #complexity;));
            }
            if has_directives {
                field_sets
                    .push(quote!(field.directive_invocations = ::std::vec![ #(#directives),* ];));
            }
            if has_requires_scopes {
                field_sets
                    .push(quote!(field.requires_scopes = ::std::vec![ #(#requires_scopes),* ];));
            }

            schema_fields.push(quote! {
                let mut field = #crate_name::registry::MetaField::new(
                    ::std::string::ToString::to_string(#field_name),
                    <#ty as #crate_name::OutputType>::create_type_info(registry),
                );
                #(#field_sets)*
                fields.insert(::std::string::ToString::to_string(#field_name), field);
            });
        } else {
            schema_fields.push(quote! {
                <#ty as #crate_name::OutputType>::create_type_info(registry);
                if let #crate_name::registry::MetaType::Object { fields: obj_fields, .. } =
                    registry.create_fake_output_type::<#ty>() {
                    fields.extend(obj_fields);
                }
            });
        }

        let guard_map_err = quote! {
            .map_err(|err| ctx.set_error_path(err.into_server_error(ctx.item.pos)))
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

        let vis = &field.vis;
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
                    #guard
                    let obj: #ty = #block;
                    return #crate_name::resolver_utils::resolve_simple_field_value(ctx, &obj).await;
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
    let has_cache_control = object_args.cache_control.no_cache
        || object_args.cache_control.max_age != 0
        || !object_args.cache_control.is_public();
    let has_keys = !matches!(object_args.resolvability, Resolvability::Resolvable);
    let has_visible = !matches!(object_args.visible, None | Some(args::Visible::None));
    let has_tags = !object_args.tags.is_empty();
    let has_directives = !object_directives.is_empty();
    let has_requires_scopes = !object_args.requires_scopes.is_empty();
    let field_count = schema_fields.len();

    let mut object_builder_base = Vec::new();
    object_builder_base.push(quote!(.rust_typename(::std::any::type_name::<Self>())));
    if has_desc {
        object_builder_base.push(quote!(.description(#desc)));
    }
    if has_cache_control {
        object_builder_base.push(quote!(.cache_control(#cache_control)));
    }
    if extends {
        object_builder_base.push(quote!(.extends(true)));
    }
    if shareable {
        object_builder_base.push(quote!(.shareable(true)));
    }
    if !resolvable {
        object_builder_base.push(quote!(.resolvable(false)));
    }
    if has_visible {
        object_builder_base.push(quote!(.visible(#visible)));
    }
    if inaccessible {
        object_builder_base.push(quote!(.inaccessible(true)));
    }
    if interface_object {
        object_builder_base.push(quote!(.interface_object(true)));
    }
    if has_tags {
        object_builder_base.push(quote!(.tags(::std::vec![ #(#tags),* ])));
    }
    if has_directives {
        object_builder_base
            .push(quote!(.directive_invocations(::std::vec![ #(#object_directives),* ])));
    }
    if has_requires_scopes {
        object_builder_base.push(quote!(.requires_scopes(::std::vec![ #(#requires_scopes),* ])));
    }

    let mut object_builder = object_builder_base.clone();
    if has_keys {
        object_builder.push(quote!(.keys(#keys)));
    }
    let object_builder_concretes = object_builder_base;

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
        quote! { #crate_name::resolver_utils::resolve_container_serial(ctx, self).await }
    } else {
        quote! { #crate_name::resolver_utils::resolve_container(ctx, self).await }
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
            #boxed_trait
            impl #impl_generics #crate_name::OutputType for #ident #ty_generics #where_clause {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    #gql_typename
                }

                fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                    registry.create_output_type::<Self, _>(#crate_name::registry::MetaTypeId::Object, |registry| {
                        #crate_name::registry::ObjectBuilder::new(
                            #gql_typename_string,
                            {
                                let mut fields = #crate_name::indexmap::IndexMap::with_capacity(#field_count);
                                #(#schema_fields)*
                                #concat_complex_fields
                                fields
                            },
                        )
                        #(#object_builder)*
                        .build()
                    })
                }

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
                    registry.create_output_type::<Self, _>(#crate_name::registry::MetaTypeId::Object, |registry| {
                        #crate_name::registry::ObjectBuilder::new(
                            ::std::string::ToString::to_string(name),
                            {
                                let mut fields = #crate_name::indexmap::IndexMap::with_capacity(#field_count);
                                #(#schema_fields)*
                                ::std::iter::Extend::extend(&mut fields, complex_fields.clone());
                                fields
                            },
                        )
                        #(#object_builder_concretes)*
                        .build()
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
                #boxed_trait
                impl #def_bounds #crate_name::OutputType for #concrete_type {
                    fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                        ::std::borrow::Cow::Borrowed(#gql_typename)
                    }

                    fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                        let mut fields = #crate_name::indexmap::IndexMap::with_capacity(#field_count);
                        #concat_complex_fields
                        Self::__internal_create_type_info_simple_object(registry, #gql_typename, fields)
                    }

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
