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

pub fn parse_value(
    s: &str,
) -> Result<graphql_parser::query::Value, graphql_parser::query::ParseError> {
    let mut doc =
        graphql_parser::query::parse_query(&format!("query ($a:Int!={}) {{ dummy }}", s))?;
    let definition = doc.definitions.remove(0);
    if let graphql_parser::query::Definition::Operation(
        graphql_parser::query::OperationDefinition::Query(graphql_parser::query::Query {
            mut variable_definitions,
            ..
        }),
    ) = definition
    {
        let var = variable_definitions.remove(0);
        Ok(var.default_value.unwrap())
    } else {
        unreachable!()
    }
}
