use proc_macro::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::{
    Block, Error, FnArg, ImplItem, ItemImpl, Pat, ReturnType, Type, TypeImplTrait, TypeParamBound,
    TypeReference,
};

use crate::args::{self, RenameRuleExt, RenameTarget, SubscriptionField};
use crate::output_type::OutputType;
use crate::utils::{
    generate_default, generate_guards, generate_validator, get_cfg_attrs, get_crate_name,
    get_param_getter_ident, get_rustdoc, get_type_path_and_name, parse_graphql_attrs,
    remove_graphql_attrs, visible_fn, GeneratorResult,
};

pub fn generate(
    subscription_args: &args::Subscription,
    item_impl: &mut ItemImpl,
) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(subscription_args.internal);
    let (self_ty, self_name) = get_type_path_and_name(item_impl.self_ty.as_ref())?;
    let generics = &item_impl.generics;
    let where_clause = &generics.where_clause;

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
            let field_deprecation = field
                .deprecation
                .as_ref()
                .map(|s| quote! {::std::option::Option::Some(#s)})
                .unwrap_or_else(|| quote! {::std::option::Option::None});
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
                    return Err(Error::new_spanned(&method.sig.output, "Missing type").into())
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
                },
            ) in args
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
                let default = generate_default(&default, &default_with)?;

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

                let visible = visible_fn(&arg_visible);
                schema_args.push(quote! {
                    args.insert(#name, #crate_name::registry::MetaInputValue {
                        name: #name,
                        description: #desc,
                        ty: <#ty as #crate_name::Type>::create_type_info(registry),
                        default_value: #schema_default,
                        validator: #validator,
                        visible: #visible,
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
                        r = Some(quote! { #b });
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
                    ty: <<#stream_ty as #crate_name::futures_util::stream::Stream>::Item as #crate_name::Type>::create_type_info(registry),
                    deprecation: #field_deprecation,
                    cache_control: ::std::default::Default::default(),
                    external: false,
                    requires: ::std::option::Option::None,
                    provides: ::std::option::Option::None,
                    visible: #visible,
                    compute_complexity: ::std::option::Option::None,
                });
            });

            let create_field_stream = quote! {
                self.#ident(ctx, #(#use_params),*)
                    .await
                    .map_err(|err| {
                        err.into_server_error().at(ctx.item.pos)
                    })?
            };

            let guard = match &field.guard {
                Some(meta_list) => generate_guards(&crate_name, meta_list)?,
                None => None,
            };
            let guard = guard.map(|guard| quote! {
                #guard.check(ctx).await.map_err(|err| err.into_server_error().at(ctx.item.pos))?;
            });

            let stream_fn = quote! {
                #(#get_params)*
                #guard
                let field_name = ::std::sync::Arc::new(::std::clone::Clone::clone(&ctx.item.node.response_key().node));
                let field = ::std::sync::Arc::new(::std::clone::Clone::clone(&ctx.item));

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
                            let resolve_id = #crate_name::ResolveId {
                                parent: ::std::option::Option::Some(0),
                                current: 1,
                            };
                            let inc_resolve_id = ::std::sync::atomic::AtomicUsize::new(1);
                            let ctx_selection_set = query_env.create_context(
                                &schema_env,
                                ::std::option::Option::Some(#crate_name::QueryPathNode {
                                    parent: ::std::option::Option::None,
                                    segment: #crate_name::QueryPathSegment::Name(&field_name),
                                }),
                                &field.node.selection_set,
                                resolve_id,
                                &inc_resolve_id,
                            );
                            let ctx_extension = #crate_name::extensions::ExtensionContext {
                                schema_data: &schema_env.data,
                                query_data: &query_env.ctx_data,
                            };

                            query_env.extensions.execution_start(&ctx_extension);

                            #[allow(bare_trait_objects)]
                            let ri = #crate_name::extensions::ResolveInfo {
                                resolve_id,
                                path_node: ctx_selection_set.path_node.as_ref().unwrap(),
                                parent_type: #gql_typename,
                                return_type: &<<#stream_ty as #crate_name::futures_util::stream::Stream>::Item as #crate_name::Type>::qualified_type_name(),
                            };

                            query_env.extensions.resolve_start(&ctx_extension, &ri);

                            let res = #crate_name::OutputType::resolve(&msg, &ctx_selection_set, &*field).await;

                            query_env.extensions.resolve_end(&ctx_extension, &ri);
                            query_env.extensions.execution_end(&ctx_extension);

                            res
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
                        if item.is_err() {
                            *errored = true;
                        }
                        #crate_name::futures_util::future::ready(::std::option::Option::Some(item))
                    },
                ))
            };

            create_stream.push(quote! {
                #(#cfg_attrs)*
                if ctx.item.node.name.node == #field_name {
                    return ::std::option::Option::Some(::std::boxed::Box::pin(
                        #crate_name::futures_util::stream::TryStreamExt::try_flatten(
                            #crate_name::futures_util::stream::once((move || async move { #stream_fn })())
                        )
                    ));
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
        impl #generics #crate_name::Type for #self_ty #where_clause {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed(#gql_typename)
            }

            #[allow(bare_trait_objects)]
            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                registry.create_type::<Self, _>(|registry| #crate_name::registry::MetaType::Object {
                    name: ::std::borrow::ToOwned::to_owned(#gql_typename),
                    description: #desc,
                    fields: {
                        let mut fields = #crate_name::indexmap::IndexMap::new();
                        #(#schema_fields)*
                        fields
                    },
                    cache_control: ::std::default::Default::default(),
                    extends: false,
                    keys: ::std::option::Option::None,
                    visible: ::std::option::Option::None,
                })
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #[allow(unused_braces, unused_variables)]
        impl #crate_name::SubscriptionType for #self_ty #where_clause {
            fn create_field_stream<'__life>(
                &'__life self,
                ctx: &'__life #crate_name::Context<'__life>,
            ) -> ::std::option::Option<::std::pin::Pin<::std::boxed::Box<dyn #crate_name::futures_util::stream::Stream<Item = #crate_name::ServerResult<#crate_name::Value>> + ::std::marker::Send + '__life>>> {
                #(#create_stream)*
                ::std::option::Option::None
            }
        }
    };
    Ok(expanded.into())
}
