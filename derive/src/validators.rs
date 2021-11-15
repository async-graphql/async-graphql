use darling::util::SpannedValue;
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
    #[darling(default, multiple)]
    custom: Vec<SpannedValue<String>>,
    #[darling(default)]
    list: bool,
}

impl Validators {
    pub fn create_validators(
        &self,
        crate_name: &TokenStream,
        value: TokenStream,
        ty: TokenStream,
        map_err: Option<TokenStream>,
    ) -> Result<TokenStream> {
        let mut codes = Vec::new();
        let mut value = value;
        let mut container = None;

        if self.list {
            container = Some(quote!(#value));
            value = quote!(__item);
        }

        if let Some(n) = &self.multiple_of {
            codes.push(quote! {
                #crate_name::validators::multiple_of(#value, #n)
            });
        }

        if let Some(n) = &self.maximum {
            codes.push(quote! {
                #crate_name::validators::maximum(#value, #n)
            });
        }

        if let Some(n) = &self.minimum {
            codes.push(quote! {
                #crate_name::validators::minimum(#value, #n)
            });
        }

        if let Some(n) = &self.max_length {
            codes.push(quote! {
                #crate_name::validators::max_length(#value, #n)
            });
        }

        if let Some(n) = &self.min_length {
            codes.push(quote! {
                #crate_name::validators::min_length(#value, #n)
            });
        }

        if let Some(n) = &self.max_items {
            codes.push(quote! {
                #crate_name::validators::max_items(#value, #n)
            });
        }

        if let Some(n) = &self.min_items {
            codes.push(quote! {
                #crate_name::validators::min_items(#value, #n)
            });
        }

        for s in &self.custom {
            let expr: Expr = syn::parse_str(s)?;
            codes.push(quote! {
                #crate_name::CustomValidator::check(&(#expr), &ctx, #value).await
                    .map_err(|err_msg| #crate_name::InputValueError::<#ty>::custom(err_msg))
            });
        }

        let codes = codes.into_iter().map(|s| quote!(#s  #map_err ?));

        if let Some(container) = container {
            Ok(quote! {
                for __item in #container {
                    #(#codes;)*
                }
            })
        } else {
            Ok(quote!(#(#codes;)*))
        }
    }
}
