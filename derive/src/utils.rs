use itertools::Itertools;
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::crate_name;
use quote::quote;
use syn::{
    Attribute, AttributeArgs, DeriveInput, Error, Expr, Ident, Lit, Meta, MetaList, NestedMeta,
    Result,
};

pub fn get_crate_name(internal: bool) -> TokenStream {
    if internal {
        quote! { crate }
    } else {
        let name = crate_name("async-graphql").unwrap_or_else(|_| "async_graphql".to_owned());
        let id = Ident::new(&name, Span::call_site());
        quote! { #id }
    }
}

pub fn add_container_attrs(
    derive: TokenStream,
    container_attrs: AttributeArgs,
    item: TokenStream,
) -> Result<TokenStream> {
    let internal = container_attrs.iter().any(|meta| {
        if let NestedMeta::Meta(Meta::Path(p)) = meta {
            p.is_ident("internal")
        } else {
            false
        }
    });
    let crate_name = get_crate_name(internal);
    let expanded = quote! {
       #[derive(#crate_name::#derive)]
       #[graphql(#(#container_attrs),*)]
       #item
    };
    Ok(expanded)
}

pub fn parse_derive(input: TokenStream) -> Result<(proc_macro::TokenStream, DeriveInput)> {
    let mut input: DeriveInput = syn::parse2(input)?;
    let attrs = &mut input.attrs;
    let graphql_attr = attrs
        .iter()
        .find_position(|attr| attr.path.is_ident("graphql"));

    if let Some((pos, _attr)) = graphql_attr {
        let attribute = attrs.remove(pos);
        let args = attribute.parse_args::<TokenStream>()?;
        Ok((args.into(), input))
    } else {
        Ok((TokenStream::new().into(), input))
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
                            params.push(quote! { #name: (#expr).into() });
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
                return Ok(quote! { Some(::std::sync::Arc::new(#validator)) });
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

pub fn parse_post_guards(crate_name: &TokenStream, args: &MetaList) -> Result<Option<TokenStream>> {
    for arg in &args.nested {
        if let NestedMeta::Meta(Meta::List(ls)) = arg {
            if ls.path.is_ident("post_guard") {
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
                            guards = Some(
                                quote! { #crate_name::guard::PostGuardExt::and(#guard, #guards) },
                            );
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

pub fn parse_default(lit: &Lit) -> Result<TokenStream> {
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
        )),
    }
}

pub fn parse_default_with(lit: &Lit) -> Result<TokenStream> {
    if let Lit::Str(str) = lit {
        let str = str.value();
        let tokens: TokenStream = str.parse()?;
        Ok(quote! { (#tokens) })
    } else {
        Err(Error::new_spanned(
            &lit,
            "Attribute 'default' should be a string.",
        ))
    }
}

pub fn get_param_getter_ident(name: &str) -> Ident {
    Ident::new(&format!("__{}_getter", name), Span::call_site())
}

pub fn feature_block(
    crate_name: &TokenStream,
    features: &[String],
    field_name: &str,
    block: TokenStream,
) -> TokenStream {
    if !features.is_empty() {
        let error_message = format!(
            "`{}` is only available if the features `{}` are enabled",
            field_name,
            features.join(",")
        );
        quote!({
            #[cfg(not(all(#(feature = #features),*)))]
            {
                return Err(#crate_name::FieldError::from(#error_message)).map_err(::std::convert::Into::into);
            }
            #[cfg(all(#(feature = #features),*))]
            {
                #block
            }
        })
    } else {
        block
    }
}
