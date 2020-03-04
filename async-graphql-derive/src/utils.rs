use graphql_parser::parse_query;
use graphql_parser::query::{Definition, OperationDefinition, ParseError, Query, Value};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

pub fn get_crate_name(internal: bool) -> TokenStream {
    match internal {
        true => quote! { crate },
        false => {
            let id = Ident::new("async_graphql", Span::call_site());
            quote! { #id }
        }
    }
}

pub fn parse_value(s: &str) -> Result<Value, ParseError> {
    let mut doc = parse_query(&format!("query ($a:Int!={}) {{ dummy }}", s))?;
    let definition = doc.definitions.remove(0);
    if let Definition::Operation(OperationDefinition::Query(Query {
        mut variable_definitions,
        ..
    })) = definition
    {
        let var = variable_definitions.remove(0);
        Ok(var.default_value.unwrap())
    } else {
        unreachable!()
    }
}

pub fn build_value_repr(crate_name: &TokenStream, value: &Value) -> TokenStream {
    match value {
        Value::Variable(_) => unreachable!(),
        Value::Int(n) => {
            let n = n.as_i64().unwrap();
            quote! { #crate_name::Value::Int((#n as i32).into()) }
        }
        Value::Float(n) => {
            quote! { #crate_name::Value::Float(#n) }
        }
        Value::String(s) => {
            quote! { #crate_name::Value::String(#s.to_string()) }
        }
        Value::Boolean(n) => {
            quote! { #crate_name::Value::Boolean(#n) }
        }
        Value::Null => {
            quote! { #crate_name::Value::Null }
        }
        Value::Enum(n) => {
            quote! { #crate_name::Value::Enum(#n.to_string()) }
        }
        Value::List(ls) => {
            let members = ls
                .iter()
                .map(|v| build_value_repr(crate_name, v))
                .collect::<Vec<_>>();
            quote! { #crate_name::Value::List(vec![#(#members),*]) }
        }
        Value::Object(obj) => {
            let members = obj
                .iter()
                .map(|(n, v)| {
                    let value = build_value_repr(crate_name, v);
                    quote! {
                        obj.insert(#n.to_string(), #value);
                    }
                })
                .collect::<Vec<_>>();
            quote! {
                {
                    let mut obj = std::collections::BTreeMap::new();
                    #(#members)*
                    obj
                }
            }
        }
    }
}
