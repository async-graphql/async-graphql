use proc_macro::TokenStream;
use quote::quote;
use syn::ItemImpl;

use crate::{
    args::{self, RenameTarget},
    utils::{
        GeneratorResult, gen_boxed_trait, get_crate_name, get_rustdoc, get_type_path_and_name,
        visible_fn,
    },
};

pub fn generate(
    scalar_args: &args::Scalar,
    item_impl: &mut ItemImpl,
) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(scalar_args.internal);
    let boxed_trait = gen_boxed_trait(&crate_name);
    let self_name = get_type_path_and_name(item_impl.self_ty.as_ref())?.1;
    let gql_typename = if !scalar_args.name_type {
        let name = scalar_args
            .name
            .clone()
            .unwrap_or_else(|| RenameTarget::Type.rename(self_name.clone()));
        quote!(::std::borrow::Cow::Borrowed(#name))
    } else {
        quote!(<Self as #crate_name::TypeName>::type_name())
    };

    let desc = if scalar_args.use_type_description {
        quote! { ::std::option::Option::Some(::std::string::ToString::to_string(<Self as #crate_name::Description>::description())) }
    } else {
        get_rustdoc(&item_impl.attrs)?
            .map(|s| quote!(::std::option::Option::Some(::std::string::ToString::to_string(#s))))
            .unwrap_or_else(|| quote!(::std::option::Option::None))
    };

    let self_ty = &item_impl.self_ty;
    let generic = &item_impl.generics;
    let where_clause = &item_impl.generics.where_clause;
    let visible = visible_fn(&scalar_args.visible);
    let inaccessible = scalar_args.inaccessible;
    let tags = scalar_args
        .tags
        .iter()
        .map(|tag| quote!(::std::string::ToString::to_string(#tag)))
        .collect::<Vec<_>>();
    let requires_scopes = scalar_args
        .requires_scopes
        .iter()
        .map(|scopes| quote!(::std::string::ToString::to_string(#scopes)))
        .collect::<Vec<_>>();
    let specified_by_url = match &scalar_args.specified_by_url {
        Some(specified_by_url) => {
            quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#specified_by_url)) }
        }
        None => quote! { ::std::option::Option::None },
    };

    let expanded = quote! {
        #item_impl

        #[allow(clippy::all, clippy::pedantic)]
        impl #generic #crate_name::InputType for #self_ty #where_clause {
            type RawValueType = Self;

            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                #gql_typename
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                registry.create_input_type::<#self_ty, _>(#crate_name::registry::MetaTypeId::Scalar, |_| #crate_name::registry::MetaType::Scalar {
                    name: ::std::borrow::Cow::into_owned(#gql_typename),
                    description: #desc,
                    is_valid: ::std::option::Option::Some(::std::sync::Arc::new(|value| <#self_ty as #crate_name::ScalarType>::is_valid(value))),
                    visible: #visible,
                    inaccessible: #inaccessible,
                    tags: ::std::vec![ #(#tags),* ],
                    specified_by_url: #specified_by_url,
                    directive_invocations: ::std::vec::Vec::new(),
                    requires_scopes: ::std::vec![ #(#requires_scopes),* ],
                })
            }

            fn parse(value: ::std::option::Option<#crate_name::Value>) -> #crate_name::InputValueResult<Self> {
                <#self_ty as #crate_name::ScalarType>::parse(value.unwrap_or_default())
            }

            fn to_value(&self) -> #crate_name::Value {
                <#self_ty as #crate_name::ScalarType>::to_value(self)
            }

            fn as_raw_value(&self) -> ::std::option::Option<&Self::RawValueType> {
                ::std::option::Option::Some(self)
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        impl #generic #crate_name::OutputTypeMarker for #self_ty #where_clause {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                #gql_typename
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                registry.create_output_type::<#self_ty, _>(#crate_name::registry::MetaTypeId::Scalar, |_| #crate_name::registry::MetaType::Scalar {
                    name: ::std::borrow::Cow::into_owned(#gql_typename),
                    description: #desc,
                    is_valid: ::std::option::Option::Some(::std::sync::Arc::new(|value| <#self_ty as #crate_name::ScalarType>::is_valid(value))),
                    visible: #visible,
                    inaccessible: #inaccessible,
                    tags: ::std::vec![ #(#tags),* ],
                    specified_by_url: #specified_by_url,
                    directive_invocations: ::std::vec::Vec::new(),
                    requires_scopes: ::std::vec![ #(#requires_scopes),* ],
                })
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #boxed_trait
        impl #generic #crate_name::OutputType for #self_ty #where_clause {

            async fn resolve(
                &self,
                _: &#crate_name::ContextSelectionSet<'_>,
                _field: &#crate_name::Positioned<#crate_name::parser::types::Field>
            ) -> #crate_name::ServerResult<#crate_name::Value> {
                ::std::result::Result::Ok(#crate_name::ScalarType::to_value(self))
            }
        }
    };
    Ok(expanded.into())
}
