use proc_macro::TokenStream;
use quote::quote;

use crate::args;
use crate::utils::{get_crate_name, get_rustdoc, GeneratorResult};

pub fn generate(desc_args: &args::Description) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(desc_args.internal);
    let ident = &desc_args.ident;
    let generics = &desc_args.generics;
    let where_clause = &generics.where_clause;
    let doc = get_rustdoc(&desc_args.attrs)?.unwrap_or_default();
    let expanded = quote! {
        impl #generics #crate_name::Description for #ident #generics #where_clause {
            fn description() -> &'static str {
                #doc
            }
        }
    };
    Ok(expanded.into())
}
