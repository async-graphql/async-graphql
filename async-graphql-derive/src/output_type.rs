use syn::{Error, GenericArgument, PathArguments, Result, Type};

pub enum OutputType<'a> {
    Value(&'a Type),
    Result(&'a Type, &'a Type),
}

impl<'a> OutputType<'a> {
    pub fn parse(input: &'a Type) -> Result<Self> {
        let ty = if let Type::Path(p) = input {
            if p.path.segments.last().unwrap().ident == "Result" {
                if let PathArguments::AngleBracketed(args) = &p.path.segments[0].arguments {
                    if args.args.len() == 0 {
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
        } else if let Type::Reference(_) = input {
            OutputType::Value(input)
        } else {
            return Err(Error::new_spanned(input, "Invalid type"));
        };
        Ok(ty)
    }

    pub fn value_type(&self) -> &Type {
        match self {
            OutputType::Value(ty) => ty,
            OutputType::Result(_, ty) => ty,
        }
    }
}
