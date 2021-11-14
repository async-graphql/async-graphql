use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;

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
    custom: Vec<String>,
}

impl Validators {
    pub fn create_validators(
        &self,
        crate_name: &TokenStream,
        value: TokenStream,
        map_err: TokenStream,
    ) -> TokenStream {
        let mut codes = Vec::new();

        if let Some(n) = &self.multiple_of {
            codes.push(quote! {
                #crate_name::validators::multiple_of(#value, #n) #map_err
            });
        }

        if let Some(n) = &self.maximum {
            codes.push(quote! {
                #crate_name::validators::maximum(#value, #n) #map_err
            });
        }

        if let Some(n) = &self.minimum {
            codes.push(quote! {
                #crate_name::validators::minimum(#value, #n) #map_err
            });
        }

        if let Some(n) = &self.max_length {
            codes.push(quote! {
                #crate_name::validators::max_length(#value, #n) #map_err
            });
        }

        if let Some(n) = &self.min_length {
            codes.push(quote! {
                #crate_name::validators::min_length(#value, #n) #map_err
            });
        }

        if let Some(n) = &self.max_items {
            codes.push(quote! {
                #crate_name::validators::max_items(#value, #n) #map_err
            });
        }

        if let Some(n) = &self.min_items {
            codes.push(quote! {
                #crate_name::validators::min_items(#value, #n) #map_err
            });
        }

        quote!(#(#codes;)*)
    }
}
