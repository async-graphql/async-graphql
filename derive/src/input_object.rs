use darling::ast::Data;
use proc_macro::TokenStream;
use quote::quote;
use syn::{ext::IdentExt, Error};

use crate::{
    args::{self, RenameRuleExt, RenameTarget},
    utils::{generate_default, get_crate_name, get_rustdoc, visible_fn, GeneratorResult},
};

pub fn generate(object_args: &args::InputObject) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let (impl_generics, ty_generics, where_clause) = object_args.generics.split_for_impl();
    let ident = &object_args.ident;
    let inaccessible = object_args.inaccessible;
    let tags = object_args
        .tags
        .iter()
        .map(|tag| quote!(::std::string::ToString::to_string(#tag)))
        .collect::<Vec<_>>();
    let s = match &object_args.data {
        Data::Struct(s) => s,
        _ => {
            return Err(
                Error::new_spanned(ident, "InputObject can only be applied to an struct.").into(),
            )
        }
    };

    for field in &s.fields {
        if field.ident.is_none() {
            return Err(Error::new_spanned(ident, "All fields must be named.").into());
        }
    }

    let gql_typename = if !object_args.name_type {
        let name = object_args
            .input_name
            .clone()
            .or_else(|| object_args.name.clone())
            .unwrap_or_else(|| RenameTarget::Type.rename(ident.to_string()));

        quote!(::std::borrow::Cow::Borrowed(#name))
    } else {
        quote!(<Self as #crate_name::TypeName>::type_name())
    };

    let desc = get_rustdoc(&object_args.attrs)?
        .map(|s| quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#s)) })
        .unwrap_or_else(|| quote! {::std::option::Option::None});

    let mut get_fields = Vec::new();
    let mut put_fields = Vec::new();
    let mut fields = Vec::new();
    let mut schema_fields = Vec::new();
    let mut flatten_fields = Vec::new();
    let mut federation_fields = Vec::new();

    for field in &s.fields {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let name = field.name.clone().unwrap_or_else(|| {
            object_args
                .rename_fields
                .rename(ident.unraw().to_string(), RenameTarget::Field)
        });
        let inaccessible = field.inaccessible;
        let tags = field
            .tags
            .iter()
            .map(|tag| quote!(::std::string::ToString::to_string(#tag)))
            .collect::<Vec<_>>();

        if field.skip || field.skip_input {
            get_fields.push(quote! {
                let #ident: #ty = ::std::default::Default::default();
            });
            fields.push(ident);
            continue;
        }

        federation_fields.push((ty, name.clone()));

        let process_with = match field.process_with.as_ref() {
            Some(fn_path) => quote! { #fn_path(&mut #ident); },
            None => Default::default(),
        };

        let validators = field
            .validator
            .clone()
            .unwrap_or_default()
            .create_validators(
                &crate_name,
                quote!(&#ident),
                Some(quote!(.map_err(#crate_name::InputValueError::propagate))),
            )?;

        if field.flatten {
            flatten_fields.push((ident, ty));

            schema_fields.push(quote! {
                #crate_name::static_assertions::assert_impl_one!(#ty: #crate_name::InputObjectType);
                <#ty as  #crate_name::InputType>::create_type_info(registry);
                if let #crate_name::registry::MetaType::InputObject { input_fields, .. } =
                    registry.create_fake_input_type::<#ty>() {
                    fields.extend(input_fields);
                }
            });

            get_fields.push(quote! {
                #[allow(unused_mut)]
                let mut #ident: #ty = #crate_name::InputType::parse(
                    ::std::option::Option::Some(#crate_name::Value::Object(::std::clone::Clone::clone(&obj)))
                ).map_err(#crate_name::InputValueError::propagate)?;
                #process_with
                #validators
            });

            fields.push(ident);

            put_fields.push(quote! {
                if let #crate_name::Value::Object(values) = #crate_name::InputType::to_value(&self.#ident) {
                    map.extend(values);
                }
            });
            continue;
        }

        let desc = get_rustdoc(&field.attrs)?
            .map(|s| quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#s)) })
            .unwrap_or_else(|| quote! {::std::option::Option::None});
        let default = generate_default(&field.default, &field.default_with)?;
        let schema_default = default
            .as_ref()
            .map(|value| {
                quote! {
                    ::std::option::Option::Some(::std::string::ToString::to_string(
                        &<#ty as #crate_name::InputType>::to_value(&#value)
                    ))
                }
            })
            .unwrap_or_else(|| quote!(::std::option::Option::None));
        let secret = field.secret;

        if let Some(default) = default {
            get_fields.push(quote! {
                #[allow(non_snake_case)]
                let #ident: #ty = {
                    match obj.get(#name) {
                        ::std::option::Option::Some(value) => {
                            #[allow(unused_mut)]
                            let mut #ident = #crate_name::InputType::parse(::std::option::Option::Some(::std::clone::Clone::clone(&value)))
                                .map_err(#crate_name::InputValueError::propagate)?;
                            #process_with
                            #ident

                        },
                        ::std::option::Option::None => #default,
                    }
                };
                #validators
            });
        } else {
            get_fields.push(quote! {
                #[allow(non_snake_case, unused_mut)]
                let mut #ident: #ty = #crate_name::InputType::parse(obj.get(#name).cloned())
                    .map_err(#crate_name::InputValueError::propagate)?;
                #process_with
                #validators
            });
        }

        put_fields.push(quote! {
            map.insert(
                #crate_name::Name::new(#name),
                #crate_name::InputType::to_value(&self.#ident)
            );
        });

        fields.push(ident);
        let visible = visible_fn(&field.visible);
        schema_fields.push(quote! {
            fields.insert(::std::borrow::ToOwned::to_owned(#name), #crate_name::registry::MetaInputValue {
                name: ::std::string::ToString::to_string(#name),
                description: #desc,
                ty: <#ty as #crate_name::InputType>::create_type_info(registry),
                default_value: #schema_default,
                visible: #visible,
                inaccessible: #inaccessible,
                tags: ::std::vec![ #(#tags),* ],
                is_secret: #secret,
            });
        })
    }

    if get_fields.is_empty() {
        return Err(Error::new_spanned(
            ident,
            "A GraphQL Input Object type must define one or more input fields.",
        )
        .into());
    }

    let visible = visible_fn(&object_args.visible);

    let get_federation_fields = {
        let fields = federation_fields.into_iter().map(|(ty, name)| {
            quote! {
                if let ::std::option::Option::Some(fields) = <#ty as #crate_name::InputType>::federation_fields() {
                    res.push(::std::format!("{} {}", #name, fields));
                } else {
                    res.push(::std::string::ToString::to_string(#name));
                }
            }
        });
        quote! {
            let mut res = ::std::vec::Vec::new();
            #(#fields)*
            ::std::option::Option::Some(::std::format!("{{ {} }}", res.join(" ")))
        }
    };

    let obj_validator = object_args
        .validator
        .as_ref()
        .map(|expr| quote! { #crate_name::CustomValidator::check(&#expr, &obj)?; });

    let expanded = if object_args.concretes.is_empty() {
        quote! {
            #[allow(clippy::all, clippy::pedantic)]
            impl #crate_name::InputType for #ident {
                type RawValueType = Self;

                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    #gql_typename
                }

                fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                    registry.create_input_type::<Self, _>(#crate_name::registry::MetaTypeId::InputObject, |registry| #crate_name::registry::MetaType::InputObject {
                        name: ::std::borrow::Cow::into_owned(#gql_typename),
                        description: #desc,
                        input_fields: {
                            let mut fields = #crate_name::indexmap::IndexMap::new();
                            #(#schema_fields)*
                            fields
                        },
                        visible: #visible,
                        inaccessible: #inaccessible,
                        tags: ::std::vec![ #(#tags),* ],
                        rust_typename: ::std::option::Option::Some(::std::any::type_name::<Self>()),
                        oneof: false,
                    })
                }

                fn parse(value: ::std::option::Option<#crate_name::Value>) -> #crate_name::InputValueResult<Self> {
                    if let ::std::option::Option::Some(#crate_name::Value::Object(obj)) = value {
                        #(#get_fields)*
                        let obj = Self { #(#fields),* };
                        #obj_validator
                        ::std::result::Result::Ok(obj)
                    } else {
                        ::std::result::Result::Err(#crate_name::InputValueError::expected_type(value.unwrap_or_default()))
                    }
                }

                fn to_value(&self) -> #crate_name::Value {
                    let mut map = #crate_name::indexmap::IndexMap::new();
                    #(#put_fields)*
                    #crate_name::Value::Object(map)
                }

                fn federation_fields() -> ::std::option::Option<::std::string::String> {
                    #get_federation_fields
                }

                fn as_raw_value(&self) -> ::std::option::Option<&Self::RawValueType> {
                    ::std::option::Option::Some(self)
                }
            }

            impl #crate_name::InputObjectType for #ident {}
        }
    } else {
        let mut code = Vec::new();

        code.push(quote! {
            #[allow(clippy::all, clippy::pedantic)]
            impl #impl_generics #ident #ty_generics #where_clause {
                fn __internal_create_type_info_input_object(registry: &mut #crate_name::registry::Registry, name: &str) -> ::std::string::String where Self: #crate_name::InputType {
                    registry.create_input_type::<Self, _>(#crate_name::registry::MetaTypeId::InputObject, |registry| #crate_name::registry::MetaType::InputObject {
                        name: ::std::borrow::ToOwned::to_owned(name),
                        description: #desc,
                        input_fields: {
                            let mut fields = #crate_name::indexmap::IndexMap::new();
                            #(#schema_fields)*
                            fields
                        },
                        visible: #visible,
                        inaccessible: #inaccessible,
                        tags: ::std::vec![ #(#tags),* ],
                        rust_typename: ::std::option::Option::Some(::std::any::type_name::<Self>()),
                        oneof: false,
                    })
                }

                fn __internal_parse(value: ::std::option::Option<#crate_name::Value>) -> #crate_name::InputValueResult<Self> where Self: #crate_name::InputType {
                    if let ::std::option::Option::Some(#crate_name::Value::Object(obj)) = value {
                        #(#get_fields)*
                        let obj = Self { #(#fields),* };
                        #obj_validator
                        ::std::result::Result::Ok(obj)
                    } else {
                        ::std::result::Result::Err(#crate_name::InputValueError::expected_type(value.unwrap_or_default()))
                    }
                }

                fn __internal_to_value(&self) -> #crate_name::Value where Self: #crate_name::InputType {
                    let mut map = #crate_name::indexmap::IndexMap::new();
                    #(#put_fields)*
                    #crate_name::Value::Object(map)
                }

                fn __internal_federation_fields() -> ::std::option::Option<::std::string::String> where Self: #crate_name::InputType {
                    #get_federation_fields
                }
            }
        });

        for concrete in &object_args.concretes {
            let gql_typename = &concrete.name;
            let params = &concrete.params.0;
            let concrete_type = quote! { #ident<#(#params),*> };

            let expanded = quote! {
                #[allow(clippy::all, clippy::pedantic)]
                impl #crate_name::InputType for #concrete_type {
                    type RawValueType = Self;

                    fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                        ::std::borrow::Cow::Borrowed(#gql_typename)
                    }

                    fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                        Self::__internal_create_type_info_input_object(registry, #gql_typename)
                    }

                    fn parse(value: ::std::option::Option<#crate_name::Value>) -> #crate_name::InputValueResult<Self> {
                        Self::__internal_parse(value)
                    }

                    fn to_value(&self) -> #crate_name::Value {
                        self.__internal_to_value()
                    }

                    fn federation_fields() -> ::std::option::Option<::std::string::String> {
                        Self::__internal_federation_fields()
                    }

                    fn as_raw_value(&self) -> ::std::option::Option<&Self::RawValueType> {
                        ::std::option::Option::Some(self)
                    }
                }

                impl #crate_name::InputObjectType for #concrete_type {}
            };
            code.push(expanded);
        }
        quote!(#(#code)*)
    };

    Ok(expanded.into())
}
