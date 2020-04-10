use crate::args;
use crate::output_type::OutputType;
use crate::utils::{build_value_repr, check_reserved_name, get_crate_name};
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, FnArg, ImplItem, ItemImpl, Pat, Result, ReturnType, Type, TypeReference};

pub fn generate(object_args: &args::Object, item_impl: &mut ItemImpl) -> Result<TokenStream> {
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
        _ => return Err(Error::new_spanned(&item_impl.self_ty, "Invalid type")),
    };
    let generics = &item_impl.generics;
    let extends = object_args.extends;

    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| self_name.clone());
    check_reserved_name(&gql_typename, object_args.internal)?;

    let desc = object_args
        .desc
        .as_ref()
        .map(|s| quote! {Some(#s)})
        .unwrap_or_else(|| quote! {None});

    let mut resolvers = Vec::new();
    let mut schema_fields = Vec::new();
    let mut find_entities = Vec::new();
    let mut add_keys = Vec::new();
    let mut create_entity_types = Vec::new();

    for item in &mut item_impl.items {
        if let ImplItem::Method(method) = item {
            if let Some(field) = args::Field::parse(&method.attrs)? {
                if method.sig.asyncness.is_none() {
                    return Err(Error::new_spanned(
                        &method.sig.output,
                        "Must be asynchronous",
                    ));
                }

                let field_name = field
                    .name
                    .clone()
                    .unwrap_or_else(|| method.sig.ident.to_string().to_camel_case());
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
                let ty = match &method.sig.output {
                    ReturnType::Type(_, ty) => OutputType::parse(ty)?,
                    ReturnType::Default => {
                        return Err(Error::new_spanned(&method.sig.output, "Missing type"))
                    }
                };
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

                let mut arg_ctx = false;
                let mut args = Vec::new();

                for (idx, arg) in method.sig.inputs.iter_mut().enumerate() {
                    if let FnArg::Receiver(receiver) = arg {
                        if idx != 0 {
                            return Err(Error::new_spanned(
                                receiver,
                                "The self receiver must be the first parameter.",
                            ));
                        }
                    } else if let FnArg::Typed(pat) = arg {
                        if idx == 0 {
                            return Err(Error::new_spanned(
                                pat,
                                "The self receiver must be the first parameter.",
                            ));
                        }

                        match (&*pat.pat, &*pat.ty) {
                            (Pat::Ident(arg_ident), Type::Path(arg_ty)) => {
                                args.push((
                                    arg_ident,
                                    arg_ty,
                                    args::Argument::parse(&crate_name, &pat.attrs)?,
                                ));
                                pat.attrs.clear();
                            }
                            (_, Type::Reference(TypeReference { elem, .. })) => {
                                if let Type::Path(path) = elem.as_ref() {
                                    if idx != 1
                                        || path.path.segments.last().unwrap().ident != "Context"
                                    {
                                        return Err(Error::new_spanned(
                                            arg,
                                            "The Context must be the second argument.",
                                        ));
                                    }
                                    arg_ctx = true;
                                }
                            }
                            _ => return Err(Error::new_spanned(arg, "Invalid argument type.")),
                        }
                    }
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
                        validator,
                    },
                ) in args
                {
                    let name = name
                        .clone()
                        .unwrap_or_else(|| ident.ident.to_string().to_camel_case());
                    let desc = desc
                        .as_ref()
                        .map(|s| quote! {Some(#s)})
                        .unwrap_or_else(|| quote! {None});
                    let schema_default = default
                        .as_ref()
                        .map(|v| {
                            let s = v.to_string();
                            quote! {Some(#s)}
                        })
                        .unwrap_or_else(|| quote! {None});

                    schema_args.push(quote! {
                        args.insert(#name, #crate_name::registry::InputValue {
                            name: #name,
                            description: #desc,
                            ty: <#ty as #crate_name::Type>::create_type_info(registry),
                            default_value: #schema_default,
                            validator: #validator,
                        });
                    });

                    use_params.push(quote! { #ident });

                    let default = match &default {
                        Some(default) => {
                            let repr = build_value_repr(&crate_name, &default);
                            quote! {|| #repr }
                        }
                        None => quote! { || #crate_name::Value::Null },
                    };

                    get_params.push(quote! {
                        let #ident: #ty = ctx.param_value(#name, field.position, #default)?;
                    });
                }

                let schema_ty = ty.value_type();

                schema_fields.push(quote! {
                    fields.insert(#field_name.to_string(), #crate_name::registry::Field {
                        name: #field_name.to_string(),
                        description: #field_desc,
                        args: {
                            let mut args = std::collections::HashMap::new();
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

                let ctx_param = if arg_ctx {
                    quote! { &ctx, }
                } else {
                    quote! {}
                };

                let field_ident = &method.sig.ident;
                let resolve_obj = match &ty {
                    OutputType::Value(_) => quote! {
                        self.#field_ident(#ctx_param #(#use_params),*).await
                    },
                    OutputType::Result(_, _) => {
                        quote! {
                            {
                                let res:#crate_name::FieldResult<_> = self.#field_ident(#ctx_param #(#use_params),*).await;
                                res.map_err(|err| err.into_error_with_path(field.position, ctx.path_node.as_ref().unwrap().to_json()))?
                            }
                        }
                    }
                };

                resolvers.push(quote! {
                    if field.name.as_str() == #field_name {
                        #(#get_params)*
                        let ctx_obj = ctx.with_selection_set(&field.selection_set);
                        return #crate_name::OutputValueType::resolve(&#resolve_obj, &ctx_obj, field.position).await;
                    }
                });

                method.attrs.remove(
                    method
                        .attrs
                        .iter()
                        .enumerate()
                        .find(|(_, a)| a.path.is_ident("field"))
                        .map(|(idx, _)| idx)
                        .unwrap(),
                );
            } else if method.attrs.iter().any(|attr| attr.path.is_ident("entity")) {
                let ty = match &method.sig.output {
                    ReturnType::Type(_, ty) => OutputType::parse(ty)?,
                    ReturnType::Default => {
                        return Err(Error::new_spanned(&method.sig.output, "Missing type"))
                    }
                };
                let mut arg_ctx = false;
                let mut args = Vec::new();

                for (idx, arg) in method.sig.inputs.iter_mut().enumerate() {
                    if let FnArg::Receiver(receiver) = arg {
                        if idx != 0 {
                            return Err(Error::new_spanned(
                                receiver,
                                "The self receiver must be the first parameter.",
                            ));
                        }
                    } else if let FnArg::Typed(pat) = arg {
                        if idx == 0 {
                            return Err(Error::new_spanned(
                                pat,
                                "The self receiver must be the first parameter.",
                            ));
                        }

                        match (&*pat.pat, &*pat.ty) {
                            (Pat::Ident(arg_ident), Type::Path(arg_ty)) => {
                                args.push((
                                    arg_ident,
                                    arg_ty,
                                    args::Argument::parse(&crate_name, &pat.attrs)?,
                                ));
                                pat.attrs.clear();
                            }
                            (_, Type::Reference(TypeReference { elem, .. })) => {
                                if let Type::Path(path) = elem.as_ref() {
                                    if idx != 1
                                        || path.path.segments.last().unwrap().ident != "Context"
                                    {
                                        return Err(Error::new_spanned(
                                            arg,
                                            "The Context must be the second argument.",
                                        ));
                                    }
                                    arg_ctx = true;
                                }
                            }
                            _ => return Err(Error::new_spanned(arg, "Invalid argument type.")),
                        }
                    }
                }

                let entity_type = ty.value_type();
                let mut key_pat = Vec::new();
                let mut key_getter = Vec::new();
                let mut use_keys = Vec::new();
                let mut keys = Vec::new();
                let mut keys_str = String::new();

                for (ident, ty, args::Argument { name, .. }) in &args {
                    let name = name
                        .clone()
                        .unwrap_or_else(|| ident.ident.to_string().to_camel_case());

                    if !keys_str.is_empty() {
                        keys_str.push(' ');
                    }
                    keys_str.push_str(&name);

                    key_pat.push(quote! {
                        Some(#ident)
                    });
                    key_getter.push(quote! {
                        params.get(#name).and_then(|value| {
                            let value: Option<#ty> = #crate_name::InputValueType::parse(value);
                            value
                        })
                    });
                    keys.push(name);
                    use_keys.push(ident);
                }
                add_keys.push(quote! { registry.add_keys(&<#entity_type as #crate_name::Type>::type_name(), #keys_str); });
                create_entity_types.push(quote! { <#entity_type as #crate_name::Type>::create_type_info(registry); });

                let field_ident = &method.sig.ident;
                let ctx_param = if arg_ctx {
                    quote! { &ctx, }
                } else {
                    quote! {}
                };
                let do_find = match &ty {
                    OutputType::Value(_) => quote! {
                        self.#field_ident(#ctx_param #(#use_keys),*).await
                    },
                    OutputType::Result(_, _) => {
                        quote! { self.#field_ident(#ctx_param #(#use_keys),*).await? }
                    }
                };

                find_entities.push((
                    args.len(),
                    quote! {
                        if typename == &<#entity_type as #crate_name::Type>::type_name() {
                            if let (#(#key_pat),*) = (#(#key_getter),*) {
                                let ctx_obj = ctx.with_selection_set(&ctx.selection_set);
                                return #crate_name::OutputValueType::resolve(&#do_find, &ctx_obj, pos).await;
                            }
                        }
                    },
                ));

                method.attrs.remove(
                    method
                        .attrs
                        .iter()
                        .enumerate()
                        .find(|(_, a)| a.path.is_ident("entity"))
                        .map(|(idx, _)| idx)
                        .unwrap(),
                );
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

    find_entities.sort_by(|(a, _), (b, _)| b.cmp(a));
    let find_entities_iter = find_entities.iter().map(|(_, code)| code);

    let expanded = quote! {
        #item_impl

        impl #generics #crate_name::Type for #self_ty {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> String {
                let ty = registry.create_type::<Self, _>(|registry| #crate_name::registry::Type::Object {
                    name: #gql_typename.to_string(),
                    description: #desc,
                    fields: {
                        let mut fields = std::collections::HashMap::new();
                        #(#schema_fields)*
                        fields
                    },
                    cache_control: #cache_control,
                    extends: #extends,
                    keys: None,
                });
                #(#create_entity_types)*
                #(#add_keys)*
                ty
            }
        }

        #[#crate_name::async_trait::async_trait]
        impl#generics #crate_name::ObjectType for #self_ty {
            async fn resolve_field(&self, ctx: &#crate_name::Context<'_>, field: &#crate_name::graphql_parser::query::Field) -> #crate_name::Result<#crate_name::serde_json::Value> {
                #(#resolvers)*
                Err(#crate_name::QueryError::FieldNotFound {
                    field_name: field.name.clone(),
                    object: #gql_typename.to_string(),
                }.into_error(field.position))
            }

            async fn find_entity(&self, ctx: &#crate_name::Context<'_>, pos: #crate_name::Pos, params: &#crate_name::Value) -> #crate_name::Result<serde_json::Value> {
                let params = match params {
                    #crate_name::Value::Object(params) => params,
                    _ => return Err(#crate_name::QueryError::EntityNotFound.into_error(pos)),
                };
                let typename = if let Some(#crate_name::Value::String(typename)) = params.get("__typename") {
                    typename
                } else {
                    return Err(#crate_name::QueryError::TypeNameNotExists.into_error(pos));
                };
                #(#find_entities_iter)*
                Err(#crate_name::QueryError::EntityNotFound.into_error(pos))
            }
        }

        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::OutputValueType for #self_ty {
            async fn resolve(value: &Self, ctx: &#crate_name::ContextSelectionSet<'_>, pos: #crate_name::Pos) -> #crate_name::Result<#crate_name::serde_json::Value> {
                #crate_name::do_resolve(ctx, value).await
            }
        }
    };
    Ok(expanded.into())
}
