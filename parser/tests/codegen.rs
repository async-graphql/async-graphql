//! `pest_derive` crate has large dependency tree, and, as a build dependency,
//! it imposes these deps onto our consumers.
//!
//! To avoid that, let's just dump generated code to string into this
//! repository, and add a test that checks that the code is fresh.
use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
};

const PREAMBLE: &str = r#"
//! This is @generated code, do not edit by hand.
//! See `graphql.pest` and `tests/codegen.rs`.
#![allow(unused_attributes)]
use super::GraphQLParser;
"#;

#[test]
fn generated_code_is_fresh() {
    let input = format!(
        r###"
#[derive(Parser)]
#[grammar = r#"graphql.pest"#]
struct GraphQLParser;
"###,
    )
    .parse::<proc_macro2::TokenStream>()
    .unwrap();

    let tokens = pest_generator::derive_parser(input.into(), false);
    let current =
        String::from_utf8(fs::read("./src/parse/generated.rs").unwrap_or_default()).unwrap();

    let current_content = match current.len() > PREAMBLE.len() {
        true => &current[PREAMBLE.len()..],
        false => current.as_str(),
    };

    let new = tokens.to_string();
    let is_up_to_date = normalize(&current_content) == normalize(&new);

    if is_up_to_date {
        return;
    }

    let code = format!("{}\n{}", PREAMBLE, reformat(&new));
    fs::write("./src/parse/generated.rs", code).unwrap();
    panic!("Generated code in the repository is outdated, updating...");
}

fn reformat(code: &str) -> String {
    let mut cmd = Command::new("rustfmt")
        .args(&["--config", "tab_spaces=2"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    cmd.stdin
        .take()
        .unwrap()
        .write_all(code.as_bytes())
        .unwrap();
    let output = cmd.wait_with_output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}

fn normalize(code: &str) -> String {
    code.replace(|c: char| c.is_ascii_whitespace() || "{},".contains(c), "")
}
