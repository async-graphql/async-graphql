use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Error, GenericArgument, PathArguments, Result, Type};

pub enum OutputType<'a> {
    Value(&'a Type),
    Result(&'a Type, &'a Type),
}

impl<'a> OutputType<'a> {
    pub fn parse(input: &'a Type) -> Result<Self> {
        let ty = if let Type::Path(p) = input {
            if p.path.segments.last().unwrap().ident == "FieldResult" {
                if let PathArguments::AngleBracketed(args) =
                    &p.path.segments.last().unwrap().arguments
                {
                    if args.args.is_empty() {
                        return Err(Error::new_spanned(input, "Invalid type"));
                    }
                    let mut res = None;
                    for arg in &args.args {
                        if let GenericArgument::Type(value_ty) = arg {
                            res = Some(OutputType::Result(input, value_ty));
                            break;
                        }
                    }
                    if res.is_none() {
                        return Err(Error::new_spanned(input, "Invalid type"));
                    }
                    res.unwrap()
                } else {
                    return Err(Error::new_spanned(input, "Invalid type"));
                }
            } else {
                OutputType::Value(input)
            }
        } else {
            OutputType::Value(input)
        };
        Ok(ty)
    }

    pub fn value_type(&self) -> Type {
        let tokens = match self {
            OutputType::Value(ty) => quote! {#ty},
            OutputType::Result(_, ty) => quote! {#ty},
        };
        let mut ty = syn::parse2::<syn::Type>(tokens).unwrap();
        Self::remove_lifecycle(&mut ty);
        ty
    }

    fn remove_lifecycle(ty: &mut Type) {
        match ty {
            Type::Reference(r) => {
                r.lifetime = None;
                Self::remove_lifecycle(&mut r.elem);
            }
            Type::Path(r) => {
                for s in &mut r.path.segments {
                    if let PathArguments::AngleBracketed(args) = &mut s.arguments {
                        for arg in &mut args.args {
                            match arg {
                                GenericArgument::Lifetime(lt) => {
                                    lt.ident = Ident::new("_", Span::call_site());
                                }
                                GenericArgument::Type(ty) => {
                                    Self::remove_lifecycle(ty);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
