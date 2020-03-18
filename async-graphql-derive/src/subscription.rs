use crate::args;
use crate::utils::{build_value_repr, get_crate_name};
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, FnArg, ImplItem, ItemImpl, Pat, Result, ReturnType, Type};

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

    let mut create_types = Vec::new();
    let mut filters = Vec::new();
    let mut schema_fields = Vec::new();

    for item in &mut item_impl.items {
        if let ImplItem::Method(method) = item {
            if let Some(field) = args::Field::parse(&method.attrs)? {
                let ident = &method.sig.ident;
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

                if method.sig.inputs.len() < 2 {
                    return Err(Error::new_spanned(
                        &method.sig.inputs,
                        "The filter function needs at least two arguments",
                    ));
                }

                if method.sig.asyncness.is_some() {
                    return Err(Error::new_spanned(
                        &method.sig.inputs,
                        "The filter function must be synchronous",
                    ));
                }

                let mut res_typ_ok = false;
                if let ReturnType::Type(_, res_ty) = &method.sig.output {
                    if let Type::Path(p) = res_ty.as_ref() {
                        if p.path.is_ident("bool") {
                            res_typ_ok = true;
                        }
                    }
                }
                if !res_typ_ok {
                    return Err(Error::new_spanned(
                        &method.sig.output,
                        "The filter function must return a boolean value",
                    ));
                }

                match &method.sig.inputs[0] {
                    FnArg::Receiver(_) => {}
                    _ => {
                        return Err(Error::new_spanned(
                            &method.sig.inputs[0],
                            "The first argument must be self receiver",
                        ));
                    }
                }

                let ty = if let FnArg::Typed(ty) = &method.sig.inputs[1] {
                    match ty.ty.as_ref() {
                        Type::Reference(r) => r.elem.as_ref().clone(),
                        _ => {
                            return Err(Error::new_spanned(ty, "Incorrect object type"));
                        }
                    }
                } else {
                    return Err(Error::new_spanned(
                        &method.sig.inputs[1],
                        "Incorrect object type",
                    ));
                };

                let mut args = Vec::new();

                for arg in method.sig.inputs.iter_mut().skip(2) {
                    if let FnArg::Typed(pat) = arg {
                        match (&*pat.pat, &*pat.ty) {
                            (Pat::Ident(arg_ident), Type::Path(arg_ty)) => {
                                args.push((arg_ident, arg_ty, args::Argument::parse(&pat.attrs)?));
                                pat.attrs.clear();
                            }
                            _ => {
                                return Err(Error::new_spanned(arg, "Incorrect argument type"));
                            }
                        }
                    } else {
                        return Err(Error::new_spanned(arg, "Incorrect argument type"));
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
                            ty: <#ty as #crate_name::GQLType>::create_type_info(registry),
                            default_value: #schema_default,
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
                        let #ident: #ty = ctx_field.param_value(#name, #default)?;
                    });
                }

                schema_fields.push(quote! {
                    fields.insert(#field_name, #crate_name::registry::Field {
                        name: #field_name,
                        description: #field_desc,
                        args: {
                            let mut args = std::collections::HashMap::new();
                            #(#schema_args)*
                            args
                        },
                        ty: <#ty as #crate_name::GQLType>::create_type_info(registry),
                        deprecation: #field_deprecation,
                    });
                });

                create_types.push(quote! {
                    if field.name.as_str() == #field_name {
                        types.insert(std::any::TypeId::of::<#ty>(), field.clone());
                        return Ok(());
                    }
                });

                filters.push(quote! {
                    if let Some(msg) = msg.downcast_ref::<#ty>() {
                        #(#get_params)*
                        if self.#ident(msg, #(#use_params)*) {
                            let ctx_selection_set = ctx_field.with_item(&field.selection_set);
                            let value =
                                #crate_name::GQLOutputValue::resolve(msg, &ctx_selection_set).await?;
                            return Ok(Some(value));
                        }
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
                    fields: {
                        let mut fields = std::collections::HashMap::new();
                        #(#schema_fields)*
                        fields
                    },
                })
            }
        }

        #[#crate_name::async_trait::async_trait]
        impl #crate_name::GQLSubscription for SubscriptionRoot {
            fn create_type(field: &#crate_name::graphql_parser::query::Field, types: &mut std::collections::HashMap<std::any::TypeId, #crate_name::graphql_parser::query::Field>) -> #crate_name::Result<()> {
                use #crate_name::ErrorWithPosition;
                #(#create_types)*
                #crate_name::anyhow::bail!(#crate_name::QueryError::FieldNotFound {
                    field_name: field.name.clone(),
                    object: #gql_typename.to_string(),
                }
                .with_position(field.position));
            }

            async fn resolve(
                &self,
                ctx: &#crate_name::ContextBase<'_, ()>,
                types: &std::collections::HashMap<std::any::TypeId, #crate_name::graphql_parser::query::Field>,
                msg: &(dyn std::any::Any + Send + Sync),
            ) -> #crate_name::Result<Option<serde_json::Value>> {
                let tid = msg.type_id();
                if let Some(field) = types.get(&tid) {
                    let ctx_field = ctx.with_item(field);
                    #(#filters)*
                }
                Ok(None)
            }
        }
    };
    Ok(expanded.into())
}
