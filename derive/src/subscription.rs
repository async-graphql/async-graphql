use proc_macro::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::{
    Block, Error, FnArg, ImplItem, ItemImpl, Pat, ReturnType, Type, TypeImplTrait, TypeParamBound,
    TypeReference,
};

use crate::args::{self, ComplexityType, RenameRuleExt, RenameTarget, SubscriptionField};
use crate::output_type::OutputType;
use crate::utils::{
    gen_deprecation, generate_default, generate_guards, generate_validator, get_cfg_attrs,
    get_crate_name, get_param_getter_ident, get_rustdoc, get_type_path_and_name,
    parse_complexity_expr, parse_graphql_attrs, remove_graphql_attrs, visible_fn, GeneratorResult,
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

    let gql_typename = subscription_args
        .name
        .clone()
        .unwrap_or_else(|| RenameTarget::Type.rename(self_name.clone()));

    let desc = if subscription_args.use_type_description {
        quote! { ::std::option::Option::Some(<Self as #crate_name::Description>::description()) }
    } else {
        get_rustdoc(&item_impl.attrs)?
            .map(|s| quote!(::std::option::Option::Some(#s)))
            .unwrap_or_else(|| quote!(::std::option::Option::None))
    };

    let mut create_stream = Vec::new();
    let mut schema_fields = Vec::new();

    for item in &mut item_impl.items {
        if let ImplItem::Method(method) = item {
            let field: SubscriptionField = parse_graphql_attrs(&method.attrs)?.unwrap_or_default();
            if field.skip {
                remove_graphql_attrs(&mut method.attrs);
                continue;
            }

            let ident = &method.sig.ident;
            let field_name = field.name.clone().unwrap_or_else(|| {
                subscription_args
                    .rename_fields
                    .rename(method.sig.ident.unraw().to_string(), RenameTarget::Field)
            });
            let field_desc = get_rustdoc(&method.attrs)?
                .map(|s| quote! {::std::option::Option::Some(#s)})
                .unwrap_or_else(|| quote! {::std::option::Option::None});
            let field_deprecation = gen_deprecation(&field.deprecation, &crate_name);
            let cfg_attrs = get_cfg_attrs(&method.attrs);

            if method.sig.asyncness.is_none() {
                return Err(Error::new_spanned(
                    &method,
                    "The subscription stream function must be asynchronous",
                )
                .into());
            }

            let ty = match &method.sig.output {
                ReturnType::Type(_, ty) => OutputType::parse(ty)?,
                ReturnType::Default => {
                    return Err(Error::new_spanned(
                        &method.sig.output,
                        "Resolver must have a return type",
                    )
                    .into())
                }
            };

            let mut create_ctx = true;
            let mut args = Vec::new();

            for (idx, arg) in method.sig.inputs.iter_mut().enumerate() {
                if let FnArg::Receiver(receiver) = arg {
                    if idx != 0 {
                        return Err(Error::new_spanned(
                            receiver,
                            "The self receiver must be the first parameter.",
                        )
                        .into());
                    }
                } else if let FnArg::Typed(pat) = arg {
                    if idx == 0 {
                        return Err(Error::new_spanned(
                            pat,
                            "The self receiver must be the first parameter.",
                        )
                        .into());
                    }

                    match (&*pat.pat, &*pat.ty) {
                        (Pat::Ident(arg_ident), Type::Path(arg_ty)) => {
                            args.push((
                                arg_ident.clone(),
                                arg_ty.clone(),
                                parse_graphql_attrs::<args::SubscriptionFieldArgument>(&pat.attrs)?
                                    .unwrap_or_default(),
                            ));
                            remove_graphql_attrs(&mut pat.attrs);
                        }
                        (arg, Type::Reference(TypeReference { elem, .. })) => {
                            if let Type::Path(path) = elem.as_ref() {
                                if idx != 1 || path.path.segments.last().unwrap().ident != "Context"
                                {
                                    return Err(Error::new_spanned(
                                        arg,
                                        "Only types that implement `InputType` can be used as input arguments.",
                                    )
                                    .into());
                                } else {
                                    create_ctx = false;
                                }
                            }
                        }
                        _ => {
                            return Err(Error::new_spanned(arg, "Incorrect argument type").into());
                        }
                    }
                } else {
                    return Err(Error::new_spanned(arg, "Incorrect argument type").into());
                }
            }

            if create_ctx {
                let arg = syn::parse2::<FnArg>(quote! { _: &#crate_name::Context<'_> }).unwrap();
                method.sig.inputs.insert(1, arg);
            }

            let mut schema_args = Vec::new();
            let mut use_params = Vec::new();
            let mut get_params = Vec::new();

            for (
                ident,
                ty,
                args::SubscriptionFieldArgument {
                    name,
                    desc,
                    default,
                    default_with,
                    validator,
                    visible: arg_visible,
                    secret,
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
                    .map(|s| quote! {::std::option::Option::Some(#s)})
                    .unwrap_or_else(|| quote! {::std::option::Option::None});
                let default = generate_default(default, default_with)?;

                let validator = match &validator {
                    Some(meta) => {
                        let stream = generate_validator(&crate_name, meta)?;
                        quote!(::std::option::Option::Some(#stream))
                    }
                    None => quote!(::std::option::Option::None),
                };

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
                schema_args.push(quote! {
                    args.insert(#name, #crate_name::registry::MetaInputValue {
                        name: #name,
                        description: #desc,
                        ty: <#ty as #crate_name::InputType>::create_type_info(registry),
                        default_value: #schema_default,
                        validator: #validator,
                        visible: #visible,
                        is_secret: #secret,
                    });
                });

                use_params.push(quote! { #ident });

                let default = match default {
                    Some(default) => quote! { ::std::option::Option::Some(|| -> #ty { #default }) },
                    None => quote! { ::std::option::Option::None },
                };
                let param_getter_name = get_param_getter_ident(&ident.ident.to_string());
                get_params.push(quote! {
                    #[allow(non_snake_case)]
                    let #param_getter_name = || -> #crate_name::ServerResult<#ty> { ctx.param_value(#name, #default) };
                    #[allow(non_snake_case)]
                    let #ident: #ty = ctx.param_value(#name, #default)?;
                });
            }

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
                match complexity {
                    ComplexityType::Const(n) => {
                        quote! { ::std::option::Option::Some(#crate_name::registry::ComplexityType::Const(#n)) }
                    }
                    ComplexityType::Fn(s) => {
                        let (variables, expr) = parse_complexity_expr(s)?;
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
                                    subscription_args.rename_args.rename(
                                        ident.ident.unraw().to_string(),
                                        RenameTarget::Argument,
                                    )
                                });
                                parse_args.push(quote! {
                                        let #ident: #ty = __ctx.param_value(__variables_definition, __field, #name, #default)?;
                                    });
                            }
                        }
                        quote! {
                            Some(#crate_name::registry::ComplexityType::Fn(|__ctx, __variables_definition, __field, child_complexity| {
                                #(#parse_args)*
                                ::std::result::Result::Ok(#expr)
                            }))
                        }
                    }
                }
            } else {
                quote! { ::std::option::Option::None }
            };

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
                    ty: <<#stream_ty as #crate_name::futures_util::stream::Stream>::Item as #crate_name::OutputType>::create_type_info(registry),
                    deprecation: #field_deprecation,
                    cache_control: ::std::default::Default::default(),
                    external: false,
                    requires: ::std::option::Option::None,
                    provides: ::std::option::Option::None,
                    visible: #visible,
                    compute_complexity: #complexity,
                });
            });

            let create_field_stream = quote! {
                self.#ident(ctx, #(#use_params),*)
                    .await
                    .map_err(|err| {
                        ::std::convert::Into::<#crate_name::Error>::into(err).into_server_error(ctx.item.pos)
                            .with_path(::std::vec![#crate_name::PathSegment::Field(::std::borrow::ToOwned::to_owned(&*field_name))])
                    })?
            };

            let guard = match &field.guard {
                Some(meta_list) => generate_guards(&crate_name, meta_list)?,
                None => None,
            };
            let guard = guard.map(|guard| quote! {
                #guard.check(ctx).await.map_err(|err| {
                    err.into_server_error(ctx.item.pos)
                        .with_path(::std::vec![#crate_name::PathSegment::Field(::std::borrow::ToOwned::to_owned(&*field_name))])
                })?;
            });
            let stream_fn = quote! {
                let field_name = ::std::clone::Clone::clone(&ctx.item.node.response_key().node);
                let field = ::std::sync::Arc::new(::std::clone::Clone::clone(&ctx.item));
                #(#get_params)*
                #guard

                let pos = ctx.item.pos;
                let schema_env = ::std::clone::Clone::clone(&ctx.schema_env);
                let query_env = ::std::clone::Clone::clone(&ctx.query_env);
                let stream = #crate_name::futures_util::stream::StreamExt::then(#create_field_stream, {
                    let field_name = ::std::clone::Clone::clone(&field_name);
                    move |msg| {
                        let schema_env = ::std::clone::Clone::clone(&schema_env);
                        let query_env = ::std::clone::Clone::clone(&query_env);
                        let field = ::std::clone::Clone::clone(&field);
                        let field_name = ::std::clone::Clone::clone(&field_name);
                        async move {
                            let ctx_selection_set = query_env.create_context(
                                &schema_env,
                                ::std::option::Option::Some(#crate_name::QueryPathNode {
                                    parent: ::std::option::Option::None,
                                    segment: #crate_name::QueryPathSegment::Name(&field_name),
                                }),
                                &field.node.selection_set,
                            );

                            let mut execute_fut = async {
                                #[allow(bare_trait_objects)]
                                let ri = #crate_name::extensions::ResolveInfo {
                                    path_node: ctx_selection_set.path_node.as_ref().unwrap(),
                                    parent_type: #gql_typename,
                                    return_type: &<<#stream_ty as #crate_name::futures_util::stream::Stream>::Item as #crate_name::OutputType>::qualified_type_name(),
                                    name: field.node.name.node.as_str(),
                                    alias: field.node.alias.as_ref().map(|alias| alias.node.as_str()),
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
                            };
                            #crate_name::futures_util::pin_mut!(execute_fut);
                            ::std::result::Result::Ok(query_env.extensions.execute(query_env.operation_name.as_deref(), &mut execute_fut).await)
                        }
                    }
                });
                #crate_name::ServerResult::Ok(#crate_name::futures_util::stream::StreamExt::scan(
                    stream,
                    false,
                    |errored, item| {
                        if *errored {
                            return #crate_name::futures_util::future::ready(::std::option::Option::None);
                        }
                        match &item {
                            ::std::result::Result::Err(_) => *errored = true,
                            ::std::result::Result::Ok(resp) if resp.is_err() => *errored = true,
                            _ => {}
                        }
                        #crate_name::futures_util::future::ready(::std::option::Option::Some(item))
                    },
                ))
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
            &self_ty,
            "A GraphQL Object type must define one or more fields.",
        )
        .into());
    }

    let expanded = quote! {
        #item_impl

        #[allow(clippy::all, clippy::pedantic)]
        #[allow(unused_braces, unused_variables)]
        impl #generics #crate_name::SubscriptionType for #self_ty #where_clause {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed(#gql_typename)
            }

            #[allow(bare_trait_objects)]
            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                registry.create_subscription_type::<Self, _>(|registry| #crate_name::registry::MetaType::Object {
                    name: ::std::borrow::ToOwned::to_owned(#gql_typename),
                    description: #desc,
                    fields: {
                        let mut fields = #crate_name::indexmap::IndexMap::new();
                        #(#schema_fields)*
                        fields
                    },
                    cache_control: ::std::default::Default::default(),
                    extends: #extends,
                    keys: ::std::option::Option::None,
                    visible: ::std::option::Option::None,
                    is_subscription: true,
                    rust_typename: ::std::any::type_name::<Self>(),
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
