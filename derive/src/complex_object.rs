use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use std::iter::FromIterator;
use std::str::FromStr;
use syn::ext::IdentExt;
use syn::{
    punctuated::Punctuated, Block, Error, FnArg, ImplItem, ItemImpl, Pat, ReturnType, Token, Type,
    TypeReference,
};

use crate::args::{self, ComplexityType, RenameRuleExt, RenameTarget};
use crate::output_type::OutputType;
use crate::utils::{
    extract_input_args, gen_deprecation, generate_default, generate_guards, generate_validator,
    get_cfg_attrs, get_crate_name, get_param_getter_ident, get_rustdoc, get_type_path_and_name,
    parse_complexity_expr, parse_graphql_attrs, remove_graphql_attrs, visible_fn, GeneratorResult,
};

pub fn generate(
    object_args: &args::ComplexObject,
    item_impl: &mut ItemImpl,
) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let (self_ty, _) = get_type_path_and_name(item_impl.self_ty.as_ref())?;
    let generics = &item_impl.generics;
    let where_clause = &item_impl.generics.where_clause;

    let mut resolvers = Vec::new();
    let mut schema_fields = Vec::new();

    // Computation of the derivated fields
    let mut derived_impls = vec![];
    for item in &mut item_impl.items {
        if let ImplItem::Method(method) = item {
            let method_args: args::ObjectField =
                parse_graphql_attrs(&method.attrs)?.unwrap_or_default();

            for derived in method_args.derived {
                if derived.name.is_some() && derived.into.is_some() {
                    let base_function_name = &method.sig.ident;
                    let name = derived.name.unwrap();
                    let with = derived.with;
                    let into = Type::Verbatim(
                        proc_macro2::TokenStream::from_str(&derived.into.unwrap()).unwrap(),
                    );

                    let mut new_impl = method.clone();
                    new_impl.sig.ident = name;
                    new_impl.sig.output =
                        syn::parse2::<ReturnType>(quote! { -> #crate_name::Result<#into> })
                            .expect("invalid result type");

                    let should_create_context = new_impl
                        .sig
                        .inputs
                        .iter()
                        .nth(1)
                        .map(|x| {
                            if let FnArg::Typed(pat) = x {
                                if let Type::Reference(TypeReference { elem, .. }) = &*pat.ty {
                                    if let Type::Path(path) = elem.as_ref() {
                                        return path.path.segments.last().unwrap().ident
                                            != "Context";
                                    }
                                }
                            };
                            true
                        })
                        .unwrap_or(true);

                    if should_create_context {
                        let arg_ctx = syn::parse2::<FnArg>(quote! { ctx: &Context<'_> })
                            .expect("invalid arg type");
                        new_impl.sig.inputs.insert(1, arg_ctx);
                    }

                    let other_atts: Punctuated<Ident, Token![,]> = Punctuated::from_iter(
                        new_impl
                            .sig
                            .inputs
                            .iter()
                            .filter_map(|x| match x {
                                FnArg::Typed(pat) => match &*pat.pat {
                                    Pat::Ident(ident) => Some(Ok(ident.ident.clone())),
                                    _ => Some(Err(Error::new_spanned(
                                        &pat,
                                        "Must be a simple argument",
                                    ))),
                                },
                                FnArg::Receiver(_) => None,
                            })
                            .collect::<Result<Vec<Ident>, Error>>()?
                            .into_iter(),
                    );

                    let new_block = match with {
                        Some(with) => quote!({
                            ::std::result::Result::Ok(#with(#self_ty::#base_function_name(&self, #other_atts).await?))
                        }),
                        None => quote!({
                            {
                                ::std::result::Result::Ok(#self_ty::#base_function_name(&self, #other_atts).await?.into())
                            }
                        }),
                    };

                    new_impl.block = syn::parse2::<Block>(new_block).expect("invalid block");

                    derived_impls.push(ImplItem::Method(new_impl));
                }
            }
        }
    }
    item_impl.items.append(&mut derived_impls);

    for item in &mut item_impl.items {
        if let ImplItem::Method(method) = item {
            let method_args: args::ObjectField =
                parse_graphql_attrs(&method.attrs)?.unwrap_or_default();
            if method_args.skip {
                remove_graphql_attrs(&mut method.attrs);
                continue;
            }

            let field_name = method_args.name.clone().unwrap_or_else(|| {
                object_args
                    .rename_fields
                    .rename(method.sig.ident.unraw().to_string(), RenameTarget::Field)
            });
            let field_desc = get_rustdoc(&method.attrs)?
                .map(|s| quote! { ::std::option::Option::Some(#s) })
                .unwrap_or_else(|| quote! {::std::option::Option::None});
            let field_deprecation = gen_deprecation(&method_args.deprecation, &crate_name);
            let external = method_args.external;
            let requires = match &method_args.requires {
                Some(requires) => quote! { ::std::option::Option::Some(#requires) },
                None => quote! { ::std::option::Option::None },
            };
            let provides = match &method_args.provides {
                Some(provides) => quote! { ::std::option::Option::Some(#provides) },
                None => quote! { ::std::option::Option::None },
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

            let args = extract_input_args(&crate_name, method)?;
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
                    visible,
                    secret,
                    ..
                },
            ) in &args
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

                let validator = match &validator {
                    Some(meta) => {
                        let stream = generate_validator(&crate_name, meta)?;
                        quote!(::std::option::Option::Some(#stream))
                    }
                    None => quote!(::std::option::Option::None),
                };

                let visible = visible_fn(visible);
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

                let param_ident = &ident.ident;
                use_params.push(quote! { #param_ident });

                let default = match default {
                    Some(default) => {
                        quote! { ::std::option::Option::Some(|| -> #ty { #default }) }
                    }
                    None => quote! { ::std::option::Option::None },
                };
                // We're generating a new identifier,
                // so remove the 'r#` prefix if present
                let param_getter_name = get_param_getter_ident(&ident.ident.unraw().to_string());
                get_params.push(quote! {
                        #[allow(non_snake_case)]
                        let #param_getter_name = || -> #crate_name::ServerResult<#ty> { ctx.param_value(#name, #default) };
                        #[allow(non_snake_case)]
                        let #ident: #ty = #param_getter_name()?;
                    });
            }

            let schema_ty = ty.value_type();
            let visible = visible_fn(&method_args.visible);

            let complexity = if let Some(complexity) = &method_args.complexity {
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
                                args::Argument {
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
                                    object_args.rename_args.rename(
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
                                Ok(#expr)
                            }))
                        }
                    }
                }
            } else {
                quote! { ::std::option::Option::None }
            };

            schema_fields.push(quote! {
                #(#cfg_attrs)*
                fields.push((#field_name.to_string(), #crate_name::registry::MetaField {
                    name: ::std::borrow::ToOwned::to_owned(#field_name),
                    description: #field_desc,
                    args: {
                        let mut args = #crate_name::indexmap::IndexMap::new();
                        #(#schema_args)*
                        args
                    },
                    ty: <#schema_ty as #crate_name::OutputType>::create_type_info(registry),
                    deprecation: #field_deprecation,
                    cache_control: #cache_control,
                    external: #external,
                    provides: #provides,
                    requires: #requires,
                    visible: #visible,
                    compute_complexity: #complexity,
                }));
            });

            let field_ident = &method.sig.ident;
            if let OutputType::Value(inner_ty) = &ty {
                let block = &method.block;
                let new_block = quote!({
                    {
                        ::std::result::Result::Ok(async move {
                            let value:#inner_ty = #block;
                            value
                        }.await)
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
                    res.map_err(|err| ::std::convert::Into::<#crate_name::Error>::into(err).into_server_error(ctx.item.pos))?
                }
            };

            let guard = match &method_args.guard {
                Some(meta_list) => generate_guards(&crate_name, meta_list)?,
                None => None,
            };

            let guard = guard.map(|guard| {
                quote! {
                    #guard.check(ctx).await
                        .map_err(|err| err.into_server_error(ctx.item.pos))?;
                }
            });

            resolvers.push(quote! {
                #(#cfg_attrs)*
                if ctx.item.node.name.node == #field_name {
                    #(#get_params)*
                    #guard
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    let res = #resolve_obj;
                    return #crate_name::OutputType::resolve(&res, &ctx_obj, ctx.item).await.map(::std::option::Option::Some);
                }
            });

            remove_graphql_attrs(&mut method.attrs);
        }
    }

    let expanded = quote! {
        #item_impl

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::ComplexObject for #self_ty #where_clause {
            fn fields(registry: &mut #crate_name::registry::Registry) -> ::std::vec::Vec<(::std::string::String, #crate_name::registry::MetaField)> {
                let mut fields = ::std::vec::Vec::new();
                #(#schema_fields)*
                fields
            }

            async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                #(#resolvers)*
                ::std::result::Result::Ok(::std::option::Option::None)
            }
        }
    };
    Ok(expanded.into())
}
