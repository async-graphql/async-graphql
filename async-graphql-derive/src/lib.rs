#![allow(clippy::cognitive_complexity)]
#![forbid(unsafe_code)]

extern crate proc_macro;

mod args;
mod r#enum;
mod input_object;
mod interface;
mod object;
mod output_type;
mod scalar;
mod simple_object;
mod subscription;
mod union;
mod utils;

use crate::utils::get_crate_name;
use proc_macro::TokenStream;
use quote::quote;
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
pub fn SimpleObject(args: TokenStream, input: TokenStream) -> TokenStream {
    let object_args = match args::Object::parse(parse_macro_input!(args as AttributeArgs)) {
        Ok(object_args) => object_args,
        Err(err) => return err.to_compile_error().into(),
    };
    let mut derive_input = parse_macro_input!(input as DeriveInput);
    match simple_object::generate(&object_args, &mut derive_input) {
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

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn DataSource(args: TokenStream, input: TokenStream) -> TokenStream {
    let datasource_args = match args::DataSource::parse(parse_macro_input!(args as AttributeArgs)) {
        Ok(datasource_args) => datasource_args,
        Err(err) => return err.to_compile_error().into(),
    };
    let input2: proc_macro2::TokenStream = input.clone().into();
    let _item_impl = parse_macro_input!(input as ItemImpl);
    let crate_name = get_crate_name(datasource_args.internal);
    let expanded = quote! {
        #[#crate_name::async_trait::async_trait]
        #input2
    };
    expanded.into()
}

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Scalar(args: TokenStream, input: TokenStream) -> TokenStream {
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
