use crate::args;
use crate::utils::{get_crate_name, get_rustdoc, GeneratorResult};
use darling::ast::Data;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Error, LitInt};

pub fn generate(object_args: &args::MergedSubscription) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let ident = &object_args.ident;
    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| ident.to_string());

    let desc = get_rustdoc(&object_args.attrs)?
        .map(|s| quote! { Some(#s) })
        .unwrap_or_else(|| quote! {None});

    let s = match &object_args.data {
        Data::Struct(e) => e,
        _ => {
            return Err(Error::new_spanned(
                &ident,
                "MergedSubscription can only be applied to an struct.",
            )
            .into())
        }
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
        impl #crate_name::SubscriptionType for #ident {
            fn create_field_stream<'a>(
                &'a self,
                ctx: &'a #crate_name::Context<'a>
            ) -> ::std::pin::Pin<::std::boxed::Box<dyn #crate_name::futures::Stream<Item = #crate_name::Result<#crate_name::serde_json::Value>> + Send + 'a>> {
                ::std::boxed::Box::pin(#crate_name::async_stream::stream! {
                    let obj = #create_merged_obj;
                    let mut stream = obj.create_field_stream(ctx);
                    while let Some(item) = #crate_name::futures::stream::StreamExt::next(&mut stream).await {
                        yield item;
                    }
                })
            }
        }
    };
    Ok(expanded.into())
}
