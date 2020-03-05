use crate::utils::parse_value;
use graphql_parser::query::Value;
use syn::{Attribute, AttributeArgs, Error, Meta, NestedMeta, Result};

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
}

impl Argument {
    pub fn parse(attrs: &[Attribute]) -> Result<Self> {
        let mut name = None;
        let mut desc = None;
        let mut default = None;

        for attr in attrs {
            match attr.parse_meta() {
                Ok(Meta::List(ls)) if ls.path.is_ident("arg") => {
                    for meta in &ls.nested {
                        match meta {
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
                                            "Attribute 'deprecation' should be a string.",
                                        ));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            name,
            desc,
            default,
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
                        match meta {
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
                            _ => {}
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
                        match meta {
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
                            _ => {}
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
}

impl InputField {
    pub fn parse(attrs: &[Attribute]) -> Result<Self> {
        let mut internal = false;
        let mut name = None;
        let mut desc = None;
        let mut default = None;

        for attr in attrs {
            if attr.path.is_ident("field") {
                if let Ok(Meta::List(args)) = attr.parse_meta() {
                    for meta in args.nested {
                        match meta {
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
                                } else if nv.path.is_ident("default") {
                                    if let syn::Lit::Str(lit) = nv.lit {
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
                }
            }
        }

        Ok(Self {
            internal,
            name,
            desc,
            default,
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
