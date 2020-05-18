use async_graphql_parser::Value;
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::crate_name;
use quote::quote;
use syn::{Attribute, Error, Expr, Ident, Lit, Meta, MetaList, NestedMeta, Result};

pub fn get_crate_name(internal: bool) -> TokenStream {
    if internal {
        quote! { crate }
    } else {
        let name = crate_name("async-graphql").unwrap_or_else(|_| "async_graphql".to_owned());
        let id = Ident::new(&name, Span::call_site());
        quote! { #id }
    }
}

pub fn build_value_repr(crate_name: &TokenStream, value: &Value) -> TokenStream {
    match value {
        Value::Variable(_) => unreachable!(),
        Value::Int(n) => {
            quote! { #crate_name::Value::Int(#n) }
        }
        Value::Float(n) => {
            quote! { #crate_name::Value::Float(#n) }
        }
        Value::String(s) => {
            quote! { #crate_name::Value::String(#s.to_string().into()) }
        }
        Value::Boolean(n) => {
            quote! { #crate_name::Value::Boolean(#n) }
        }
        Value::Null => {
            quote! { #crate_name::Value::Null }
        }
        Value::Enum(n) => {
            quote! { #crate_name::Value::Enum(#n.to_string()) }
        }
        Value::List(ls) => {
            let members = ls
                .iter()
                .map(|v| build_value_repr(crate_name, v))
                .collect::<Vec<_>>();
            quote! { #crate_name::Value::List(vec![#(#members),*]) }
        }
        Value::Object(obj) => {
            let members = obj
                .iter()
                .map(|(n, v)| {
                    let value = build_value_repr(crate_name, v);
                    quote! {
                        obj.insert(#n.to_string().into(), #value);
                    }
                })
                .collect::<Vec<_>>();
            quote! {
                {
                    let mut obj = std::collections::BTreeMap::new();
                    #(#members)*
                    #crate_name::Value::Object(obj)
                }
            }
        }
        Value::Upload(_) => quote! { #crate_name::Value::Null },
    }
}

pub fn check_reserved_name(name: &str, internal: bool) -> Result<()> {
    if internal {
        return Ok(());
    }
    if name.ends_with("Connection") {
        Err(Error::new(
            Span::call_site(),
            "The name ending with 'Connection' is reserved",
        ))
    } else if name == "PageInfo" {
        Err(Error::new(
            Span::call_site(),
            "The name 'PageInfo' is reserved",
        ))
    } else {
        Ok(())
    }
}

fn parse_nested_validator(
    crate_name: &TokenStream,
    nested_meta: &NestedMeta,
) -> Result<TokenStream> {
    let mut params = Vec::new();

    match nested_meta {
        NestedMeta::Meta(Meta::List(ls)) => {
            if ls.path.is_ident("and") {
                let mut validators = Vec::new();
                for nested_meta in &ls.nested {
                    validators.push(parse_nested_validator(crate_name, nested_meta)?);
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
                    validators.push(parse_nested_validator(crate_name, nested_meta)?);
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
                            params.push(quote! { #name: #expr.into() });
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Value must be string literal",
                            ));
                        }
                    } else {
                        return Err(Error::new_spanned(
                            nested_meta,
                            "Invalid property for validator",
                        ));
                    }
                }
                Ok(quote! { #ty { #(#params),* } })
            }
        }
        NestedMeta::Meta(Meta::Path(ty)) => Ok(quote! { #ty {} }),
        NestedMeta::Meta(Meta::NameValue(_)) | NestedMeta::Lit(_) => {
            Err(Error::new_spanned(nested_meta, "Invalid validator"))
        }
    }
}

pub fn parse_validator(crate_name: &TokenStream, args: &MetaList) -> Result<TokenStream> {
    for arg in &args.nested {
        if let NestedMeta::Meta(Meta::List(ls)) = arg {
            if ls.path.is_ident("validator") {
                if ls.nested.len() > 1 {
                    return Err(Error::new_spanned(ls,
                                                  "Only one validator can be defined. You can connect combine validators with `and` or `or`"));
                }
                if ls.nested.is_empty() {
                    return Err(Error::new_spanned(
                        ls,
                        "At least one validator must be defined",
                    ));
                }
                let validator = parse_nested_validator(crate_name, &ls.nested[0])?;
                return Ok(quote! { Some(std::sync::Arc::new(#validator)) });
            }
        }
    }
    Ok(quote! {None})
}

pub fn parse_guards(crate_name: &TokenStream, args: &MetaList) -> Result<Option<TokenStream>> {
    for arg in &args.nested {
        if let NestedMeta::Meta(Meta::List(ls)) = arg {
            if ls.path.is_ident("guard") {
                let mut guards = None;

                for item in &ls.nested {
                    if let NestedMeta::Meta(Meta::List(ls)) = item {
                        let ty = &ls.path;
                        let mut params = Vec::new();
                        for attr in &ls.nested {
                            if let NestedMeta::Meta(Meta::NameValue(nv)) = attr {
                                let name = &nv.path;
                                if let Lit::Str(value) = &nv.lit {
                                    let value_str = value.value();
                                    if value_str.starts_with('@') {
                                        let id = Ident::new(&value_str[1..], value.span());
                                        params.push(quote! { #name: &#id });
                                    } else {
                                        let expr = syn::parse_str::<Expr>(&value_str)?;
                                        params.push(quote! { #name: #expr.into() });
                                    }
                                } else {
                                    return Err(Error::new_spanned(
                                        &nv.lit,
                                        "Value must be string literal",
                                    ));
                                }
                            } else {
                                return Err(Error::new_spanned(attr, "Invalid property for guard"));
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
                        return Err(Error::new_spanned(item, "Invalid guard"));
                    }
                }

                return Ok(guards);
            }
        }
    }

    Ok(None)
}

pub fn get_rustdoc(attrs: &[Attribute]) -> Result<Option<String>> {
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
