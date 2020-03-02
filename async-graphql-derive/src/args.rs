use syn::{Attribute, AttributeArgs, Error, Meta, NestedMeta, Result, Type};

#[derive(Debug)]
pub struct Object {
    pub internal: bool,
    pub auto_impl: bool,
    pub name: Option<String>,
    pub desc: Option<String>,
}

impl Object {
    pub fn parse(args: AttributeArgs) -> Result<Self> {
        let mut internal = false;
        let mut auto_impl = false;
        let mut name = None;
        let mut desc = None;

        for arg in args {
            match arg {
                NestedMeta::Meta(Meta::Path(p)) if p.is_ident("internal") => {
                    internal = true;
                }
                NestedMeta::Meta(Meta::Path(p)) if p.is_ident("auto_impl") => {
                    auto_impl = true;
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
            auto_impl,
            name,
            desc,
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
    pub name: Option<String>,
    pub desc: Option<String>,
    pub is_attr: bool,
    pub attr_type: Option<Type>,
    pub arguments: Vec<Argument>,
}

impl Field {
    pub fn parse(attrs: &[Attribute]) -> Result<Option<Self>> {
        let mut is_field = false;
        let mut name = None;
        let mut desc = None;
        let mut is_attr = false;
        let mut attr_type = None;
        let mut arguments = Vec::new();

        for attr in attrs {
            if attr.path.is_ident("field") {
                is_field = true;
                if let Ok(Meta::List(args)) = attr.parse_meta() {
                    for meta in args.nested {
                        match meta {
                            NestedMeta::Meta(Meta::Path(p)) if p.is_ident("attr") => {
                                is_attr = true;
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
                                } else if nv.path.is_ident("attr_type") {
                                    if let syn::Lit::Str(lit) = &nv.lit {
                                        if let Ok(ty) = syn::parse_str::<syn::Type>(&lit.value()) {
                                            attr_type = Some(ty);
                                        } else {
                                            return Err(Error::new_spanned(&lit, "expect type"));
                                        }
                                    } else {
                                        return Err(Error::new_spanned(
                                            &nv.lit,
                                            "Attribute 'attr_type' should be a string.",
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
                                                    if let Ok(ty2) =
                                                        syn::parse_str::<syn::Type>(&lit.value())
                                                    {
                                                        ty = Some(ty2);
                                                    } else {
                                                        return Err(Error::new_spanned(
                                                            &lit,
                                                            "expect type",
                                                        ));
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
                }
            }
        }

        if is_field {
            Ok(Some(Self {
                name,
                desc,
                is_attr,
                attr_type,
                arguments,
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
