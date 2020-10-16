use proc_macro::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::{Block, Error, FnArg, ImplItem, ItemImpl, Pat, ReturnType, Type, TypeReference};

use crate::args::{self, RenameRuleExt, RenameTarget};
use crate::output_type::OutputType;
use crate::utils::{
    generate_default, generate_guards, generate_validator, get_cfg_attrs, get_crate_name,
    get_param_getter_ident, get_rustdoc, parse_graphql_attrs, remove_graphql_attrs,
    GeneratorResult,
};

pub fn generate(
    object_args: &args::Object,
    item_impl: &mut ItemImpl,
) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let (self_ty, self_name) = match item_impl.self_ty.as_ref() {
        Type::Path(path) => (
            path,
            path.path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap(),
        ),
        _ => return Err(Error::new_spanned(&item_impl.self_ty, "Invalid type").into()),
    };
    let generics = &item_impl.generics;
    let where_clause = &item_impl.generics.where_clause;
    let extends = object_args.extends;
    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| RenameTarget::Type.rename(self_name.clone()));

    let desc = get_rustdoc(&item_impl.attrs)?
        .map(|s| quote!(::std::option::Option::Some(#s)))
        .unwrap_or_else(|| quote!(::std::option::Option::None));

    let mut resolvers = Vec::new();
    let mut schema_fields = Vec::new();
    let mut find_entities = Vec::new();
    let mut add_keys = Vec::new();
    let mut create_entity_types = Vec::new();

    for item in &mut item_impl.items {
        if let ImplItem::Method(method) = item {
            let method_args: args::ObjectField =
                parse_graphql_attrs(&method.attrs)?.unwrap_or_default();

            if method_args.entity {
                let cfg_attrs = get_cfg_attrs(&method.attrs);

                if method.sig.asyncness.is_none() {
                    return Err(Error::new_spanned(&method, "Must be asynchronous").into());
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
                                    parse_graphql_attrs::<args::Argument>(&pat.attrs)?
                                        .unwrap_or_default(),
                                ));
                                remove_graphql_attrs(&mut pat.attrs);
                            }
                            (arg, Type::Reference(TypeReference { elem, .. })) => {
                                if let Type::Path(path) = elem.as_ref() {
                                    if idx != 1
                                        || path.path.segments.last().unwrap().ident != "Context"
                                    {
                                        return Err(Error::new_spanned(
                                            arg,
                                            "The Context must be the second argument.",
                                        )
                                        .into());
                                    } else {
                                        create_ctx = false;
                                    }
                                }
                            }
                            _ => {
                                return Err(Error::new_spanned(arg, "Invalid argument type.").into())
                            }
                        }
                    }
                }

                if create_ctx {
                    let arg =
                        syn::parse2::<FnArg>(quote! { _: &#crate_name::Context<'_> }).unwrap();
                    method.sig.inputs.insert(1, arg);
                }

                let entity_type = ty.value_type();
                let mut key_pat = Vec::new();
                let mut key_getter = Vec::new();
                let mut use_keys = Vec::new();
                let mut keys = Vec::new();
                let mut keys_str = String::new();
                let mut requires_getter = Vec::new();
                let all_key = args.iter().all(|(_, _, arg)| !arg.key);

                if args.is_empty() {
                    return Err(Error::new_spanned(
                        method,
                        "Entity need to have at least one key.",
                    )
                    .into());
                }

                for (ident, ty, args::Argument { name, key, .. }) in &args {
                    let is_key = all_key || *key;
                    let name = name.clone().unwrap_or_else(|| {
                        object_args
                            .rename_args
                            .rename(ident.ident.unraw().to_string(), RenameTarget::Argument)
                    });

                    if is_key {
                        if !keys_str.is_empty() {
                            keys_str.push(' ');
                        }
                        keys_str.push_str(&name);

                        key_pat.push(quote! {
                            ::std::option::Option::Some(#ident)
                        });
                        key_getter.push(quote! {
                            params.get(#name).and_then(|value| {
                                let value: ::std::option::Option<#ty> = #crate_name::InputValueType::parse(::std::option::Option::Some(::std::clone::Clone::clone(&value))).ok();
                                value
                            })
                        });
                        keys.push(name);
                        use_keys.push(ident);
                    } else {
                        // requires
                        requires_getter.push(quote! {
                            let #ident: #ty = #crate_name::InputValueType::parse(params.get(#name).cloned()).
                                map_err(|err| err.into_server_error().at(ctx.item.pos))?;
                        });
                        use_keys.push(ident);
                    }
                }

                add_keys.push(quote! { registry.add_keys(&<#entity_type as #crate_name::Type>::type_name(), #keys_str); });
                create_entity_types.push(
                    quote! { <#entity_type as #crate_name::Type>::create_type_info(registry); },
                );

                let field_ident = &method.sig.ident;
                if let OutputType::Value(inner_ty) = &ty {
                    let block = &method.block;
                    let new_block = quote!({
                        {
                            let value:#inner_ty = async move #block.await;
                            ::std::result::Result::Ok(value)
                        }
                    });
                    method.block = syn::parse2::<Block>(new_block).expect("invalid block");
                    method.sig.output =
                        syn::parse2::<ReturnType>(quote! { -> #crate_name::Result<#inner_ty> })
                            .expect("invalid result type");
                }
                let do_find = quote! { self.#field_ident(ctx, #(#use_keys),*).await.map_err(|err| err.into_server_error().at(ctx.item.pos))? };

                find_entities.push((
                    args.len(),
                    quote! {
                        #(#cfg_attrs)*
                        if typename == &<#entity_type as #crate_name::Type>::type_name() {
                            if let (#(#key_pat),*) = (#(#key_getter),*) {
                                #(#requires_getter)*
                                let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                                return #crate_name::OutputValueType::resolve(&#do_find, &ctx_obj, ctx.item).await.map(::std::option::Option::Some);
                            }
                        }
                    },
                ));
            } else if !method_args.skip {
                if method.sig.asyncness.is_none() {
                    return Err(Error::new_spanned(&method, "Must be asynchronous").into());
                }

                let field_name = method_args.name.clone().unwrap_or_else(|| {
                    object_args
                        .rename_fields
                        .rename(method.sig.ident.unraw().to_string(), RenameTarget::Field)
                });
                let field_desc = get_rustdoc(&method.attrs)?
                    .map(|s| quote! { ::std::option::Option::Some(#s) })
                    .unwrap_or_else(|| quote! {::std::option::Option::None});
                let field_deprecation = method_args
                    .deprecation
                    .as_ref()
                    .map(|s| quote! { ::std::option::Option::Some(#s) })
                    .unwrap_or_else(|| quote! {::std::option::Option::None});
                let external = method_args.external;
                let requires = match &method_args.requires {
                    Some(requires) => quote! { ::std::option::Option::Some(#requires) },
                    None => quote! { ::std::option::Option::None },
                };
                let provides = match &method_args.provides {
                    Some(provides) => quote! { ::std::option::Option::Some(#provides) },
                    None => quote! { ::std::option::Option::None },
                };
                let ty = match &method.sig.output {
                    ReturnType::Type(_, ty) => OutputType::parse(ty)?,
                    ReturnType::Default => {
                        return Err(Error::new_spanned(&method.sig.output, "Missing type").into())
                    }
                };
                let cache_control = {
                    let public = method_args.cache_control.is_public();
                    let max_age = method_args.cache_control.max_age;
                    quote! {
                        #crate_name::CacheControl {
                            public: #public,
                            max_age: #max_age,
                        }
                    }
                };
                let cfg_attrs = get_cfg_attrs(&method.attrs);

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
                                    parse_graphql_attrs::<args::Argument>(&pat.attrs)?
                                        .unwrap_or_default(),
                                ));
                                remove_graphql_attrs(&mut pat.attrs);
                            }
                            (arg, Type::Reference(TypeReference { elem, .. })) => {
                                if let Type::Path(path) = elem.as_ref() {
                                    if idx != 1
                                        || path.path.segments.last().unwrap().ident != "Context"
                                    {
                                        return Err(Error::new_spanned(
                                            arg,
                                            "The Context must be the second argument.",
                                        )
                                        .into());
                                    }

                                    create_ctx = false;
                                }
                            }
                            _ => {
                                return Err(Error::new_spanned(arg, "Invalid argument type.").into())
                            }
                        }
                    }
                }

                if create_ctx {
                    let arg =
                        syn::parse2::<FnArg>(quote! { _: &#crate_name::Context<'_> }).unwrap();
                    method.sig.inputs.insert(1, arg);
                }

                let mut schema_args = Vec::new();
                let mut use_params = Vec::new();
                let mut get_params = Vec::new();

                for (
                    ident,
                    ty,
                    args::Argument {
                        name,
                        desc,
                        default,
                        default_with,
                        validator,
                        ..
                    },
                ) in args
                {
                    let name = name.clone().unwrap_or_else(|| {
                        object_args
                            .rename_args
                            .rename(ident.ident.unraw().to_string(), RenameTarget::Argument)
                    });
                    let desc = desc
                        .as_ref()
                        .map(|s| quote! {::std::option::Option::Some(#s)})
                        .unwrap_or_else(|| quote! {::std::option::Option::None});
                    let default = generate_default(&default, &default_with)?;
                    let schema_default = default
                        .as_ref()
                        .map(|value| {
                            quote! {
                                ::std::option::Option::Some(::std::string::ToString::to_string(
                                    &<#ty as #crate_name::InputValueType>::to_value(&#value)
                                ))
                            }
                        })
                        .unwrap_or_else(|| quote! {::std::option::Option::None});

                    let validator = match &validator {
                        Some(meta) => {
                            let stream = generate_validator(&crate_name, meta)?;
                            quote!(::std::option::Option::Some(#stream))
                        }
                        None => quote!(::std::option::Option::None),
                    };

                    schema_args.push(quote! {
                        args.insert(#name, #crate_name::registry::MetaInputValue {
                            name: #name,
                            description: #desc,
                            ty: <#ty as #crate_name::Type>::create_type_info(registry),
                            default_value: #schema_default,
                            validator: #validator,
                        });
                    });

                    let param_ident = &ident.ident;
                    use_params.push(quote! { #param_ident });

                    let default = match default {
                        Some(default) => {
                            quote! { ::std::option::Option::Some(|| -> #ty { #default }) }
                        }
                        None => quote! { ::std::option::Option::None },
                    };
                    let param_getter_name = get_param_getter_ident(&ident.ident.to_string());
                    get_params.push(quote! {
                        #[allow(non_snake_case)]
                        let #param_getter_name = || -> #crate_name::ServerResult<#ty> { ctx.param_value(#name, #default) };
                        #[allow(non_snake_case)]
                        let #ident: #ty = #param_getter_name()?;
                    });
                }

                let schema_ty = ty.value_type();

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
                        ty: <#schema_ty as #crate_name::Type>::create_type_info(registry),
                        deprecation: #field_deprecation,
                        cache_control: #cache_control,
                        external: #external,
                        provides: #provides,
                        requires: #requires,
                    });
                });

                let field_ident = &method.sig.ident;
                if let OutputType::Value(inner_ty) = &ty {
                    let block = &method.block;
                    let new_block = quote!({
                        {
                            let value:#inner_ty = async move #block.await;
                            ::std::result::Result::Ok(value)
                        }
                    });
                    method.block = syn::parse2::<Block>(new_block).expect("invalid block");
                    method.sig.output =
                        syn::parse2::<ReturnType>(quote! { -> #crate_name::Result<#inner_ty> })
                            .expect("invalid result type");
                }

                let resolve_obj = quote! {
                    {
                        let res = self.#field_ident(ctx, #(#use_params),*).await;
                        res.map_err(|err| err.into_server_error().at(ctx.item.pos))?
                    }
                };

                let guard = match &method_args.guard {
                    Some(meta_list) => generate_guards(&crate_name, meta_list)?,
                    None => None,
                };

                let guard = guard.map(|guard| {
                    quote! {
                        #guard.check(ctx).await
                            .map_err(|err| err.into_server_error().at(ctx.item.pos))?;
                    }
                });

                resolvers.push(quote! {
                    #(#cfg_attrs)*
                    if ctx.item.node.name.node == #field_name {
                        #(#get_params)*
                        #guard
                        let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                        let res = #resolve_obj;
                        return #crate_name::OutputValueType::resolve(&res, &ctx_obj, ctx.item).await.map(::std::option::Option::Some);
                    }
                });
            }

            remove_graphql_attrs(&mut method.attrs);
        }
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

    find_entities.sort_by(|(a, _), (b, _)| b.cmp(a));
    let find_entities_iter = find_entities.iter().map(|(_, code)| code);

    let expanded = quote! {
        #item_impl

        #[allow(clippy::all, clippy::pedantic)]
        impl #generics #crate_name::Type for #self_ty #where_clause {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                let ty = registry.create_type::<Self, _>(|registry| #crate_name::registry::MetaType::Object {
                    name: ::std::borrow::ToOwned::to_owned(#gql_typename),
                    description: #desc,
                    fields: {
                        let mut fields = #crate_name::indexmap::IndexMap::new();
                        #(#schema_fields)*
                        fields
                    },
                    cache_control: #cache_control,
                    extends: #extends,
                    keys: ::std::option::Option::None,
                });
                #(#create_entity_types)*
                #(#add_keys)*
                ty
            }
        }

        #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
        #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
        #[#crate_name::async_trait::async_trait]
        impl#generics #crate_name::resolver_utils::ContainerType for #self_ty #where_clause {
            async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                #(#resolvers)*
                ::std::result::Result::Ok(::std::option::Option::None)
            }

            async fn find_entity(&self, ctx: &#crate_name::Context<'_>, params: &#crate_name::Value) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                let params = match params {
                    #crate_name::Value::Object(params) => params,
                    _ => return ::std::result::Result::Ok(::std::option::Option::None),
                };
                let typename = if let ::std::option::Option::Some(#crate_name::Value::String(typename)) = params.get("__typename") {
                    typename
                } else {
                    return ::std::result::Result::Err(
                        #crate_name::ServerError::new(r#""__typename" must be an existing string."#)
                            .at(ctx.item.pos)
                    );
                };
                #(#find_entities_iter)*
                ::std::result::Result::Ok(::std::option::Option::None)
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::OutputValueType for #self_ty #where_clause {
            async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::Value> {
                #crate_name::resolver_utils::resolve_container(ctx, self).await
            }
        }

        impl #generics #crate_name::ObjectType for #self_ty #where_clause {}
    };
    Ok(expanded.into())
}
