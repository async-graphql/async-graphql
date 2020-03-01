use crate::args;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Result};

pub fn generate(object_args: &args::Object, input: &DeriveInput) -> Result<TokenStream> {
    let ident = &input.ident;
    let s = match &input.data {
        Data::Struct(s) => s,
        _ => return Err(Error::new_spanned(input, "It should be a struct.")),
    };

    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| ident.to_string());

    let mut get_fields = Vec::new();
    let mut get_json_fields = Vec::new();
    let mut fields = Vec::new();

    for field in &s.fields {
        let field_args = args::InputField::parse(&field.attrs)?;
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let name = field_args.name.unwrap_or_else(|| ident.to_string());
        get_fields.push(quote! {
            let #ident:#ty = async_graphql::GQLInputValue::parse(obj.remove(#name).unwrap_or(async_graphql::Value::Null))?;
        });
        get_json_fields.push(quote! {
            let #ident:#ty = async_graphql::GQLInputValue::parse_from_json(obj.remove(#name).unwrap_or(async_graphql::serde_json::Value::Null))?;
        });
        fields.push(ident);
    }

    let expanded = quote! {
        #input

        impl async_graphql::GQLType for #ident {
            fn type_name() -> String {
                #gql_typename.to_string()
            }
        }

        impl async_graphql::GQLInputValue for #ident {
            fn parse(value: async_graphql::Value) -> async_graphql::Result<Self> {
                if let async_graphql::Value::Object(mut obj) = value {
                    #(#get_fields)*
                    Ok(Self { #(#fields),* })
                } else {
                    Err(async_graphql::QueryError::ExpectedType {
                        expect: Self::type_name(),
                        actual: value,
                    }.into())
                }
            }

            fn parse_from_json(value: async_graphql::serde_json::Value) -> async_graphql::Result<Self> {
                if let async_graphql::serde_json::Value::Object(mut obj) = value {
                    #(#get_json_fields)*
                    Ok(Self { #(#fields),* })
                } else {
                    Err(async_graphql::QueryError::ExpectedJsonType {
                        expect: Self::type_name(),
                        actual: value,
                    }.into())
                }
            }
        }

        impl async_graphql::GQLInputObject for #ident {}
    };
    println!("{}", expanded.to_string());
    Ok(expanded.into())
}
