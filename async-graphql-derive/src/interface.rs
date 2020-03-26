use crate::args;
use crate::args::{InterfaceField, InterfaceFieldArgument};
use crate::output_type::OutputType;
use crate::utils::{build_value_repr, check_reserved_name, get_crate_name};
use inflector::Inflector;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Result, Type};

pub fn generate(interface_args: &args::Interface, input: &DeriveInput) -> Result<TokenStream> {
    let crate_name = get_crate_name(interface_args.internal);
    let ident = &input.ident;
    let generics = &input.generics;
    let attrs = &input.attrs;
    let vis = &input.vis;
    let s = match &input.data {
        Data::Struct(s) => s,
        _ => return Err(Error::new_spanned(input, "It should be a struct.")),
    };
    let fields = match &s.fields {
        Fields::Unnamed(fields) => fields,
        _ => return Err(Error::new_spanned(input, "All fields must be unnamed.")),
    };
    let mut enum_names = Vec::new();
    let mut enum_items = Vec::new();
    let mut type_into_impls = Vec::new();
    let gql_typename = interface_args
        .name
        .clone()
        .unwrap_or_else(|| ident.to_string());
    check_reserved_name(&gql_typename, interface_args.internal)?;
    let desc = interface_args
        .desc
        .as_ref()
        .map(|s| quote! {Some(#s)})
        .unwrap_or_else(|| quote! {None});
    let mut registry_types = Vec::new();
    let mut possible_types = Vec::new();
    let mut collect_inline_fields = Vec::new();

    for field in &fields.unnamed {
        if let Type::Path(p) = &field.ty {
            let enum_name = &p.path.segments.last().unwrap().ident;
            enum_names.push(enum_name);
            enum_items.push(quote! { #enum_name(#p) });
            type_into_impls.push(quote! {
                impl #generics From<#p> for #ident #generics {
                    fn from(obj: #p) -> Self {
                        #ident::#enum_name(obj)
                    }
                }
            });
            registry_types.push(quote! {
                <#p as #crate_name::Type>::create_type_info(registry);
                registry.add_implements(&<#p as #crate_name::Type>::type_name(), #gql_typename);
            });
            possible_types.push(quote! {
                possible_types.insert(<#p as #crate_name::Type>::type_name().to_string());
            });
            collect_inline_fields.push(quote! {
                // if name == <#p as #crate_name::Type>::type_name() {
                //     if let #ident::#enum_name(obj) = self {
                //         return &obj;
                //     }
                //     unreachable!()
                // }
            });
        } else {
            return Err(Error::new_spanned(field, "Invalid type"));
        }
    }

    let mut methods = Vec::new();
    let mut schema_fields = Vec::new();
    let mut resolvers = Vec::new();

    for InterfaceField {
        name,
        desc,
        ty,
        args,
        deprecation,
        context,
    } in &interface_args.fields
    {
        let method_name = Ident::new(name, Span::call_site());
        let name = name.to_camel_case();
        let mut calls = Vec::new();
        let mut use_params = Vec::new();
        let mut decl_params = Vec::new();
        let mut get_params = Vec::new();
        let mut schema_args = Vec::new();

        if *context {
            decl_params.push(quote! { ctx: &'ctx #crate_name::Context<'ctx> });
            use_params.push(quote! { ctx });
        }

        for InterfaceFieldArgument {
            name,
            desc,
            ty,
            default,
        } in args
        {
            let ident = Ident::new(name, Span::call_site());
            let name = name.to_camel_case();
            decl_params.push(quote! { #ident: #ty });
            use_params.push(quote! { #ident });

            let param_default = match &default {
                Some(default) => {
                    let repr = build_value_repr(&crate_name, &default);
                    quote! {|| #repr }
                }
                None => quote! { || #crate_name::Value::Null },
            };
            get_params.push(quote! {
                let #ident: #ty = ctx.param_value(#name, #param_default)?;
            });

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
                    validator: None,
                });
            });
        }

        for enum_name in &enum_names {
            calls.push(quote! {
                #ident::#enum_name(obj) => obj.#method_name(#(#use_params),*).await
            });
        }

        let ctx_lifetime = if *context {
            quote! { <'ctx> }
        } else {
            quote! {}
        };

        methods.push(quote! {
            async fn #method_name #ctx_lifetime(&self, #(#decl_params),*) -> #ty {
                match self {
                    #(#calls,)*
                }
            }
        });

        let desc = desc
            .as_ref()
            .map(|s| quote! {Some(#s)})
            .unwrap_or_else(|| quote! {None});
        let deprecation = deprecation
            .as_ref()
            .map(|s| quote! {Some(#s)})
            .unwrap_or_else(|| quote! {None});

        let ty = OutputType::parse(ty)?;
        let schema_ty = ty.value_type();

        schema_fields.push(quote! {
            fields.insert(#name.to_string(), #crate_name::registry::Field {
                name: #name.to_string(),
                description: #desc,
                args: {
                    let mut args = std::collections::HashMap::new();
                    #(#schema_args)*
                    args
                },
                ty: <#schema_ty as #crate_name::Type>::create_type_info(registry),
                deprecation: #deprecation,
                cache_control: Default::default(),
            });
        });

        let resolve_obj = match &ty {
            OutputType::Value(_) => quote! {
                self.#method_name(#(#use_params),*).await
            },
            OutputType::Result(_, _) => {
                quote! {
                    self.#method_name(#(#use_params),*).await.
                        map_err(|err| err.with_position(field.position))?
                }
            }
        };

        resolvers.push(quote! {
            if field.name.as_str() == #name {
                #(#get_params)*
                let ctx_obj = ctx.with_selection_set(&field.selection_set);
                return #crate_name::OutputValueType::resolve(&#resolve_obj, &ctx_obj).await.
                    map_err(|err| err.with_position(field.position).into());
            }
        });
    }

    let expanded = quote! {
        #(#attrs)*
        #vis enum #ident #generics { #(#enum_items),* }

        #(#type_into_impls)*

        impl #generics #ident #generics {
            #(#methods)*
        }

        impl #generics #crate_name::Type for #ident #generics {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> String {
                registry.create_type::<Self, _>(|registry| {
                    #(#registry_types)*

                    #crate_name::registry::Type::Interface {
                        name: #gql_typename.to_string(),
                        description: #desc,
                        fields: {
                            let mut fields = std::collections::HashMap::new();
                            #(#schema_fields)*
                            fields
                        },
                        possible_types: {
                            let mut possible_types = std::collections::HashSet::new();
                            #(#possible_types)*
                            possible_types
                        },
                    }
                })
            }
        }

        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::ObjectType for #ident #generics {
            async fn resolve_field(&self, ctx: &#crate_name::Context<'_>, field: &#crate_name::graphql_parser::query::Field) -> #crate_name::Result<#crate_name::serde_json::Value> {
                use #crate_name::ErrorWithPosition;

                #(#resolvers)*

                #crate_name::anyhow::bail!(#crate_name::QueryError::FieldNotFound {
                    field_name: field.name.clone(),
                    object: #gql_typename.to_string(),
                }
                .with_position(field.position));
            }

            fn collect_inline_fields<'a>(
                &'a self,
                name: &str,
                ctx: #crate_name::ContextSelectionSet<'a>,
                futures: &mut Vec<#crate_name::BoxFieldFuture<'a>>,
            ) -> #crate_name::Result<()> {
                #(#collect_inline_fields)*
                #crate_name::anyhow::bail!(#crate_name::QueryError::UnrecognizedInlineFragment {
                    object: #gql_typename.to_string(),
                    name: name.to_string(),
                });
            }
        }

        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::OutputValueType for #ident #generics {
            async fn resolve(value: &Self, ctx: &#crate_name::ContextSelectionSet<'_>) -> #crate_name::Result<#crate_name::serde_json::Value> {
                #crate_name::do_resolve(ctx, value).await
            }
        }
    };
    Ok(expanded.into())
}
