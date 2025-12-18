use darling::ast::{Data, Style};
use proc_macro::TokenStream;
use quote::quote;
use syn::Error;

use crate::{
    args::{self, NewTypeName, RenameTarget},
    utils::{GeneratorResult, gen_boxed_trait, get_crate_name, get_rustdoc, visible_fn},
};

pub fn generate(newtype_args: &args::NewType) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(newtype_args.internal);
    let boxed_trait = gen_boxed_trait(&crate_name);
    let ident = &newtype_args.ident;
    let (impl_generics, ty_generics, where_clause) = newtype_args.generics.split_for_impl();
    let inaccessible = newtype_args.inaccessible;
    let tags = newtype_args
        .tags
        .iter()
        .map(|tag| quote!(::std::string::ToString::to_string(#tag)))
        .collect::<Vec<_>>();
    let gql_typename = match &newtype_args.name {
        NewTypeName::New(name) => Some(name.clone()),
        NewTypeName::Rust => Some(RenameTarget::Type.rename(ident.to_string())),
        NewTypeName::Original => None,
    };
    let desc = get_rustdoc(&newtype_args.attrs)?
        .map(|s| quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#s)) })
        .unwrap_or_else(|| quote! {::std::option::Option::None});
    let visible = visible_fn(&newtype_args.visible);

    let fields = match &newtype_args.data {
        Data::Struct(e) => e,
        _ => {
            return Err(
                Error::new_spanned(ident, "NewType can only be applied to an struct.").into(),
            );
        }
    };

    if fields.style == Style::Tuple && fields.fields.len() != 1 {
        return Err(Error::new_spanned(ident, "Invalid type.").into());
    }
    let inner_ty = &fields.fields[0];
    let type_name = match &gql_typename {
        Some(name) => quote! { ::std::borrow::Cow::Borrowed(#name) },
        None => quote! { <#inner_ty as #crate_name::InputType>::type_name() },
    };
    let create_type_info = if let Some(name) = &gql_typename {
        let specified_by_url = match &newtype_args.specified_by_url {
            Some(specified_by_url) => quote! { ::std::option::Option::Some(#specified_by_url) },
            None => quote! { ::std::option::Option::None },
        };

        quote! {
            registry.create_input_type::<#ident, _>(#crate_name::registry::MetaTypeId::Scalar, |_| #crate_name::registry::MetaType::Scalar {
                name: ::std::borrow::ToOwned::to_owned(#name),
                description: #desc,
                is_valid: ::std::option::Option::Some(::std::sync::Arc::new(|value| <#ident as #crate_name::ScalarType>::is_valid(value))),
                visible: #visible,
                inaccessible: #inaccessible,
                tags: ::std::vec![ #(#tags),* ],
                specified_by_url: #specified_by_url,
                directive_invocations: ::std::vec::Vec::new(),
                requires_scopes: ::std::vec::Vec::new(),
            })
        }
    } else {
        quote! { <#inner_ty as #crate_name::InputType>::create_type_info(registry) }
    };

    let expanded = quote! {
        #[allow(clippy::all, clippy::pedantic)]
        impl #impl_generics #crate_name::ScalarType for #ident #ty_generics #where_clause {
            fn parse(value: #crate_name::Value) -> #crate_name::InputValueResult<Self> {
                <#inner_ty as #crate_name::ScalarType>::parse(value).map(#ident).map_err(#crate_name::InputValueError::propagate)
            }

            fn to_value(&self) -> #crate_name::Value {
                <#inner_ty as #crate_name::ScalarType>::to_value(&self.0)
            }
        }

        impl #impl_generics ::std::convert::From<#inner_ty> for #ident #ty_generics #where_clause {
            fn from(value: #inner_ty) -> Self {
                Self(value)
            }
        }

        #[allow(clippy::from_over_into)]
        impl #impl_generics ::std::convert::Into<#inner_ty> for #ident #ty_generics #where_clause {
            fn into(self) -> #inner_ty {
                self.0
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        impl #impl_generics #crate_name::InputType for #ident #ty_generics #where_clause {
            type RawValueType = #inner_ty;

            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                #type_name
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                #create_type_info
            }

            fn parse(value: ::std::option::Option<#crate_name::Value>) -> #crate_name::InputValueResult<Self> {
                <#ident as #crate_name::ScalarType>::parse(value.unwrap_or_default())
            }

            fn to_value(&self) -> #crate_name::Value {
                <#ident as #crate_name::ScalarType>::to_value(self)
            }

            fn as_raw_value(&self) -> ::std::option::Option<&Self::RawValueType> {
                self.0.as_raw_value()
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        impl #impl_generics #crate_name::OutputTypeMarker for #ident #ty_generics #where_clause {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                #type_name
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                #create_type_info
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #boxed_trait
        impl #impl_generics #crate_name::OutputType for #ident #ty_generics #where_clause {

            async fn resolve(
                &self,
                _: &#crate_name::ContextSelectionSet<'_>,
                _field: &#crate_name::Positioned<#crate_name::parser::types::Field>
            ) -> #crate_name::ServerResult<#crate_name::Value> {
                Ok(#crate_name::ScalarType::to_value(self))
            }
        }
    };

    Ok(expanded.into())
}
