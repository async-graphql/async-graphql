use crate::utils::{parse_validators, parse_value};
use graphql_parser::query::Value;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, AttributeArgs, Error, Meta, MetaList, NestedMeta, Result, Type};

#[derive(Debug)]
pub struct Object {
    pub internal: bool,
    pub name: Option<String>,
    pub desc: Option<String>,
}

impl Object {
    pub fn parse(args: AttributeArgs) -> Result<Self> {
        let mut internal = false;
        let mut name = None;
        let mut desc = None;

        for arg in args {
            match arg {
                NestedMeta::Meta(Meta::Path(p)) if p.is_ident("internal") => {
                    internal = true;
                }
                NestedMeta::Meta(Meta::NameValue(nv)) => {
                    if nv.path.is_ident("name") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            name = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'name' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("desc") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            desc = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'desc' should be a string.",
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            internal,
            name,
            desc,
        })
    }
}

#[derive(Debug)]
pub struct Argument {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub default: Option<Value>,
    pub validators: TokenStream,
}

impl Argument {
    pub fn parse(attrs: &[Attribute]) -> Result<Self> {
        let mut name = None;
        let mut desc = None;
        let mut default = None;
        let mut validators = quote! { Default::default() };

        for attr in attrs {
            match attr.parse_meta() {
                Ok(Meta::List(ls)) if ls.path.is_ident("arg") => {
                    for meta in &ls.nested {
                        if let NestedMeta::Meta(Meta::NameValue(nv)) = meta {
                            if nv.path.is_ident("name") {
                                if let syn::Lit::Str(lit) = &nv.lit {
                                    name = Some(lit.value());
                                } else {
                                    return Err(Error::new_spanned(
                                        &nv.lit,
                                        "Attribute 'name' should be a string.",
                                    ));
                                }
                            } else if nv.path.is_ident("desc") {
                                if let syn::Lit::Str(lit) = &nv.lit {
                                    desc = Some(lit.value());
                                } else {
                                    return Err(Error::new_spanned(
                                        &nv.lit,
                                        "Attribute 'desc' should be a string.",
                                    ));
                                }
                            } else if nv.path.is_ident("default") {
                                if let syn::Lit::Str(lit) = &nv.lit {
                                    match parse_value(&lit.value()) {
                                        Ok(Value::Variable(_)) => {
                                            return Err(Error::new_spanned(
                                                &nv.lit,
                                                "The default cannot be a variable",
                                            ))
                                        }
                                        Ok(value) => default = Some(value),
                                        Err(err) => {
                                            return Err(Error::new_spanned(
                                                &nv.lit,
                                                format!("Invalid value: {}", err),
                                            ));
                                        }
                                    }
                                } else {
                                    return Err(Error::new_spanned(
                                        &nv.lit,
                                        "Attribute 'default' should be a string.",
                                    ));
                                }
                            }
                        }
                    }

                    validators = parse_validators(&ls)?;
                }
                _ => {}
            }
        }

        Ok(Self {
            name,
            desc,
            default,
            validators,
        })
    }
}

#[derive(Debug)]
pub struct Field {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub deprecation: Option<String>,
}

impl Field {
    pub fn parse(attrs: &[Attribute]) -> Result<Option<Self>> {
        let mut is_field = false;
        let mut name = None;
        let mut desc = None;
        let mut deprecation = None;

        for attr in attrs {
            match attr.parse_meta() {
                Ok(Meta::Path(p)) if p.is_ident("field") => {
                    is_field = true;
                }
                Ok(Meta::List(ls)) if ls.path.is_ident("field") => {
                    is_field = true;
                    for meta in &ls.nested {
                        if let NestedMeta::Meta(Meta::NameValue(nv)) = meta {
                            if nv.path.is_ident("name") {
                                if let syn::Lit::Str(lit) = &nv.lit {
                                    name = Some(lit.value());
                                } else {
                                    return Err(Error::new_spanned(
                                        &nv.lit,
                                        "Attribute 'name' should be a string.",
                                    ));
                                }
                            } else if nv.path.is_ident("desc") {
                                if let syn::Lit::Str(lit) = &nv.lit {
                                    desc = Some(lit.value());
                                } else {
                                    return Err(Error::new_spanned(
                                        &nv.lit,
                                        "Attribute 'desc' should be a string.",
                                    ));
                                }
                            } else if nv.path.is_ident("deprecation") {
                                if let syn::Lit::Str(lit) = &nv.lit {
                                    deprecation = Some(lit.value());
                                } else {
                                    return Err(Error::new_spanned(
                                        &nv.lit,
                                        "Attribute 'deprecation' should be a string.",
                                    ));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if is_field {
            Ok(Some(Self {
                name,
                desc,
                deprecation,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug)]
pub struct Enum {
    pub internal: bool,
    pub name: Option<String>,
    pub desc: Option<String>,
}

impl Enum {
    pub fn parse(args: AttributeArgs) -> Result<Self> {
        let mut internal = false;
        let mut name = None;
        let mut desc = None;

        for arg in args {
            match arg {
                NestedMeta::Meta(Meta::Path(p)) if p.is_ident("internal") => {
                    internal = true;
                }
                NestedMeta::Meta(Meta::NameValue(nv)) => {
                    if nv.path.is_ident("name") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            name = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'name' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("desc") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            desc = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'desc' should be a string.",
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            internal,
            name,
            desc,
        })
    }
}

#[derive(Debug)]
pub struct EnumItem {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub deprecation: Option<String>,
}

impl EnumItem {
    pub fn parse(attrs: &[Attribute]) -> Result<Self> {
        let mut name = None;
        let mut desc = None;
        let mut deprecation = None;

        for attr in attrs {
            if attr.path.is_ident("item") {
                if let Ok(Meta::List(args)) = attr.parse_meta() {
                    for meta in args.nested {
                        if let NestedMeta::Meta(Meta::NameValue(nv)) = meta {
                            if nv.path.is_ident("name") {
                                if let syn::Lit::Str(lit) = nv.lit {
                                    name = Some(lit.value());
                                } else {
                                    return Err(Error::new_spanned(
                                        &nv.lit,
                                        "Attribute 'name' should be a string.",
                                    ));
                                }
                            } else if nv.path.is_ident("desc") {
                                if let syn::Lit::Str(lit) = nv.lit {
                                    desc = Some(lit.value());
                                } else {
                                    return Err(Error::new_spanned(
                                        &nv.lit,
                                        "Attribute 'desc' should be a string.",
                                    ));
                                }
                            } else if nv.path.is_ident("deprecation") {
                                if let syn::Lit::Str(lit) = nv.lit {
                                    deprecation = Some(lit.value());
                                } else {
                                    return Err(Error::new_spanned(
                                        &nv.lit,
                                        "Attribute 'deprecation' should be a string.",
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(Self {
            name,
            desc,
            deprecation,
        })
    }
}

#[derive(Debug)]
pub struct InputField {
    pub internal: bool,
    pub name: Option<String>,
    pub desc: Option<String>,
    pub default: Option<Value>,
    pub validators: TokenStream,
}

impl InputField {
    pub fn parse(attrs: &[Attribute]) -> Result<Self> {
        let mut internal = false;
        let mut name = None;
        let mut desc = None;
        let mut default = None;
        let mut validators = quote! { Default::default() };

        for attr in attrs {
            if attr.path.is_ident("field") {
                if let Ok(Meta::List(args)) = &attr.parse_meta() {
                    for meta in &args.nested {
                        match meta {
                            NestedMeta::Meta(Meta::Path(p)) if p.is_ident("internal") => {
                                internal = true;
                            }
                            NestedMeta::Meta(Meta::NameValue(nv)) => {
                                if nv.path.is_ident("name") {
                                    if let syn::Lit::Str(lit) = &nv.lit {
                                        name = Some(lit.value());
                                    } else {
                                        return Err(Error::new_spanned(
                                            &nv.lit,
                                            "Attribute 'name' should be a string.",
                                        ));
                                    }
                                } else if nv.path.is_ident("desc") {
                                    if let syn::Lit::Str(lit) = &nv.lit {
                                        desc = Some(lit.value());
                                    } else {
                                        return Err(Error::new_spanned(
                                            &nv.lit,
                                            "Attribute 'desc' should be a string.",
                                        ));
                                    }
                                } else if nv.path.is_ident("default") {
                                    if let syn::Lit::Str(lit) = &nv.lit {
                                        match parse_value(&lit.value()) {
                                            Ok(Value::Variable(_)) => {
                                                return Err(Error::new_spanned(
                                                    &lit,
                                                    "The default cannot be a variable",
                                                ))
                                            }
                                            Ok(value) => default = Some(value),
                                            Err(err) => {
                                                return Err(Error::new_spanned(
                                                    &lit,
                                                    format!("Invalid value: {}", err),
                                                ));
                                            }
                                        }
                                    } else {
                                        return Err(Error::new_spanned(
                                            &nv.lit,
                                            "Attribute 'default' should be a string.",
                                        ));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    validators = parse_validators(&args)?;
                }
            }
        }

        Ok(Self {
            internal,
            name,
            desc,
            default,
            validators,
        })
    }
}

#[derive(Debug)]
pub struct InputObject {
    pub internal: bool,
    pub name: Option<String>,
    pub desc: Option<String>,
}

impl InputObject {
    pub fn parse(args: AttributeArgs) -> Result<Self> {
        let mut internal = false;
        let mut name = None;
        let mut desc = None;

        for arg in args {
            match arg {
                NestedMeta::Meta(Meta::Path(p)) if p.is_ident("internal") => {
                    internal = true;
                }
                NestedMeta::Meta(Meta::NameValue(nv)) => {
                    if nv.path.is_ident("name") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            name = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'name' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("desc") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            desc = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'desc' should be a string.",
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            internal,
            name,
            desc,
        })
    }
}

#[derive(Debug)]
pub struct InterfaceFieldArgument {
    pub name: String,
    pub desc: Option<String>,
    pub ty: Type,
    pub default: Option<Value>,
}

impl InterfaceFieldArgument {
    pub fn parse(ls: &MetaList) -> Result<Self> {
        let mut name = None;
        let mut desc = None;
        let mut ty = None;
        let mut default = None;

        for meta in &ls.nested {
            if let NestedMeta::Meta(Meta::NameValue(nv)) = meta {
                if nv.path.is_ident("name") {
                    if let syn::Lit::Str(lit) = &nv.lit {
                        name = Some(lit.value());
                    } else {
                        return Err(Error::new_spanned(
                            &nv.lit,
                            "Attribute 'name' should be a string.",
                        ));
                    }
                } else if nv.path.is_ident("desc") {
                    if let syn::Lit::Str(lit) = &nv.lit {
                        desc = Some(lit.value());
                    } else {
                        return Err(Error::new_spanned(
                            &nv.lit,
                            "Attribute 'desc' should be a string.",
                        ));
                    }
                } else if nv.path.is_ident("type") {
                    if let syn::Lit::Str(lit) = &nv.lit {
                        if let Ok(ty2) = syn::parse_str::<syn::Type>(&lit.value()) {
                            ty = Some(ty2);
                        } else {
                            return Err(Error::new_spanned(&lit, "Expect type"));
                        }
                    } else {
                        return Err(Error::new_spanned(
                            &nv.lit,
                            "Attribute 'type' should be a string.",
                        ));
                    }
                } else if nv.path.is_ident("default") {
                    if let syn::Lit::Str(lit) = &nv.lit {
                        match parse_value(&lit.value()) {
                            Ok(Value::Variable(_)) => {
                                return Err(Error::new_spanned(
                                    &nv.lit,
                                    "The default cannot be a variable",
                                ))
                            }
                            Ok(value) => default = Some(value),
                            Err(err) => {
                                return Err(Error::new_spanned(
                                    &nv.lit,
                                    format!("Invalid value: {}", err),
                                ));
                            }
                        }
                    } else {
                        return Err(Error::new_spanned(
                            &nv.lit,
                            "Attribute 'default' should be a string.",
                        ));
                    }
                }
            }
        }

        if name.is_none() {
            return Err(Error::new_spanned(ls, "Missing name"));
        }

        if ty.is_none() {
            return Err(Error::new_spanned(ls, "Missing type"));
        }

        Ok(Self {
            name: name.unwrap(),
            desc,
            ty: ty.unwrap(),
            default,
        })
    }
}

#[derive(Debug)]
pub struct InterfaceField {
    pub name: String,
    pub desc: Option<String>,
    pub ty: Type,
    pub args: Vec<InterfaceFieldArgument>,
    pub deprecation: Option<String>,
    pub context: bool,
}

impl InterfaceField {
    pub fn parse(ls: &MetaList) -> Result<Self> {
        let mut name = None;
        let mut desc = None;
        let mut ty = None;
        let mut args = Vec::new();
        let mut deprecation = None;
        let mut context = false;

        for meta in &ls.nested {
            match meta {
                NestedMeta::Meta(Meta::Path(p)) if p.is_ident("context") => {
                    context = true;
                }
                NestedMeta::Meta(Meta::NameValue(nv)) => {
                    if nv.path.is_ident("name") {
                        if let syn::Lit::Str(lit) = &nv.lit {
                            name = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'name' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("desc") {
                        if let syn::Lit::Str(lit) = &nv.lit {
                            desc = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'desc' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("type") {
                        if let syn::Lit::Str(lit) = &nv.lit {
                            if let Ok(ty2) = syn::parse_str::<syn::Type>(&lit.value()) {
                                ty = Some(ty2);
                            } else {
                                return Err(Error::new_spanned(&lit, "Expect type"));
                            }
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'type' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("deprecation") {
                        if let syn::Lit::Str(lit) = &nv.lit {
                            deprecation = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'deprecation' should be a string.",
                            ));
                        }
                    }
                }
                NestedMeta::Meta(Meta::List(ls)) if ls.path.is_ident("arg") => {
                    args.push(InterfaceFieldArgument::parse(ls)?);
                }
                _ => {}
            }
        }

        if name.is_none() {
            return Err(Error::new_spanned(ls, "Missing name"));
        }

        if ty.is_none() {
            return Err(Error::new_spanned(ls, "Missing type"));
        }

        Ok(Self {
            name: name.unwrap(),
            desc,
            ty: ty.unwrap(),
            args,
            deprecation,
            context,
        })
    }
}

#[derive(Debug)]
pub struct Interface {
    pub internal: bool,
    pub name: Option<String>,
    pub desc: Option<String>,
    pub fields: Vec<InterfaceField>,
}

impl Interface {
    pub fn parse(args: AttributeArgs) -> Result<Self> {
        let mut internal = false;
        let mut name = None;
        let mut desc = None;
        let mut fields = Vec::new();

        for arg in args {
            match arg {
                NestedMeta::Meta(Meta::Path(p)) if p.is_ident("internal") => {
                    internal = true;
                }
                NestedMeta::Meta(Meta::Path(p)) if p.is_ident("internal") => {
                    internal = true;
                }
                NestedMeta::Meta(Meta::NameValue(nv)) => {
                    if nv.path.is_ident("name") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            name = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'name' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("desc") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            desc = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'desc' should be a string.",
                            ));
                        }
                    }
                }
                NestedMeta::Meta(Meta::List(ls)) if ls.path.is_ident("field") => {
                    fields.push(InterfaceField::parse(&ls)?);
                }
                _ => {}
            }
        }

        Ok(Self {
            internal,
            name,
            desc,
            fields,
        })
    }
}
