#![allow(clippy::cognitive_complexity)]

extern crate proc_macro;

mod args;
mod r#enum;
mod input_object;
mod interface;
mod object;
mod output_type;
mod subscription;
mod union;
mod utils;

use proc_macro::TokenStream;
use syn::parse_macro_input;
use syn::{AttributeArgs, DeriveInput, ItemImpl};

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Object(args: TokenStream, input: TokenStream) -> TokenStream {
    let object_args = match args::Object::parse(parse_macro_input!(args as AttributeArgs)) {
        Ok(object_args) => object_args,
        Err(err) => return err.to_compile_error().into(),
    };
    let mut item_impl = parse_macro_input!(input as ItemImpl);
    match object::generate(&object_args, &mut item_impl) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Enum(args: TokenStream, input: TokenStream) -> TokenStream {
    let enum_args = match args::Enum::parse(parse_macro_input!(args as AttributeArgs)) {
        Ok(enum_args) => enum_args,
        Err(err) => return err.to_compile_error().into(),
    };
    let input = parse_macro_input!(input as DeriveInput);
    match r#enum::generate(&enum_args, &input) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn InputObject(args: TokenStream, input: TokenStream) -> TokenStream {
    let object_args = match args::InputObject::parse(parse_macro_input!(args as AttributeArgs)) {
        Ok(enum_args) => enum_args,
        Err(err) => return err.to_compile_error().into(),
    };
    let input = parse_macro_input!(input as DeriveInput);
    match input_object::generate(&object_args, &input) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Interface(args: TokenStream, input: TokenStream) -> TokenStream {
    let interface_args = match args::Interface::parse(parse_macro_input!(args as AttributeArgs)) {
        Ok(interface_args) => interface_args,
        Err(err) => return err.to_compile_error().into(),
    };
    let input = parse_macro_input!(input as DeriveInput);
    match interface::generate(&interface_args, &input) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Union(args: TokenStream, input: TokenStream) -> TokenStream {
    let interface_args = match args::Interface::parse(parse_macro_input!(args as AttributeArgs)) {
        Ok(interface_args) => interface_args,
        Err(err) => return err.to_compile_error().into(),
    };
    let input = parse_macro_input!(input as DeriveInput);
    match union::generate(&interface_args, &input) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Subscription(args: TokenStream, input: TokenStream) -> TokenStream {
    let object_args = match args::Object::parse(parse_macro_input!(args as AttributeArgs)) {
        Ok(object_args) => object_args,
        Err(err) => return err.to_compile_error().into(),
    };
    let mut item_impl = parse_macro_input!(input as ItemImpl);
    match subscription::generate(&object_args, &mut item_impl) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}
