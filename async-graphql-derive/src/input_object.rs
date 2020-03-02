use crate::args;
use crate::utils::get_crate_name;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Result};

pub fn generate(object_args: &args::Object, input: &DeriveInput) -> Result<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
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
            let #ident:#ty = #crate_name::GQLInputValue::parse(obj.remove(#name).unwrap_or(#crate_name::Value::Null))?;
        });
        get_json_fields.push(quote! {
            let #ident:#ty = #crate_name::GQLInputValue::parse_from_json(obj.remove(#name).unwrap_or(#crate_name::serde_json::Value::Null))?;
        });
        fields.push(ident);
    }

    let expanded = quote! {
        #input

        impl #crate_name::GQLType for #ident {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#gql_typename)
            }
        }

        impl #crate_name::GQLInputValue for #ident {
            fn parse(value: #crate_name::Value) -> #crate_name::Result<Self> {
                use #crate_name::GQLType;

                if let #crate_name::Value::Object(mut obj) = value {
                    #(#get_fields)*
                    Ok(Self { #(#fields),* })
                } else {
                    Err(#crate_name::QueryError::ExpectedType {
                        expect: Self::type_name(),
                        actual: value,
                    }.into())
                }
            }

            fn parse_from_json(value: #crate_name::serde_json::Value) -> #crate_name::Result<Self> {
                use #crate_name::GQLType;
                if let #crate_name::serde_json::Value::Object(mut obj) = value {
                    #(#get_json_fields)*
                    Ok(Self { #(#fields),* })
                } else {
                    Err(#crate_name::QueryError::ExpectedJsonType {
                        expect: Self::type_name(),
                        actual: value,
                    }.into())
                }
            }
        }

        impl #crate_name::GQLInputObject for #ident {}
    };
    Ok(expanded.into())
}
