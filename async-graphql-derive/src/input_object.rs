use crate::args;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Error, Ident, Result};

pub fn generate(object_args: &args::Object, input: &DeriveInput) -> Result<TokenStream> {
    let attrs = &input.attrs;
    let vis = &input.vis;
    let ident = &input.ident;
    let s = match &input.data {
        Data::Struct(s) => s,
        _ => return Err(Error::new_spanned(input, "It should be a struct.")),
    };

    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| ident.to_string());

    for field in &s.fields {
        let field_args = args::Field::parse(&field.attrs)?;
        let ident = field.ident.as_ref().unwrap();
    }

    let expanded = quote! {
        #input

        impl async_graphql::Type for #ident {
            fn type_name() -> String {
                #gql_typename.to_string()
            }
        }

        impl async_graphql::GQLInputObject for #ident {

        }
    };
    Ok(expanded.into())
}
