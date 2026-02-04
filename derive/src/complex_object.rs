use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    Block, Error, FnArg, ImplItem, ItemImpl, Pat, ReturnType, Token, Type, TypeReference,
    ext::IdentExt, punctuated::Punctuated,
};

use crate::{
    args::{self, RenameRuleExt, RenameTarget, TypeDirectiveLocation},
    output_type::OutputType,
    utils::{
        GeneratorResult, extract_input_args, gen_boxed_trait, gen_deprecation, gen_directive_calls,
        generate_default, generate_guards, get_cfg_attrs, get_crate_path, get_rustdoc,
        get_type_path_and_name, parse_complexity_expr, parse_graphql_attrs, remove_graphql_attrs,
        visible_fn,
    },
};

pub fn generate(
    object_args: &args::ComplexObject,
    item_impl: &mut ItemImpl,
) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_path(&object_args.crate_path, object_args.internal);
    let boxed_trait = gen_boxed_trait(&crate_name);
    let (self_ty, _) = get_type_path_and_name(item_impl.self_ty.as_ref())?;
    let generics = &item_impl.generics;
    let where_clause = &item_impl.generics.where_clause;

    let mut resolvers = Vec::new();
    let mut schema_fields = Vec::new();

    // Computation of the derived fields
    let mut derived_impls = vec![];
    for item in &mut item_impl.items {
        if let ImplItem::Fn(method) = item {
            let method_args: args::ComplexObjectField =
                parse_graphql_attrs(&method.attrs)?.unwrap_or_default();

            for derived in method_args.derived {
                if let Some(name) = derived.name
                    && let Some(into) = derived.into
                {
                    let base_function_name = &method.sig.ident;
                    let with = derived.with;
                    let into = Type::Verbatim(proc_macro2::TokenStream::from_str(&into).unwrap());

                    let mut new_impl = method.clone();
                    new_impl.sig.ident = name;
                    new_impl.sig.output =
                        syn::parse2::<ReturnType>(quote! { -> #crate_name::Result<#into> })
                            .expect("invalid result type");

                    let should_create_context = new_impl.sig.inputs.iter().nth(1).is_none_or(|x| {
                        if let FnArg::Typed(pat) = x
                            && let Type::Reference(TypeReference { elem, .. }) = &*pat.ty
                            && let Type::Path(path) = elem.as_ref()
                        {
                            return path.path.segments.last().unwrap().ident != "Context";
                        };
                        true
                    });

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
                                        pat,
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

                    derived_impls.push(ImplItem::Fn(new_impl));
                }
            }
        }
    }
    item_impl.items.append(&mut derived_impls);

    for item in &mut item_impl.items {
        if let ImplItem::Fn(method) = item {
            let method_args: args::ComplexObjectField =
                parse_graphql_attrs(&method.attrs)?.unwrap_or_default();
            if method_args.skip {
                remove_graphql_attrs(&mut method.attrs);
                continue;
            }
            let is_async = method.sig.asyncness.is_some();
            let cfg_attrs = get_cfg_attrs(&method.attrs);

            if method_args.flatten {
                // Only used to inject the context placeholder if required.
                extract_input_args::<args::Argument>(&crate_name, method)?;

                let ty = match &method.sig.output {
                    ReturnType::Type(_, ty) => OutputType::parse(ty)?,
                    ReturnType::Default => {
                        return Err(Error::new_spanned(
                            &method.sig.output,
                            "Flatten resolver must have a return type",
                        )
                        .into());
                    }
                };
                let ty = ty.value_type();
                let ident = &method.sig.ident;

                schema_fields.push(quote! {
                    #crate_name::static_assertions_next::assert_impl_one!(#ty: #crate_name::ObjectType);
                    <#ty>::create_type_info(registry);
                    if let #crate_name::registry::MetaType::Object { fields: obj_fields, .. } =
                        registry.create_fake_output_type::<#ty>() {
                        fields.extend(obj_fields);
                    }
                });

                let flattened_resolver = if is_async {
                    quote! {
                        #(#cfg_attrs)*
                        if let ::std::option::Option::Some(value) = #crate_name::ContainerType::resolve_field(&self.#ident(ctx).await, ctx).await? {
                            return ::std::result::Result::Ok(std::option::Option::Some(value));
                        }
                    }
                } else {
                    quote! {
                        #(#cfg_attrs)*
                        let value = self.#ident(ctx);
                        if let ::std::option::Option::Some(value) = #crate_name::ContainerType::resolve_field(&value, ctx).await? {
                            return ::std::result::Result::Ok(std::option::Option::Some(value));
                        }
                    }
                };
                resolvers.push(flattened_resolver);

                remove_graphql_attrs(&mut method.attrs);
                continue;
            }

            let field_name = method_args.name.clone().unwrap_or_else(|| {
                object_args
                    .rename_fields
                    .rename(method.sig.ident.unraw().to_string(), RenameTarget::Field)
            });
            let field_desc_value = get_rustdoc(&method.attrs)?;
            let has_field_desc = field_desc_value.is_some();
            let field_desc = field_desc_value
                .map(|s| quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#s)) })
                .unwrap_or_else(|| quote! {::std::option::Option::None});
            let field_deprecation = gen_deprecation(&method_args.deprecation, &crate_name);
            let external = method_args.external;
            let shareable = method_args.shareable;
            let directives = gen_directive_calls(
                &crate_name,
                &method_args.directives,
                TypeDirectiveLocation::FieldDefinition,
            );
            let override_from = match &method_args.override_from {
                Some(from) => {
                    quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#from)) }
                }
                None => quote! { ::std::option::Option::None },
            };
            let inaccessible = method_args.inaccessible;
            let tags = method_args
                .tags
                .iter()
                .map(|tag| quote!(::std::string::ToString::to_string(#tag)))
                .collect::<Vec<_>>();
            let requires_scopes = method_args
                .requires_scopes
                .iter()
                .map(|scopes| quote!(::std::string::ToString::to_string(#scopes)))
                .collect::<Vec<_>>();
            let requires = match &method_args.requires {
                Some(requires) => {
                    quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#requires)) }
                }
                None => quote! { ::std::option::Option::None },
            };
            let provides = match &method_args.provides {
                Some(provides) => {
                    quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#provides)) }
                }
                None => quote! { ::std::option::Option::None },
            };
            let cache_control = {
                let public = method_args.cache_control.is_public();
                let max_age = if method_args.cache_control.no_cache {
                    -1
                } else {
                    method_args.cache_control.max_age as i32
                };
                quote! {
                    #crate_name::CacheControl {
                        public: #public,
                        max_age: #max_age,
                    }
                }
            };

            let has_cache_control = method_args.cache_control.no_cache
                || method_args.cache_control.max_age != 0
                || !method_args.cache_control.is_public();
            let has_deprecation =
                !matches!(method_args.deprecation, args::Deprecation::NoDeprecated);
            let has_external = external;
            let has_shareable = shareable;
            let has_inaccessible = inaccessible;
            let has_requires = method_args.requires.is_some();
            let has_provides = method_args.provides.is_some();
            let has_override_from = method_args.override_from.is_some();
            let has_visible = !matches!(method_args.visible, None | Some(args::Visible::None));
            let has_tags = !method_args.tags.is_empty();
            let has_complexity = method_args.complexity.is_some();
            let has_directives = !method_args.directives.is_empty();
            let has_requires_scopes = !method_args.requires_scopes.is_empty();

            let args = extract_input_args::<args::Argument>(&crate_name, method)?;
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
                    process_with,
                    visible,
                    inaccessible,
                    tags,
                    secret,
                    directives,
                    deprecation,
                    ..
                },
            ) in &args
            {
                let name = name.clone().unwrap_or_else(|| {
                    object_args
                        .rename_args
                        .rename(ident.ident.unraw().to_string(), RenameTarget::Argument)
                });
                let has_desc = desc.is_some();
                let default = generate_default(default, default_with)?;
                let schema_default = default.as_ref().map(|value| {
                    quote! {
                        ::std::option::Option::Some(::std::string::ToString::to_string(
                            &<#ty as #crate_name::InputType>::to_value(&#value)
                        ))
                    }
                });

                let has_visible = visible.is_some();
                let visible = visible_fn(visible);
                let has_tags = !tags.is_empty();
                let tags = tags
                    .iter()
                    .map(|tag| quote!(::std::string::ToString::to_string(#tag)))
                    .collect::<Vec<_>>();
                let has_directives = !directives.is_empty();
                let directives = gen_directive_calls(
                    &crate_name,
                    directives,
                    TypeDirectiveLocation::ArgumentDefinition,
                );
                let has_deprecation = !matches!(deprecation, args::Deprecation::NoDeprecated);
                let deprecation_expr = gen_deprecation(deprecation, &crate_name);

                let mut arg_sets = Vec::new();
                if has_desc {
                    let desc = desc.as_ref().expect("checked desc");
                    arg_sets.push(quote! {
                        arg.description = ::std::option::Option::Some(::std::string::ToString::to_string(#desc));
                    });
                }
                if let Some(schema_default) = schema_default {
                    arg_sets.push(quote!(arg.default_value = #schema_default;));
                }
                if has_deprecation {
                    arg_sets.push(quote!(arg.deprecation = #deprecation_expr;));
                }
                if has_visible {
                    arg_sets.push(quote!(arg.visible = #visible;));
                }
                if *inaccessible {
                    arg_sets.push(quote!(arg.inaccessible = true;));
                }
                if has_tags {
                    arg_sets.push(quote!(arg.tags = ::std::vec![ #(#tags),* ];));
                }
                if *secret {
                    arg_sets.push(quote!(arg.is_secret = true;));
                }
                if has_directives {
                    arg_sets
                        .push(quote!(arg.directive_invocations = ::std::vec![ #(#directives),* ];));
                }

                schema_args.push(quote! {
                    {
                        let mut arg = #crate_name::registry::MetaInputValue::new(
                            ::std::string::ToString::to_string(#name),
                            <#ty as #crate_name::InputType>::create_type_info(registry),
                        );
                        #(#arg_sets)*
                        field.args.insert(::std::string::ToString::to_string(#name), arg);
                    }
                });

                let param_ident = &ident.ident;
                use_params.push(quote! { #param_ident });

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

            let output = method.sig.output.clone();
            let ty = match &output {
                ReturnType::Type(_, ty) => OutputType::parse(ty)?,
                ReturnType::Default => {
                    return Err(
                        Error::new_spanned(&output, "Resolver must have a return type").into(),
                    );
                }
            };
            let schema_ty = ty.value_type();
            let visible = visible_fn(&method_args.visible);

            let complexity = if let Some(complexity) = &method_args.complexity {
                let (variables, expr) = parse_complexity_expr(complexity.clone())?;
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
                            object_args
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
                        Ok(#expr)
                    })
                }
            } else {
                quote! { ::std::option::Option::None }
            };

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
                #(#cfg_attrs)*
                {
                    let mut field = #crate_name::registry::MetaField::new(
                        ::std::string::ToString::to_string(#field_name),
                        <#schema_ty as #crate_name::OutputType>::create_type_info(registry),
                    );
                    #(#schema_args)*
                    #(#field_sets)*
                    fields.push((::std::string::ToString::to_string(#field_name), field));
                }
            });

            let field_ident = &method.sig.ident;
            if is_async {
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
            }

            let resolve_obj = if is_async {
                quote! {
                    {
                        let res = self.#field_ident(ctx, #(#use_params),*).await;
                        res.map_err(|err| ::std::convert::Into::<#crate_name::Error>::into(err).into_server_error(ctx.item.pos))
                    }
                }
            } else {
                match &ty {
                    OutputType::Value(_) => {
                        quote! {
                            ::std::result::Result::Ok(self.#field_ident(ctx, #(#use_params),*))
                        }
                    }
                    OutputType::Result(_) => {
                        quote! {
                            self.#field_ident(ctx, #(#use_params),*)
                                .map_err(|err| {
                                    ::std::convert::Into::<#crate_name::Error>::into(err)
                                        .into_server_error(ctx.item.pos)
                                })
                        }
                    }
                }
            };

            let guard_map_err = quote! {
                .map_err(|err| err.into_server_error(ctx.item.pos))
            };
            let guard = match method_args.guard.as_ref().or(object_args.guard.as_ref()) {
                Some(code) => Some(generate_guards(&crate_name, code, guard_map_err)?),
                None => None,
            };

            let resolve_block = if is_async {
                quote! {
                    let f = async move {
                        #(#get_params)*
                        #guard
                        #resolve_obj
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return #crate_name::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
            } else {
                quote! {
                    #(#get_params)*
                    #guard
                    let obj = #resolve_obj.map_err(|err| ctx.set_error_path(err))?;
                    return #crate_name::resolver_utils::resolve_simple_field_value(ctx, &obj).await;
                }
            };

            resolvers.push(quote! {
                #(#cfg_attrs)*
                if ctx.item.node.name.node == #field_name {
                    #resolve_block
                }
            });

            remove_graphql_attrs(&mut method.attrs);
        }
    }

    let expanded = quote! {
        #item_impl

        #[allow(clippy::all, clippy::pedantic)]
        #boxed_trait
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
