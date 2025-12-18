use darling::ast::Data;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Error, LitInt};

use crate::{
    args::{self, RenameTarget, TypeDirectiveLocation},
    utils::{
        GeneratorResult, gen_boxed_trait, gen_directive_calls, get_crate_name, get_rustdoc,
        visible_fn,
    },
};

pub fn generate(object_args: &args::MergedObject) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let boxed_trait = gen_boxed_trait(&crate_name);
    let ident = &object_args.ident;
    let (impl_generics, ty_generics, where_clause) = object_args.generics.split_for_impl();
    let extends = object_args.extends;
    let shareable = object_args.shareable;
    let inaccessible = object_args.inaccessible;
    let interface_object = object_args.interface_object;
    let tags = object_args
        .tags
        .iter()
        .map(|tag| quote!(::std::string::ToString::to_string(#tag)))
        .collect::<Vec<_>>();
    let gql_typename = if !object_args.name_type {
        let name = object_args
            .name
            .clone()
            .unwrap_or_else(|| RenameTarget::Type.rename(ident.to_string()));
        quote!(::std::borrow::Cow::Borrowed(#name))
    } else {
        quote!(<Self as #crate_name::TypeName>::type_name())
    };

    let directives = gen_directive_calls(&object_args.directives, TypeDirectiveLocation::Object);

    let desc = get_rustdoc(&object_args.attrs)?
        .map(|s| quote! { ::std::option::Option::Some(::std::string::ToString::to_string(#s)) })
        .unwrap_or_else(|| quote! {::std::option::Option::None});

    let s = match &object_args.data {
        Data::Struct(e) => e,
        _ => {
            return Err(Error::new_spanned(
                ident,
                "MergedObject can only be applied to an struct.",
            )
            .into());
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

    let visible = visible_fn(&object_args.visible);
    let resolve_container = if object_args.serial {
        if cfg!(feature = "boxed-trait") {
            quote! { #crate_name::resolver_utils::resolve_container_serial(ctx, &self as &dyn #crate_name::resolver_utils::ContainerType, &self).await }
        } else {
            quote! { #crate_name::resolver_utils::resolve_container_serial(ctx, self).await }
        }
    } else {
        if cfg!(feature = "boxed-trait") {
            quote! { #crate_name::resolver_utils::resolve_container(ctx, &self as &dyn #crate_name::resolver_utils::ContainerType, &self).await }
        } else {
            quote! { #crate_name::resolver_utils::resolve_container(ctx, self).await }
        }
    };

    let expanded = quote! {
        #[allow(clippy::all, clippy::pedantic)]
        #boxed_trait
        impl #impl_generics #crate_name::resolver_utils::ContainerType for #ident #ty_generics #where_clause {
            async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                #create_merged_obj.resolve_field(ctx).await
            }

            async fn find_entity(&self, ctx: &#crate_name::Context<'_>, params: &#crate_name::Value) ->  #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
               #create_merged_obj.find_entity(ctx, params).await
            }
        }

        impl #impl_generics #crate_name::OutputTypeMarker for #ident #ty_generics #where_clause {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                #gql_typename
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> ::std::string::String {
                registry.create_output_type::<Self, _>(#crate_name::registry::MetaTypeId::Object, |registry| {
                    let mut fields = ::std::default::Default::default();
                    let mut cache_control = ::std::default::Default::default();

                    if let #crate_name::registry::MetaType::Object {
                        fields: obj_fields,
                        cache_control: obj_cache_control,
                        ..
                    } = registry.create_fake_output_type::<#merged_type>() {
                        fields = obj_fields;
                        cache_control = obj_cache_control;
                    }

                    #crate_name::registry::MetaType::Object {
                        name: ::std::borrow::Cow::into_owned(#gql_typename),
                        description: #desc,
                        fields,
                        cache_control,
                        extends: #extends,
                        shareable: #shareable,
                        resolvable: true,
                        inaccessible: #inaccessible,
                        interface_object: #interface_object,
                        tags: ::std::vec![ #(#tags),* ],
                        keys: ::std::option::Option::None,
                        visible: #visible,
                        is_subscription: false,
                        rust_typename: ::std::option::Option::Some(::std::any::type_name::<Self>()),
                        directive_invocations: ::std::vec![ #(#directives),* ],
                        requires_scopes: ::std::vec![],
                    }
                })
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #boxed_trait
        impl #impl_generics #crate_name::OutputType for #ident #ty_generics #where_clause {

            async fn resolve(&self, ctx: &#crate_name::ContextSelectionSet<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::Value> {
                #resolve_container
            }
        }

        impl #impl_generics #crate_name::ObjectType for #ident #ty_generics #where_clause {}
    };
    Ok(expanded.into())
}
