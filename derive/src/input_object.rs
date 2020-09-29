use crate::args;
use crate::utils::{
    generate_default, generate_validator, get_crate_name, get_rustdoc, GeneratorResult,
};
use darling::ast::Data;
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::Error;

pub fn generate(object_args: &args::InputObject) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let ident = &object_args.ident;
    let s = match &object_args.data {
        Data::Struct(s) => s,
        _ => {
            return Err(
                Error::new_spanned(ident, "InputObject can only be applied to an struct.").into(),
            )
        }
    };

    let mut struct_fields = Vec::new();
    for field in &s.fields {
        let vis = &field.vis;
        let ty = &field.ty;
        let ident = &field.ident;
        let attrs = field
            .attrs
            .iter()
            .filter(|attr| !attr.path.is_ident("field"))
            .collect::<Vec<_>>();
        struct_fields.push(quote! {
            #(#attrs)*
            #vis #ident: #ty
        });
    }

    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| ident.to_string());

    let desc = get_rustdoc(&object_args.attrs)?
        .map(|s| quote! { Some(#s) })
        .unwrap_or_else(|| quote! {None});

    let mut get_fields = Vec::new();
    let mut put_fields = Vec::new();
    let mut fields = Vec::new();
    let mut schema_fields = Vec::new();
    let mut flatten_fields = Vec::new();

    for field in &s.fields {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let name = field
            .name
            .clone()
            .unwrap_or_else(|| ident.unraw().to_string().to_camel_case());

        if field.flatten {
            flatten_fields.push((ident, ty));

            schema_fields.push(quote! {
                #crate_name::static_assertions::assert_impl_one!(#ty: #crate_name::InputObjectType);
                #ty::create_type_info(registry);
                if let Some(#crate_name::registry::MetaType::InputObject{ input_fields, .. }) =
                    registry.types.remove(&*<#ty as #crate_name::Type>::type_name()) {
                    fields.extend(input_fields);
                }
            });

            get_fields.push(quote! {
                let #ident: #ty = #crate_name::InputValueType::parse(Some(#crate_name::Value::Object(obj.clone())))?;
            });

            fields.push(ident);

            put_fields.push(quote! {
                if let #crate_name::Value::Object(values) = #crate_name::InputValueType::to_value(&self.#ident) {
                    map.extend(values);
                }
            });
            continue;
        }

        let validator = match &field.validator {
            Some(meta) => {
                let stream = generate_validator(&crate_name, meta)?;
                quote!(Some(#stream))
            }
            None => quote!(None),
        };
        let desc = get_rustdoc(&field.attrs)?
            .map(|s| quote! { Some(#s) })
            .unwrap_or_else(|| quote! {None});
        let default = generate_default(&field.default, &field.default_with)?;
        let schema_default = default
            .as_ref()
            .map(|value| {
                quote! {Some( <#ty as #crate_name::InputValueType>::to_value(&#value).to_string() )}
            })
            .unwrap_or_else(|| quote! {None});

        if let Some(default) = default {
            get_fields.push(quote! {
                let #ident: #ty = {
                    match obj.get(#name) {
                        Some(value) => #crate_name::InputValueType::parse(Some(value.clone()))?,
                        None => #default,
                    }
                };
            });
        } else {
            get_fields.push(quote! {
                let #ident:#ty = #crate_name::InputValueType::parse(obj.get(#name).cloned())?;
            });
        }

        put_fields.push(quote! {
            map.insert(
                #crate_name::parser::types::Name::new_unchecked(#name.to_owned()),
                #crate_name::InputValueType::to_value(&self.#ident)
            );
        });

        fields.push(ident);
        schema_fields.push(quote! {
            fields.insert(#name.to_string(), #crate_name::registry::MetaInputValue {
                name: #name,
                description: #desc,
                ty: <#ty as #crate_name::Type>::create_type_info(registry),
                default_value: #schema_default,
                validator: #validator,
            });
        })
    }

    let expanded = quote! {
        #[allow(clippy::all, clippy::pedantic)]
        impl #crate_name::Type for #ident {
            fn type_name() -> ::std::borrow::Cow<'static, str> {
                ::std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> String {
                registry.create_type::<Self, _>(|registry| #crate_name::registry::MetaType::InputObject {
                    name: #gql_typename.to_string(),
                    description: #desc,
                    input_fields: {
                        let mut fields = #crate_name::indexmap::IndexMap::new();
                        #(#schema_fields)*
                        fields
                    }
                })
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        impl #crate_name::InputValueType for #ident {
            fn parse(value: Option<#crate_name::Value>) -> #crate_name::InputValueResult<Self> {
                if let Some(#crate_name::Value::Object(obj)) = value {
                    #(#get_fields)*
                    Ok(Self { #(#fields),* })
                } else {
                    Err(#crate_name::InputValueError::ExpectedType(value.unwrap_or_default()))
                }
            }

            fn to_value(&self) -> #crate_name::Value {
                let mut map = ::std::collections::BTreeMap::new();
                #(#put_fields)*
                #crate_name::Value::Object(map)
            }
        }

        impl #crate_name::InputObjectType for #ident {}
    };
    Ok(expanded.into())
}
