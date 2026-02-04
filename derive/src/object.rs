use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    Block, Error, Expr, FnArg, ImplItem, ItemImpl, Pat, PatIdent, ReturnType, Token, Type,
    TypeReference, ext::IdentExt, punctuated::Punctuated,
};

use crate::{
    args::{self, RenameRuleExt, RenameTarget, Resolvability, TypeDirectiveLocation},
    output_type::OutputType,
    utils::{
        GeneratorResult, extract_input_args, gen_boxed_trait, gen_deprecation, gen_directive_calls,
        generate_default, generate_guards, get_cfg_attrs, get_crate_path, get_rustdoc,
        get_type_path_and_name, parse_complexity_expr, parse_graphql_attrs, remove_graphql_attrs,
        visible_fn,
    },
    validators::Validators,
};

pub fn generate(
    object_args: &args::Object,
    item_impl: &mut ItemImpl,
) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_path(&object_args.crate_path, object_args.internal);
    let boxed_trait = gen_boxed_trait(&crate_name);
    let (self_ty, self_name) = get_type_path_and_name(item_impl.self_ty.as_ref())?;
    let (impl_generics, _, where_clause) = item_impl.generics.split_for_impl();
    let extends = object_args.extends;
    let shareable = object_args.shareable;
    let inaccessible = object_args.inaccessible;
    let interface_object = object_args.interface_object;
    let resolvable = matches!(object_args.resolvability, Resolvability::Resolvable);
    let tags = object_args
        .tags
        .iter()
        .map(|tag| quote!(::std::string::ToString::to_string(#tag)))
        .collect::<Vec<_>>();
    let requires_scopes = object_args
        .requires_scopes
        .iter()
        .map(|scopes| quote!(::std::string::ToString::to_string(#scopes)))
        .collect::<Vec<_>>();

    let directives = gen_directive_calls(
        &crate_name,
        &object_args.directives,
        TypeDirectiveLocation::Object,
    );
    let gql_typename = if !object_args.name_type {
        object_args
            .name
            .as_ref()
            .map(|name| quote!(::std::borrow::Cow::Borrowed(#name)))
            .unwrap_or_else(|| {
                let name = RenameTarget::Type.rename(self_name.clone());
                quote!(::std::borrow::Cow::Borrowed(#name))
            })
    } else {
        quote!(<Self as #crate_name::TypeName>::type_name())
    };
    let gql_typename_string = if !object_args.name_type {
        let name = object_args
            .name
            .as_ref()
            .cloned()
            .unwrap_or_else(|| RenameTarget::Type.rename(self_name.clone()));
        quote!(::std::string::ToString::to_string(#name))
    } else {
        quote!(::std::string::ToString::to_string(&#gql_typename))
    };

    let rustdoc = get_rustdoc(&item_impl.attrs)?;
    let has_desc = object_args.use_type_description || rustdoc.is_some();
    let desc = if object_args.use_type_description {
        quote! { ::std::option::Option::Some(::std::string::ToString::to_string(<Self as #crate_name::Description>::description())) }
    } else {
        rustdoc
            .map(|s| quote!(::std::option::Option::Some(::std::string::ToString::to_string(#s))))
            .unwrap_or_else(|| quote!(::std::option::Option::None))
    };

    let mut flattened_resolvers = Vec::new();
    let mut resolvers = Vec::new();
    let mut schema_fields = Vec::new();
    let mut find_entities = Vec::new();
    let mut add_keys = Vec::new();
    let mut create_entity_types = Vec::new();

    let mut unresolvable_key = String::new();

    // Computation of the derived fields
    let mut derived_impls = vec![];
    for item in &mut item_impl.items {
        if let ImplItem::Fn(method) = item {
            let method_args: args::ObjectField =
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
            let method_args: args::ObjectField =
                parse_graphql_attrs(&method.attrs)?.unwrap_or_default();

            if method_args.entity {
                let cfg_attrs = get_cfg_attrs(&method.attrs);

                if method.sig.asyncness.is_none() {
                    return Err(Error::new_spanned(method, "Must be asynchronous").into());
                }

                let args = extract_input_args::<args::Argument>(&crate_name, method)?;

                let output = method.sig.output.clone();
                let ty = match &output {
                    ReturnType::Type(_, ty) => OutputType::parse(ty)?,
                    ReturnType::Default => {
                        return Err(Error::new_spanned(
                            &output,
                            "Resolver must have a return type",
                        )
                        .into());
                    }
                };

                let entity_type = ty.value_type();
                let mut key_pat = Vec::new();
                let mut key_getter = Vec::new();
                let mut use_keys = Vec::new();
                let mut get_federation_key = Vec::new();
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
                        get_federation_key.push(quote! {
                            if let Some(fields) = <#ty as #crate_name::InputType>::federation_fields() {
                                key_str.push(format!("{} {}", #name, fields));
                            } else {
                                key_str.push(#name.to_string());
                            }
                        });

                        key_pat.push(quote! {
                            ::std::option::Option::Some(#ident)
                        });
                        key_getter.push(quote! {
                            params.get(#name).and_then(|value| {
                                let value: ::std::option::Option<#ty> = #crate_name::InputType::parse(::std::option::Option::Some(::std::clone::Clone::clone(&value))).ok();
                                value
                            })
                        });
                    } else {
                        // requires
                        requires_getter.push(quote! {
                            let #ident: #ty = #crate_name::InputType::parse(params.get(#name).cloned()).
                                map_err(|err| err.into_server_error(ctx.item.pos))?;
                        });
                    }
                    use_keys.push(ident);
                }

                add_keys.push(quote! {
                    {
                        let mut key_str = Vec::new();
                        #(#get_federation_key)*
                        registry.add_keys(&<#entity_type as #crate_name::OutputType>::type_name(), &key_str.join(" "));
                    }
                });
                create_entity_types.push(
                    quote! { <#entity_type as #crate_name::OutputType>::create_type_info(registry); },
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
                let do_find = quote! {
                    self.#field_ident(ctx, #(#use_keys),*)
                        .await.map_err(|err| ::std::convert::Into::<#crate_name::Error>::into(err)
                        .into_server_error(ctx.item.pos))
                };

                find_entities.push((
                    args.len(),
                    quote! {
                        #(#cfg_attrs)*
                        if typename == &<#entity_type as #crate_name::OutputType>::type_name() {
                            if let (#(#key_pat),*) = (#(#key_getter),*) {
                                let f = async move {
                                    #(#requires_getter)*
                                    #do_find
                                };
                                let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                                let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                                return #crate_name::OutputType::resolve(&obj, &ctx_obj, ctx.item).await.map(::std::option::Option::Some);
                            }
                        }
                    },
                ));
            } else if !method_args.skip {
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

                    flattened_resolvers.push(flattened_resolver);

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

                unresolvable_key.push_str(&field_name);
                unresolvable_key.push(' ');

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
                let mut params = Vec::new();

                for (
                    ident,
                    ty,
                    args::Argument {
                        name,
                        desc,
                        default,
                        default_with,
                        process_with,
                        validator,
                        visible,
                        secret,
                        inaccessible,
                        tags,
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
                    let has_deprecation = !matches!(deprecation, args::Deprecation::NoDeprecated);
                    let deprecation_expr = gen_deprecation(deprecation, &crate_name);
                    let has_directives = !directives.is_empty();
                    let directives = gen_directive_calls(
                        &crate_name,
                        directives,
                        TypeDirectiveLocation::ArgumentDefinition,
                    );

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
                        arg_sets.push(
                            quote!(arg.directive_invocations = ::std::vec![ #(#directives),* ];),
                        );
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

                    params.push(FieldResolverParameter {
                        ty,
                        name,
                        default,
                        process_with,
                        validator,
                        ident: ident.clone(),
                    });
                }

                let output = method.sig.output.clone();
                let ty = match &output {
                    ReturnType::Type(_, ty) => OutputType::parse(ty)?,
                    ReturnType::Default => {
                        return Err(Error::new_spanned(
                            &output,
                            "Resolver must have a return type",
                        )
                        .into());
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
                        ::std::option::Option::Some(|__ctx, __variables_definition, __field, child_complexity| {
                            #(#parse_args)*
                            ::std::result::Result::Ok(#expr)
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
                    field_sets.push(
                        quote!(field.directive_invocations = ::std::vec![ #(#directives),* ];),
                    );
                }
                if has_requires_scopes {
                    field_sets.push(
                        quote!(field.requires_scopes = ::std::vec![ #(#requires_scopes),* ];),
                    );
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
                        fields.insert(::std::string::ToString::to_string(#field_name), field);
                    }
                });

                let field_ident = &method.sig.ident;
                if is_async && let OutputType::Value(inner_ty) = &ty {
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

                let extract_params = params
                    .iter()
                    .map(|param| generate_parameter_extraction(&crate_name, param))
                    .collect::<Result<Vec<_>, _>>()?;
                let use_params = params.iter().map(
                    |FieldResolverParameter {
                         ident: PatIdent { ident, .. },
                         ..
                     }| ident,
                );

                let guard_map_err = quote! {
                    .map_err(|err| ctx.set_error_path(err.into_server_error(ctx.item.pos)))
                };
                let guard = match method_args.guard.as_ref().or(object_args.guard.as_ref()) {
                    Some(code) => generate_guards(&crate_name, code, guard_map_err)?,
                    None => Default::default(),
                };

                let resolve_body = if is_async {
                    quote! {
                        return #crate_name::resolver_utils::resolve_field_async(
                            ctx,
                            self.#field_ident(ctx, #(#use_params),*)
                        )
                        .await;
                    }
                } else {
                    match &ty {
                        OutputType::Value(_) => {
                            quote! {
                                let obj: #schema_ty = self.#field_ident(ctx, #(#use_params),*);
                                return #crate_name::resolver_utils::resolve_simple_field_value(ctx, &obj).await;
                            }
                        }
                        OutputType::Result(_) => {
                            quote! {
                                let obj: #schema_ty = self.#field_ident(ctx, #(#use_params),*)
                                    .map_err(|err| {
                                        let err = ::std::convert::Into::<#crate_name::Error>::into(err)
                                            .into_server_error(ctx.item.pos);
                                        ctx.set_error_path(err)
                                    })?;
                                return #crate_name::resolver_utils::resolve_simple_field_value(ctx, &obj).await;
                            }
                        }
                    }
                };

                resolvers.push(quote! {
                    #(#cfg_attrs)*
                    #field_name => {
                        #(#extract_params)*
                        #guard
                        #resolve_body
                    }
                });
            }

            remove_graphql_attrs(&mut method.attrs);
        }
    }

    let cache_control = {
        let public = object_args.cache_control.is_public();
        let max_age = if object_args.cache_control.no_cache {
            -1
        } else {
            object_args.cache_control.max_age as i32
        };
        quote! {
            #crate_name::CacheControl {
                public: #public,
                max_age: #max_age,
            }
        }
    };

    find_entities.sort_by(|(a, _), (b, _)| b.cmp(a));
    let find_entities_iter = find_entities.iter().map(|(_, code)| code);

    if resolvers.is_empty() && create_entity_types.is_empty() {
        return Err(Error::new_spanned(
            self_ty,
            "A GraphQL Object type must define one or more fields.",
        )
        .into());
    }

    let keys = match &object_args.resolvability {
        Resolvability::Resolvable => quote!(::std::option::Option::None),
        Resolvability::Unresolvable { key: Some(key) } => quote!(::std::option::Option::Some(
            ::std::vec![ ::std::string::ToString::to_string(#key)]
        )),
        Resolvability::Unresolvable { key: None } => {
            unresolvable_key.pop(); // need to remove the trailing space

            quote!(::std::option::Option::Some(
                ::std::vec![ ::std::string::ToString::to_string(#unresolvable_key)]
            ))
        }
    };

    let visible = visible_fn(&object_args.visible);
    let resolve_container = if object_args.serial {
        quote! { #crate_name::resolver_utils::resolve_container_serial(ctx, self).await }
    } else {
        quote! { #crate_name::resolver_utils::resolve_container(ctx, self).await }
    };

    let has_cache_control = object_args.cache_control.no_cache
        || object_args.cache_control.max_age != 0
        || !object_args.cache_control.is_public();
    let has_keys = !matches!(object_args.resolvability, Resolvability::Resolvable);
    let has_visible = !matches!(object_args.visible, None | Some(args::Visible::None));
    let has_tags = !object_args.tags.is_empty();
    let has_directives = !directives.is_empty();
    let has_requires_scopes = !object_args.requires_scopes.is_empty();
    let field_count = schema_fields.len();

    let mut object_builder_base = Vec::new();
    object_builder_base.push(quote!(.rust_typename(::std::any::type_name::<Self>())));
    if has_desc {
        object_builder_base.push(quote!(.description(#desc)));
    }
    if has_cache_control {
        object_builder_base.push(quote!(.cache_control(#cache_control)));
    }
    if extends {
        object_builder_base.push(quote!(.extends(true)));
    }
    if shareable {
        object_builder_base.push(quote!(.shareable(true)));
    }
    if !resolvable {
        object_builder_base.push(quote!(.resolvable(false)));
    }
    if has_keys {
        object_builder_base.push(quote!(.keys(#keys)));
    }
    if has_visible {
        object_builder_base.push(quote!(.visible(#visible)));
    }
    if inaccessible {
        object_builder_base.push(quote!(.inaccessible(true)));
    }
    if interface_object {
        object_builder_base.push(quote!(.interface_object(true)));
    }
    if has_tags {
        object_builder_base.push(quote!(.tags(::std::vec![ #(#tags),* ])));
    }
    if has_directives {
        object_builder_base.push(quote!(.directive_invocations(::std::vec![ #(#directives),* ])));
    }

    let mut object_builder = object_builder_base.clone();
    if has_requires_scopes {
        object_builder.push(quote!(.requires_scopes(::std::vec![ #(#requires_scopes),* ])));
    }
    let object_builder_concretes = object_builder_base;

    let resolve_field_resolver_match = generate_field_match(resolvers)?;

    let expanded = if object_args.concretes.is_empty() {
        quote! {
            #item_impl

            #[doc(hidden)]
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
                #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
                #boxed_trait
                impl #impl_generics #crate_name::resolver_utils::ContainerType for #self_ty #where_clause {
                    async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                        #resolve_field_resolver_match
                        #(#flattened_resolvers)*
                        ::std::result::Result::Ok(::std::option::Option::None)
                    }

                    async fn find_entity(&self, ctx: &#crate_name::Context<'_>, params: &#crate_name::Value) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                        if let ::std::option::Option::Some((params, typename)) =
                            #crate_name::resolver_utils::find_entity_params(ctx, params)?
                        {
                            #(#find_entities_iter)*
                        }
                        ::std::result::Result::Ok(::std::option::Option::None)
                    }
                }

                #[allow(clippy::all, clippy::pedantic)]
                #boxed_trait
                impl #impl_generics #crate_name::OutputType for #self_ty #where_clause {
                    fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                        #gql_typename
                    }

                    fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                        let ty = registry.create_output_type::<Self, _>(#crate_name::registry::MetaTypeId::Object, |registry| {
                            #crate_name::registry::ObjectBuilder::new(
                                #gql_typename_string,
                                {
                                    let mut fields = #crate_name::indexmap::IndexMap::with_capacity(#field_count);
                                    #(#schema_fields)*
                                    fields
                                },
                            )
                            #(#object_builder)*
                            .build()
                        });
                        #(#create_entity_types)*
                        #(#add_keys)*
                        ty
                    }

                    async fn resolve(
                        &self,
                        ctx: &#crate_name::ContextSelectionSet<'_>,
                        _field: &#crate_name::Positioned<#crate_name::parser::types::Field>
                    ) -> #crate_name::ServerResult<#crate_name::Value> {
                        #resolve_container
                    }
                }

                impl #impl_generics #crate_name::ObjectType for #self_ty #where_clause {}
            };
        }
    } else {
        let mut codes = Vec::new();

        codes.push(quote! {
            #item_impl

            #[doc(hidden)]
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                impl #impl_generics #self_ty #where_clause {
                    fn __internal_create_type_info(registry: &mut #crate_name::registry::Registry, name: &str) -> ::std::string::String  where Self: #crate_name::OutputType {
                        let ty = registry.create_output_type::<Self, _>(#crate_name::registry::MetaTypeId::Object, |registry| {
                            #crate_name::registry::ObjectBuilder::new(
                                ::std::string::ToString::to_string(name),
                                {
                                    let mut fields = #crate_name::indexmap::IndexMap::with_capacity(#field_count);
                                    #(#schema_fields)*
                                    fields
                                },
                            )
                            #(#object_builder_concretes)*
                            .build()
                        });
                        #(#create_entity_types)*
                        #(#add_keys)*
                        ty
                    }

                    async fn __internal_resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> where Self: #crate_name::ContainerType {
                        #resolve_field_resolver_match
                        #(#flattened_resolvers)*
                        ::std::result::Result::Ok(::std::option::Option::None)
                    }

                    async fn __internal_find_entity(&self, ctx: &#crate_name::Context<'_>, params: &#crate_name::Value) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                        if let ::std::option::Option::Some((params, typename)) =
                            #crate_name::resolver_utils::find_entity_params(ctx, params)?
                        {
                            #(#find_entities_iter)*
                        }
                        ::std::result::Result::Ok(::std::option::Option::None)
                    }
                }
            };
        });

        for concrete in &object_args.concretes {
            let gql_typename = &concrete.name;
            let params = &concrete.params.0;
            let ty = {
                let s = quote!(#self_ty).to_string();
                match s.rfind('<') {
                    Some(pos) => syn::parse_str(&s[..pos]).unwrap(),
                    None => self_ty.clone(),
                }
            };
            let concrete_type = quote! { #ty<#(#params),*> };

            let def_bounds = if !concrete.bounds.0.is_empty() {
                let bounds = concrete.bounds.0.iter().map(|b| quote!(#b));
                Some(quote!(<#(#bounds),*>))
            } else {
                None
            };

            codes.push(quote! {
                #boxed_trait
                impl #def_bounds #crate_name::resolver_utils::ContainerType for #concrete_type {
                    async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                        self.__internal_resolve_field(ctx).await
                    }

                    async fn find_entity(&self, ctx: &#crate_name::Context<'_>, params: &#crate_name::Value) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                        self.__internal_find_entity(ctx, params).await
                    }
                }

                #boxed_trait
                impl #def_bounds #crate_name::OutputType for #concrete_type {
                    fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                        ::std::borrow::Cow::Borrowed(#gql_typename)
                    }

                    fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                        Self::__internal_create_type_info(registry, #gql_typename)
                    }

                    async fn resolve(
                        &self,
                        ctx: &#crate_name::ContextSelectionSet<'_>,
                        _field: &#crate_name::Positioned<#crate_name::parser::types::Field>
                    ) -> #crate_name::ServerResult<#crate_name::Value> {
                        #resolve_container
                    }
                }

                impl #def_bounds #crate_name::ObjectType for #concrete_type {}
            });
        }

        quote!(#(#codes)*)
    };

    Ok(expanded.into())
}

fn generate_field_match(
    resolvers: Vec<proc_macro2::TokenStream>,
) -> GeneratorResult<proc_macro2::TokenStream> {
    if resolvers.is_empty() {
        return Ok(quote!());
    }

    Ok(quote! {
        match ctx.item.node.name.node.as_str() {
            #(#resolvers)*
            _ => {}
        }
    })
}

fn generate_parameter_extraction(
    crate_name: &syn::Path,
    parameter: &FieldResolverParameter,
) -> GeneratorResult<proc_macro2::TokenStream> {
    let FieldResolverParameter {
        ty,
        name,
        default,
        process_with,
        validator,
        ident,
    } = parameter;

    let default = match default {
        Some(default) => {
            quote! { ::std::option::Option::Some(|| -> #ty { #default }) }
        }
        None => quote! { ::std::option::Option::None },
    };

    let process_with = match process_with.as_ref() {
        Some(fn_path) => quote! { #fn_path(&mut #ident); },
        None => Default::default(),
    };

    let validators = (*validator).clone().unwrap_or_default().create_validators(
        crate_name,
        quote!(&#ident),
        Some(quote!(.map_err(|err| ctx.set_error_path(err.into_server_error(__pos))))),
    )?;

    let mut non_mut_ident = ident.clone();
    non_mut_ident.mutability = None;
    let ident = &ident.ident;
    Ok(quote! {
        #[allow(non_snake_case, unused_variables, unused_mut)]
        // Todo: if there are no processors we can drop the mut.
        let (__pos, mut #non_mut_ident) = ctx.param_value::<#ty>(#name, #default)
            .map_err(|err| ctx.set_error_path(err))?;
        #process_with
        #validators
        #[allow(non_snake_case, unused_variables)]
        let #ident = #non_mut_ident;
    })
}

struct FieldResolverParameter<'a> {
    ty: &'a Type,
    process_with: &'a Option<Expr>,
    validator: &'a Option<Validators>,
    ident: PatIdent,
    name: String,
    default: Option<proc_macro2::TokenStream>,
}
