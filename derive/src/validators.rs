use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Expr, Lit, Result};

#[derive(Clone)]
pub enum Number {
    F64(f64),
    I64(i64),
}

impl FromMeta for Number {
    fn from_value(value: &Lit) -> darling::Result<Self> {
        match value {
            Lit::Int(n) => Ok(Number::I64(n.base10_parse::<i64>()?)),
            Lit::Float(n) => Ok(Number::F64(n.base10_parse::<f64>()?)),
            _ => Err(darling::Error::unexpected_type("number")),
        }
    }
}

impl ToTokens for Number {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Number::F64(n) => tokens.extend(quote!(#n as f64)),
            Number::I64(n) => tokens.extend(quote!(#n as i64)),
        }
    }
}

#[derive(FromMeta, Default, Clone)]
pub struct Validators {
    #[darling(default)]
    multiple_of: Option<Number>,
    #[darling(default)]
    min_password_strength: Option<u8>,
    #[darling(default)]
    maximum: Option<Number>,
    #[darling(default)]
    minimum: Option<Number>,
    #[darling(default)]
    max_length: Option<usize>,
    #[darling(default)]
    min_length: Option<usize>,
    #[darling(default)]
    max_items: Option<usize>,
    #[darling(default)]
    min_items: Option<usize>,
    #[darling(default)]
    chars_max_length: Option<usize>,
    #[darling(default)]
    chars_min_length: Option<usize>,
    #[darling(default)]
    email: bool,
    #[darling(default)]
    url: bool,
    #[darling(default)]
    ip: bool,
    #[darling(default)]
    regex: Option<String>,
    #[darling(default, multiple)]
    custom: Vec<Expr>,
    #[darling(default)]
    list: bool,
}

impl Validators {
    pub fn create_validators(
        &self,
        crate_name: &TokenStream,
        value: TokenStream,
        map_err: Option<TokenStream>,
    ) -> Result<TokenStream> {
        let mut list_validators = Vec::new();
        let mut elem_validators = Vec::new();
        let mut codes = Vec::new();

        if let Some(n) = &self.max_items {
            list_validators.push(quote! {
                #crate_name::validators::max_items(__raw_value, #n)
            });
        }

        if let Some(n) = &self.min_items {
            list_validators.push(quote! {
                #crate_name::validators::min_items(__raw_value, #n)
            });
        }

        if let Some(n) = &self.multiple_of {
            elem_validators.push(quote! {
                #crate_name::validators::multiple_of(__raw_value, #n)
            });
        }

        if let Some(n) = &self.min_password_strength {
            elem_validators.push(quote! {
                #crate_name::validators::min_password_strength(__raw_value, #n)
            });
        }

        if let Some(n) = &self.maximum {
            elem_validators.push(quote! {
                #crate_name::validators::maximum(__raw_value, #n)
            });
        }

        if let Some(n) = &self.minimum {
            elem_validators.push(quote! {
                #crate_name::validators::minimum(__raw_value, #n)
            });
        }

        if let Some(n) = &self.max_length {
            elem_validators.push(quote! {
                #crate_name::validators::max_length(__raw_value, #n)
            });
        }

        if let Some(n) = &self.min_length {
            elem_validators.push(quote! {
                #crate_name::validators::min_length(__raw_value, #n)
            });
        }

        if let Some(n) = &self.chars_max_length {
            elem_validators.push(quote! {
                #crate_name::validators::chars_max_length(__raw_value, #n)
            });
        }

        if let Some(n) = &self.chars_min_length {
            elem_validators.push(quote! {
                #crate_name::validators::chars_min_length(__raw_value, #n)
            });
        }

        if self.email {
            elem_validators.push(quote! {
                #crate_name::validators::email(__raw_value)
            });
        }

        if self.url {
            elem_validators.push(quote! {
                #crate_name::validators::url(__raw_value)
            });
        }

        if self.ip {
            elem_validators.push(quote! {
                #crate_name::validators::ip(__raw_value)
            });
        }

        if let Some(re) = &self.regex {
            elem_validators.push(quote! {
                #crate_name::validators::regex(__raw_value, #re)
            });
        }

        if !list_validators.is_empty() {
            codes.push(quote! {
                if let ::std::option::Option::Some(__raw_value) = #crate_name::InputType::as_raw_value(#value) {
                    #(#list_validators #map_err ?;)*
                }
            });
        }

        if !elem_validators.is_empty() {
            if self.list {
                codes.push(quote! {
                    if let ::std::option::Option::Some(value) = #crate_name::InputType::as_raw_value(#value) {
                        for __item in value {
                            if let ::std::option::Option::Some(__raw_value) = #crate_name::InputType::as_raw_value(__item) {
                                #(#elem_validators #map_err ?;)*
                            }
                        }
                    }
                });
            } else {
                codes.push(quote! {
                    if let ::std::option::Option::Some(__raw_value) = #crate_name::InputType::as_raw_value(#value) {
                        #(#elem_validators #map_err ?;)*
                    }
                });
            }
        }

        for expr in &self.custom {
            if self.list {
                codes.push(quote! {
                    if let ::std::option::Option::Some(value) = #crate_name::InputType::as_raw_value(#value) {
                        for __item in value {
                            if let ::std::option::Option::Some(__raw_value) = #crate_name::InputType::as_raw_value(__item) {
                                #crate_name::CustomValidator::check(&(#expr), __raw_value) #map_err ?;
                            }
                        }
                    }
                });
            } else {
                codes.push(quote! {
                    if let ::std::option::Option::Some(__raw_value) = #crate_name::InputType::as_raw_value(#value) {
                        #crate_name::CustomValidator::check(&(#expr), __raw_value) #map_err ?;
                    }
                });
            }
        }

        if codes.is_empty() {
            return Ok(quote!());
        }

        Ok(quote!(#(#codes)*))
    }
}
