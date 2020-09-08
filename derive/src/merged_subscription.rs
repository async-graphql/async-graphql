use crate::args;
use crate::utils::{get_crate_name, get_rustdoc};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Error, LitInt, Result};

pub fn generate(object_args: &args::Object, input: &DeriveInput) -> Result<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let ident = &input.ident;
    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| ident.to_string());

    let desc = object_args
        .desc
        .clone()
        .or_else(|| get_rustdoc(&input.attrs).ok().flatten())
        .map(|s| quote! { Some(#s) })
        .unwrap_or_else(|| quote! {None});

    let s = match &input.data {
        Data::Struct(e) => e,
        _ => return Err(Error::new_spanned(input, "It should be a struct")),
    };

    let mut types = Vec::new();
    for field in &s.fields {
        types.push(&field.ty);
    }

    let create_merged_obj = {
        let mut obj = quote! { #crate_name::MergedObjectSubscriptionTail };
        for i in 0..types.len() {
            let n = LitInt::new(&format!("{}", i), Span::call_site());
            obj = quote! { #crate_name::MergedObject(&self.#n, #obj) };
        }
        quote! {
            #obj
        }
    };

    let merged_type = {
        let mut obj = quote! { #crate_name::MergedObjectTail };
        for ty in &types {
            obj = quote! { #crate_name::MergedObject::<#ty, #obj> };
        }
        obj
    };

    let expanded = quote! {
        #[allow(clippy::all, clippy::pedantic)]
        impl #crate_name::Type for #ident {
            fn type_name() -> ::std::borrow::Cow<'static, str> {
                ::std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> String {
                registry.create_type::<Self, _>(|registry| {
                    #merged_type::create_type_info(registry);

                    let mut fields = Default::default();

                    if let Some(#crate_name::registry::MetaType::Object {
                        fields: obj_fields,
                        ..
                    }) = registry.types.remove(&*#merged_type::type_name()) {
                        fields = obj_fields;
                    }

                    #crate_name::registry::MetaType::Object {
                        name: #gql_typename.to_string(),
                        description: #desc,
                        fields,
                        cache_control: Default::default(),
                        extends: false,
                        keys: None,
                    }
                })
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait]
        impl #crate_name::SubscriptionType for #ident {
            async fn create_field_stream(&self, idx: usize, ctx: &#crate_name::Context<'_>, schema_env: #crate_name::SchemaEnv, query_env: #crate_name::QueryEnv) -> #crate_name::Result<::std::pin::Pin<Box<dyn #crate_name::futures::Stream<Item = #crate_name::Result<#crate_name::serde_json::Value>> + Send>>> {
                #create_merged_obj.create_field_stream(idx, ctx, schema_env, query_env).await
            }
        }
    };
    Ok(expanded.into())
}
