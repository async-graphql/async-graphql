use crate::args;
use darling::FromMeta;
use proc_macro2::{Span, TokenStream, TokenTree};
use proc_macro_crate::crate_name;
use quote::quote;
use syn::{Attribute, Error, Expr, Ident, Lit, LitStr, Meta, NestedMeta};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("{0}")]
    Syn(#[from] syn::Error),

    #[error("{0}")]
    Darling(#[from] darling::Error),
}

impl GeneratorError {
    pub fn write_errors(self) -> TokenStream {
        match self {
            GeneratorError::Syn(err) => err.to_compile_error(),
            GeneratorError::Darling(err) => err.write_errors(),
        }
    }
}

pub type GeneratorResult<T> = std::result::Result<T, GeneratorError>;

pub fn get_crate_name(internal: bool) -> TokenStream {
    if internal {
        quote! { crate }
    } else {
        let name = crate_name("async-graphql").unwrap_or_else(|_| "async_graphql".to_owned());
        TokenTree::from(Ident::new(&name, Span::call_site())).into()
    }
}

fn generate_nested_validator(
    crate_name: &TokenStream,
    nested_meta: &NestedMeta,
) -> GeneratorResult<TokenStream> {
    let mut params = Vec::new();
    match nested_meta {
        NestedMeta::Meta(Meta::List(ls)) => {
            if ls.path.is_ident("and") {
                let mut validators = Vec::new();
                for nested_meta in &ls.nested {
                    validators.push(generate_nested_validator(crate_name, nested_meta)?);
                }
                Ok(validators
                    .into_iter()
                    .fold(None, |acc, item| match acc {
                        Some(prev) => Some(quote! { #crate_name::validators::InputValueValidatorExt::and(#prev, #item) }),
                        None => Some(item),
                    })
                    .unwrap())
            } else if ls.path.is_ident("or") {
                let mut validators = Vec::new();
                for nested_meta in &ls.nested {
                    validators.push(generate_nested_validator(crate_name, nested_meta)?);
                }
                Ok(validators
                    .into_iter()
                    .fold(None, |acc, item| match acc {
                        Some(prev) => Some(quote! { #crate_name::validators::InputValueValidatorExt::or(#prev, #item) }),
                        None => Some(item),
                    })
                    .unwrap())
            } else {
                let ty = &ls.path;
                for item in &ls.nested {
                    if let NestedMeta::Meta(Meta::NameValue(nv)) = item {
                        let name = &nv.path;
                        if let Lit::Str(value) = &nv.lit {
                            let expr = syn::parse_str::<Expr>(&value.value())?;
                            params.push(quote! { #name: (#expr).into() });
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Value must be string literal",
                            )
                            .into());
                        }
                    } else {
                        return Err(Error::new_spanned(
                            nested_meta,
                            "Invalid property for validator",
                        )
                        .into());
                    }
                }
                Ok(quote! { #ty { #(#params),* } })
            }
        }
        NestedMeta::Meta(Meta::Path(ty)) => Ok(quote! { #ty {} }),
        NestedMeta::Meta(Meta::NameValue(_)) | NestedMeta::Lit(_) => {
            Err(Error::new_spanned(nested_meta, "Invalid validator").into())
        }
    }
}

pub fn generate_validator(crate_name: &TokenStream, args: &Meta) -> GeneratorResult<TokenStream> {
    match args {
        Meta::List(args) => {
            if args.nested.len() > 1 {
                return Err(Error::new_spanned(args, "Only one validator can be defined. You can connect combine validators with `and` or `or`").into());
            }
            if args.nested.is_empty() {
                return Err(
                    Error::new_spanned(args, "At least one validator must be defined").into(),
                );
            }
            let validator = generate_nested_validator(crate_name, &args.nested[0])?;
            Ok(quote! { ::std::sync::Arc::new(#validator) })
        }
        _ => Err(Error::new_spanned(args, "Invalid validator").into()),
    }
}

pub fn generate_guards(
    crate_name: &TokenStream,
    args: &Meta,
) -> GeneratorResult<Option<TokenStream>> {
    match args {
        Meta::List(args) => {
            let mut guards = None;
            for item in &args.nested {
                if let NestedMeta::Meta(Meta::List(ls)) = item {
                    let ty = &ls.path;
                    let mut params = Vec::new();
                    for attr in &ls.nested {
                        if let NestedMeta::Meta(Meta::NameValue(nv)) = attr {
                            let name = &nv.path;
                            if let Lit::Str(value) = &nv.lit {
                                let value_str = value.value();
                                if value_str.starts_with('@') {
                                    let getter_name = get_param_getter_ident(&value_str[1..]);
                                    params.push(quote! { #name: #getter_name()? });
                                } else {
                                    let expr = syn::parse_str::<Expr>(&value_str)?;
                                    params.push(quote! { #name: (#expr).into() });
                                }
                            } else {
                                return Err(Error::new_spanned(
                                    &nv.lit,
                                    "Value must be string literal",
                                )
                                .into());
                            }
                        } else {
                            return Err(
                                Error::new_spanned(attr, "Invalid property for guard").into()
                            );
                        }
                    }
                    let guard = quote! { #ty { #(#params),* } };
                    if guards.is_none() {
                        guards = Some(guard);
                    } else {
                        guards =
                            Some(quote! { #crate_name::guard::GuardExt::and(#guard, #guards) });
                    }
                } else {
                    return Err(Error::new_spanned(item, "Invalid guard").into());
                }
            }
            Ok(guards)
        }
        _ => Err(Error::new_spanned(args, "Invalid guards").into()),
    }
}

pub fn generate_post_guards(
    crate_name: &TokenStream,
    args: &Meta,
) -> GeneratorResult<Option<TokenStream>> {
    match args {
        Meta::List(args) => {
            let mut guards = None;
            for item in &args.nested {
                if let NestedMeta::Meta(Meta::List(ls)) = item {
                    let ty = &ls.path;
                    let mut params = Vec::new();
                    for attr in &ls.nested {
                        if let NestedMeta::Meta(Meta::NameValue(nv)) = attr {
                            let name = &nv.path;
                            if let Lit::Str(value) = &nv.lit {
                                let value_str = value.value();
                                if value_str.starts_with('@') {
                                    let getter_name = get_param_getter_ident(&value_str[1..]);
                                    params.push(quote! { #name: #getter_name()? });
                                } else {
                                    let expr = syn::parse_str::<Expr>(&value_str)?;
                                    params.push(quote! { #name: (#expr).into() });
                                }
                            } else {
                                return Err(Error::new_spanned(
                                    &nv.lit,
                                    "Value must be string literal",
                                )
                                .into());
                            }
                        } else {
                            return Err(
                                Error::new_spanned(attr, "Invalid property for guard").into()
                            );
                        }
                    }
                    let guard = quote! { #ty { #(#params),* } };
                    if guards.is_none() {
                        guards = Some(guard);
                    } else {
                        guards =
                            Some(quote! { #crate_name::guard::PostGuardExt::and(#guard, #guards) });
                    }
                } else {
                    return Err(Error::new_spanned(item, "Invalid guard").into());
                }
            }
            Ok(guards)
        }
        _ => Err(Error::new_spanned(args, "Invalid guards").into()),
    }
}

pub fn get_rustdoc(attrs: &[Attribute]) -> GeneratorResult<Option<String>> {
    let mut full_docs = String::new();
    for attr in attrs {
        match attr.parse_meta()? {
            Meta::NameValue(nv) if nv.path.is_ident("doc") => {
                if let Lit::Str(doc) = nv.lit {
                    let doc = doc.value();
                    let doc_str = doc.trim();
                    if !full_docs.is_empty() {
                        full_docs += "\n";
                    }
                    full_docs += doc_str;
                }
            }
            _ => {}
        }
    }
    Ok(if full_docs.is_empty() {
        None
    } else {
        Some(full_docs)
    })
}

fn generate_default_value(lit: &Lit) -> GeneratorResult<TokenStream> {
    match lit {
        Lit::Str(value) =>{
            let value = value.value();
            Ok(quote!({ #value.to_string() }))
        }
        Lit::Int(value) => {
            let value = value.base10_parse::<i32>()?;
            Ok(quote!({ #value as i32 }))
        }
        Lit::Float(value) => {
            let value = value.base10_parse::<f64>()?;
            Ok(quote!({ #value as f64 }))
        }
        Lit::Bool(value) => {
            let value = value.value;
            Ok(quote!({ #value }))
        }
        _ => Err(Error::new_spanned(
            lit,
            "The default value type only be string, integer, float and boolean, other types should use default_with",
        ).into()),
    }
}

fn generate_default_with(lit: &LitStr) -> GeneratorResult<TokenStream> {
    let str = lit.value();
    let tokens: TokenStream = str
        .parse()
        .map_err(|err| GeneratorError::Syn(syn::Error::from(err)))?;
    Ok(quote! { (#tokens) })
}

pub fn generate_default(
    default: &Option<args::DefaultValue>,
    default_with: &Option<LitStr>,
) -> GeneratorResult<Option<TokenStream>> {
    match (default, default_with) {
        (Some(args::DefaultValue::Default), _) => Ok(Some(quote! { Default::default() })),
        (Some(args::DefaultValue::Value(lit)), _) => Ok(Some(generate_default_value(lit)?)),
        (None, Some(lit)) => Ok(Some(generate_default_with(lit)?)),
        (None, None) => Ok(None),
    }
}

pub fn get_param_getter_ident(name: &str) -> Ident {
    Ident::new(&format!("__{}_getter", name), Span::call_site())
}

pub fn get_cfg_attrs(attrs: &[Attribute]) -> Vec<Attribute> {
    attrs
        .iter()
        .filter(|attr| !attr.path.segments.is_empty() && attr.path.segments[0].ident == "cfg")
        .cloned()
        .collect()
}

pub fn parse_graphql_attrs<T: FromMeta>(attrs: &[Attribute]) -> GeneratorResult<Option<T>> {
    for attr in attrs {
        if attr.path.is_ident("graphql") {
            let meta = attr.parse_meta()?;
            return Ok(Some(T::from_meta(&meta)?));
        }
    }
    Ok(None)
}

pub fn remove_graphql_attrs(attrs: &mut Vec<Attribute>) {
    if let Some((idx, _)) = attrs
        .iter()
        .enumerate()
        .find(|(_, a)| a.path.is_ident("graphql"))
    {
        attrs.remove(idx);
    }
}
