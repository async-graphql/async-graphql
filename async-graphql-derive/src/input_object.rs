use crate::args;
use crate::utils::{build_value_repr, get_crate_name};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Result};

pub fn generate(object_args: &args::InputObject, input: &DeriveInput) -> Result<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let ident = &input.ident;
    let attrs = &input.attrs;
    let vis = &input.vis;
    let s = match &input.data {
        Data::Struct(s) => s,
        _ => return Err(Error::new_spanned(input, "It should be a struct.")),
    };

    let mut struct_fields = Vec::new();
    for field in &s.fields {
        let vis = &field.vis;
        let ty = &field.ty;
        let ident = &field.ident;
        struct_fields.push(quote! {
            #vis #ident: #ty
        });
    }
    let new_struct = quote! {
        #(#attrs)*
        #vis struct #ident {
            #(#struct_fields),*
        }
    };

    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| ident.to_string());
    let desc = object_args
        .desc
        .as_ref()
        .map(|s| quote! {Some(#s)})
        .unwrap_or_else(|| quote! {None});

    let mut get_fields = Vec::new();
    let mut fields = Vec::new();
    let mut schema_fields = Vec::new();

    for field in &s.fields {
        let field_args = args::InputField::parse(&field.attrs)?;
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let name = field_args.name.unwrap_or_else(|| ident.to_string());
        let desc = field_args
            .desc
            .as_ref()
            .map(|s| quote! {Some(#s)})
            .unwrap_or_else(|| quote! {None});
        let default = field_args
            .default
            .as_ref()
            .map(|v| {
                let s = v.to_string();
                quote! {Some(#s)}
            })
            .unwrap_or_else(|| quote! {None});

        if let Some(default) = &field_args.default {
            let default_repr = build_value_repr(&crate_name, default);
            get_fields.push(quote! {
                let #ident:#ty = {
                    match obj.get(#name) {
                        Some(value) => #crate_name::GQLInputValue::parse(value)?,
                        None => {
                            let default = #default_repr;
                            #crate_name::GQLInputValue::parse(&default)?
                        }
                    }
                };
            });
        } else {
            get_fields.push(quote! {
                let #ident:#ty = #crate_name::GQLInputValue::parse(obj.get(#name).unwrap_or(&#crate_name::Value::Null))?;
            });
        }

        fields.push(ident);
        schema_fields.push(quote! {
            #crate_name::registry::InputValue {
                name: #name,
                description: #desc,
                ty: <#ty as #crate_name::GQLType>::create_type_info(registry),
                default_value: #default,
            }
        })
    }

    let expanded = quote! {
        #new_struct

        impl #crate_name::GQLType for #ident {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> String {
                registry.create_type::<Self, _>(|registry| #crate_name::registry::Type::InputObject {
                    name: #gql_typename,
                    description: #desc,
                    input_fields: vec![#(#schema_fields),*]
                })
            }
        }

        impl #crate_name::GQLInputValue for #ident {
            fn parse(value: &#crate_name::Value) -> Option<Self> {
                use #crate_name::GQLType;

                if let #crate_name::Value::Object(obj) = value {
                    #(#get_fields)*
                    Some(Self { #(#fields),* })
                } else {
                    None
                }
            }
        }

        impl #crate_name::GQLInputObject for #ident {}
    };
    Ok(expanded.into())
}
