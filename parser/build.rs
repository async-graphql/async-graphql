use std::{error::Error, fs};

const PREAMBLE: &str = "\
//! This is @generated code, do not edit by hand.
//! See `graphql.pest` and `tests/codegen.rs`.
#![allow(unused_attributes)]
use super::GraphQLParser;
";

fn main() -> Result<(), Box<dyn Error>> {
    generated_code()?;
    println!("cargo:rerun-if-changed=src/graphql.pest");
    Ok(())
}

fn generated_code() -> Result<(), Box<dyn Error>> {
    let input = r###"
#[derive(Parser)]
#[grammar = r#"graphql.pest"#]
struct GraphQLParser;
"###
    .parse::<proc_macro2::TokenStream>()
    .unwrap();
    let tokens = pest_generator::derive_parser(input, false);
    let new = tokens.to_string();
    let code = format!("{}\n{}", PREAMBLE, &new);
    let current_code = fs::read_to_string("./src/parse/generated.rs").unwrap();
    if current_code != code {
        fs::write("./src/parse/generated.rs", code).unwrap();
    }
    Ok(())
}
