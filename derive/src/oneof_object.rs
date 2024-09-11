use darling::ast::{Data, Style};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, Type};

use crate::{
    args,
    args::{RenameRuleExt, RenameTarget, TypeDirectiveLocation},
    utils::{gen_directive_calls, get_crate_name, get_rustdoc, visible_fn, GeneratorResult},
};

pub fn generate(object_args: &args::OneofObject) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let (impl_generics, ty_generics, where_clause) = object_args.generics.split_for_impl();
    let ident = &object_args.ident;
    let desc = get_rustdoc(&object_args.attrs)?
        .map(|s| quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#s)) })
        .unwrap_or_else(|| quote! {::std::option::Option::None});
    let inaccessible = object_args.inaccessible;
    let tags = object_args
        .tags
        .iter()
        .map(|tag| quote!(::std::string::ToString::to_string(#tag)))
        .collect::<Vec<_>>();
    let directives =
        gen_directive_calls(&object_args.directives, TypeDirectiveLocation::InputObject);
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
    let s = match &object_args.data {
        Data::Enum(s) => s,
        _ => {
            return Err(
                Error::new_spanned(ident, "InputObject can only be applied to an enum.").into(),
            )
        }
    };

    let mut enum_names = Vec::new();
    let mut schema_fields = Vec::new();
    let mut parse_item = Vec::new();
    let mut put_fields = Vec::new();

    for variant in s {
        let enum_name = &variant.ident;
        let field_name = variant.name.clone().unwrap_or_else(|| {
            object_args
                .rename_fields
                .rename(enum_name.to_string(), RenameTarget::Field)
        });
        let inaccessible = variant.inaccessible;
        let tags = variant
            .tags
            .iter()
            .map(|tag| quote!(::std::string::ToString::to_string(#tag)))
            .collect::<Vec<_>>();
        let desc = get_rustdoc(&variant.attrs)?
            .map(|s| quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#s)) })
            .unwrap_or_else(|| quote! {::std::option::Option::None});
        let ty = match variant.fields.style {
            Style::Tuple if variant.fields.fields.len() == 1 => &variant.fields.fields[0],
            Style::Tuple => {
                return Err(Error::new_spanned(
                    enum_name,
                    "Only single value variants are supported",
                )
                .into())
            }
            Style::Unit => {
                return Err(
                    Error::new_spanned(enum_name, "Empty variants are not supported").into(),
                )
            }
            Style::Struct => {
                return Err(Error::new_spanned(
                    enum_name,
                    "Variants with named fields are not supported",
                )
                .into())
            }
        };

        // Type may be wrapped in `Type::Group` if the type comes from a macro
        // substitution, so unwrap it.
        let ty = match ty {
            Type::Group(tg) => &*tg.elem,
            ty => ty,
        };

        let directives = gen_directive_calls(
            &variant.directives,
            TypeDirectiveLocation::InputFieldDefinition,
        );

        if let Type::Path(_) = ty {
            enum_names.push(enum_name);

            let secret = variant.secret;
            let visible = visible_fn(&variant.visible);

            schema_fields.push(quote! {
                fields.insert(::std::borrow::ToOwned::to_owned(#field_name), #crate_name::registry::MetaInputValue {
                    name: ::std::string::ToString::to_string(#field_name),
                    description: #desc,
                    ty: <::std::option::Option<#ty> as #crate_name::InputType>::create_type_info(registry),
                    default_value: ::std::option::Option::None,
                    visible: #visible,
                    inaccessible: #inaccessible,
                    tags: ::std::vec![ #(#tags),* ],
                    is_secret: #secret,
                    directive_invocations: ::std::vec![ #(#directives),* ],
                });
            });

            let validators = variant
                .validator
                .clone()
                .unwrap_or_default()
                .create_validators(
                    &crate_name,
                    quote!(&value),
                    Some(quote!(.map_err(#crate_name::InputValueError::propagate))),
                )?;

            parse_item.push(quote! {
                if obj.contains_key(#field_name) && obj.len() == 1 {
                    let value = #crate_name::InputType::parse(obj.remove(#field_name)).map_err(#crate_name::InputValueError::propagate)?;
                    #validators
                    return ::std::result::Result::Ok(Self::#enum_name(value));
                }
            });

            put_fields.push(quote! {
                Self::#enum_name(value) => {
                    map.insert(#crate_name::Name::new(#field_name), #crate_name::InputType::to_value(value));
                }
            });
        } else {
            return Err(Error::new_spanned(ty, "Invalid type").into());
        }
    }

    let visible = visible_fn(&object_args.visible);
    let expanded = if object_args.concretes.is_empty() {
        quote! {
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
                        oneof: true,
                        directive_invocations: ::std::vec![ #(#directives),* ],
                    })
                }

                fn parse(value: ::std::option::Option<#crate_name::Value>) -> #crate_name::InputValueResult<Self> {
                    if let ::std::option::Option::Some(#crate_name::Value::Object(mut obj)) = value {
                        #(#parse_item)*
                        ::std::result::Result::Err(#crate_name::InputValueError::expected_type(#crate_name::Value::Object(obj)))
                    } else {
                        ::std::result::Result::Err(#crate_name::InputValueError::expected_type(value.unwrap_or_default()))
                    }
                }

                fn to_value(&self) -> #crate_name::Value {
                    let mut map = #crate_name::indexmap::IndexMap::new();
                    match self {
                        #(#put_fields)*
                    }
                    #crate_name::Value::Object(map)
                }

                fn federation_fields() -> ::std::option::Option<::std::string::String> {
                    ::std::option::Option::None
                }

                fn as_raw_value(&self) -> ::std::option::Option<&Self::RawValueType> {
                    ::std::option::Option::Some(self)
                }
            }

            impl #crate_name::InputObjectType for #ident {}
            impl #crate_name::OneofObjectType for #ident {}
        }
    } else {
        let mut code = Vec::new();

        code.push(quote! {
            #[allow(clippy::all, clippy::pedantic)]
            impl #impl_generics #ident #ty_generics #where_clause {
                fn __internal_create_type_info(registry: &mut #crate_name::registry::Registry, name: &str) -> ::std::string::String where Self: #crate_name::InputType {
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
                        oneof: true,
                        directive_invocations: ::std::vec![ #(#directives),* ],
                    })
                }

                fn __internal_parse(value: ::std::option::Option<#crate_name::Value>) -> #crate_name::InputValueResult<Self> where Self: #crate_name::InputType {
                    if let ::std::option::Option::Some(#crate_name::Value::Object(mut obj)) = value {
                        #(#parse_item)*
                        ::std::result::Result::Err(#crate_name::InputValueError::expected_type(#crate_name::Value::Object(obj)))
                    } else {
                        ::std::result::Result::Err(#crate_name::InputValueError::expected_type(value.unwrap_or_default()))
                    }
                }

                fn __internal_to_value(&self) -> #crate_name::Value where Self: #crate_name::InputType {
                    let mut map = #crate_name::indexmap::IndexMap::new();
                    match self {
                        #(#put_fields)*
                    }
                    #crate_name::Value::Object(map)
                }
            }
        });

        for concrete in &object_args.concretes {
            let gql_typename = &concrete.name;
            let params = &concrete.params.0;
            let concrete_type = quote! { #ident<#(#params),*> };

            let def_bounds = if !concrete.bounds.0.is_empty() {
                let bounds = concrete.bounds.0.iter().map(|b| quote!(#b));
                Some(quote!(<#(#bounds),*>))
            } else {
                None
            };

            let expanded = quote! {
                #[allow(clippy::all, clippy::pedantic)]
                impl #def_bounds #crate_name::InputType for #concrete_type {
                    type RawValueType = Self;

                    fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                        ::std::borrow::Cow::Borrowed(#gql_typename)
                    }

                    fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                        Self::__internal_create_type_info(registry, #gql_typename)
                    }

                    fn parse(value: ::std::option::Option<#crate_name::Value>) -> #crate_name::InputValueResult<Self> {
                        Self::__internal_parse(value)
                    }

                    fn to_value(&self) -> #crate_name::Value {
                        self.__internal_to_value()
                    }

                    fn federation_fields() -> ::std::option::Option<::std::string::String> {
                        ::std::option::Option::None
                    }

                    fn as_raw_value(&self) -> ::std::option::Option<&Self::RawValueType> {
                        ::std::option::Option::Some(self)
                    }
                }

                impl #def_bounds #crate_name::InputObjectType for #concrete_type {}
                impl #def_bounds #crate_name::OneofObjectType for #concrete_type {}
            };
            code.push(expanded);
        }
        quote!(#(#code)*)
    };

    Ok(expanded.into())
}
