use std::collections::HashSet;

use darling::FromMeta;
use once_cell::sync::Lazy;
use proc_macro2::{Span, TokenStream, TokenTree};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use regex::Regex;
use syn::visit::Visit;
use syn::{
    Attribute, Error, Expr, ExprPath, FnArg, Ident, ImplItemMethod, Lit, LitStr, Meta, NestedMeta,
    Pat, PatIdent, Type, TypeGroup, TypeParamBound, TypeReference,
};
use thiserror::Error;

use crate::args;
use crate::args::{Argument, Deprecation, Visible};

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
        let name = match crate_name("async-graphql") {
            Ok(FoundCrate::Name(name)) => name,
            Ok(FoundCrate::Itself) | Err(_) => "async_graphql".to_string(),
        };
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
            } else if ls.path.is_ident("list") {
                if ls.nested.len() > 1 {
                    return Err(Error::new_spanned(
                        ls,
                        "Only one validator can be wrapped with list.",
                    )
                    .into());
                }
                if ls.nested.is_empty() {
                    return Err(
                        Error::new_spanned(ls, "At least one validator must be defined").into(),
                    );
                }
                let validator = generate_nested_validator(crate_name, &ls.nested[0])?;
                Ok(quote! {
                    #crate_name::validators::List(#validator)
                })
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
        Meta::List(args) => match args.path.get_ident().map(ToString::to_string) {
            Some(ident) if ident == "guard" => {
                if args.nested.len() != 1 {
                    return Err(Error::new_spanned(
                        args,
                        "Chained rules isn't possible anymore, please use operators.",
                    )
                    .into());
                }
                if let NestedMeta::Meta(rule) = &args.nested[0] {
                    generate_guards(crate_name, rule)
                } else {
                    Err(Error::new_spanned(&args.nested[0], "Invalid rule.").into())
                }
            }
            Some(ident) if ident == "and" => {
                if args.nested.len() != 2 {
                    return Err(
                        Error::new_spanned(args, "and operator support only 2 operands.").into(),
                    );
                }
                let first_rule: Option<TokenStream>;
                let second_rule: Option<TokenStream>;
                if let NestedMeta::Meta(rule) = &args.nested[0] {
                    first_rule = generate_guards(crate_name, rule)?;
                } else {
                    return Err(Error::new_spanned(&args.nested[0], "Invalid rule.").into());
                }
                if let NestedMeta::Meta(rule) = &args.nested[1] {
                    second_rule = generate_guards(crate_name, rule)?;
                } else {
                    return Err(Error::new_spanned(&args.nested[1], "Invalid rule.").into());
                }
                Ok(Some(
                    quote! { #crate_name::guard::GuardExt::and(#first_rule, #second_rule) },
                ))
            }
            Some(ident) if ident == "or" => {
                if args.nested.len() != 2 {
                    return Err(
                        Error::new_spanned(args, "or operator support only 2 operands.").into(),
                    );
                }
                let first_rule: Option<TokenStream>;
                let second_rule: Option<TokenStream>;
                if let NestedMeta::Meta(rule) = &args.nested[0] {
                    first_rule = generate_guards(crate_name, rule)?;
                } else {
                    return Err(Error::new_spanned(&args.nested[0], "Invalid rule.").into());
                }
                if let NestedMeta::Meta(rule) = &args.nested[1] {
                    second_rule = generate_guards(crate_name, rule)?;
                } else {
                    return Err(Error::new_spanned(&args.nested[1], "Invalid rule.").into());
                }
                Ok(Some(
                    quote! { #crate_name::guard::GuardExt::or(#first_rule, #second_rule) },
                ))
            }
            Some(ident) if ident == "chain" => {
                if args.nested.len() < 2 {
                    return Err(Error::new_spanned(
                        args,
                        "chain operator need at least 1 operand.",
                    )
                    .into());
                }
                let mut guards: Option<TokenStream> = None;
                for arg in &args.nested {
                    if let NestedMeta::Meta(rule) = &arg {
                        let guard = generate_guards(crate_name, rule)?;
                        if guards.is_none() {
                            guards = guard;
                        } else {
                            guards =
                                Some(quote! { #crate_name::guard::GuardExt::and(#guards, #guard) });
                        }
                    }
                }
                Ok(guards)
            }
            Some(ident) if ident == "race" => {
                if args.nested.len() < 2 {
                    return Err(
                        Error::new_spanned(args, "race operator need at least 1 operand.").into(),
                    );
                }
                let mut guards: Option<TokenStream> = None;
                for arg in &args.nested {
                    if let NestedMeta::Meta(rule) = &arg {
                        let guard = generate_guards(crate_name, rule)?;
                        if guards.is_none() {
                            guards = guard;
                        } else {
                            guards =
                                Some(quote! { #crate_name::guard::GuardExt::or(#guards, #guard) });
                        }
                    }
                }
                Ok(guards)
            }
            _ => {
                let ty = &args.path;
                let mut params = Vec::new();
                for attr in &args.nested {
                    if let NestedMeta::Meta(Meta::NameValue(nv)) = attr {
                        let name = &nv.path;
                        if let Lit::Str(value) = &nv.lit {
                            let value_str = value.value();
                            if let Some(value_str) = value_str.strip_prefix('@') {
                                let getter_name = get_param_getter_ident(value_str);
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
                        return Err(Error::new_spanned(attr, "Invalid property for guard").into());
                    }
                }
                Ok(Some(quote! { #ty { #(#params),* } }))
            }
        },
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
            Ok(quote!({ ::std::borrow::ToOwned::to_owned(#value) }))
        }
        Lit::Int(value) => {
            let value = value.base10_parse::<i32>()?;
            Ok(quote!({ ::std::convert::TryInto::try_into(#value).unwrap() }))
        }
        Lit::Float(value) => {
            let value = value.base10_parse::<f64>()?;
            Ok(quote!({ ::std::convert::TryInto::try_into(#value) }))
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
        (Some(args::DefaultValue::Default), _) => {
            Ok(Some(quote! { ::std::default::Default::default() }))
        }
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

pub fn get_type_path_and_name(ty: &Type) -> GeneratorResult<(&Type, String)> {
    match ty {
        Type::Path(path) => Ok((
            ty,
            path.path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap(),
        )),
        Type::Group(TypeGroup { elem, .. }) => get_type_path_and_name(elem),
        Type::TraitObject(trait_object) => Ok((
            ty,
            trait_object
                .bounds
                .iter()
                .find_map(|bound| match bound {
                    TypeParamBound::Trait(t) => {
                        Some(t.path.segments.last().map(|s| s.ident.to_string()).unwrap())
                    }
                    _ => None,
                })
                .unwrap(),
        )),
        _ => Err(Error::new_spanned(ty, "Invalid type").into()),
    }
}

pub fn visible_fn(visible: &Option<Visible>) -> TokenStream {
    match visible {
        None | Some(Visible::None) => quote! { ::std::option::Option::None },
        Some(Visible::HiddenAlways) => quote! { ::std::option::Option::Some(|_| false) },
        Some(Visible::FnName(name)) => {
            quote! { ::std::option::Option::Some(#name) }
        }
    }
}

pub fn parse_complexity_expr(s: &str) -> GeneratorResult<(HashSet<String>, Expr)> {
    #[derive(Default)]
    struct VisitComplexityExpr {
        variables: HashSet<String>,
    }

    impl<'a> Visit<'a> for VisitComplexityExpr {
        fn visit_expr_path(&mut self, i: &'a ExprPath) {
            if let Some(ident) = i.path.get_ident() {
                if ident != "child_complexity" {
                    self.variables.insert(ident.to_string());
                }
            }
        }
    }

    let expr: Expr = syn::parse_str(s)?;
    let mut visit = VisitComplexityExpr::default();
    visit.visit_expr(&expr);
    Ok((visit.variables, expr))
}

pub fn gen_deprecation(deprecation: &Deprecation, crate_name: &TokenStream) -> TokenStream {
    match deprecation {
        Deprecation::NoDeprecated => {
            quote! { #crate_name::registry::Deprecation::NoDeprecated }
        }
        Deprecation::Deprecated {
            reason: Some(reason),
        } => {
            quote! { #crate_name::registry::Deprecation::Deprecated { reason: ::std::option::Option::Some(#reason) } }
        }
        Deprecation::Deprecated { reason: None } => {
            quote! { #crate_name::registry::Deprecation::Deprecated { reason: ::std::option::Option::None } }
        }
    }
}

pub fn extract_input_args(
    crate_name: &proc_macro2::TokenStream,
    method: &mut ImplItemMethod,
) -> GeneratorResult<Vec<(PatIdent, Type, Argument)>> {
    let mut args = Vec::new();
    let mut create_ctx = true;

    if method.sig.inputs.is_empty() {
        return Err(Error::new_spanned(
            &method.sig,
            "The self receiver must be the first parameter.",
        )
        .into());
    }

    for (idx, arg) in method.sig.inputs.iter_mut().enumerate() {
        if let FnArg::Receiver(receiver) = arg {
            if idx != 0 {
                return Err(Error::new_spanned(
                    receiver,
                    "The self receiver must be the first parameter.",
                )
                .into());
            }
        } else if let FnArg::Typed(pat) = arg {
            if idx == 0 {
                return Err(Error::new_spanned(
                    pat,
                    "The self receiver must be the first parameter.",
                )
                .into());
            }

            match (&*pat.pat, &*pat.ty) {
                (Pat::Ident(arg_ident), Type::Reference(TypeReference { elem, .. })) => {
                    if let Type::Path(path) = elem.as_ref() {
                        if idx != 1 || path.path.segments.last().unwrap().ident != "Context" {
                            args.push((
                                arg_ident.clone(),
                                pat.ty.as_ref().clone(),
                                parse_graphql_attrs::<args::Argument>(&pat.attrs)?
                                    .unwrap_or_default(),
                            ));
                        } else {
                            create_ctx = false;
                        }
                    }
                }
                (Pat::Ident(arg_ident), ty) => {
                    args.push((
                        arg_ident.clone(),
                        ty.clone(),
                        parse_graphql_attrs::<args::Argument>(&pat.attrs)?.unwrap_or_default(),
                    ));
                    remove_graphql_attrs(&mut pat.attrs);
                }
                _ => {
                    return Err(Error::new_spanned(arg, "Invalid argument type.").into());
                }
            }
        }
    }

    if create_ctx {
        let arg = syn::parse2::<FnArg>(quote! { _: &#crate_name::Context<'_> }).unwrap();
        method.sig.inputs.insert(1, arg);
    }

    Ok(args)
}

#[derive(Debug)]
pub enum DerivedIntoCoercion {
    Unknown = 1,
    VecToVec = 2,
    OptionToOption = 3,
    OptionVecToOptionVec = 4,
    VecOptionToVecOption = 5,
}

static CHECK_OPTION: Lazy<Regex> = Lazy::new(|| Regex::new("^Option <(.*?) >$").unwrap());
static CHECK_VEC: Lazy<Regex> = Lazy::new(|| Regex::new("^Vec <(.*?) >$").unwrap());
static CHECK_VEC_OPTION: Lazy<Regex> =
    Lazy::new(|| Regex::new("^Vec < Option <(.*?)> >$").unwrap());
static CHECK_OPTION_VEC: Lazy<Regex> =
    Lazy::new(|| Regex::new("^Option < Vec <(.*?)> >$").unwrap());

/// The into argument for a derive field won't be able to transform everythings:
/// Without the specialization from Rust, we can't implement things like From between Vec<T> ->
/// Vec<U> or Option<T> -> Option<U>.
/// But there are cases which you want to have this coercion derived, so to have it working
/// until the specialization feature comes, we manually check coercion for the most usual cases
/// which are:
///
/// - Vec<T> -> Vec<U>
/// - Option<T> -> Option<U>
/// - Option<Vec<T>> -> Option<Vec<U>>
/// - Vec<Option<T>> -> Vec<Option<U>>
pub fn derive_type_coercion<S1: AsRef<str>, S2: AsRef<str>>(
    base_type: S1,
    target_type: S2,
) -> DerivedIntoCoercion {
    if CHECK_OPTION_VEC.find(base_type.as_ref()).is_some()
        && CHECK_OPTION_VEC.find(target_type.as_ref()).is_some()
    {
        return DerivedIntoCoercion::OptionVecToOptionVec;
    }

    if CHECK_VEC_OPTION.find(base_type.as_ref()).is_some()
        && CHECK_VEC_OPTION.find(target_type.as_ref()).is_some()
    {
        return DerivedIntoCoercion::VecOptionToVecOption;
    }

    if CHECK_VEC.find(base_type.as_ref()).is_some()
        && CHECK_VEC.find(target_type.as_ref()).is_some()
    {
        return DerivedIntoCoercion::VecToVec;
    }

    if CHECK_OPTION.find(base_type.as_ref()).is_some()
        && CHECK_OPTION.find(target_type.as_ref()).is_some()
    {
        return DerivedIntoCoercion::OptionToOption;
    }

    DerivedIntoCoercion::Unknown
}
