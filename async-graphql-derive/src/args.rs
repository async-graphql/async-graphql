use syn::{Attribute, AttributeArgs, Error, Meta, MetaList, NestedMeta, Result, Type};

#[derive(Debug)]
pub struct Object {
    pub internal: bool,
    pub name: Option<String>,
    pub desc: Option<String>,
    pub fields: Vec<Field>,
}

impl Object {
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
                    fields.push(Field::parse(&ls)?);
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

#[derive(Debug)]
pub struct Argument {
    pub name: String,
    pub desc: Option<String>,
    pub ty: Type,
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub resolver: Option<String>,
    pub desc: Option<String>,
    pub ty: Type,
    pub is_owned: bool,
    pub arguments: Vec<Argument>,
}

impl Field {
    fn parse(ls: &MetaList) -> Result<Self> {
        let mut name = None;
        let mut resolver = None;
        let mut desc = None;
        let mut ty = None;
        let mut is_owned = false;
        let mut arguments = Vec::new();

        for meta in &ls.nested {
            match meta {
                NestedMeta::Meta(Meta::Path(p)) if p.is_ident("owned") => {
                    is_owned = true;
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
                    } else if nv.path.is_ident("resolver") {
                        if let syn::Lit::Str(lit) = &nv.lit {
                            resolver = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'resolver' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("type") {
                        if let syn::Lit::Str(lit) = &nv.lit {
                            if let Ok(ty2) = syn::parse_str::<syn::Type>(&lit.value()) {
                                ty = Some(ty2);
                            } else {
                                return Err(Error::new_spanned(&lit, "Expect type"));
                            }
                            desc = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'type' should be a string.",
                            ));
                        }
                    }
                }
                NestedMeta::Meta(Meta::List(ls)) => {
                    if ls.path.is_ident("arg") {
                        let mut name = None;
                        let mut desc = None;
                        let mut ty = None;

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
                                            return Err(Error::new_spanned(&lit, "expect type"));
                                        }
                                    } else {
                                        return Err(Error::new_spanned(
                                            &nv.lit,
                                            "Attribute 'type' should be a string.",
                                        ));
                                    }
                                }
                            }
                        }

                        if name.is_none() {
                            return Err(Error::new_spanned(ls, "missing name."));
                        }

                        if ty.is_none() {
                            return Err(Error::new_spanned(ls, "missing type."));
                        }

                        arguments.push(Argument {
                            name: name.unwrap(),
                            desc,
                            ty: ty.unwrap(),
                        });
                    }
                }
                _ => {}
            }
        }

        if name.is_none() {
            return Err(Error::new_spanned(ls, "missing name."));
        }

        if ty.is_none() {
            return Err(Error::new_spanned(ls, "missing type."));
        }

        Ok(Self {
            name: name.unwrap(),
            resolver,
            desc,
            ty: ty.unwrap(),
            is_owned,
            arguments,
        })
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
}

impl EnumItem {
    pub fn parse(attrs: &[Attribute]) -> Result<Self> {
        let mut name = None;
        let mut desc = None;

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
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(Self { name, desc })
    }
}

#[derive(Debug)]
pub struct InputField {
    pub internal: bool,
    pub name: Option<String>,
    pub desc: Option<String>,
}

impl InputField {
    pub fn parse(attrs: &[Attribute]) -> Result<Self> {
        let mut internal = false;
        let mut name = None;
        let mut desc = None;

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
        })
    }
}
