//! `pest_derive` crate has large dependency tree, and, as a build dependency,
//! it imposes these deps onto our consumers.
//!
//! To avoid that, let's just dump generated code to string into this
//! repository, and add a test that checks that the code is fresh.
use std::fs;

const PREAMBLE: &str = r#"
//! This is @generated code, do not edit by hand.
//! See `graphql.pest` and `tests/codegen.rs`.
#![allow(unused_attributes)]
use super::GraphQLParser;
"#;

#[test]
fn generated_code_is_fresh() {
    let input = r###"
#[derive(Parser)]
#[grammar = r#"graphql.pest"#]
struct GraphQLParser;
"###
    .to_string()
    .parse::<proc_macro2::TokenStream>()
    .unwrap();

    let tokens = pest_generator::derive_parser(input, false);
    let current = String::from_utf8(fs::read("./src/parse/generated.rs").unwrap()).unwrap();

    let current_content = match current.len() > PREAMBLE.len() {
        true => &current[PREAMBLE.len()..],
        false => current.as_str(),
    };

    let new = tokens.to_string();
    let is_up_to_date = normalize(current_content) == normalize(&new);

    if is_up_to_date {
        return;
    }

    let code = format!("{PREAMBLE}\n{new}");
    fs::write("./src/parse/generated.rs", code).unwrap();
    panic!("Generated code in the repository is outdated, updating...");
}

fn normalize(code: &str) -> String {
    code.replace(|c: char| c.is_ascii_whitespace() || "{},".contains(c), "")
}
