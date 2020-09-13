#![allow(clippy::cognitive_complexity)]
#![forbid(unsafe_code)]

extern crate proc_macro;

mod args;
mod r#enum;
mod input_object;
mod interface;
mod merged_object;
mod merged_subscription;
mod object;
mod output_type;
mod scalar;
mod simple_object;
mod subscription;
mod union;
mod utils;

use crate::utils::parse_derive;
use proc_macro::TokenStream;
use syn::parse_macro_input;
use syn::{AttributeArgs, ItemImpl};

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn GQLObject(args: TokenStream, input: TokenStream) -> TokenStream {
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

#[proc_macro_derive(GQLSimpleObject, attributes(field, graphql))]
pub fn derive_simple_object(input: TokenStream) -> TokenStream {
    let (args, input) = match parse_derive(input.into()) {
        Ok(r) => r,
        Err(err) => return err.to_compile_error().into(),
    };
    let object_args = match args::Object::parse(parse_macro_input!(args as AttributeArgs)) {
        Ok(object_args) => object_args,
        Err(err) => return err.to_compile_error().into(),
    };
    match simple_object::generate(&object_args, &input) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(GQLEnum, attributes(item, graphql))]
pub fn derive_enum(input: TokenStream) -> TokenStream {
    let (args, input) = match parse_derive(input.into()) {
        Ok(r) => r,
        Err(err) => return err.to_compile_error().into(),
    };
    let enum_args = match args::Enum::parse(parse_macro_input!(args as AttributeArgs)) {
        Ok(enum_args) => enum_args,
        Err(err) => return err.to_compile_error().into(),
    };
    match r#enum::generate(&enum_args, &input) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(GQLInputObject, attributes(field, graphql))]
pub fn derive_input_object(input: TokenStream) -> TokenStream {
    let (args, input) = match parse_derive(input.into()) {
        Ok(r) => r,
        Err(err) => return err.to_compile_error().into(),
    };
    let object_args = match args::InputObject::parse(parse_macro_input!(args as AttributeArgs)) {
        Ok(object_args) => object_args,
        Err(err) => return err.to_compile_error().into(),
    };
    match input_object::generate(&object_args, &input) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(GQLInterface, attributes(graphql))]
pub fn derive_interface(input: TokenStream) -> TokenStream {
    let (args, input) = match parse_derive(input.into()) {
        Ok(r) => r,
        Err(err) => return err.to_compile_error().into(),
    };
    let interface_args = match args::Interface::parse(parse_macro_input!(args as AttributeArgs)) {
        Ok(interface_args) => interface_args,
        Err(err) => return err.to_compile_error().into(),
    };
    match interface::generate(&interface_args, &input) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(GQLUnion, attributes(graphql))]
pub fn derive_union(input: TokenStream) -> TokenStream {
    let (args, input) = match parse_derive(input.into()) {
        Ok(r) => r,
        Err(err) => return err.to_compile_error().into(),
    };
    let union_args = match args::Interface::parse(parse_macro_input!(args as AttributeArgs)) {
        Ok(union_args) => union_args,
        Err(err) => return err.to_compile_error().into(),
    };
    match union::generate(&union_args, &input) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn GQLSubscription(args: TokenStream, input: TokenStream) -> TokenStream {
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

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn GQLScalar(args: TokenStream, input: TokenStream) -> TokenStream {
    let scalar_args = match args::Scalar::parse(parse_macro_input!(args as AttributeArgs)) {
        Ok(scalar_args) => scalar_args,
        Err(err) => return err.to_compile_error().into(),
    };
    let mut item_impl = parse_macro_input!(input as ItemImpl);
    match scalar::generate(&scalar_args, &mut item_impl) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(GQLMergedObject, attributes(item, graphql))]
pub fn derive_merged_object(input: TokenStream) -> TokenStream {
    let (args, input) = match parse_derive(input.into()) {
        Ok(r) => r,
        Err(err) => return err.to_compile_error().into(),
    };
    let object_args = match args::Object::parse(parse_macro_input!(args as AttributeArgs)) {
        Ok(object_args) => object_args,
        Err(err) => return err.to_compile_error().into(),
    };
    match merged_object::generate(&object_args, &input) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(GQLMergedSubscription, attributes(item, graphql))]
pub fn derive_merged_subscription(input: TokenStream) -> TokenStream {
    let (args, input) = match parse_derive(input.into()) {
        Ok(r) => r,
        Err(err) => return err.to_compile_error().into(),
    };
    let object_args = match args::Object::parse(parse_macro_input!(args as AttributeArgs)) {
        Ok(object_args) => object_args,
        Err(err) => return err.to_compile_error().into(),
    };
    match merged_subscription::generate(&object_args, &input) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}
