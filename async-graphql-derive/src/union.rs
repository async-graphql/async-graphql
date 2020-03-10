use crate::args;
use crate::utils::get_crate_name;
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
    let desc = interface_args
        .desc
        .as_ref()
        .map(|s| quote! {Some(#s)})
        .unwrap_or_else(|| quote! {None});
    let mut registry_types = Vec::new();
    let mut possible_types = Vec::new();
    let mut inline_fragment_resolvers = Vec::new();

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
                <#p as #crate_name::GQLType>::create_type_info(registry);
            });
            possible_types.push(quote! {
                possible_types.insert(<#p as #crate_name::GQLType>::type_name().to_string());
            });
            inline_fragment_resolvers.push(quote! {
                if name == <#p as #crate_name::GQLType>::type_name() {
                    if let #ident::#enum_name(obj) = self {
                        #crate_name::do_resolve(ctx, obj, result).await?;
                    }
                    return Ok(());
                }
            });
        } else {
            return Err(Error::new_spanned(field, "Invalid type"));
        }
    }

    let expanded = quote! {
        #(#attrs)*
        #vis enum #ident #generics { #(#enum_items),* }

        #(#type_into_impls)*

        impl #generics #crate_name::GQLType for #ident #generics {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> String {
                registry.create_type::<Self, _>(|registry| {
                    #(#registry_types)*

                    #crate_name::registry::Type::Union {
                        name: #gql_typename,
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
        impl #generics #crate_name::GQLObject for #ident #generics {
            async fn resolve_field(&self, ctx: &#crate_name::Context<'_>, field: &#crate_name::graphql_parser::query::Field) -> #crate_name::Result<#crate_name::serde_json::Value> {
                use #crate_name::ErrorWithPosition;
                anyhow::bail!(#crate_name::QueryError::FieldNotFound {
                    field_name: field.name.clone(),
                    object: #gql_typename.to_string(),
                }
                .with_position(field.position));
            }

            async fn resolve_inline_fragment(&self, name: &str, ctx: &#crate_name::ContextSelectionSet<'_>, result: &mut #crate_name::serde_json::Map<String, serde_json::Value>) -> #crate_name::Result<()> {
                #(#inline_fragment_resolvers)*
                anyhow::bail!(#crate_name::QueryError::UnrecognizedInlineFragment {
                    object: #gql_typename.to_string(),
                    name: name.to_string(),
                });
            }
        }
    };
    Ok(expanded.into())
}
