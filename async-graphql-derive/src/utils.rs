use graphql_parser::parse_query;
use graphql_parser::query::{Definition, OperationDefinition, ParseError, Query, Value};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Error, Ident, Meta, MetaList, NestedMeta, Result};

pub fn get_crate_name(internal: bool) -> TokenStream {
    if internal {
        quote! { crate }
    } else {
        let id = Ident::new("async_graphql", Span::call_site());
        quote! { #id }
    }
}

pub fn parse_value(s: &str) -> std::result::Result<Value, ParseError> {
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

pub fn check_reserved_name(name: &str, internal: bool) -> Result<()> {
    if internal {
        return Ok(());
    }
    if name.ends_with("Connection") {
        Err(Error::new(
            Span::call_site(),
            "The name ending with 'Connection' is reserved",
        ))
    } else if name == "PageInfo" {
        Err(Error::new(
            Span::call_site(),
            "The name 'PageInfo' is reserved",
        ))
    } else {
        Ok(())
    }
}

pub fn parse_validators(args: &MetaList) -> Result<TokenStream> {
    let mut validators = Vec::new();
    for arg in &args.nested {
        if let NestedMeta::Meta(Meta::List(ls)) = arg {
            if ls.path.is_ident("validator") {
                let mut ty = None;
                let mut params = Vec::new();
                for item in &ls.nested {
                    match item {
                        NestedMeta::Meta(Meta::Path(p)) => {
                            ty = Some(p);
                        }
                        NestedMeta::Meta(Meta::NameValue(nv)) => {
                            let name = &nv.path;
                            let value = &nv.lit;
                            params.push(quote! { #name: #value });
                        }
                        _ => {}
                    }
                }
                validators.push(quote! { Box::new(#ty { #(#params),* }) });
            }
        }
    }
    Ok(quote! { std::sync::Arc::new(vec![#(#validators)*]) })
}
