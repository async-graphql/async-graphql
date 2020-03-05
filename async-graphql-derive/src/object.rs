use crate::args;
use crate::utils::{build_value_repr, get_crate_name};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Error, FnArg, GenericArgument, ImplItem, ItemImpl, Pat, PathArguments, Result, ReturnType,
    Type, TypeReference,
};

enum OutputType<'a> {
    Value(&'a Type),
    ValueRef(&'a TypeReference),
    Result(&'a Type, &'a Type),
}

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

    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| self_name.clone());
    let desc = object_args
        .desc
        .as_ref()
        .map(|s| quote! {Some(#s)})
        .unwrap_or_else(|| quote! {None});

    let mut resolvers = Vec::new();
    let mut schema_fields = Vec::new();

    for item in &mut item_impl.items {
        if let ImplItem::Method(method) = item {
            if let Some(field) = args::Field::parse(&method.attrs)? {
                let field_name = field
                    .name
                    .clone()
                    .unwrap_or_else(|| method.sig.ident.to_string());
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
                let ty = match &method.sig.output {
                    ReturnType::Type(_, ty) => {
                        if let Type::Path(p) = ty.as_ref() {
                            if p.path.is_ident("Result") {
                                if let PathArguments::AngleBracketed(args) =
                                    &p.path.segments[0].arguments
                                {
                                    if args.args.len() == 0 {
                                        return Err(Error::new_spanned(
                                            &method.sig.output,
                                            "Invalid type",
                                        ));
                                    }
                                    let mut res = None;
                                    for arg in &args.args {
                                        if let GenericArgument::Type(value_ty) = arg {
                                            res = Some(OutputType::Result(ty, value_ty));
                                            break;
                                        }
                                    }
                                    if res.is_none() {
                                        return Err(Error::new_spanned(
                                            &method.sig.output,
                                            "Invalid type",
                                        ));
                                    }
                                    res.unwrap()
                                } else {
                                    return Err(Error::new_spanned(
                                        &method.sig.output,
                                        "Invalid type",
                                    ));
                                }
                            } else {
                                OutputType::Value(ty)
                            }
                        } else if let Type::Reference(ty) = ty.as_ref() {
                            OutputType::ValueRef(ty)
                        } else {
                            return Err(Error::new_spanned(&method.sig.output, "Invalid type"));
                        }
                    }
                    ReturnType::Default => {
                        return Err(Error::new_spanned(&method.sig.output, "Missing type"));
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
                            // 第一个参数必须是self
                            return Err(Error::new_spanned(
                                pat,
                                "The self receiver must be the first parameter.",
                            ));
                        }

                        match (&*pat.pat, &*pat.ty) {
                            (Pat::Ident(arg_ident), Type::Path(arg_ty)) => {
                                args.push((arg_ident, arg_ty, args::Argument::parse(&pat.attrs)?));
                                pat.attrs.clear();
                            }
                            (_, Type::Reference(TypeReference { elem, .. })) => {
                                if let Type::Path(path) = elem.as_ref() {
                                    if idx != 1
                                        || path.path.segments.last().unwrap().ident.to_string()
                                            != "Context"
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
                    },
                ) in args
                {
                    let name = name.clone().unwrap_or_else(|| ident.ident.to_string());
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
                        #crate_name::registry::InputValue {
                            name: #name,
                            description: #desc,
                            ty: <#ty as #crate_name::GQLType>::create_type_info(registry),
                            default_value: #schema_default,
                        }
                    });

                    use_params.push(quote! { #ident });

                    let default = match &default {
                        Some(default) => {
                            let repr = build_value_repr(&crate_name, &default);
                            quote! {Some(|| #repr) }
                        }
                        None => quote! { None },
                    };
                    get_params.push(quote! {
                        let #ident: #ty = ctx_field.param_value(#name, #default)?;
                    });
                }

                let schema_ty = match &ty {
                    OutputType::Result(_, value_ty) => value_ty,
                    OutputType::Value(value_ty) => value_ty,
                    OutputType::ValueRef(r) => r.elem.as_ref(),
                };
                schema_fields.push(quote! {
                    #crate_name::registry::Field {
                        name: #field_name,
                        description: #field_desc,
                        args: vec![#(#schema_args),*],
                        ty: <#schema_ty as #crate_name::GQLType>::create_type_info(registry),
                        deprecation: #field_deprecation,
                    }
                });

                let ctx_field = match arg_ctx {
                    true => quote! { &ctx_field, },
                    false => quote! {},
                };

                let field_ident = &method.sig.ident;
                let resolve_obj = match &ty {
                    OutputType::Value(_) | OutputType::ValueRef(_) => quote! {
                        self.#field_ident(#ctx_field #(#use_params),*).await
                    },
                    OutputType::Result(_, _) => {
                        quote! {
                            self.#field_ident(#ctx_field #(#use_params),*).await.
                                map_err(|err| err.with_position(field.position))?
                        }
                    }
                };

                resolvers.push(quote! {
                    if field.name.as_str() == #field_name {
                        #(#get_params)*
                        let obj = #resolve_obj;
                        let ctx_obj = ctx_field.with_item(&field.selection_set);
                        let value = obj.resolve(&ctx_obj).await.
                            map_err(|err| err.with_position(field.position))?;
                        let name = field.alias.clone().unwrap_or_else(|| field.name.clone());
                        result.insert(name, value.into());
                        continue;
                    }
                });

                method.attrs.clear();
            }
        }
    }

    let expanded = quote! {
        #item_impl

        impl #generics #crate_name::GQLType for #self_ty {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> String {
                registry.create_type::<Self, _>(|registry| #crate_name::registry::Type::Object {
                    name: #gql_typename,
                    description: #desc,
                    fields: vec![#(#schema_fields),*]
                })
            }
        }

        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::GQLOutputValue for #self_ty {
            async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>) -> #crate_name::Result<#crate_name::serde_json::Value> {
                use #crate_name::ErrorWithPosition;

                if ctx.items.is_empty() {
                    #crate_name::anyhow::bail!(#crate_name::QueryError::MustHaveSubFields {
                        object: #gql_typename,
                    }.with_position(ctx.span.0));
                }

                let mut result = #crate_name::serde_json::Map::<String, #crate_name::serde_json::Value>::new();
                for selection in &ctx.items {
                    match selection {
                        #crate_name::graphql_parser::query::Selection::Field(field) => {
                            let ctx_field = ctx.with_item(field);
                            if ctx_field.is_skip_this()? {
                                continue;
                            }
                            if field.name.as_str() == "__typename" {
                                let name = field.alias.clone().unwrap_or_else(|| field.name.clone());
                                result.insert(name, #gql_typename.into());
                                continue;
                            }
                            if field.name.as_str() == "__schema" {
                                continue;
                            }
                            #(#resolvers)*
                            #crate_name::anyhow::bail!(#crate_name::QueryError::FieldNotFound {
                                field_name: field.name.clone(),
                                object: #gql_typename,
                            }.with_position(field.position));
                        }
                        _ => {}
                    }
                }

                Ok(#crate_name::serde_json::Value::Object(result))
            }
        }

        impl#generics #crate_name::GQLObject for #self_ty {}
    };
    Ok(expanded.into())
}
