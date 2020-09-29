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

    let types: Vec<_> = s.fields.iter().map(|field| &field.ty).collect();

    let create_field_stream: proc_macro2::TokenStream = (0..types.len())
        .map(|i| {
            let n = LitInt::new(&i.to_string(), Span::call_site());
            quote!(.or_else(|| #crate_name::SubscriptionType::create_field_stream(&self.#n, ctx)))
        })
        .collect();

    let merged_type = types.iter().fold(
        quote!(#crate_name::MergedObjectTail),
        |obj, ty| quote!(#crate_name::MergedObject::<#ty, #obj>),
    );

    let expanded = quote! {
        #[allow(clippy::all, clippy::pedantic)]
        impl #crate_name::Type for #ident {
            fn type_name() -> ::std::borrow::Cow<'static, str> {
                ::std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                registry.create_type::<Self, _>(|registry| {
                    #merged_type::create_type_info(registry);

                    let mut fields = ::std::default::Default::default();

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
                        cache_control: ::std::default::Default::default(),
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
            ) -> Option<::std::pin::Pin<::std::boxed::Box<dyn #crate_name::futures::Stream<Item = #crate_name::ServerResult<#crate_name::serde_json::Value>> + ::std::marker::Send + 'a>>> {
                None #create_field_stream
            }
        }
    };
    Ok(expanded.into())
}
