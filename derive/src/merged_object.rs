use crate::args;
use crate::utils::{get_crate_name, get_rustdoc, GeneratorResult};
use darling::ast::Data;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Error, LitInt};

pub fn generate(object_args: &args::MergedObject) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let ident = &object_args.ident;
    let extends = object_args.extends;
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
            return Err(
                Error::new_spanned(ident, "MergedObject can only be applied to an struct.").into(),
            )
        }
    };

    let mut types = Vec::new();
    for field in &s.fields {
        types.push(&field.ty);
    }

    let create_merged_obj = {
        let mut obj = quote! { #crate_name::MergedObjectTail };
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
                    let mut cache_control = Default::default();

                    if let Some(#crate_name::registry::MetaType::Object {
                        fields: obj_fields,
                        cache_control: obj_cache_control,
                        ..
                    }) = registry.types.remove(&*#merged_type::type_name()) {
                        fields = obj_fields;
                        cache_control = obj_cache_control;
                    }

                    #crate_name::registry::MetaType::Object {
                        name: #gql_typename.to_string(),
                        description: #desc,
                        fields,
                        cache_control,
                        extends: #extends,
                        keys: None,
                    }
                })
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait]
        impl #crate_name::resolver_utils::ObjectType for #ident {
            async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::serde_json::Value>> {
                #create_merged_obj.resolve_field(ctx).await
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait]
        impl #crate_name::OutputValueType for #ident {
            async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::serde_json::Value> {
                #crate_name::resolver_utils::resolve_object(ctx, self).await
            }
        }
    };
    Ok(expanded.into())
}
