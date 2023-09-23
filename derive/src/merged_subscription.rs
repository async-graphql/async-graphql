use darling::ast::Data;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Error, LitInt};

use crate::{
    args::{self, RenameTarget},
    utils::{get_crate_name, get_rustdoc, visible_fn, GeneratorResult},
};

pub fn generate(object_args: &args::MergedSubscription) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let ident = &object_args.ident;
    let (impl_generics, ty_generics, where_clause) = object_args.generics.split_for_impl();
    let extends = object_args.extends;
    let gql_typename = if !object_args.name_type {
        let name = object_args
            .name
            .clone()
            .unwrap_or_else(|| RenameTarget::Type.rename(ident.to_string()));
        quote!(::std::borrow::Cow::Borrowed(#name))
    } else {
        quote!(<Self as #crate_name::TypeName>::type_name())
    };

    let desc = get_rustdoc(&object_args.attrs)?
        .map(|s| quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#s)) })
        .unwrap_or_else(|| quote! {::std::option::Option::None});

    let s = match &object_args.data {
        Data::Struct(e) => e,
        _ => {
            return Err(Error::new_spanned(
                ident,
                "MergedSubscription can only be applied to an struct.",
            )
            .into())
        }
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

    let visible = visible_fn(&object_args.visible);
    let expanded = quote! {
        #[allow(clippy::all, clippy::pedantic)]
        impl #impl_generics #crate_name::SubscriptionType for #ident #ty_generics #where_clause {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                #gql_typename
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                registry.create_subscription_type::<Self, _>(|registry| {
                    let mut fields = ::std::default::Default::default();

                    if let #crate_name::registry::MetaType::Object {
                        fields: obj_fields,
                        ..
                    } = registry.create_fake_subscription_type::<#merged_type>() {
                        fields = obj_fields;
                    }

                    #crate_name::registry::MetaType::Object {
                        name: ::std::borrow::Cow::into_owned(#gql_typename),
                        description: #desc,
                        fields,
                        cache_control: ::std::default::Default::default(),
                        extends: #extends,
                        keys: ::std::option::Option::None,
                        visible: #visible,
                        shareable: false,
                        resolvable: true,
                        inaccessible: false,
                        interface_object: false,
                        tags: ::std::default::Default::default(),
                        is_subscription: true,
                        rust_typename: ::std::option::Option::Some(::std::any::type_name::<Self>()),
                        directive_invocations: ::std::default::Default::default(),
                    }
                })
            }

            fn create_field_stream<'__life>(
                &'__life self,
                ctx: &'__life #crate_name::Context<'__life>
            ) -> ::std::option::Option<::std::pin::Pin<::std::boxed::Box<dyn #crate_name::futures_util::stream::Stream<Item = #crate_name::Response> + ::std::marker::Send + '__life>>> {
                ::std::option::Option::None #create_field_stream
            }
        }
    };
    Ok(expanded.into())
}
