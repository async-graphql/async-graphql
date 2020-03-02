use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

pub fn get_crate_name(internal: bool) -> TokenStream {
    match internal {
        true => quote! { crate },
        false => {
            let id = Ident::new("async_graphql", Span::call_site());
            quote! { #id }
        }
    }
}
