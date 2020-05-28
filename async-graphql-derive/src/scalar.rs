use crate::args;
use crate::utils::{check_reserved_name, get_crate_name, get_rustdoc};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, ItemImpl, Result, Type};

pub fn generate(scalar_args: &args::Scalar, item_impl: &mut ItemImpl) -> Result<TokenStream> {
    let self_name = match item_impl.self_ty.as_ref() {
        Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap(),
        _ => return Err(Error::new_spanned(&item_impl.self_ty, "Invalid type")),
    };
    let gql_typename = scalar_args
        .name
        .clone()
        .unwrap_or_else(|| self_name.clone());
    check_reserved_name(&gql_typename, scalar_args.internal)?;
    let desc = scalar_args
        .desc
        .clone()
        .or_else(|| get_rustdoc(&item_impl.attrs).ok().flatten())
        .map(|s| quote! { Some(#s) })
        .unwrap_or_else(|| quote! {None});
    let self_ty = &item_impl.self_ty;
    let generic = &item_impl.generics;
    let where_clause = &item_impl.generics.where_clause;
    let crate_name = get_crate_name(scalar_args.internal);
    let expanded = quote! {
        #item_impl

        impl #generic #crate_name::Type for #self_ty #where_clause {
            fn type_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> String {
                registry.create_type::<#self_ty, _>(|_| #crate_name::registry::MetaType::Scalar {
                    name: #gql_typename.to_string(),
                    description: #desc,
                    is_valid: |value| <#self_ty as #crate_name::ScalarType>::is_valid(value),
                })
            }
        }

        impl #generic #crate_name::InputValueType for #self_ty #where_clause {
            fn parse(value: #crate_name::Value) -> #crate_name::InputValueResult<Self> {
                <#self_ty as #crate_name::ScalarType>::parse(value)
            }

            fn to_value(&self) -> #crate_name::Value {
                <#self_ty as #crate_name::ScalarType>::to_value(self)
            }
        }

        #[allow(clippy::ptr_arg)]
        #[#crate_name::async_trait::async_trait]
        impl #generic #crate_name::OutputValueType for #self_ty #where_clause {
            async fn resolve(
                &self,
                _: &#crate_name::ContextSelectionSet<'_>,
                _field: &#crate_name::Positioned<#crate_name::parser::query::Field>
            ) -> #crate_name::Result<#crate_name::serde_json::Value> {
                Ok(#crate_name::ScalarType::to_value(self).into())
            }
        }
    };
    Ok(expanded.into())
}
