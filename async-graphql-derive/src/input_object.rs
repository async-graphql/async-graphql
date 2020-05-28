use crate::args;
use crate::utils::{check_reserved_name, get_crate_name, get_rustdoc};
use inflector::Inflector;
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
        let attrs = field
            .attrs
            .iter()
            .filter(|attr| !attr.path.is_ident("field"))
            .collect::<Vec<_>>();
        struct_fields.push(quote! {
            #(#attrs)*
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
    check_reserved_name(&gql_typename, object_args.internal)?;

    let desc = object_args
        .desc
        .clone()
        .or_else(|| get_rustdoc(&input.attrs).ok().flatten())
        .map(|s| quote! { Some(#s) })
        .unwrap_or_else(|| quote! {None});

    let mut get_fields = Vec::new();
    let mut put_fields = Vec::new();
    let mut fields = Vec::new();
    let mut schema_fields = Vec::new();

    for field in &s.fields {
        let field_args = args::InputField::parse(&crate_name, &field.attrs)?;
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let validator = &field_args.validator;
        let name = field_args
            .name
            .unwrap_or_else(|| ident.to_string().to_camel_case());
        let desc = field_args
            .desc
            .as_ref()
            .map(|s| quote! {Some(#s)})
            .unwrap_or_else(|| quote! {None});
        let schema_default = field_args
            .default
            .as_ref()
            .map(|value| {
                quote! {Some( <#ty as #crate_name::InputValueType>::to_value(&#value).to_string() )}
            })
            .unwrap_or_else(|| quote! {None});

        if let Some(default) = &field_args.default {
            get_fields.push(quote! {
                let #ident: #ty = {
                    match obj.get(#name) {
                        Some(value) => #crate_name::InputValueType::parse(Some(value.clone()))?,
                        None => #default,
                    }
                };
            });
        } else {
            get_fields.push(quote! {
                let #ident:#ty = #crate_name::InputValueType::parse(obj.get(#name).cloned())?;
            });
        }

        put_fields.push(quote! {
            map.insert(#name.to_string(), #crate_name::InputValueType::to_value(&self.#ident));
        });

        fields.push(ident);
        schema_fields.push(quote! {
            fields.insert(#name.to_string(), #crate_name::registry::MetaInputValue {
                name: #name,
                description: #desc,
                ty: <#ty as #crate_name::Type>::create_type_info(registry),
                default_value: #schema_default,
                validator: #validator,
            });
        })
    }

    let expanded = quote! {
        #new_struct

        impl #crate_name::Type for #ident {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> String {
                registry.create_type::<Self, _>(|registry| #crate_name::registry::MetaType::InputObject {
                    name: #gql_typename.to_string(),
                    description: #desc,
                    input_fields: {
                        let mut fields = #crate_name::indexmap::IndexMap::new();
                        #(#schema_fields)*
                        fields
                    }
                })
            }
        }

        impl #crate_name::InputValueType for #ident {
            fn parse(value: Option<#crate_name::Value>) -> #crate_name::InputValueResult<Self> {
                use #crate_name::Type;

                if let Some(#crate_name::Value::Object(obj)) = value {
                    #(#get_fields)*
                    Ok(Self { #(#fields),* })
                } else {
                    Err(#crate_name::InputValueError::ExpectedType(value.unwrap_or_default()))
                }
            }

            fn to_value(&self) -> #crate_name::Value {
                let mut map = std::collections::BTreeMap::new();
                #(#put_fields)*
                #crate_name::Value::Object(map)
            }
        }

        impl #crate_name::InputObjectType for #ident {}
    };
    Ok(expanded.into())
}
