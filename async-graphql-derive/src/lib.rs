#![allow(clippy::cognitive_complexity)]

extern crate proc_macro;

mod args;
mod r#enum;
mod input_object;
mod interface;
mod object;
mod output_type;
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
    let item_impl = parse_macro_input!(input as ItemImpl);
    let crate_name = get_crate_name(datasource_args.internal);
    let expanded = quote! {
        #[#crate_name::async_trait::async_trait]
        #item_impl
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
    let item_impl = parse_macro_input!(input as ItemImpl);
    let self_ty = &item_impl.self_ty;
    let generic = &item_impl.generics;
    let where_clause = &item_impl.generics.where_clause;
    let crate_name = get_crate_name(scalar_args.internal);
    let expanded = quote! {
        #item_impl

        impl #generic #crate_name::Type for #self_ty #where_clause {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(<#self_ty as #crate_name::ScalarType>::type_name())
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> String {
                registry.create_type::<#self_ty, _>(|_| #crate_name::registry::Type::Scalar {
                    name: <#self_ty as #crate_name::ScalarType>::type_name().to_string(),
                    description: <#self_ty>::description(),
                    is_valid: |value| <#self_ty as #crate_name::ScalarType>::is_valid(value),
                })
            }
        }

        impl #generic #crate_name::InputValueType for #self_ty #where_clause {
            fn parse(value: &#crate_name::Value) -> #crate_name::InputValueResult<Self> {
                <#self_ty as #crate_name::ScalarType>::parse(value)
            }
        }

        #[allow(clippy::ptr_arg)]
        #[#crate_name::async_trait::async_trait]
        impl #generic #crate_name::OutputValueType for #self_ty #where_clause {
            async fn resolve(
                &self,
                _: &#crate_name::ContextSelectionSet<'_>,
                _pos: #crate_name::Pos,
            ) -> #crate_name::Result<#crate_name::serde_json::Value> {
                self.to_json()
            }
        }
    };
    expanded.into()
}
