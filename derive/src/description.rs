use proc_macro::TokenStream;
use quote::quote;

use crate::args;
use crate::utils::{get_crate_name, get_rustdoc, GeneratorResult};

pub fn generate(desc_args: &args::Description) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(desc_args.internal);
    let ident = &desc_args.ident;
    let (impl_generics, ty_generics, where_clause) = desc_args.generics.split_for_impl();
    let doc = get_rustdoc(&desc_args.attrs)?.unwrap_or_default();
    let expanded = quote! {
        impl #impl_generics #crate_name::Description for #ident #ty_generics #where_clause {
            fn description() -> &'static str {
                #doc
            }
        }
    };
    Ok(expanded.into())
}
