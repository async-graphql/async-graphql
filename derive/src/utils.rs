use std::collections::HashSet;

use darling::FromMeta;
use proc_macro2::{Span, TokenStream, TokenTree};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{
    visit::Visit, visit_mut, visit_mut::VisitMut, Attribute, Error, Expr, ExprLit, ExprPath, FnArg,
    Ident, ImplItemFn, Lifetime, Lit, LitStr, Meta, Pat, PatIdent, Type, TypeGroup, TypeParamBound,
    TypeReference,
};
use thiserror::Error;

use crate::args::{self, Deprecation, TypeDirectiveLocation, Visible};

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

pub fn generate_guards(
    crate_name: &TokenStream,
    expr: &Expr,
    map_err: TokenStream,
) -> GeneratorResult<TokenStream> {
    let code = quote! {{
        use #crate_name::GuardExt;
        #expr
    }};
    Ok(quote! {
        #crate_name::Guard::check(&#code, &ctx).await #map_err ?;
    })
}

pub fn get_rustdoc(attrs: &[Attribute]) -> GeneratorResult<Option<String>> {
    let mut full_docs = String::new();
    for attr in attrs {
        if let Meta::NameValue(nv) = &attr.meta {
            if nv.path.is_ident("doc") {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(doc), ..
                }) = &nv.value
                {
                    let doc = doc.value();
                    let doc_str = doc.trim();
                    if !full_docs.is_empty() {
                        full_docs += "\n";
                    }
                    full_docs += doc_str;
                }
            }
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
            Ok(quote!({ ::std::convert::TryInto::try_into(#value).unwrap_or_default() }))
        }
        Lit::Float(value) => {
            let value = value.base10_parse::<f64>()?;
            Ok(quote!({ ::std::convert::TryInto::try_into(#value).unwrap_or_default() }))
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

pub fn get_cfg_attrs(attrs: &[Attribute]) -> Vec<Attribute> {
    attrs
        .iter()
        .filter(|attr| !attr.path().segments.is_empty() && attr.path().segments[0].ident == "cfg")
        .cloned()
        .collect()
}

pub fn parse_graphql_attrs<T: FromMeta + Default>(
    attrs: &[Attribute],
) -> GeneratorResult<Option<T>> {
    for attr in attrs {
        if attr.path().is_ident("graphql") {
            return Ok(Some(T::from_meta(&attr.meta)?));
        }
    }
    Ok(None)
}

pub fn remove_graphql_attrs(attrs: &mut Vec<Attribute>) {
    if let Some((idx, _)) = attrs
        .iter()
        .enumerate()
        .find(|(_, a)| a.path().is_ident("graphql"))
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

pub fn parse_complexity_expr(expr: Expr) -> GeneratorResult<(HashSet<String>, Expr)> {
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
            quote! { #crate_name::registry::Deprecation::Deprecated { reason: ::std::option::Option::Some(::std::string::ToString::to_string(#reason)) } }
        }
        Deprecation::Deprecated { reason: None } => {
            quote! { #crate_name::registry::Deprecation::Deprecated { reason: ::std::option::Option::None } }
        }
    }
}

pub fn extract_input_args<T: FromMeta + Default>(
    crate_name: &proc_macro2::TokenStream,
    method: &mut ImplItemFn,
) -> GeneratorResult<Vec<(PatIdent, Type, T)>> {
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
                                parse_graphql_attrs::<T>(&pat.attrs)?.unwrap_or_default(),
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
                        parse_graphql_attrs::<T>(&pat.attrs)?.unwrap_or_default(),
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

pub struct RemoveLifetime;

impl VisitMut for RemoveLifetime {
    fn visit_lifetime_mut(&mut self, i: &mut Lifetime) {
        i.ident = Ident::new("_", Span::call_site());
        visit_mut::visit_lifetime_mut(self, i);
    }
}

pub fn gen_directive_calls(
    directive_calls: &[Expr],
    location: TypeDirectiveLocation,
) -> Vec<TokenStream> {
    directive_calls
        .iter()
        .map(|directive| {
            let directive_name = if let Expr::Call(expr) = directive {
                if let Expr::Path(ref expr) = *expr.func {
                    expr.path.segments.first().map(|s| s.ident.clone())
                } else {
                    None
                }
            } else {
                None
            }
            .expect(
                "Directive invocation expression format must be <directive_name>::apply(<args>)",
            );
            let identifier = location.location_trait_identifier();
            quote!({
                <#directive_name as async_graphql::registry::location_traits::#identifier>::check();
                <#directive_name as async_graphql::TypeDirective>::register(&#directive_name, registry);
                #directive
            })
        })
        .collect::<Vec<_>>()
}
