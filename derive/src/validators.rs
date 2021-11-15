use darling::util::SpannedValue;
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Result};

#[derive(FromMeta, Default, Clone)]
pub struct Validators {
    #[darling(default)]
    multiple_of: Option<f64>,
    #[darling(default)]
    maximum: Option<f64>,
    #[darling(default)]
    minimum: Option<f64>,
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
        Ok(quote!(#(#codes;)*))
    }
}
