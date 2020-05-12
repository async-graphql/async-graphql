use crate::args;
use crate::output_type::OutputType;
use crate::utils::{build_value_repr, check_reserved_name, get_crate_name, get_rustdoc};
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Block, Error, FnArg, ImplItem, ItemImpl, Pat, Result, ReturnType, Type, TypeImplTrait,
    TypeReference,
};

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
    let where_clause = &generics.where_clause;

    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| self_name.clone());
    check_reserved_name(&gql_typename, object_args.internal)?;

    let desc = object_args
        .desc
        .clone()
        .or_else(|| get_rustdoc(&item_impl.attrs).ok().flatten())
        .map(|s| quote! { Some(#s) })
        .unwrap_or_else(|| quote! {None});

    let mut create_stream = Vec::new();
    let mut schema_fields = Vec::new();

    for item in &mut item_impl.items {
        if let ImplItem::Method(method) = item {
            if let Some(field) = args::Field::parse(&crate_name, &method.attrs)? {
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

                if method.sig.asyncness.is_none() {
                    return Err(Error::new_spanned(
                        &method,
                        "The subscription stream function must be asynchronous",
                    ));
                }

                let ty = match &method.sig.output {
                    ReturnType::Type(_, ty) => OutputType::parse(ty)?,
                    ReturnType::Default => {
                        return Err(Error::new_spanned(&method.sig.output, "Missing type"))
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
                                    arg_ident.clone(),
                                    arg_ty.clone(),
                                    args::Argument::parse(&crate_name, &pat.attrs)?,
                                ));
                                pat.attrs.clear();
                            }
                            (arg, Type::Reference(TypeReference { elem, .. })) => {
                                if let Type::Path(path) = elem.as_ref() {
                                    if idx != 1
                                        || path.path.segments.last().unwrap().ident != "Context"
                                    {
                                        return Err(Error::new_spanned(
                                            arg,
                                            "The Context must be the second argument.",
                                        ));
                                    } else {
                                        create_ctx = false;
                                    }
                                }
                            }
                            _ => {
                                return Err(Error::new_spanned(arg, "Incorrect argument type"));
                            }
                        }
                    } else {
                        return Err(Error::new_spanned(arg, "Incorrect argument type"));
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
                        let #ident: #ty = ctx.param_value(#name, #default)?;
                    });
                }

                let res_ty = ty.value_type();
                let stream_ty = if let Type::ImplTrait(TypeImplTrait { bounds, .. }) = &res_ty {
                    quote! { #bounds }
                } else {
                    quote! { #res_ty }
                };

                if let OutputType::Value(inner_ty) = &ty {
                    let block = &method.block;
                    let new_block = quote!({
                        {
                            let value = (move || { async move #block })().await;
                            Ok(value)
                        }
                    });
                    method.block = syn::parse2::<Block>(new_block).expect("invalid block");
                    method.sig.output = syn::parse2::<ReturnType>(
                        quote! { -> #crate_name::FieldResult<#inner_ty> },
                    )
                    .expect("invalid result type");
                }

                schema_fields.push(quote! {
                    fields.insert(#field_name.to_string(), #crate_name::registry::Field {
                        name: #field_name.to_string(),
                        description: #field_desc,
                        args: {
                            let mut args = std::collections::HashMap::new();
                            #(#schema_args)*
                            args
                        },
                        ty: <<#stream_ty as #crate_name::futures::stream::Stream>::Item as #crate_name::Type>::create_type_info(registry),
                        deprecation: #field_deprecation,
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                    });
                });

                let create_field_stream = quote! {
                    #crate_name::futures::stream::StreamExt::fuse(self.#ident(ctx, #(#use_params),*).await.
                        map_err(|err| err.into_error_with_path(ctx.position(), ctx.path_node.as_ref().unwrap().to_json()))?)
                };

                let guard = field.guard.map(|guard| quote! {
                    #guard.check(ctx).await.map_err(|err| err.into_error_with_path(ctx.position(), ctx.path_node.as_ref().unwrap().to_json()))?;
                });

                create_stream.push(quote! {
                    if ctx.name.node == #field_name {
                        use #crate_name::futures::stream::{StreamExt, TryStreamExt};

                        #(#get_params)*
                        #guard
                        let field_name = std::sync::Arc::new(ctx.result_name().to_string());

                        // I think the code here is safe because the lifetime of selection_set is always less than the environment.
                        let field_selection_set = unsafe {
                            (&ctx.selection_set
                                as *const Positioned<#crate_name::parser::ast::SelectionSet>)
                                .as_ref()
                                .unwrap()
                        };

                        let schema = schema.clone();
                        let pos = ctx.position();
                        let environment = environment.clone();
                        let stream = #create_field_stream.then({
                            let field_name = field_name.clone();
                            move |msg| {
                                let environment = environment.clone();
                                let field_selection_set = field_selection_set.clone();
                                let schema = schema.clone();
                                let field_name = field_name.clone();
                                async move {
                                    let resolve_id = std::sync::atomic::AtomicUsize::default();
                                    let ctx_selection_set = environment.create_context(
                                        &schema,
                                        Some(#crate_name::QueryPathNode {
                                            parent: None,
                                            segment: #crate_name::QueryPathSegment::Name(&field_name),
                                        }),
                                        field_selection_set,
                                        &resolve_id,
                                    );
                                    #crate_name::OutputValueType::resolve(&msg, &ctx_selection_set, pos).await
                                }
                            }
                        })
                        .map_ok(move |value| #crate_name::serde_json::json!({ field_name.as_str(): value }))
                        .scan(true, |state, item| {
                            if !*state {
                                return #crate_name::futures::future::ready(None);
                            }
                            if item.is_err() {
                                *state = false;
                            }
                            return #crate_name::futures::future::ready(Some(item));
                        });
                        return Ok(Box::pin(stream));
                    }
                });
            }

            if let Some((idx, _)) = method
                .attrs
                .iter()
                .enumerate()
                .find(|(_, a)| a.path.is_ident("field"))
            {
                method.attrs.remove(idx);
            }
        }
    }

    let expanded = quote! {
        #item_impl

        impl #generics #crate_name::Type for #self_ty #where_clause {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#gql_typename)
            }

            #[allow(bare_trait_objects)]
            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> String {
                registry.create_type::<Self, _>(|registry| #crate_name::registry::Type::Object {
                    name: #gql_typename.to_string(),
                    description: #desc,
                    fields: {
                        let mut fields = std::collections::HashMap::new();
                        #(#schema_fields)*
                        fields
                    },
                    cache_control: Default::default(),
                    extends: false,
                    keys: None,
                })
            }
        }

        #[#crate_name::async_trait::async_trait]
        impl #crate_name::SubscriptionType for #self_ty #where_clause {
            #[allow(unused_variables)]
            #[allow(bare_trait_objects)]
            async fn create_field_stream<Query, Mutation>(
                &self,
                idx: usize,
                ctx: &#crate_name::Context<'_>,
                schema: &#crate_name::Schema<Query, Mutation, Self>,
                environment: std::sync::Arc<#crate_name::Environment>,
            ) -> #crate_name::Result<std::pin::Pin<Box<dyn #crate_name::futures::Stream<Item = #crate_name::Result<#crate_name::serde_json::Value>> + Send>>>
            where
                Query: #crate_name::ObjectType + Send + Sync + 'static,
                Mutation: #crate_name::ObjectType + Send + Sync + 'static,
                Self: Send + Sync + 'static + Sized,
            {
                use #crate_name::futures::StreamExt;

                #(#create_stream)*
                Err(#crate_name::QueryError::FieldNotFound {
                    field_name: ctx.name.to_string(),
                    object: #gql_typename.to_string(),
                }.into_error(ctx.position()))
            }
        }
    };
    Ok(expanded.into())
}
