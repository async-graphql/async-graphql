use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Block, Error, ImplItem, ItemImpl, ReturnType, Type, TypeImplTrait, TypeParamBound,
    ext::IdentExt,
};

use crate::{
    args::{self, RenameRuleExt, RenameTarget, SubscriptionField, TypeDirectiveLocation},
    output_type::OutputType,
    utils::{
        GeneratorResult, extract_input_args, gen_deprecation, gen_directive_calls,
        generate_default, generate_guards, get_cfg_attrs, get_crate_name, get_rustdoc,
        get_type_path_and_name, parse_complexity_expr, parse_graphql_attrs, remove_graphql_attrs,
        visible_fn,
    },
};

pub fn generate(
    subscription_args: &args::Subscription,
    item_impl: &mut ItemImpl,
) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(subscription_args.internal);
    let (self_ty, self_name) = get_type_path_and_name(item_impl.self_ty.as_ref())?;
    let generics = &item_impl.generics;
    let where_clause = &item_impl.generics.where_clause;
    let extends = subscription_args.extends;
    let directives =
        gen_directive_calls(&subscription_args.directives, TypeDirectiveLocation::Object);

    let gql_typename = if !subscription_args.name_type {
        let name = subscription_args
            .name
            .clone()
            .unwrap_or_else(|| RenameTarget::Type.rename(self_name.clone()));
        quote!(::std::borrow::Cow::Borrowed(#name))
    } else {
        quote!(<Self as #crate_name::TypeName>::type_name())
    };

    let desc = if subscription_args.use_type_description {
        quote! { ::std::option::Option::Some(::std::string::ToString::to_string(<Self as #crate_name::Description>::description())) }
    } else {
        get_rustdoc(&item_impl.attrs)?
            .map(|s| quote!(::std::option::Option::Some(::std::string::ToString::to_string(#s))))
            .unwrap_or_else(|| quote!(::std::option::Option::None))
    };

    let mut create_stream = Vec::new();
    let mut schema_fields = Vec::new();

    for item in &mut item_impl.items {
        if let ImplItem::Fn(method) = item {
            let field: SubscriptionField = parse_graphql_attrs(&method.attrs)?.unwrap_or_default();
            if field.skip {
                remove_graphql_attrs(&mut method.attrs);
                continue;
            }

            let ident = method.sig.ident.clone();
            let field_name = field.name.clone().unwrap_or_else(|| {
                subscription_args
                    .rename_fields
                    .rename(method.sig.ident.unraw().to_string(), RenameTarget::Field)
            });
            let field_desc = get_rustdoc(&method.attrs)?
                .map(|s| quote! {::std::option::Option::Some(::std::string::ToString::to_string(#s))})
                .unwrap_or_else(|| quote! {::std::option::Option::None});
            let field_deprecation = gen_deprecation(&field.deprecation, &crate_name);
            let cfg_attrs = get_cfg_attrs(&method.attrs);

            if method.sig.asyncness.is_none() {
                return Err(Error::new_spanned(
                    method,
                    "The subscription stream function must be asynchronous",
                )
                .into());
            }

            let mut schema_args = Vec::new();
            let mut use_params = Vec::new();
            let mut get_params = Vec::new();
            let args = extract_input_args::<args::SubscriptionFieldArgument>(&crate_name, method)?;

            for (
                ident,
                ty,
                args::SubscriptionFieldArgument {
                    name,
                    desc,
                    default,
                    default_with,
                    validator,
                    process_with,
                    visible: arg_visible,
                    secret,
                    deprecation,
                },
            ) in &args
            {
                let name = name.clone().unwrap_or_else(|| {
                    subscription_args
                        .rename_args
                        .rename(ident.ident.unraw().to_string(), RenameTarget::Argument)
                });
                let desc = desc
                    .as_ref()
                    .map(|s| quote! {::std::option::Option::Some(::std::string::ToString::to_string(#s))})
                    .unwrap_or_else(|| quote! {::std::option::Option::None});
                let default = generate_default(default, default_with)?;

                let schema_default = default
                    .as_ref()
                    .map(|value| {
                        quote! {
                            ::std::option::Option::Some(::std::string::ToString::to_string(
                                &<#ty as #crate_name::InputType>::to_value(&#value)
                            ))
                        }
                    })
                    .unwrap_or_else(|| quote! {::std::option::Option::None});

                let visible = visible_fn(arg_visible);
                let deprecation = gen_deprecation(deprecation, &crate_name);

                schema_args.push(quote! {
                    args.insert(::std::borrow::ToOwned::to_owned(#name), #crate_name::registry::MetaInputValue {
                            name: ::std::string::ToString::to_string(#name),
                            description: #desc,
                            ty: <#ty as #crate_name::InputType>::create_type_info(registry),
                            deprecation: #deprecation,
                            default_value: #schema_default,
                            visible: #visible,
                            inaccessible: false,
                            tags: ::std::default::Default::default(),
                            is_secret: #secret,
                            directive_invocations: ::std::vec![],
                        });
                    });

                use_params.push(quote! { #ident });

                let default = match default {
                    Some(default) => {
                        quote! { ::std::option::Option::Some(|| -> #ty { #default }) }
                    }
                    None => quote! { ::std::option::Option::None },
                };

                let param_ident = &ident.ident;
                let process_with = match process_with.as_ref() {
                    Some(fn_path) => quote! { #fn_path(&mut #param_ident); },
                    None => Default::default(),
                };

                let validators = validator.clone().unwrap_or_default().create_validators(
                    &crate_name,
                    quote!(&#ident),
                    Some(quote!(.map_err(|err| err.into_server_error(__pos)))),
                )?;

                let mut non_mut_ident = ident.clone();
                non_mut_ident.mutability = None;
                get_params.push(quote! {
                    #[allow(non_snake_case, unused_mut)]
                    let (__pos, mut #non_mut_ident) = ctx.param_value::<#ty>(#name, #default)?;
                    #process_with
                    #validators
                    #[allow(non_snake_case)]
                    let #ident = #non_mut_ident;
                });
            }

            let ty = match &method.sig.output {
                ReturnType::Type(_, ty) => OutputType::parse(ty)?,
                ReturnType::Default => {
                    return Err(Error::new_spanned(
                        &method.sig.output,
                        "Resolver must have a return type",
                    )
                    .into());
                }
            };
            let res_ty = ty.value_type();
            let stream_ty = if let Type::ImplTrait(TypeImplTrait { bounds, .. }) = &res_ty {
                let mut r = None;
                for b in bounds {
                    if let TypeParamBound::Trait(b) = b {
                        r = Some(quote! { dyn #b });
                    }
                }
                quote! { #r }
            } else {
                quote! { #res_ty }
            };

            if let OutputType::Value(inner_ty) = &ty {
                let block = &method.block;
                let new_block = quote!({
                    {
                        let value = (move || { async move #block })().await;
                        ::std::result::Result::Ok(value)
                    }
                });
                method.block = syn::parse2::<Block>(new_block).expect("invalid block");
                method.sig.output =
                    syn::parse2::<ReturnType>(quote! { -> #crate_name::Result<#inner_ty> })
                        .expect("invalid result type");
            }

            let visible = visible_fn(&field.visible);
            let complexity = if let Some(complexity) = &field.complexity {
                let (variables, expr) = parse_complexity_expr(complexity.clone())?;
                let mut parse_args = Vec::new();
                for variable in variables {
                    if let Some((
                        ident,
                        ty,
                        args::SubscriptionFieldArgument {
                            name,
                            default,
                            default_with,
                            ..
                        },
                    )) = args
                        .iter()
                        .find(|(pat_ident, _, _)| pat_ident.ident == variable)
                    {
                        let default = match generate_default(default, default_with)? {
                            Some(default) => {
                                quote! { ::std::option::Option::Some(|| -> #ty { #default }) }
                            }
                            None => quote! { ::std::option::Option::None },
                        };
                        let name = name.clone().unwrap_or_else(|| {
                            subscription_args
                                .rename_args
                                .rename(ident.ident.unraw().to_string(), RenameTarget::Argument)
                        });
                        parse_args.push(quote! {
                            let #ident: #ty = __ctx.param_value(__variables_definition, __field, #name, #default)?;
                        });
                    }
                }
                quote! {
                    Some(|__ctx, __variables_definition, __field, child_complexity| {
                        #(#parse_args)*
                        ::std::result::Result::Ok(#expr)
                    })
                }
            } else {
                quote! { ::std::option::Option::None }
            };

            let directives =
                gen_directive_calls(&field.directives, TypeDirectiveLocation::FieldDefinition);

            schema_fields.push(quote! {
                #(#cfg_attrs)*
                fields.insert(::std::borrow::ToOwned::to_owned(#field_name), #crate_name::registry::MetaField {
                    name: ::std::borrow::ToOwned::to_owned(#field_name),
                    description: #field_desc,
                    args: {
                        let mut args = #crate_name::indexmap::IndexMap::new();
                        #(#schema_args)*
                        args
                    },
                    ty: <<#stream_ty as #crate_name::futures_util::stream::Stream>::Item as #crate_name::OutputTypeMarker>::create_type_info(registry),
                    deprecation: #field_deprecation,
                    cache_control: ::std::default::Default::default(),
                    external: false,
                    requires: ::std::option::Option::None,
                    provides: ::std::option::Option::None,
                    shareable: false,
                    override_from: ::std::option::Option::None,
                    visible: #visible,
                    inaccessible: false,
                    tags: ::std::default::Default::default(),
                    compute_complexity: #complexity,
                    directive_invocations: ::std::vec![ #(#directives),* ],
                    requires_scopes: ::std::vec![],
                });
            });

            let create_field_stream = quote! {
                self.#ident(ctx, #(#use_params),*)
                    .await
                    .map_err(|err| {
                        ::std::convert::Into::<#crate_name::Error>::into(err).into_server_error(ctx.item.pos)
                            .with_path(::std::vec![#crate_name::PathSegment::Field(::std::borrow::ToOwned::to_owned(&*field_name))])
                    })
            };

            let guard_map_err = quote! {
                .map_err(|err| {
                    err.into_server_error(ctx.item.pos)
                        .with_path(::std::vec![#crate_name::PathSegment::Field(::std::borrow::ToOwned::to_owned(&*field_name))])
                })
            };
            let guard = match field.guard.as_ref().or(subscription_args.guard.as_ref()) {
                Some(code) => Some(generate_guards(&crate_name, code, guard_map_err)?),
                None => None,
            };
            let stream_fn = quote! {
                let field_name = ::std::clone::Clone::clone(&ctx.item.node.response_key().node);
                let field = ::std::sync::Arc::new(::std::clone::Clone::clone(&ctx.item));

                let f = async {
                    #(#get_params)*
                    #guard
                    #create_field_stream
                };
                let stream = f.await.map_err(|err| ctx.set_error_path(err))?;

                let pos = ctx.item.pos;
                let schema_env = ::std::clone::Clone::clone(&ctx.schema_env);
                let query_env = ::std::clone::Clone::clone(&ctx.query_env);
                let stream = #crate_name::futures_util::stream::StreamExt::then(stream, {
                    let field_name = ::std::clone::Clone::clone(&field_name);
                    move |msg| {
                        let schema_env = ::std::clone::Clone::clone(&schema_env);
                        let query_env = ::std::clone::Clone::clone(&query_env);
                        let field = ::std::clone::Clone::clone(&field);
                        let field_name = ::std::clone::Clone::clone(&field_name);
                        async move {
                            let f = |execute_data: ::std::option::Option<#crate_name::Data>| {
                                let schema_env = ::std::clone::Clone::clone(&schema_env);
                                let query_env = ::std::clone::Clone::clone(&query_env);
                                async move {
                                    let ctx_selection_set = query_env.create_context(
                                        &schema_env,
                                        ::std::option::Option::Some(#crate_name::QueryPathNode {
                                            parent: ::std::option::Option::None,
                                            segment: #crate_name::QueryPathSegment::Name(&field_name),
                                        }),
                                        &field.node.selection_set,
                                        execute_data.as_ref(),
                                    );

                                    let parent_type = #gql_typename;
                                    #[allow(bare_trait_objects)]
                                    let ri = #crate_name::extensions::ResolveInfo {
                                        path_node: ctx_selection_set.path_node.as_ref().unwrap(),
                                        parent_type: &parent_type,
                                        return_type: &<<#stream_ty as #crate_name::futures_util::stream::Stream>::Item as #crate_name::OutputTypeMarker>::qualified_type_name(),
                                        name: field.node.name.node.as_str(),
                                        alias: field.node.alias.as_ref().map(|alias| alias.node.as_str()),
                                        is_for_introspection: false,
                                        field: &field.node,
                                    };
                                    let resolve_fut = async {
                                        #crate_name::OutputType::resolve(&msg, &ctx_selection_set, &*field)
                                            .await
                                            .map(::std::option::Option::Some)
                                    };
                                    #crate_name::futures_util::pin_mut!(resolve_fut);
                                    let mut resp = query_env.extensions.resolve(ri, &mut resolve_fut).await.map(|value| {
                                        let mut map = #crate_name::indexmap::IndexMap::new();
                                        map.insert(::std::clone::Clone::clone(&field_name), value.unwrap_or_default());
                                        #crate_name::Response::new(#crate_name::Value::Object(map))
                                    })
                                    .unwrap_or_else(|err| #crate_name::Response::from_errors(::std::vec![err]));

                                    use ::std::iter::Extend;
                                    resp.errors.extend(::std::mem::take(&mut *query_env.errors.lock().unwrap()));
                                    resp
                                }
                            };
                            ::std::result::Result::Ok(query_env.extensions.execute(query_env.operation_name.as_deref(), f).await)
                        }
                    }
                });
                #crate_name::ServerResult::Ok(stream)
            };

            create_stream.push(quote! {
                #(#cfg_attrs)*
                if ctx.item.node.name.node == #field_name {
                    let stream = #crate_name::futures_util::stream::TryStreamExt::try_flatten(
                        #crate_name::futures_util::stream::once((move || async move { #stream_fn })())
                    );
                    let stream = #crate_name::futures_util::StreamExt::map(stream, |res| match res {
                        ::std::result::Result::Ok(resp) => resp,
                        ::std::result::Result::Err(err) => #crate_name::Response::from_errors(::std::vec![err]),
                    });
                    return ::std::option::Option::Some(::std::boxed::Box::pin(stream));
                }
            });

            remove_graphql_attrs(&mut method.attrs);
        }
    }

    if create_stream.is_empty() {
        return Err(Error::new_spanned(
            self_ty,
            "A GraphQL Object type must define one or more fields.",
        )
        .into());
    }

    let visible = visible_fn(&subscription_args.visible);

    let expanded = quote! {
        #item_impl

        #[allow(clippy::all, clippy::pedantic)]
        #[allow(unused_braces, unused_variables)]
        impl #generics #crate_name::SubscriptionType for #self_ty #where_clause {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                #gql_typename
            }

            #[allow(bare_trait_objects)]
            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                registry.create_subscription_type::<Self, _>(|registry| #crate_name::registry::MetaType::Object {
                    name: ::std::borrow::Cow::into_owned(#gql_typename),
                    description: #desc,
                    fields: {
                        let mut fields = #crate_name::indexmap::IndexMap::new();
                        #(#schema_fields)*
                        fields
                    },
                    cache_control: ::std::default::Default::default(),
                    extends: #extends,
                    keys: ::std::option::Option::None,
                    visible: #visible,
                    shareable: false,
                    resolvable: true,
                    inaccessible: false,
                    interface_object: false,
                    tags: ::std::default::Default::default(),
                    is_subscription: true,
                    rust_typename: ::std::option::Option::Some(::std::any::type_name::<Self>()),
                    directive_invocations: ::std::vec![ #(#directives),* ],
                    requires_scopes: ::std::vec![],
                })
            }

            fn create_field_stream<'__life>(
                &'__life self,
                ctx: &'__life #crate_name::Context<'_>,
            ) -> ::std::option::Option<::std::pin::Pin<::std::boxed::Box<dyn #crate_name::futures_util::stream::Stream<Item = #crate_name::Response> + ::std::marker::Send + '__life>>> {
                #(#create_stream)*
                ::std::option::Option::None
            }
        }
    };

    Ok(expanded.into())
}
