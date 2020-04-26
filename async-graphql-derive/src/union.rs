use crate::args;
use crate::utils::{check_reserved_name, get_crate_name};
use proc_macro::TokenStream;
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
    let mut get_introspection_typename = Vec::new();

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
            });
            possible_types.push(quote! {
                possible_types.insert(<#p as #crate_name::Type>::type_name().to_string());
            });
            collect_inline_fields.push(quote! {
                if let #ident::#enum_name(obj) = self {
                    return obj.collect_inline_fields(name, pos, ctx, futures);
                }
            });
            get_introspection_typename.push(quote! {
                #ident::#enum_name(obj) => <#p as #crate_name::Type>::type_name()
            })
        } else {
            return Err(Error::new_spanned(field, "Invalid type"));
        }
    }

    let expanded = quote! {
        #(#attrs)*
        #vis enum #ident #generics { #(#enum_items),* }

        #(#type_into_impls)*

        impl #generics #crate_name::Type for #ident #generics {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn introspection_type_name(&self) -> std::borrow::Cow<'static, str> {
                match self {
                    #(#get_introspection_typename),*
                }
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> String {
                registry.create_type::<Self, _>(|registry| {
                    #(#registry_types)*

                    #crate_name::registry::Type::Union {
                        name: #gql_typename.to_string(),
                        description: #desc,
                        possible_types: {
                            let mut possible_types = std::collections::HashSet::new();
                            #(#possible_types)*
                            possible_types
                        }
                    }
                })
            }
        }

        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::ObjectType for #ident #generics {
            async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::Result<#crate_name::serde_json::Value> {
                Err(#crate_name::QueryError::FieldNotFound {
                    field_name: ctx.name.clone(),
                    object: #gql_typename.to_string(),
                }.into_error(ctx.position))
            }

            fn collect_inline_fields<'a>(
                &'a self,
                name: &str,
                pos: #crate_name::Pos,
                ctx: &#crate_name::ContextSelectionSet<'a>,
                futures: &mut Vec<#crate_name::BoxFieldFuture<'a>>,
            ) -> #crate_name::Result<()> {
                #(#collect_inline_fields)*
                Ok(())
            }
        }

        #[#crate_name::async_trait::async_trait]
        impl #generics #crate_name::OutputValueType for #ident #generics {
            async fn resolve(value: &Self, ctx: &#crate_name::ContextSelectionSet<'_>, pos: #crate_name::Pos) -> #crate_name::Result<#crate_name::serde_json::Value> {
                #crate_name::do_resolve(ctx, value).await
            }
        }
    };
    Ok(expanded.into())
}
