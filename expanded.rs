#![feature(prelude_import)]
#[macro_use]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use async_graphql::*;
extern crate test;
#[rustc_test_marker = "test_union_simple_object"]
#[doc(hidden)]
pub const test_union_simple_object: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("test_union_simple_object"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/union.rs",
        start_line: 4usize,
        start_col: 14usize,
        end_line: 4usize,
        end_col: 38usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(test_union_simple_object()),
    ),
};
pub fn test_union_simple_object() {
    let body = async {
        struct MyObj {
            id: i32,
            title: String,
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl MyObj {
            #[inline]
            #[allow(missing_docs)]
            async fn id(
                &self,
                ctx: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<&i32> {
                ::std::result::Result::Ok(&self.id)
            }
            #[inline]
            #[allow(missing_docs)]
            async fn title(
                &self,
                ctx: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<&String> {
                ::std::result::Result::Ok(&self.title)
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::resolver_utils::ContainerType for MyObj {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = {
                        if ctx.item.node.name.node == "id" {
                            let f = async move {
                                __self
                                    .id(ctx)
                                    .await
                                    .map_err(|err| err.into_server_error(ctx.item.pos))
                            };
                            let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                            let ctx_obj = ctx
                                .with_selection_set(&ctx.item.node.selection_set);
                            return async_graphql::OutputType::resolve(
                                    &obj,
                                    &ctx_obj,
                                    ctx.item,
                                )
                                .await
                                .map(::std::option::Option::Some);
                        }
                        if ctx.item.node.name.node == "title" {
                            let f = async move {
                                __self
                                    .title(ctx)
                                    .await
                                    .map_err(|err| err.into_server_error(ctx.item.pos))
                            };
                            let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                            let ctx_obj = ctx
                                .with_selection_set(&ctx.item.node.selection_set);
                            return async_graphql::OutputType::resolve(
                                    &obj,
                                    &ctx_obj,
                                    ctx.item,
                                )
                                .await
                                .map(::std::option::Option::Some);
                        }
                        ::std::result::Result::Ok(::std::option::Option::None)
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputTypeMarker for MyObj {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("MyObj")
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Object,
                        |registry| async_graphql::registry::MetaType::Object {
                            name: ::std::borrow::Cow::into_owned(
                                ::std::borrow::Cow::Borrowed("MyObj"),
                            ),
                            description: ::std::option::Option::None,
                            fields: {
                                let mut fields = async_graphql::indexmap::IndexMap::new();
                                fields
                                    .insert(
                                        ::std::borrow::ToOwned::to_owned("id"),
                                        async_graphql::registry::MetaField {
                                            name: ::std::borrow::ToOwned::to_owned("id"),
                                            description: ::std::option::Option::None,
                                            args: ::std::default::Default::default(),
                                            ty: <i32 as async_graphql::OutputTypeMarker>::create_type_info(
                                                registry,
                                            ),
                                            deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                            cache_control: async_graphql::CacheControl {
                                                public: true,
                                                max_age: 0i32,
                                            },
                                            external: false,
                                            provides: ::std::option::Option::None,
                                            requires: ::std::option::Option::None,
                                            shareable: false,
                                            inaccessible: false,
                                            tags: ::alloc::vec::Vec::new(),
                                            override_from: ::std::option::Option::None,
                                            visible: ::std::option::Option::None,
                                            compute_complexity: ::std::option::Option::None,
                                            directive_invocations: ::alloc::vec::Vec::new(),
                                            requires_scopes: ::alloc::vec::Vec::new(),
                                        },
                                    );
                                fields
                                    .insert(
                                        ::std::borrow::ToOwned::to_owned("title"),
                                        async_graphql::registry::MetaField {
                                            name: ::std::borrow::ToOwned::to_owned("title"),
                                            description: ::std::option::Option::None,
                                            args: ::std::default::Default::default(),
                                            ty: <String as async_graphql::OutputTypeMarker>::create_type_info(
                                                registry,
                                            ),
                                            deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                            cache_control: async_graphql::CacheControl {
                                                public: true,
                                                max_age: 0i32,
                                            },
                                            external: false,
                                            provides: ::std::option::Option::None,
                                            requires: ::std::option::Option::None,
                                            shareable: false,
                                            inaccessible: false,
                                            tags: ::alloc::vec::Vec::new(),
                                            override_from: ::std::option::Option::None,
                                            visible: ::std::option::Option::None,
                                            compute_complexity: ::std::option::Option::None,
                                            directive_invocations: ::alloc::vec::Vec::new(),
                                            requires_scopes: ::alloc::vec::Vec::new(),
                                        },
                                    );
                                fields
                            },
                            cache_control: async_graphql::CacheControl {
                                public: true,
                                max_age: 0i32,
                            },
                            extends: false,
                            shareable: false,
                            resolvable: true,
                            inaccessible: false,
                            interface_object: false,
                            tags: ::alloc::vec::Vec::new(),
                            keys: ::std::option::Option::None,
                            visible: ::std::option::Option::None,
                            is_subscription: false,
                            rust_typename: ::std::option::Option::Some(
                                ::std::any::type_name::<Self>(),
                            ),
                            directive_invocations: ::alloc::vec::Vec::new(),
                            requires_scopes: ::alloc::vec::Vec::new(),
                        },
                    )
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputType for MyObj {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::type_name()
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                &__self as &dyn async_graphql::resolver_utils::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::ObjectType for MyObj {}
        enum Node {
            MyObj(MyObj),
        }
        const _: () = {
            fn assert_impl() {
                let _: ::static_assertions_next::True = {
                    #[allow(unused_imports)]
                    use ::static_assertions_next::{
                        _bool::{True, False},
                        _core::{marker::PhantomData, ops::Deref},
                    };
                    trait DoesntImpl {
                        const DOES_IMPL: False = False;
                    }
                    impl<T: ?Sized> DoesntImpl for T {}
                    *{
                        struct Wrapper<T: ?Sized>(PhantomData<T>);
                        #[allow(dead_code)]
                        impl<T: ?Sized + async_graphql::ObjectType> Wrapper<T> {
                            const DOES_IMPL: True = True;
                        }
                        &<Wrapper<MyObj>>::DOES_IMPL
                    }
                };
            }
        };
        #[allow(clippy::all, clippy::pedantic)]
        impl ::std::convert::From<MyObj> for Node {
            fn from(obj: MyObj) -> Self {
                Node::MyObj(obj)
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::resolver_utils::ContainerType for Node {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = { ::std::result::Result::Ok(::std::option::Option::None) };
                    #[allow(unreachable_code)] __ret
                })
            }
            fn collect_all_fields<'__life>(
                &'__life self,
                ctx: &async_graphql::ContextSelectionSet<'__life>,
                fields: &mut async_graphql::resolver_utils::Fields<'__life>,
            ) -> async_graphql::ServerResult<()> {
                match self {
                    Node::MyObj(obj) => obj.collect_all_fields(ctx, fields),
                }
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputTypeMarker for Node {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("Node")
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                match self {
                    Node::MyObj(obj) => {
                        <MyObj as async_graphql::OutputTypeMarker>::type_name()
                    }
                }
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Union,
                        |registry| {
                            <MyObj as async_graphql::OutputTypeMarker>::create_type_info(
                                registry,
                            );
                            async_graphql::registry::MetaType::Union {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("Node"),
                                ),
                                description: ::std::option::Option::None,
                                possible_types: {
                                    let mut possible_types = async_graphql::indexmap::IndexSet::new();
                                    possible_types
                                        .insert(
                                            <MyObj as async_graphql::OutputTypeMarker>::type_name()
                                                .into_owned(),
                                        );
                                    possible_types
                                },
                                visible: ::std::option::Option::None,
                                inaccessible: false,
                                tags: ::alloc::vec::Vec::new(),
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::std::vec::Vec::new(),
                            }
                        },
                    )
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputType for Node {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::type_name()
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::introspection_type_name(&self)
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                __self as &dyn async_graphql::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::UnionType for Node {}
        struct Query;
        impl Query {
            async fn node(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<Node> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: Node = {
                                MyObj {
                                    id: 33,
                                    title: "haha".to_string(),
                                }
                                    .into()
                            };
                            value
                        }
                            .await,
                    )
                }
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __FieldIdent {
                node,
            }
            impl __FieldIdent {
                fn from_name(
                    __name: &async_graphql::Name,
                ) -> ::std::option::Option<__FieldIdent> {
                    match __name.as_str() {
                        "node" => ::std::option::Option::Some(__FieldIdent::node),
                        _ => ::std::option::Option::None,
                    }
                }
            }
            impl Query {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __node_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.node(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
            }
            #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
            #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
            impl async_graphql::resolver_utils::ContainerType for Query {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let __field = __FieldIdent::from_name(
                                &ctx.item.node.name.node,
                            );
                            match __field {
                                ::std::option::Option::Some(__FieldIdent::node) => {
                                    return __self.__node_resolver(&ctx).await;
                                }
                                None => {}
                            }
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                    params: &'life3 async_graphql::Value,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let params = match params {
                                async_graphql::Value::Object(params) => params,
                                _ => {
                                    return ::std::result::Result::Ok(
                                        ::std::option::Option::None,
                                    );
                                }
                            };
                            let typename = if let ::std::option::Option::Some(
                                async_graphql::Value::String(typename),
                            ) = params.get("__typename")
                            {
                                typename
                            } else {
                                return ::std::result::Result::Err(
                                    async_graphql::ServerError::new(
                                        r#""__typename" must be an existing string."#,
                                        ::std::option::Option::Some(ctx.item.pos),
                                    ),
                                );
                            };
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputTypeMarker for Query {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed("Query")
                }
                fn create_type_info(
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    let ty = registry
                        .create_output_type::<
                            Self,
                            _,
                        >(
                            async_graphql::registry::MetaTypeId::Object,
                            |registry| async_graphql::registry::MetaType::Object {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("Query"),
                                ),
                                description: ::std::option::Option::None,
                                fields: {
                                    let mut fields = async_graphql::indexmap::IndexMap::new();
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("node"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("node"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <Node as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                },
                                cache_control: async_graphql::CacheControl {
                                    public: true,
                                    max_age: 0i32,
                                },
                                extends: false,
                                shareable: false,
                                resolvable: true,
                                inaccessible: false,
                                interface_object: false,
                                tags: ::alloc::vec::Vec::new(),
                                keys: ::std::option::Option::None,
                                visible: ::std::option::Option::None,
                                is_subscription: false,
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::alloc::vec::Vec::new(),
                                requires_scopes: ::alloc::vec::Vec::new(),
                            },
                        );
                    ty
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputType for Query {
                fn type_name(
                    &self,
                ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    <Self as async_graphql::OutputTypeMarker>::type_name()
                }
                fn create_type_info(
                    &self,
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                    _field: &'life3 async_graphql::Positioned<
                        async_graphql::parser::types::Field,
                    >,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<async_graphql::Value>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<async_graphql::Value>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                            async_graphql::resolver_utils::resolve_container(
                                    ctx,
                                    &__self as &dyn async_graphql::resolver_utils::ContainerType,
                                )
                                .await
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            impl async_graphql::ObjectType for Query {}
        };
        let query = r#"{
            node {
                ... on MyObj {
                    id
                }
            }
        }"#;
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        match (
            &schema.execute(query).await.into_result().unwrap().data,
            &::async_graphql_value::ConstValue::Object({
                let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                let _ = object
                    .insert(
                        ::async_graphql_value::Name::new("node"),
                        ::async_graphql_value::ConstValue::Object({
                            let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                            let _ = object
                                .insert(
                                    ::async_graphql_value::Name::new("id"),
                                    ::async_graphql_value::to_value(&33).unwrap(),
                                );
                            object
                        }),
                    );
                object
            }),
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    };
    let mut body = body;
    #[allow(unused_mut)]
    let mut body = unsafe { ::tokio::macros::support::Pin::new_unchecked(&mut body) };
    let body: ::core::pin::Pin<&mut dyn ::core::future::Future<Output = ()>> = body;
    #[allow(
        clippy::expect_used,
        clippy::diverging_sub_expression,
        clippy::needless_return
    )]
    {
        return tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
extern crate test;
#[rustc_test_marker = "test_union_simple_object2"]
#[doc(hidden)]
pub const test_union_simple_object2: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("test_union_simple_object2"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/union.rs",
        start_line: 48usize,
        start_col: 14usize,
        end_line: 48usize,
        end_col: 39usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(test_union_simple_object2()),
    ),
};
pub fn test_union_simple_object2() {
    let body = async {
        struct MyObj {
            id: i32,
            title: String,
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl MyObj {
            #[inline]
            #[allow(missing_docs)]
            async fn id(
                &self,
                ctx: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<&i32> {
                ::std::result::Result::Ok(&self.id)
            }
            #[inline]
            #[allow(missing_docs)]
            async fn title(
                &self,
                ctx: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<&String> {
                ::std::result::Result::Ok(&self.title)
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::resolver_utils::ContainerType for MyObj {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = {
                        if ctx.item.node.name.node == "id" {
                            let f = async move {
                                __self
                                    .id(ctx)
                                    .await
                                    .map_err(|err| err.into_server_error(ctx.item.pos))
                            };
                            let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                            let ctx_obj = ctx
                                .with_selection_set(&ctx.item.node.selection_set);
                            return async_graphql::OutputType::resolve(
                                    &obj,
                                    &ctx_obj,
                                    ctx.item,
                                )
                                .await
                                .map(::std::option::Option::Some);
                        }
                        if ctx.item.node.name.node == "title" {
                            let f = async move {
                                __self
                                    .title(ctx)
                                    .await
                                    .map_err(|err| err.into_server_error(ctx.item.pos))
                            };
                            let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                            let ctx_obj = ctx
                                .with_selection_set(&ctx.item.node.selection_set);
                            return async_graphql::OutputType::resolve(
                                    &obj,
                                    &ctx_obj,
                                    ctx.item,
                                )
                                .await
                                .map(::std::option::Option::Some);
                        }
                        ::std::result::Result::Ok(::std::option::Option::None)
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputTypeMarker for MyObj {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("MyObj")
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Object,
                        |registry| async_graphql::registry::MetaType::Object {
                            name: ::std::borrow::Cow::into_owned(
                                ::std::borrow::Cow::Borrowed("MyObj"),
                            ),
                            description: ::std::option::Option::None,
                            fields: {
                                let mut fields = async_graphql::indexmap::IndexMap::new();
                                fields
                                    .insert(
                                        ::std::borrow::ToOwned::to_owned("id"),
                                        async_graphql::registry::MetaField {
                                            name: ::std::borrow::ToOwned::to_owned("id"),
                                            description: ::std::option::Option::None,
                                            args: ::std::default::Default::default(),
                                            ty: <i32 as async_graphql::OutputTypeMarker>::create_type_info(
                                                registry,
                                            ),
                                            deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                            cache_control: async_graphql::CacheControl {
                                                public: true,
                                                max_age: 0i32,
                                            },
                                            external: false,
                                            provides: ::std::option::Option::None,
                                            requires: ::std::option::Option::None,
                                            shareable: false,
                                            inaccessible: false,
                                            tags: ::alloc::vec::Vec::new(),
                                            override_from: ::std::option::Option::None,
                                            visible: ::std::option::Option::None,
                                            compute_complexity: ::std::option::Option::None,
                                            directive_invocations: ::alloc::vec::Vec::new(),
                                            requires_scopes: ::alloc::vec::Vec::new(),
                                        },
                                    );
                                fields
                                    .insert(
                                        ::std::borrow::ToOwned::to_owned("title"),
                                        async_graphql::registry::MetaField {
                                            name: ::std::borrow::ToOwned::to_owned("title"),
                                            description: ::std::option::Option::None,
                                            args: ::std::default::Default::default(),
                                            ty: <String as async_graphql::OutputTypeMarker>::create_type_info(
                                                registry,
                                            ),
                                            deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                            cache_control: async_graphql::CacheControl {
                                                public: true,
                                                max_age: 0i32,
                                            },
                                            external: false,
                                            provides: ::std::option::Option::None,
                                            requires: ::std::option::Option::None,
                                            shareable: false,
                                            inaccessible: false,
                                            tags: ::alloc::vec::Vec::new(),
                                            override_from: ::std::option::Option::None,
                                            visible: ::std::option::Option::None,
                                            compute_complexity: ::std::option::Option::None,
                                            directive_invocations: ::alloc::vec::Vec::new(),
                                            requires_scopes: ::alloc::vec::Vec::new(),
                                        },
                                    );
                                fields
                            },
                            cache_control: async_graphql::CacheControl {
                                public: true,
                                max_age: 0i32,
                            },
                            extends: false,
                            shareable: false,
                            resolvable: true,
                            inaccessible: false,
                            interface_object: false,
                            tags: ::alloc::vec::Vec::new(),
                            keys: ::std::option::Option::None,
                            visible: ::std::option::Option::None,
                            is_subscription: false,
                            rust_typename: ::std::option::Option::Some(
                                ::std::any::type_name::<Self>(),
                            ),
                            directive_invocations: ::alloc::vec::Vec::new(),
                            requires_scopes: ::alloc::vec::Vec::new(),
                        },
                    )
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputType for MyObj {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::type_name()
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                &__self as &dyn async_graphql::resolver_utils::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::ObjectType for MyObj {}
        enum Node {
            MyObj(MyObj),
        }
        const _: () = {
            fn assert_impl() {
                let _: ::static_assertions_next::True = {
                    #[allow(unused_imports)]
                    use ::static_assertions_next::{
                        _bool::{True, False},
                        _core::{marker::PhantomData, ops::Deref},
                    };
                    trait DoesntImpl {
                        const DOES_IMPL: False = False;
                    }
                    impl<T: ?Sized> DoesntImpl for T {}
                    *{
                        struct Wrapper<T: ?Sized>(PhantomData<T>);
                        #[allow(dead_code)]
                        impl<T: ?Sized + async_graphql::ObjectType> Wrapper<T> {
                            const DOES_IMPL: True = True;
                        }
                        &<Wrapper<MyObj>>::DOES_IMPL
                    }
                };
            }
        };
        #[allow(clippy::all, clippy::pedantic)]
        impl ::std::convert::From<MyObj> for Node {
            fn from(obj: MyObj) -> Self {
                Node::MyObj(obj)
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::resolver_utils::ContainerType for Node {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = { ::std::result::Result::Ok(::std::option::Option::None) };
                    #[allow(unreachable_code)] __ret
                })
            }
            fn collect_all_fields<'__life>(
                &'__life self,
                ctx: &async_graphql::ContextSelectionSet<'__life>,
                fields: &mut async_graphql::resolver_utils::Fields<'__life>,
            ) -> async_graphql::ServerResult<()> {
                match self {
                    Node::MyObj(obj) => obj.collect_all_fields(ctx, fields),
                }
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputTypeMarker for Node {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("Node")
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                match self {
                    Node::MyObj(obj) => {
                        <MyObj as async_graphql::OutputTypeMarker>::type_name()
                    }
                }
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Union,
                        |registry| {
                            <MyObj as async_graphql::OutputTypeMarker>::create_type_info(
                                registry,
                            );
                            async_graphql::registry::MetaType::Union {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("Node"),
                                ),
                                description: ::std::option::Option::None,
                                possible_types: {
                                    let mut possible_types = async_graphql::indexmap::IndexSet::new();
                                    possible_types
                                        .insert(
                                            <MyObj as async_graphql::OutputTypeMarker>::type_name()
                                                .into_owned(),
                                        );
                                    possible_types
                                },
                                visible: ::std::option::Option::None,
                                inaccessible: false,
                                tags: ::alloc::vec::Vec::new(),
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::std::vec::Vec::new(),
                            }
                        },
                    )
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputType for Node {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::type_name()
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::introspection_type_name(&self)
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                __self as &dyn async_graphql::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::UnionType for Node {}
        struct Query;
        impl Query {
            async fn node(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<Node> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: Node = {
                                MyObj {
                                    id: 33,
                                    title: "haha".to_string(),
                                }
                                    .into()
                            };
                            value
                        }
                            .await,
                    )
                }
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __FieldIdent {
                node,
            }
            impl __FieldIdent {
                fn from_name(
                    __name: &async_graphql::Name,
                ) -> ::std::option::Option<__FieldIdent> {
                    match __name.as_str() {
                        "node" => ::std::option::Option::Some(__FieldIdent::node),
                        _ => ::std::option::Option::None,
                    }
                }
            }
            impl Query {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __node_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.node(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
            }
            #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
            #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
            impl async_graphql::resolver_utils::ContainerType for Query {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let __field = __FieldIdent::from_name(
                                &ctx.item.node.name.node,
                            );
                            match __field {
                                ::std::option::Option::Some(__FieldIdent::node) => {
                                    return __self.__node_resolver(&ctx).await;
                                }
                                None => {}
                            }
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                    params: &'life3 async_graphql::Value,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let params = match params {
                                async_graphql::Value::Object(params) => params,
                                _ => {
                                    return ::std::result::Result::Ok(
                                        ::std::option::Option::None,
                                    );
                                }
                            };
                            let typename = if let ::std::option::Option::Some(
                                async_graphql::Value::String(typename),
                            ) = params.get("__typename")
                            {
                                typename
                            } else {
                                return ::std::result::Result::Err(
                                    async_graphql::ServerError::new(
                                        r#""__typename" must be an existing string."#,
                                        ::std::option::Option::Some(ctx.item.pos),
                                    ),
                                );
                            };
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputTypeMarker for Query {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed("Query")
                }
                fn create_type_info(
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    let ty = registry
                        .create_output_type::<
                            Self,
                            _,
                        >(
                            async_graphql::registry::MetaTypeId::Object,
                            |registry| async_graphql::registry::MetaType::Object {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("Query"),
                                ),
                                description: ::std::option::Option::None,
                                fields: {
                                    let mut fields = async_graphql::indexmap::IndexMap::new();
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("node"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("node"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <Node as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                },
                                cache_control: async_graphql::CacheControl {
                                    public: true,
                                    max_age: 0i32,
                                },
                                extends: false,
                                shareable: false,
                                resolvable: true,
                                inaccessible: false,
                                interface_object: false,
                                tags: ::alloc::vec::Vec::new(),
                                keys: ::std::option::Option::None,
                                visible: ::std::option::Option::None,
                                is_subscription: false,
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::alloc::vec::Vec::new(),
                                requires_scopes: ::alloc::vec::Vec::new(),
                            },
                        );
                    ty
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputType for Query {
                fn type_name(
                    &self,
                ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    <Self as async_graphql::OutputTypeMarker>::type_name()
                }
                fn create_type_info(
                    &self,
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                    _field: &'life3 async_graphql::Positioned<
                        async_graphql::parser::types::Field,
                    >,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<async_graphql::Value>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<async_graphql::Value>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                            async_graphql::resolver_utils::resolve_container(
                                    ctx,
                                    &__self as &dyn async_graphql::resolver_utils::ContainerType,
                                )
                                .await
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            impl async_graphql::ObjectType for Query {}
        };
        let query = r#"{
            node {
                ... on MyObj {
                    id
                }
            }
        }"#;
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        match (
            &schema.execute(query).await.into_result().unwrap().data,
            &::async_graphql_value::ConstValue::Object({
                let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                let _ = object
                    .insert(
                        ::async_graphql_value::Name::new("node"),
                        ::async_graphql_value::ConstValue::Object({
                            let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                            let _ = object
                                .insert(
                                    ::async_graphql_value::Name::new("id"),
                                    ::async_graphql_value::to_value(&33).unwrap(),
                                );
                            object
                        }),
                    );
                object
            }),
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    };
    let mut body = body;
    #[allow(unused_mut)]
    let mut body = unsafe { ::tokio::macros::support::Pin::new_unchecked(&mut body) };
    let body: ::core::pin::Pin<&mut dyn ::core::future::Future<Output = ()>> = body;
    #[allow(
        clippy::expect_used,
        clippy::diverging_sub_expression,
        clippy::needless_return
    )]
    {
        return tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
extern crate test;
#[rustc_test_marker = "test_multiple_unions"]
#[doc(hidden)]
pub const test_multiple_unions: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("test_multiple_unions"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/union.rs",
        start_line: 92usize,
        start_col: 14usize,
        end_line: 92usize,
        end_col: 34usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(test_multiple_unions()),
    ),
};
pub fn test_multiple_unions() {
    let body = async {
        struct MyObj;
        impl MyObj {
            async fn value_a(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<i32> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: i32 = { 1 };
                            value
                        }
                            .await,
                    )
                }
            }
            async fn value_b(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<i32> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: i32 = { 2 };
                            value
                        }
                            .await,
                    )
                }
            }
            async fn value_c(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<i32> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: i32 = { 3 };
                            value
                        }
                            .await,
                    )
                }
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __FieldIdent {
                value_a,
                value_b,
                value_c,
            }
            impl __FieldIdent {
                fn from_name(
                    __name: &async_graphql::Name,
                ) -> ::std::option::Option<__FieldIdent> {
                    match __name.as_str() {
                        "valueA" => ::std::option::Option::Some(__FieldIdent::value_a),
                        "valueB" => ::std::option::Option::Some(__FieldIdent::value_b),
                        "valueC" => ::std::option::Option::Some(__FieldIdent::value_c),
                        _ => ::std::option::Option::None,
                    }
                }
            }
            impl MyObj {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __value_a_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.value_a(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __value_b_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.value_b(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __value_c_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.value_c(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
            }
            #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
            #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
            impl async_graphql::resolver_utils::ContainerType for MyObj {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let __field = __FieldIdent::from_name(
                                &ctx.item.node.name.node,
                            );
                            match __field {
                                ::std::option::Option::Some(__FieldIdent::value_a) => {
                                    return __self.__value_a_resolver(&ctx).await;
                                }
                                ::std::option::Option::Some(__FieldIdent::value_b) => {
                                    return __self.__value_b_resolver(&ctx).await;
                                }
                                ::std::option::Option::Some(__FieldIdent::value_c) => {
                                    return __self.__value_c_resolver(&ctx).await;
                                }
                                None => {}
                            }
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                    params: &'life3 async_graphql::Value,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let params = match params {
                                async_graphql::Value::Object(params) => params,
                                _ => {
                                    return ::std::result::Result::Ok(
                                        ::std::option::Option::None,
                                    );
                                }
                            };
                            let typename = if let ::std::option::Option::Some(
                                async_graphql::Value::String(typename),
                            ) = params.get("__typename")
                            {
                                typename
                            } else {
                                return ::std::result::Result::Err(
                                    async_graphql::ServerError::new(
                                        r#""__typename" must be an existing string."#,
                                        ::std::option::Option::Some(ctx.item.pos),
                                    ),
                                );
                            };
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputTypeMarker for MyObj {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed("MyObj")
                }
                fn create_type_info(
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    let ty = registry
                        .create_output_type::<
                            Self,
                            _,
                        >(
                            async_graphql::registry::MetaTypeId::Object,
                            |registry| async_graphql::registry::MetaType::Object {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("MyObj"),
                                ),
                                description: ::std::option::Option::None,
                                fields: {
                                    let mut fields = async_graphql::indexmap::IndexMap::new();
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("valueA"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("valueA"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <i32 as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("valueB"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("valueB"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <i32 as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("valueC"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("valueC"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <i32 as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                },
                                cache_control: async_graphql::CacheControl {
                                    public: true,
                                    max_age: 0i32,
                                },
                                extends: false,
                                shareable: false,
                                resolvable: true,
                                inaccessible: false,
                                interface_object: false,
                                tags: ::alloc::vec::Vec::new(),
                                keys: ::std::option::Option::None,
                                visible: ::std::option::Option::None,
                                is_subscription: false,
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::alloc::vec::Vec::new(),
                                requires_scopes: ::alloc::vec::Vec::new(),
                            },
                        );
                    ty
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputType for MyObj {
                fn type_name(
                    &self,
                ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    <Self as async_graphql::OutputTypeMarker>::type_name()
                }
                fn create_type_info(
                    &self,
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                    _field: &'life3 async_graphql::Positioned<
                        async_graphql::parser::types::Field,
                    >,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<async_graphql::Value>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<async_graphql::Value>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                            async_graphql::resolver_utils::resolve_container(
                                    ctx,
                                    &__self as &dyn async_graphql::resolver_utils::ContainerType,
                                )
                                .await
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            impl async_graphql::ObjectType for MyObj {}
        };
        enum UnionA {
            MyObj(MyObj),
        }
        const _: () = {
            fn assert_impl() {
                let _: ::static_assertions_next::True = {
                    #[allow(unused_imports)]
                    use ::static_assertions_next::{
                        _bool::{True, False},
                        _core::{marker::PhantomData, ops::Deref},
                    };
                    trait DoesntImpl {
                        const DOES_IMPL: False = False;
                    }
                    impl<T: ?Sized> DoesntImpl for T {}
                    *{
                        struct Wrapper<T: ?Sized>(PhantomData<T>);
                        #[allow(dead_code)]
                        impl<T: ?Sized + async_graphql::ObjectType> Wrapper<T> {
                            const DOES_IMPL: True = True;
                        }
                        &<Wrapper<MyObj>>::DOES_IMPL
                    }
                };
            }
        };
        #[allow(clippy::all, clippy::pedantic)]
        impl ::std::convert::From<MyObj> for UnionA {
            fn from(obj: MyObj) -> Self {
                UnionA::MyObj(obj)
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::resolver_utils::ContainerType for UnionA {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = { ::std::result::Result::Ok(::std::option::Option::None) };
                    #[allow(unreachable_code)] __ret
                })
            }
            fn collect_all_fields<'__life>(
                &'__life self,
                ctx: &async_graphql::ContextSelectionSet<'__life>,
                fields: &mut async_graphql::resolver_utils::Fields<'__life>,
            ) -> async_graphql::ServerResult<()> {
                match self {
                    UnionA::MyObj(obj) => obj.collect_all_fields(ctx, fields),
                }
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputTypeMarker for UnionA {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("UnionA")
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                match self {
                    UnionA::MyObj(obj) => {
                        <MyObj as async_graphql::OutputTypeMarker>::type_name()
                    }
                }
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Union,
                        |registry| {
                            <MyObj as async_graphql::OutputTypeMarker>::create_type_info(
                                registry,
                            );
                            async_graphql::registry::MetaType::Union {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("UnionA"),
                                ),
                                description: ::std::option::Option::None,
                                possible_types: {
                                    let mut possible_types = async_graphql::indexmap::IndexSet::new();
                                    possible_types
                                        .insert(
                                            <MyObj as async_graphql::OutputTypeMarker>::type_name()
                                                .into_owned(),
                                        );
                                    possible_types
                                },
                                visible: ::std::option::Option::None,
                                inaccessible: false,
                                tags: ::alloc::vec::Vec::new(),
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::std::vec::Vec::new(),
                            }
                        },
                    )
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputType for UnionA {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::type_name()
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::introspection_type_name(&self)
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                __self as &dyn async_graphql::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::UnionType for UnionA {}
        enum UnionB {
            MyObj(MyObj),
        }
        const _: () = {
            fn assert_impl() {
                let _: ::static_assertions_next::True = {
                    #[allow(unused_imports)]
                    use ::static_assertions_next::{
                        _bool::{True, False},
                        _core::{marker::PhantomData, ops::Deref},
                    };
                    trait DoesntImpl {
                        const DOES_IMPL: False = False;
                    }
                    impl<T: ?Sized> DoesntImpl for T {}
                    *{
                        struct Wrapper<T: ?Sized>(PhantomData<T>);
                        #[allow(dead_code)]
                        impl<T: ?Sized + async_graphql::ObjectType> Wrapper<T> {
                            const DOES_IMPL: True = True;
                        }
                        &<Wrapper<MyObj>>::DOES_IMPL
                    }
                };
            }
        };
        #[allow(clippy::all, clippy::pedantic)]
        impl ::std::convert::From<MyObj> for UnionB {
            fn from(obj: MyObj) -> Self {
                UnionB::MyObj(obj)
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::resolver_utils::ContainerType for UnionB {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = { ::std::result::Result::Ok(::std::option::Option::None) };
                    #[allow(unreachable_code)] __ret
                })
            }
            fn collect_all_fields<'__life>(
                &'__life self,
                ctx: &async_graphql::ContextSelectionSet<'__life>,
                fields: &mut async_graphql::resolver_utils::Fields<'__life>,
            ) -> async_graphql::ServerResult<()> {
                match self {
                    UnionB::MyObj(obj) => obj.collect_all_fields(ctx, fields),
                }
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputTypeMarker for UnionB {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("UnionB")
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                match self {
                    UnionB::MyObj(obj) => {
                        <MyObj as async_graphql::OutputTypeMarker>::type_name()
                    }
                }
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Union,
                        |registry| {
                            <MyObj as async_graphql::OutputTypeMarker>::create_type_info(
                                registry,
                            );
                            async_graphql::registry::MetaType::Union {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("UnionB"),
                                ),
                                description: ::std::option::Option::None,
                                possible_types: {
                                    let mut possible_types = async_graphql::indexmap::IndexSet::new();
                                    possible_types
                                        .insert(
                                            <MyObj as async_graphql::OutputTypeMarker>::type_name()
                                                .into_owned(),
                                        );
                                    possible_types
                                },
                                visible: ::std::option::Option::None,
                                inaccessible: false,
                                tags: ::alloc::vec::Vec::new(),
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::std::vec::Vec::new(),
                            }
                        },
                    )
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputType for UnionB {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::type_name()
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::introspection_type_name(&self)
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                __self as &dyn async_graphql::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::UnionType for UnionB {}
        struct Query;
        impl Query {
            async fn union_a(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<UnionA> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: UnionA = { MyObj.into() };
                            value
                        }
                            .await,
                    )
                }
            }
            async fn union_b(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<UnionB> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: UnionB = { MyObj.into() };
                            value
                        }
                            .await,
                    )
                }
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __FieldIdent {
                union_a,
                union_b,
            }
            impl __FieldIdent {
                fn from_name(
                    __name: &async_graphql::Name,
                ) -> ::std::option::Option<__FieldIdent> {
                    match __name.as_str() {
                        "unionA" => ::std::option::Option::Some(__FieldIdent::union_a),
                        "unionB" => ::std::option::Option::Some(__FieldIdent::union_b),
                        _ => ::std::option::Option::None,
                    }
                }
            }
            impl Query {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __union_a_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.union_a(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __union_b_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.union_b(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
            }
            #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
            #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
            impl async_graphql::resolver_utils::ContainerType for Query {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let __field = __FieldIdent::from_name(
                                &ctx.item.node.name.node,
                            );
                            match __field {
                                ::std::option::Option::Some(__FieldIdent::union_a) => {
                                    return __self.__union_a_resolver(&ctx).await;
                                }
                                ::std::option::Option::Some(__FieldIdent::union_b) => {
                                    return __self.__union_b_resolver(&ctx).await;
                                }
                                None => {}
                            }
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                    params: &'life3 async_graphql::Value,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let params = match params {
                                async_graphql::Value::Object(params) => params,
                                _ => {
                                    return ::std::result::Result::Ok(
                                        ::std::option::Option::None,
                                    );
                                }
                            };
                            let typename = if let ::std::option::Option::Some(
                                async_graphql::Value::String(typename),
                            ) = params.get("__typename")
                            {
                                typename
                            } else {
                                return ::std::result::Result::Err(
                                    async_graphql::ServerError::new(
                                        r#""__typename" must be an existing string."#,
                                        ::std::option::Option::Some(ctx.item.pos),
                                    ),
                                );
                            };
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputTypeMarker for Query {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed("Query")
                }
                fn create_type_info(
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    let ty = registry
                        .create_output_type::<
                            Self,
                            _,
                        >(
                            async_graphql::registry::MetaTypeId::Object,
                            |registry| async_graphql::registry::MetaType::Object {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("Query"),
                                ),
                                description: ::std::option::Option::None,
                                fields: {
                                    let mut fields = async_graphql::indexmap::IndexMap::new();
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("unionA"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("unionA"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <UnionA as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("unionB"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("unionB"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <UnionB as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                },
                                cache_control: async_graphql::CacheControl {
                                    public: true,
                                    max_age: 0i32,
                                },
                                extends: false,
                                shareable: false,
                                resolvable: true,
                                inaccessible: false,
                                interface_object: false,
                                tags: ::alloc::vec::Vec::new(),
                                keys: ::std::option::Option::None,
                                visible: ::std::option::Option::None,
                                is_subscription: false,
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::alloc::vec::Vec::new(),
                                requires_scopes: ::alloc::vec::Vec::new(),
                            },
                        );
                    ty
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputType for Query {
                fn type_name(
                    &self,
                ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    <Self as async_graphql::OutputTypeMarker>::type_name()
                }
                fn create_type_info(
                    &self,
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                    _field: &'life3 async_graphql::Positioned<
                        async_graphql::parser::types::Field,
                    >,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<async_graphql::Value>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<async_graphql::Value>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                            async_graphql::resolver_utils::resolve_container(
                                    ctx,
                                    &__self as &dyn async_graphql::resolver_utils::ContainerType,
                                )
                                .await
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            impl async_graphql::ObjectType for Query {}
        };
        let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
            .register_output_type::<UnionA>()
            .finish();
        let query = r#"{
            unionA {
               ... on MyObj {
                valueA
                valueB
                valueC
              }
            }
            unionB {
                ... on MyObj {
                 valueA
                 valueB
                 valueC
               }
             }
        }"#;
        match (
            &schema.execute(query).await.into_result().unwrap().data,
            &::async_graphql_value::ConstValue::Object({
                let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                let _ = object
                    .insert(
                        ::async_graphql_value::Name::new("unionA"),
                        ::async_graphql_value::ConstValue::Object({
                            let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                            let _ = object
                                .insert(
                                    ::async_graphql_value::Name::new("valueA"),
                                    ::async_graphql_value::to_value(&1).unwrap(),
                                );
                            let _ = object
                                .insert(
                                    ::async_graphql_value::Name::new("valueB"),
                                    ::async_graphql_value::to_value(&2).unwrap(),
                                );
                            let _ = object
                                .insert(
                                    ::async_graphql_value::Name::new("valueC"),
                                    ::async_graphql_value::to_value(&3).unwrap(),
                                );
                            object
                        }),
                    );
                let _ = object
                    .insert(
                        ::async_graphql_value::Name::new("unionB"),
                        ::async_graphql_value::ConstValue::Object({
                            let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                            let _ = object
                                .insert(
                                    ::async_graphql_value::Name::new("valueA"),
                                    ::async_graphql_value::to_value(&1).unwrap(),
                                );
                            let _ = object
                                .insert(
                                    ::async_graphql_value::Name::new("valueB"),
                                    ::async_graphql_value::to_value(&2).unwrap(),
                                );
                            let _ = object
                                .insert(
                                    ::async_graphql_value::Name::new("valueC"),
                                    ::async_graphql_value::to_value(&3).unwrap(),
                                );
                            object
                        }),
                    );
                object
            }),
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    };
    let mut body = body;
    #[allow(unused_mut)]
    let mut body = unsafe { ::tokio::macros::support::Pin::new_unchecked(&mut body) };
    let body: ::core::pin::Pin<&mut dyn ::core::future::Future<Output = ()>> = body;
    #[allow(
        clippy::expect_used,
        clippy::diverging_sub_expression,
        clippy::needless_return
    )]
    {
        return tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
extern crate test;
#[rustc_test_marker = "test_multiple_objects_in_multiple_unions"]
#[doc(hidden)]
pub const test_multiple_objects_in_multiple_unions: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("test_multiple_objects_in_multiple_unions"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/union.rs",
        start_line: 169usize,
        start_col: 14usize,
        end_line: 169usize,
        end_col: 54usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(test_multiple_objects_in_multiple_unions()),
    ),
};
pub fn test_multiple_objects_in_multiple_unions() {
    let body = async {
        struct MyObjOne;
        impl MyObjOne {
            async fn value_a(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<i32> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: i32 = { 1 };
                            value
                        }
                            .await,
                    )
                }
            }
            async fn value_b(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<i32> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: i32 = { 2 };
                            value
                        }
                            .await,
                    )
                }
            }
            async fn value_c(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<i32> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: i32 = { 3 };
                            value
                        }
                            .await,
                    )
                }
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __FieldIdent {
                value_a,
                value_b,
                value_c,
            }
            impl __FieldIdent {
                fn from_name(
                    __name: &async_graphql::Name,
                ) -> ::std::option::Option<__FieldIdent> {
                    match __name.as_str() {
                        "valueA" => ::std::option::Option::Some(__FieldIdent::value_a),
                        "valueB" => ::std::option::Option::Some(__FieldIdent::value_b),
                        "valueC" => ::std::option::Option::Some(__FieldIdent::value_c),
                        _ => ::std::option::Option::None,
                    }
                }
            }
            impl MyObjOne {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __value_a_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.value_a(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __value_b_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.value_b(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __value_c_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.value_c(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
            }
            #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
            #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
            impl async_graphql::resolver_utils::ContainerType for MyObjOne {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let __field = __FieldIdent::from_name(
                                &ctx.item.node.name.node,
                            );
                            match __field {
                                ::std::option::Option::Some(__FieldIdent::value_a) => {
                                    return __self.__value_a_resolver(&ctx).await;
                                }
                                ::std::option::Option::Some(__FieldIdent::value_b) => {
                                    return __self.__value_b_resolver(&ctx).await;
                                }
                                ::std::option::Option::Some(__FieldIdent::value_c) => {
                                    return __self.__value_c_resolver(&ctx).await;
                                }
                                None => {}
                            }
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                    params: &'life3 async_graphql::Value,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let params = match params {
                                async_graphql::Value::Object(params) => params,
                                _ => {
                                    return ::std::result::Result::Ok(
                                        ::std::option::Option::None,
                                    );
                                }
                            };
                            let typename = if let ::std::option::Option::Some(
                                async_graphql::Value::String(typename),
                            ) = params.get("__typename")
                            {
                                typename
                            } else {
                                return ::std::result::Result::Err(
                                    async_graphql::ServerError::new(
                                        r#""__typename" must be an existing string."#,
                                        ::std::option::Option::Some(ctx.item.pos),
                                    ),
                                );
                            };
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputTypeMarker for MyObjOne {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed("MyObjOne")
                }
                fn create_type_info(
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    let ty = registry
                        .create_output_type::<
                            Self,
                            _,
                        >(
                            async_graphql::registry::MetaTypeId::Object,
                            |registry| async_graphql::registry::MetaType::Object {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("MyObjOne"),
                                ),
                                description: ::std::option::Option::None,
                                fields: {
                                    let mut fields = async_graphql::indexmap::IndexMap::new();
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("valueA"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("valueA"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <i32 as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("valueB"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("valueB"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <i32 as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("valueC"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("valueC"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <i32 as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                },
                                cache_control: async_graphql::CacheControl {
                                    public: true,
                                    max_age: 0i32,
                                },
                                extends: false,
                                shareable: false,
                                resolvable: true,
                                inaccessible: false,
                                interface_object: false,
                                tags: ::alloc::vec::Vec::new(),
                                keys: ::std::option::Option::None,
                                visible: ::std::option::Option::None,
                                is_subscription: false,
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::alloc::vec::Vec::new(),
                                requires_scopes: ::alloc::vec::Vec::new(),
                            },
                        );
                    ty
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputType for MyObjOne {
                fn type_name(
                    &self,
                ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    <Self as async_graphql::OutputTypeMarker>::type_name()
                }
                fn create_type_info(
                    &self,
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                    _field: &'life3 async_graphql::Positioned<
                        async_graphql::parser::types::Field,
                    >,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<async_graphql::Value>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<async_graphql::Value>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                            async_graphql::resolver_utils::resolve_container(
                                    ctx,
                                    &__self as &dyn async_graphql::resolver_utils::ContainerType,
                                )
                                .await
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            impl async_graphql::ObjectType for MyObjOne {}
        };
        struct MyObjTwo;
        impl MyObjTwo {
            async fn value_a(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<i32> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: i32 = { 1 };
                            value
                        }
                            .await,
                    )
                }
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __FieldIdent {
                value_a,
            }
            impl __FieldIdent {
                fn from_name(
                    __name: &async_graphql::Name,
                ) -> ::std::option::Option<__FieldIdent> {
                    match __name.as_str() {
                        "valueA" => ::std::option::Option::Some(__FieldIdent::value_a),
                        _ => ::std::option::Option::None,
                    }
                }
            }
            impl MyObjTwo {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __value_a_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.value_a(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
            }
            #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
            #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
            impl async_graphql::resolver_utils::ContainerType for MyObjTwo {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let __field = __FieldIdent::from_name(
                                &ctx.item.node.name.node,
                            );
                            match __field {
                                ::std::option::Option::Some(__FieldIdent::value_a) => {
                                    return __self.__value_a_resolver(&ctx).await;
                                }
                                None => {}
                            }
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                    params: &'life3 async_graphql::Value,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let params = match params {
                                async_graphql::Value::Object(params) => params,
                                _ => {
                                    return ::std::result::Result::Ok(
                                        ::std::option::Option::None,
                                    );
                                }
                            };
                            let typename = if let ::std::option::Option::Some(
                                async_graphql::Value::String(typename),
                            ) = params.get("__typename")
                            {
                                typename
                            } else {
                                return ::std::result::Result::Err(
                                    async_graphql::ServerError::new(
                                        r#""__typename" must be an existing string."#,
                                        ::std::option::Option::Some(ctx.item.pos),
                                    ),
                                );
                            };
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputTypeMarker for MyObjTwo {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed("MyObjTwo")
                }
                fn create_type_info(
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    let ty = registry
                        .create_output_type::<
                            Self,
                            _,
                        >(
                            async_graphql::registry::MetaTypeId::Object,
                            |registry| async_graphql::registry::MetaType::Object {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("MyObjTwo"),
                                ),
                                description: ::std::option::Option::None,
                                fields: {
                                    let mut fields = async_graphql::indexmap::IndexMap::new();
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("valueA"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("valueA"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <i32 as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                },
                                cache_control: async_graphql::CacheControl {
                                    public: true,
                                    max_age: 0i32,
                                },
                                extends: false,
                                shareable: false,
                                resolvable: true,
                                inaccessible: false,
                                interface_object: false,
                                tags: ::alloc::vec::Vec::new(),
                                keys: ::std::option::Option::None,
                                visible: ::std::option::Option::None,
                                is_subscription: false,
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::alloc::vec::Vec::new(),
                                requires_scopes: ::alloc::vec::Vec::new(),
                            },
                        );
                    ty
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputType for MyObjTwo {
                fn type_name(
                    &self,
                ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    <Self as async_graphql::OutputTypeMarker>::type_name()
                }
                fn create_type_info(
                    &self,
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                    _field: &'life3 async_graphql::Positioned<
                        async_graphql::parser::types::Field,
                    >,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<async_graphql::Value>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<async_graphql::Value>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                            async_graphql::resolver_utils::resolve_container(
                                    ctx,
                                    &__self as &dyn async_graphql::resolver_utils::ContainerType,
                                )
                                .await
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            impl async_graphql::ObjectType for MyObjTwo {}
        };
        enum UnionA {
            MyObjOne(MyObjOne),
            MyObjTwo(MyObjTwo),
        }
        const _: () = {
            fn assert_impl() {
                let _: ::static_assertions_next::True = {
                    #[allow(unused_imports)]
                    use ::static_assertions_next::{
                        _bool::{True, False},
                        _core::{marker::PhantomData, ops::Deref},
                    };
                    trait DoesntImpl {
                        const DOES_IMPL: False = False;
                    }
                    impl<T: ?Sized> DoesntImpl for T {}
                    *{
                        struct Wrapper<T: ?Sized>(PhantomData<T>);
                        #[allow(dead_code)]
                        impl<T: ?Sized + async_graphql::ObjectType> Wrapper<T> {
                            const DOES_IMPL: True = True;
                        }
                        &<Wrapper<MyObjOne>>::DOES_IMPL
                    }
                };
            }
        };
        #[allow(clippy::all, clippy::pedantic)]
        impl ::std::convert::From<MyObjOne> for UnionA {
            fn from(obj: MyObjOne) -> Self {
                UnionA::MyObjOne(obj)
            }
        }
        const _: () = {
            fn assert_impl() {
                let _: ::static_assertions_next::True = {
                    #[allow(unused_imports)]
                    use ::static_assertions_next::{
                        _bool::{True, False},
                        _core::{marker::PhantomData, ops::Deref},
                    };
                    trait DoesntImpl {
                        const DOES_IMPL: False = False;
                    }
                    impl<T: ?Sized> DoesntImpl for T {}
                    *{
                        struct Wrapper<T: ?Sized>(PhantomData<T>);
                        #[allow(dead_code)]
                        impl<T: ?Sized + async_graphql::ObjectType> Wrapper<T> {
                            const DOES_IMPL: True = True;
                        }
                        &<Wrapper<MyObjTwo>>::DOES_IMPL
                    }
                };
            }
        };
        #[allow(clippy::all, clippy::pedantic)]
        impl ::std::convert::From<MyObjTwo> for UnionA {
            fn from(obj: MyObjTwo) -> Self {
                UnionA::MyObjTwo(obj)
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::resolver_utils::ContainerType for UnionA {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = { ::std::result::Result::Ok(::std::option::Option::None) };
                    #[allow(unreachable_code)] __ret
                })
            }
            fn collect_all_fields<'__life>(
                &'__life self,
                ctx: &async_graphql::ContextSelectionSet<'__life>,
                fields: &mut async_graphql::resolver_utils::Fields<'__life>,
            ) -> async_graphql::ServerResult<()> {
                match self {
                    UnionA::MyObjOne(obj) => obj.collect_all_fields(ctx, fields),
                    UnionA::MyObjTwo(obj) => obj.collect_all_fields(ctx, fields),
                }
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputTypeMarker for UnionA {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("UnionA")
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                match self {
                    UnionA::MyObjOne(obj) => {
                        <MyObjOne as async_graphql::OutputTypeMarker>::type_name()
                    }
                    UnionA::MyObjTwo(obj) => {
                        <MyObjTwo as async_graphql::OutputTypeMarker>::type_name()
                    }
                }
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Union,
                        |registry| {
                            <MyObjOne as async_graphql::OutputTypeMarker>::create_type_info(
                                registry,
                            );
                            <MyObjTwo as async_graphql::OutputTypeMarker>::create_type_info(
                                registry,
                            );
                            async_graphql::registry::MetaType::Union {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("UnionA"),
                                ),
                                description: ::std::option::Option::None,
                                possible_types: {
                                    let mut possible_types = async_graphql::indexmap::IndexSet::new();
                                    possible_types
                                        .insert(
                                            <MyObjOne as async_graphql::OutputTypeMarker>::type_name()
                                                .into_owned(),
                                        );
                                    possible_types
                                        .insert(
                                            <MyObjTwo as async_graphql::OutputTypeMarker>::type_name()
                                                .into_owned(),
                                        );
                                    possible_types
                                },
                                visible: ::std::option::Option::None,
                                inaccessible: false,
                                tags: ::alloc::vec::Vec::new(),
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::std::vec::Vec::new(),
                            }
                        },
                    )
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputType for UnionA {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::type_name()
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::introspection_type_name(&self)
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                __self as &dyn async_graphql::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::UnionType for UnionA {}
        enum UnionB {
            MyObjOne(MyObjOne),
        }
        const _: () = {
            fn assert_impl() {
                let _: ::static_assertions_next::True = {
                    #[allow(unused_imports)]
                    use ::static_assertions_next::{
                        _bool::{True, False},
                        _core::{marker::PhantomData, ops::Deref},
                    };
                    trait DoesntImpl {
                        const DOES_IMPL: False = False;
                    }
                    impl<T: ?Sized> DoesntImpl for T {}
                    *{
                        struct Wrapper<T: ?Sized>(PhantomData<T>);
                        #[allow(dead_code)]
                        impl<T: ?Sized + async_graphql::ObjectType> Wrapper<T> {
                            const DOES_IMPL: True = True;
                        }
                        &<Wrapper<MyObjOne>>::DOES_IMPL
                    }
                };
            }
        };
        #[allow(clippy::all, clippy::pedantic)]
        impl ::std::convert::From<MyObjOne> for UnionB {
            fn from(obj: MyObjOne) -> Self {
                UnionB::MyObjOne(obj)
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::resolver_utils::ContainerType for UnionB {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = { ::std::result::Result::Ok(::std::option::Option::None) };
                    #[allow(unreachable_code)] __ret
                })
            }
            fn collect_all_fields<'__life>(
                &'__life self,
                ctx: &async_graphql::ContextSelectionSet<'__life>,
                fields: &mut async_graphql::resolver_utils::Fields<'__life>,
            ) -> async_graphql::ServerResult<()> {
                match self {
                    UnionB::MyObjOne(obj) => obj.collect_all_fields(ctx, fields),
                }
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputTypeMarker for UnionB {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("UnionB")
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                match self {
                    UnionB::MyObjOne(obj) => {
                        <MyObjOne as async_graphql::OutputTypeMarker>::type_name()
                    }
                }
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Union,
                        |registry| {
                            <MyObjOne as async_graphql::OutputTypeMarker>::create_type_info(
                                registry,
                            );
                            async_graphql::registry::MetaType::Union {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("UnionB"),
                                ),
                                description: ::std::option::Option::None,
                                possible_types: {
                                    let mut possible_types = async_graphql::indexmap::IndexSet::new();
                                    possible_types
                                        .insert(
                                            <MyObjOne as async_graphql::OutputTypeMarker>::type_name()
                                                .into_owned(),
                                        );
                                    possible_types
                                },
                                visible: ::std::option::Option::None,
                                inaccessible: false,
                                tags: ::alloc::vec::Vec::new(),
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::std::vec::Vec::new(),
                            }
                        },
                    )
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputType for UnionB {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::type_name()
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::introspection_type_name(&self)
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                __self as &dyn async_graphql::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::UnionType for UnionB {}
        struct Query;
        impl Query {
            async fn my_obj(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<Vec<UnionA>> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: Vec<UnionA> = {
                                <[_]>::into_vec(
                                    ::alloc::boxed::box_new([MyObjOne.into(), MyObjTwo.into()]),
                                )
                            };
                            value
                        }
                            .await,
                    )
                }
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __FieldIdent {
                my_obj,
            }
            impl __FieldIdent {
                fn from_name(
                    __name: &async_graphql::Name,
                ) -> ::std::option::Option<__FieldIdent> {
                    match __name.as_str() {
                        "myObj" => ::std::option::Option::Some(__FieldIdent::my_obj),
                        _ => ::std::option::Option::None,
                    }
                }
            }
            impl Query {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __my_obj_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.my_obj(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
            }
            #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
            #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
            impl async_graphql::resolver_utils::ContainerType for Query {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let __field = __FieldIdent::from_name(
                                &ctx.item.node.name.node,
                            );
                            match __field {
                                ::std::option::Option::Some(__FieldIdent::my_obj) => {
                                    return __self.__my_obj_resolver(&ctx).await;
                                }
                                None => {}
                            }
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                    params: &'life3 async_graphql::Value,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let params = match params {
                                async_graphql::Value::Object(params) => params,
                                _ => {
                                    return ::std::result::Result::Ok(
                                        ::std::option::Option::None,
                                    );
                                }
                            };
                            let typename = if let ::std::option::Option::Some(
                                async_graphql::Value::String(typename),
                            ) = params.get("__typename")
                            {
                                typename
                            } else {
                                return ::std::result::Result::Err(
                                    async_graphql::ServerError::new(
                                        r#""__typename" must be an existing string."#,
                                        ::std::option::Option::Some(ctx.item.pos),
                                    ),
                                );
                            };
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputTypeMarker for Query {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed("Query")
                }
                fn create_type_info(
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    let ty = registry
                        .create_output_type::<
                            Self,
                            _,
                        >(
                            async_graphql::registry::MetaTypeId::Object,
                            |registry| async_graphql::registry::MetaType::Object {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("Query"),
                                ),
                                description: ::std::option::Option::None,
                                fields: {
                                    let mut fields = async_graphql::indexmap::IndexMap::new();
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("myObj"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("myObj"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <Vec<
                                                    UnionA,
                                                > as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                },
                                cache_control: async_graphql::CacheControl {
                                    public: true,
                                    max_age: 0i32,
                                },
                                extends: false,
                                shareable: false,
                                resolvable: true,
                                inaccessible: false,
                                interface_object: false,
                                tags: ::alloc::vec::Vec::new(),
                                keys: ::std::option::Option::None,
                                visible: ::std::option::Option::None,
                                is_subscription: false,
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::alloc::vec::Vec::new(),
                                requires_scopes: ::alloc::vec::Vec::new(),
                            },
                        );
                    ty
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputType for Query {
                fn type_name(
                    &self,
                ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    <Self as async_graphql::OutputTypeMarker>::type_name()
                }
                fn create_type_info(
                    &self,
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                    _field: &'life3 async_graphql::Positioned<
                        async_graphql::parser::types::Field,
                    >,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<async_graphql::Value>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<async_graphql::Value>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                            async_graphql::resolver_utils::resolve_container(
                                    ctx,
                                    &__self as &dyn async_graphql::resolver_utils::ContainerType,
                                )
                                .await
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            impl async_graphql::ObjectType for Query {}
        };
        let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
            .register_output_type::<UnionB>()
            .finish();
        let query = r#"{
            myObj {
                ... on MyObjTwo {
                    valueA
                }
                ... on MyObjOne {
                    valueA
                    valueB
                    valueC
                }
            }
         }"#;
        match (
            &schema.execute(query).await.into_result().unwrap().data,
            &::async_graphql_value::ConstValue::Object({
                let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                let _ = object
                    .insert(
                        ::async_graphql_value::Name::new("myObj"),
                        ::async_graphql_value::ConstValue::List(
                            <[_]>::into_vec(
                                ::alloc::boxed::box_new([
                                    ::async_graphql_value::ConstValue::Object({
                                        let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                                        let _ = object
                                            .insert(
                                                ::async_graphql_value::Name::new("valueA"),
                                                ::async_graphql_value::to_value(&1).unwrap(),
                                            );
                                        let _ = object
                                            .insert(
                                                ::async_graphql_value::Name::new("valueB"),
                                                ::async_graphql_value::to_value(&2).unwrap(),
                                            );
                                        let _ = object
                                            .insert(
                                                ::async_graphql_value::Name::new("valueC"),
                                                ::async_graphql_value::to_value(&3).unwrap(),
                                            );
                                        object
                                    }),
                                    ::async_graphql_value::ConstValue::Object({
                                        let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                                        let _ = object
                                            .insert(
                                                ::async_graphql_value::Name::new("valueA"),
                                                ::async_graphql_value::to_value(&1).unwrap(),
                                            );
                                        object
                                    }),
                                ]),
                            ),
                        ),
                    );
                object
            }),
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    };
    let mut body = body;
    #[allow(unused_mut)]
    let mut body = unsafe { ::tokio::macros::support::Pin::new_unchecked(&mut body) };
    let body: ::core::pin::Pin<&mut dyn ::core::future::Future<Output = ()>> = body;
    #[allow(
        clippy::expect_used,
        clippy::diverging_sub_expression,
        clippy::needless_return
    )]
    {
        return tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
extern crate test;
#[rustc_test_marker = "test_union_field_result"]
#[doc(hidden)]
pub const test_union_field_result: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("test_union_field_result"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/union.rs",
        start_line: 246usize,
        start_col: 14usize,
        end_line: 246usize,
        end_col: 37usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(test_union_field_result()),
    ),
};
pub fn test_union_field_result() {
    let body = async {
        struct MyObj;
        impl MyObj {
            async fn value(&self, _: &async_graphql::Context<'_>) -> Result<i32> {
                Ok(10)
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __FieldIdent {
                value,
            }
            impl __FieldIdent {
                fn from_name(
                    __name: &async_graphql::Name,
                ) -> ::std::option::Option<__FieldIdent> {
                    match __name.as_str() {
                        "value" => ::std::option::Option::Some(__FieldIdent::value),
                        _ => ::std::option::Option::None,
                    }
                }
            }
            impl MyObj {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __value_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.value(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
            }
            #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
            #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
            impl async_graphql::resolver_utils::ContainerType for MyObj {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let __field = __FieldIdent::from_name(
                                &ctx.item.node.name.node,
                            );
                            match __field {
                                ::std::option::Option::Some(__FieldIdent::value) => {
                                    return __self.__value_resolver(&ctx).await;
                                }
                                None => {}
                            }
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                    params: &'life3 async_graphql::Value,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let params = match params {
                                async_graphql::Value::Object(params) => params,
                                _ => {
                                    return ::std::result::Result::Ok(
                                        ::std::option::Option::None,
                                    );
                                }
                            };
                            let typename = if let ::std::option::Option::Some(
                                async_graphql::Value::String(typename),
                            ) = params.get("__typename")
                            {
                                typename
                            } else {
                                return ::std::result::Result::Err(
                                    async_graphql::ServerError::new(
                                        r#""__typename" must be an existing string."#,
                                        ::std::option::Option::Some(ctx.item.pos),
                                    ),
                                );
                            };
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputTypeMarker for MyObj {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed("MyObj")
                }
                fn create_type_info(
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    let ty = registry
                        .create_output_type::<
                            Self,
                            _,
                        >(
                            async_graphql::registry::MetaTypeId::Object,
                            |registry| async_graphql::registry::MetaType::Object {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("MyObj"),
                                ),
                                description: ::std::option::Option::None,
                                fields: {
                                    let mut fields = async_graphql::indexmap::IndexMap::new();
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("value"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("value"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <i32 as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                },
                                cache_control: async_graphql::CacheControl {
                                    public: true,
                                    max_age: 0i32,
                                },
                                extends: false,
                                shareable: false,
                                resolvable: true,
                                inaccessible: false,
                                interface_object: false,
                                tags: ::alloc::vec::Vec::new(),
                                keys: ::std::option::Option::None,
                                visible: ::std::option::Option::None,
                                is_subscription: false,
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::alloc::vec::Vec::new(),
                                requires_scopes: ::alloc::vec::Vec::new(),
                            },
                        );
                    ty
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputType for MyObj {
                fn type_name(
                    &self,
                ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    <Self as async_graphql::OutputTypeMarker>::type_name()
                }
                fn create_type_info(
                    &self,
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                    _field: &'life3 async_graphql::Positioned<
                        async_graphql::parser::types::Field,
                    >,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<async_graphql::Value>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<async_graphql::Value>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                            async_graphql::resolver_utils::resolve_container(
                                    ctx,
                                    &__self as &dyn async_graphql::resolver_utils::ContainerType,
                                )
                                .await
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            impl async_graphql::ObjectType for MyObj {}
        };
        enum Node {
            MyObj(MyObj),
        }
        const _: () = {
            fn assert_impl() {
                let _: ::static_assertions_next::True = {
                    #[allow(unused_imports)]
                    use ::static_assertions_next::{
                        _bool::{True, False},
                        _core::{marker::PhantomData, ops::Deref},
                    };
                    trait DoesntImpl {
                        const DOES_IMPL: False = False;
                    }
                    impl<T: ?Sized> DoesntImpl for T {}
                    *{
                        struct Wrapper<T: ?Sized>(PhantomData<T>);
                        #[allow(dead_code)]
                        impl<T: ?Sized + async_graphql::ObjectType> Wrapper<T> {
                            const DOES_IMPL: True = True;
                        }
                        &<Wrapper<MyObj>>::DOES_IMPL
                    }
                };
            }
        };
        #[allow(clippy::all, clippy::pedantic)]
        impl ::std::convert::From<MyObj> for Node {
            fn from(obj: MyObj) -> Self {
                Node::MyObj(obj)
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::resolver_utils::ContainerType for Node {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = { ::std::result::Result::Ok(::std::option::Option::None) };
                    #[allow(unreachable_code)] __ret
                })
            }
            fn collect_all_fields<'__life>(
                &'__life self,
                ctx: &async_graphql::ContextSelectionSet<'__life>,
                fields: &mut async_graphql::resolver_utils::Fields<'__life>,
            ) -> async_graphql::ServerResult<()> {
                match self {
                    Node::MyObj(obj) => obj.collect_all_fields(ctx, fields),
                }
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputTypeMarker for Node {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("Node")
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                match self {
                    Node::MyObj(obj) => {
                        <MyObj as async_graphql::OutputTypeMarker>::type_name()
                    }
                }
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Union,
                        |registry| {
                            <MyObj as async_graphql::OutputTypeMarker>::create_type_info(
                                registry,
                            );
                            async_graphql::registry::MetaType::Union {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("Node"),
                                ),
                                description: ::std::option::Option::None,
                                possible_types: {
                                    let mut possible_types = async_graphql::indexmap::IndexSet::new();
                                    possible_types
                                        .insert(
                                            <MyObj as async_graphql::OutputTypeMarker>::type_name()
                                                .into_owned(),
                                        );
                                    possible_types
                                },
                                visible: ::std::option::Option::None,
                                inaccessible: false,
                                tags: ::alloc::vec::Vec::new(),
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::std::vec::Vec::new(),
                            }
                        },
                    )
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputType for Node {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::type_name()
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::introspection_type_name(&self)
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                __self as &dyn async_graphql::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::UnionType for Node {}
        struct Query;
        impl Query {
            async fn node(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<Node> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: Node = { MyObj.into() };
                            value
                        }
                            .await,
                    )
                }
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __FieldIdent {
                node,
            }
            impl __FieldIdent {
                fn from_name(
                    __name: &async_graphql::Name,
                ) -> ::std::option::Option<__FieldIdent> {
                    match __name.as_str() {
                        "node" => ::std::option::Option::Some(__FieldIdent::node),
                        _ => ::std::option::Option::None,
                    }
                }
            }
            impl Query {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __node_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.node(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
            }
            #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
            #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
            impl async_graphql::resolver_utils::ContainerType for Query {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let __field = __FieldIdent::from_name(
                                &ctx.item.node.name.node,
                            );
                            match __field {
                                ::std::option::Option::Some(__FieldIdent::node) => {
                                    return __self.__node_resolver(&ctx).await;
                                }
                                None => {}
                            }
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                    params: &'life3 async_graphql::Value,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let params = match params {
                                async_graphql::Value::Object(params) => params,
                                _ => {
                                    return ::std::result::Result::Ok(
                                        ::std::option::Option::None,
                                    );
                                }
                            };
                            let typename = if let ::std::option::Option::Some(
                                async_graphql::Value::String(typename),
                            ) = params.get("__typename")
                            {
                                typename
                            } else {
                                return ::std::result::Result::Err(
                                    async_graphql::ServerError::new(
                                        r#""__typename" must be an existing string."#,
                                        ::std::option::Option::Some(ctx.item.pos),
                                    ),
                                );
                            };
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputTypeMarker for Query {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed("Query")
                }
                fn create_type_info(
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    let ty = registry
                        .create_output_type::<
                            Self,
                            _,
                        >(
                            async_graphql::registry::MetaTypeId::Object,
                            |registry| async_graphql::registry::MetaType::Object {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("Query"),
                                ),
                                description: ::std::option::Option::None,
                                fields: {
                                    let mut fields = async_graphql::indexmap::IndexMap::new();
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("node"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("node"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <Node as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                },
                                cache_control: async_graphql::CacheControl {
                                    public: true,
                                    max_age: 0i32,
                                },
                                extends: false,
                                shareable: false,
                                resolvable: true,
                                inaccessible: false,
                                interface_object: false,
                                tags: ::alloc::vec::Vec::new(),
                                keys: ::std::option::Option::None,
                                visible: ::std::option::Option::None,
                                is_subscription: false,
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::alloc::vec::Vec::new(),
                                requires_scopes: ::alloc::vec::Vec::new(),
                            },
                        );
                    ty
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputType for Query {
                fn type_name(
                    &self,
                ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    <Self as async_graphql::OutputTypeMarker>::type_name()
                }
                fn create_type_info(
                    &self,
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                    _field: &'life3 async_graphql::Positioned<
                        async_graphql::parser::types::Field,
                    >,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<async_graphql::Value>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<async_graphql::Value>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                            async_graphql::resolver_utils::resolve_container(
                                    ctx,
                                    &__self as &dyn async_graphql::resolver_utils::ContainerType,
                                )
                                .await
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            impl async_graphql::ObjectType for Query {}
        };
        let query = r#"{
            node {
                ... on MyObj {
                    value
                }
            }
        }"#;
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        match (
            &schema.execute(query).await.into_result().unwrap().data,
            &::async_graphql_value::ConstValue::Object({
                let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                let _ = object
                    .insert(
                        ::async_graphql_value::Name::new("node"),
                        ::async_graphql_value::ConstValue::Object({
                            let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                            let _ = object
                                .insert(
                                    ::async_graphql_value::Name::new("value"),
                                    ::async_graphql_value::to_value(&10).unwrap(),
                                );
                            object
                        }),
                    );
                object
            }),
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    };
    let mut body = body;
    #[allow(unused_mut)]
    let mut body = unsafe { ::tokio::macros::support::Pin::new_unchecked(&mut body) };
    let body: ::core::pin::Pin<&mut dyn ::core::future::Future<Output = ()>> = body;
    #[allow(
        clippy::expect_used,
        clippy::diverging_sub_expression,
        clippy::needless_return
    )]
    {
        return tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
extern crate test;
#[rustc_test_marker = "test_union_flatten"]
#[doc(hidden)]
pub const test_union_flatten: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("test_union_flatten"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/union.rs",
        start_line: 289usize,
        start_col: 14usize,
        end_line: 289usize,
        end_col: 32usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(test_union_flatten()),
    ),
};
pub fn test_union_flatten() {
    let body = async {
        struct MyObj1 {
            value1: i32,
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl MyObj1 {
            #[inline]
            #[allow(missing_docs)]
            async fn value1(
                &self,
                ctx: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<&i32> {
                ::std::result::Result::Ok(&self.value1)
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::resolver_utils::ContainerType for MyObj1 {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = {
                        if ctx.item.node.name.node == "value1" {
                            let f = async move {
                                __self
                                    .value1(ctx)
                                    .await
                                    .map_err(|err| err.into_server_error(ctx.item.pos))
                            };
                            let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                            let ctx_obj = ctx
                                .with_selection_set(&ctx.item.node.selection_set);
                            return async_graphql::OutputType::resolve(
                                    &obj,
                                    &ctx_obj,
                                    ctx.item,
                                )
                                .await
                                .map(::std::option::Option::Some);
                        }
                        ::std::result::Result::Ok(::std::option::Option::None)
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputTypeMarker for MyObj1 {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("MyObj1")
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Object,
                        |registry| async_graphql::registry::MetaType::Object {
                            name: ::std::borrow::Cow::into_owned(
                                ::std::borrow::Cow::Borrowed("MyObj1"),
                            ),
                            description: ::std::option::Option::None,
                            fields: {
                                let mut fields = async_graphql::indexmap::IndexMap::new();
                                fields
                                    .insert(
                                        ::std::borrow::ToOwned::to_owned("value1"),
                                        async_graphql::registry::MetaField {
                                            name: ::std::borrow::ToOwned::to_owned("value1"),
                                            description: ::std::option::Option::None,
                                            args: ::std::default::Default::default(),
                                            ty: <i32 as async_graphql::OutputTypeMarker>::create_type_info(
                                                registry,
                                            ),
                                            deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                            cache_control: async_graphql::CacheControl {
                                                public: true,
                                                max_age: 0i32,
                                            },
                                            external: false,
                                            provides: ::std::option::Option::None,
                                            requires: ::std::option::Option::None,
                                            shareable: false,
                                            inaccessible: false,
                                            tags: ::alloc::vec::Vec::new(),
                                            override_from: ::std::option::Option::None,
                                            visible: ::std::option::Option::None,
                                            compute_complexity: ::std::option::Option::None,
                                            directive_invocations: ::alloc::vec::Vec::new(),
                                            requires_scopes: ::alloc::vec::Vec::new(),
                                        },
                                    );
                                fields
                            },
                            cache_control: async_graphql::CacheControl {
                                public: true,
                                max_age: 0i32,
                            },
                            extends: false,
                            shareable: false,
                            resolvable: true,
                            inaccessible: false,
                            interface_object: false,
                            tags: ::alloc::vec::Vec::new(),
                            keys: ::std::option::Option::None,
                            visible: ::std::option::Option::None,
                            is_subscription: false,
                            rust_typename: ::std::option::Option::Some(
                                ::std::any::type_name::<Self>(),
                            ),
                            directive_invocations: ::alloc::vec::Vec::new(),
                            requires_scopes: ::alloc::vec::Vec::new(),
                        },
                    )
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputType for MyObj1 {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::type_name()
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                &__self as &dyn async_graphql::resolver_utils::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::ObjectType for MyObj1 {}
        struct MyObj2 {
            value2: i32,
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl MyObj2 {
            #[inline]
            #[allow(missing_docs)]
            async fn value2(
                &self,
                ctx: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<&i32> {
                ::std::result::Result::Ok(&self.value2)
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::resolver_utils::ContainerType for MyObj2 {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = {
                        if ctx.item.node.name.node == "value2" {
                            let f = async move {
                                __self
                                    .value2(ctx)
                                    .await
                                    .map_err(|err| err.into_server_error(ctx.item.pos))
                            };
                            let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                            let ctx_obj = ctx
                                .with_selection_set(&ctx.item.node.selection_set);
                            return async_graphql::OutputType::resolve(
                                    &obj,
                                    &ctx_obj,
                                    ctx.item,
                                )
                                .await
                                .map(::std::option::Option::Some);
                        }
                        ::std::result::Result::Ok(::std::option::Option::None)
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputTypeMarker for MyObj2 {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("MyObj2")
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Object,
                        |registry| async_graphql::registry::MetaType::Object {
                            name: ::std::borrow::Cow::into_owned(
                                ::std::borrow::Cow::Borrowed("MyObj2"),
                            ),
                            description: ::std::option::Option::None,
                            fields: {
                                let mut fields = async_graphql::indexmap::IndexMap::new();
                                fields
                                    .insert(
                                        ::std::borrow::ToOwned::to_owned("value2"),
                                        async_graphql::registry::MetaField {
                                            name: ::std::borrow::ToOwned::to_owned("value2"),
                                            description: ::std::option::Option::None,
                                            args: ::std::default::Default::default(),
                                            ty: <i32 as async_graphql::OutputTypeMarker>::create_type_info(
                                                registry,
                                            ),
                                            deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                            cache_control: async_graphql::CacheControl {
                                                public: true,
                                                max_age: 0i32,
                                            },
                                            external: false,
                                            provides: ::std::option::Option::None,
                                            requires: ::std::option::Option::None,
                                            shareable: false,
                                            inaccessible: false,
                                            tags: ::alloc::vec::Vec::new(),
                                            override_from: ::std::option::Option::None,
                                            visible: ::std::option::Option::None,
                                            compute_complexity: ::std::option::Option::None,
                                            directive_invocations: ::alloc::vec::Vec::new(),
                                            requires_scopes: ::alloc::vec::Vec::new(),
                                        },
                                    );
                                fields
                            },
                            cache_control: async_graphql::CacheControl {
                                public: true,
                                max_age: 0i32,
                            },
                            extends: false,
                            shareable: false,
                            resolvable: true,
                            inaccessible: false,
                            interface_object: false,
                            tags: ::alloc::vec::Vec::new(),
                            keys: ::std::option::Option::None,
                            visible: ::std::option::Option::None,
                            is_subscription: false,
                            rust_typename: ::std::option::Option::Some(
                                ::std::any::type_name::<Self>(),
                            ),
                            directive_invocations: ::alloc::vec::Vec::new(),
                            requires_scopes: ::alloc::vec::Vec::new(),
                        },
                    )
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputType for MyObj2 {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::type_name()
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                &__self as &dyn async_graphql::resolver_utils::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::ObjectType for MyObj2 {}
        enum InnerUnion1 {
            A(MyObj1),
        }
        const _: () = {
            fn assert_impl() {
                let _: ::static_assertions_next::True = {
                    #[allow(unused_imports)]
                    use ::static_assertions_next::{
                        _bool::{True, False},
                        _core::{marker::PhantomData, ops::Deref},
                    };
                    trait DoesntImpl {
                        const DOES_IMPL: False = False;
                    }
                    impl<T: ?Sized> DoesntImpl for T {}
                    *{
                        struct Wrapper<T: ?Sized>(PhantomData<T>);
                        #[allow(dead_code)]
                        impl<T: ?Sized + async_graphql::ObjectType> Wrapper<T> {
                            const DOES_IMPL: True = True;
                        }
                        &<Wrapper<MyObj1>>::DOES_IMPL
                    }
                };
            }
        };
        #[allow(clippy::all, clippy::pedantic)]
        impl ::std::convert::From<MyObj1> for InnerUnion1 {
            fn from(obj: MyObj1) -> Self {
                InnerUnion1::A(obj)
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::resolver_utils::ContainerType for InnerUnion1 {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = { ::std::result::Result::Ok(::std::option::Option::None) };
                    #[allow(unreachable_code)] __ret
                })
            }
            fn collect_all_fields<'__life>(
                &'__life self,
                ctx: &async_graphql::ContextSelectionSet<'__life>,
                fields: &mut async_graphql::resolver_utils::Fields<'__life>,
            ) -> async_graphql::ServerResult<()> {
                match self {
                    InnerUnion1::A(obj) => obj.collect_all_fields(ctx, fields),
                }
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputTypeMarker for InnerUnion1 {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("InnerUnion1")
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                match self {
                    InnerUnion1::A(obj) => {
                        <MyObj1 as async_graphql::OutputTypeMarker>::type_name()
                    }
                }
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Union,
                        |registry| {
                            <MyObj1 as async_graphql::OutputTypeMarker>::create_type_info(
                                registry,
                            );
                            async_graphql::registry::MetaType::Union {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("InnerUnion1"),
                                ),
                                description: ::std::option::Option::None,
                                possible_types: {
                                    let mut possible_types = async_graphql::indexmap::IndexSet::new();
                                    possible_types
                                        .insert(
                                            <MyObj1 as async_graphql::OutputTypeMarker>::type_name()
                                                .into_owned(),
                                        );
                                    possible_types
                                },
                                visible: ::std::option::Option::None,
                                inaccessible: false,
                                tags: ::alloc::vec::Vec::new(),
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::std::vec::Vec::new(),
                            }
                        },
                    )
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputType for InnerUnion1 {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::type_name()
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::introspection_type_name(&self)
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                __self as &dyn async_graphql::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::UnionType for InnerUnion1 {}
        enum InnerUnion2 {
            B(MyObj2),
        }
        const _: () = {
            fn assert_impl() {
                let _: ::static_assertions_next::True = {
                    #[allow(unused_imports)]
                    use ::static_assertions_next::{
                        _bool::{True, False},
                        _core::{marker::PhantomData, ops::Deref},
                    };
                    trait DoesntImpl {
                        const DOES_IMPL: False = False;
                    }
                    impl<T: ?Sized> DoesntImpl for T {}
                    *{
                        struct Wrapper<T: ?Sized>(PhantomData<T>);
                        #[allow(dead_code)]
                        impl<T: ?Sized + async_graphql::ObjectType> Wrapper<T> {
                            const DOES_IMPL: True = True;
                        }
                        &<Wrapper<MyObj2>>::DOES_IMPL
                    }
                };
            }
        };
        #[allow(clippy::all, clippy::pedantic)]
        impl ::std::convert::From<MyObj2> for InnerUnion2 {
            fn from(obj: MyObj2) -> Self {
                InnerUnion2::B(obj)
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::resolver_utils::ContainerType for InnerUnion2 {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = { ::std::result::Result::Ok(::std::option::Option::None) };
                    #[allow(unreachable_code)] __ret
                })
            }
            fn collect_all_fields<'__life>(
                &'__life self,
                ctx: &async_graphql::ContextSelectionSet<'__life>,
                fields: &mut async_graphql::resolver_utils::Fields<'__life>,
            ) -> async_graphql::ServerResult<()> {
                match self {
                    InnerUnion2::B(obj) => obj.collect_all_fields(ctx, fields),
                }
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputTypeMarker for InnerUnion2 {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("InnerUnion2")
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                match self {
                    InnerUnion2::B(obj) => {
                        <MyObj2 as async_graphql::OutputTypeMarker>::type_name()
                    }
                }
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Union,
                        |registry| {
                            <MyObj2 as async_graphql::OutputTypeMarker>::create_type_info(
                                registry,
                            );
                            async_graphql::registry::MetaType::Union {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("InnerUnion2"),
                                ),
                                description: ::std::option::Option::None,
                                possible_types: {
                                    let mut possible_types = async_graphql::indexmap::IndexSet::new();
                                    possible_types
                                        .insert(
                                            <MyObj2 as async_graphql::OutputTypeMarker>::type_name()
                                                .into_owned(),
                                        );
                                    possible_types
                                },
                                visible: ::std::option::Option::None,
                                inaccessible: false,
                                tags: ::alloc::vec::Vec::new(),
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::std::vec::Vec::new(),
                            }
                        },
                    )
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputType for InnerUnion2 {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::type_name()
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::introspection_type_name(&self)
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                __self as &dyn async_graphql::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::UnionType for InnerUnion2 {}
        enum MyUnion {
            #[graphql(flatten)]
            Inner1(InnerUnion1),
            #[graphql(flatten)]
            Inner2(InnerUnion2),
        }
        const _: () = {
            fn assert_impl() {
                let _: ::static_assertions_next::True = {
                    #[allow(unused_imports)]
                    use ::static_assertions_next::{
                        _bool::{True, False},
                        _core::{marker::PhantomData, ops::Deref},
                    };
                    trait DoesntImpl {
                        const DOES_IMPL: False = False;
                    }
                    impl<T: ?Sized> DoesntImpl for T {}
                    *{
                        struct Wrapper<T: ?Sized>(PhantomData<T>);
                        #[allow(dead_code)]
                        impl<T: ?Sized + async_graphql::UnionType> Wrapper<T> {
                            const DOES_IMPL: True = True;
                        }
                        &<Wrapper<InnerUnion1>>::DOES_IMPL
                    }
                };
            }
        };
        #[allow(clippy::all, clippy::pedantic)]
        impl ::std::convert::From<InnerUnion1> for MyUnion {
            fn from(obj: InnerUnion1) -> Self {
                MyUnion::Inner1(obj)
            }
        }
        const _: () = {
            fn assert_impl() {
                let _: ::static_assertions_next::True = {
                    #[allow(unused_imports)]
                    use ::static_assertions_next::{
                        _bool::{True, False},
                        _core::{marker::PhantomData, ops::Deref},
                    };
                    trait DoesntImpl {
                        const DOES_IMPL: False = False;
                    }
                    impl<T: ?Sized> DoesntImpl for T {}
                    *{
                        struct Wrapper<T: ?Sized>(PhantomData<T>);
                        #[allow(dead_code)]
                        impl<T: ?Sized + async_graphql::UnionType> Wrapper<T> {
                            const DOES_IMPL: True = True;
                        }
                        &<Wrapper<InnerUnion2>>::DOES_IMPL
                    }
                };
            }
        };
        #[allow(clippy::all, clippy::pedantic)]
        impl ::std::convert::From<InnerUnion2> for MyUnion {
            fn from(obj: InnerUnion2) -> Self {
                MyUnion::Inner2(obj)
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::resolver_utils::ContainerType for MyUnion {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = { ::std::result::Result::Ok(::std::option::Option::None) };
                    #[allow(unreachable_code)] __ret
                })
            }
            fn collect_all_fields<'__life>(
                &'__life self,
                ctx: &async_graphql::ContextSelectionSet<'__life>,
                fields: &mut async_graphql::resolver_utils::Fields<'__life>,
            ) -> async_graphql::ServerResult<()> {
                match self {
                    MyUnion::Inner1(obj) => obj.collect_all_fields(ctx, fields),
                    MyUnion::Inner2(obj) => obj.collect_all_fields(ctx, fields),
                }
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputTypeMarker for MyUnion {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("MyUnion")
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                match self {
                    MyUnion::Inner1(obj) => {
                        <InnerUnion1 as async_graphql::OutputTypeMarker>::introspection_type_name(
                            obj,
                        )
                    }
                    MyUnion::Inner2(obj) => {
                        <InnerUnion2 as async_graphql::OutputTypeMarker>::introspection_type_name(
                            obj,
                        )
                    }
                }
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Union,
                        |registry| {
                            async_graphql::registry::MetaType::Union {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("MyUnion"),
                                ),
                                description: ::std::option::Option::None,
                                possible_types: {
                                    let mut possible_types = async_graphql::indexmap::IndexSet::new();
                                    if let async_graphql::registry::MetaType::Union {
                                        possible_types: possible_types2,
                                        ..
                                    } = registry.create_fake_output_type::<InnerUnion1>()
                                    {
                                        possible_types.extend(possible_types2);
                                    }
                                    if let async_graphql::registry::MetaType::Union {
                                        possible_types: possible_types2,
                                        ..
                                    } = registry.create_fake_output_type::<InnerUnion2>()
                                    {
                                        possible_types.extend(possible_types2);
                                    }
                                    possible_types
                                },
                                visible: ::std::option::Option::None,
                                inaccessible: false,
                                tags: ::alloc::vec::Vec::new(),
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::std::vec::Vec::new(),
                            }
                        },
                    )
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputType for MyUnion {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::type_name()
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::introspection_type_name(&self)
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                __self as &dyn async_graphql::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::UnionType for MyUnion {}
        struct Query;
        impl Query {
            async fn value1(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<MyUnion> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: MyUnion = {
                                InnerUnion1::A(MyObj1 { value1: 99 }).into()
                            };
                            value
                        }
                            .await,
                    )
                }
            }
            async fn value2(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<MyUnion> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: MyUnion = {
                                InnerUnion2::B(MyObj2 { value2: 88 }).into()
                            };
                            value
                        }
                            .await,
                    )
                }
            }
            async fn value3(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<InnerUnion1> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: InnerUnion1 = {
                                InnerUnion1::A(MyObj1 { value1: 77 })
                            };
                            value
                        }
                            .await,
                    )
                }
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __FieldIdent {
                value1,
                value2,
                value3,
            }
            impl __FieldIdent {
                fn from_name(
                    __name: &async_graphql::Name,
                ) -> ::std::option::Option<__FieldIdent> {
                    match __name.as_str() {
                        "value1" => ::std::option::Option::Some(__FieldIdent::value1),
                        "value2" => ::std::option::Option::Some(__FieldIdent::value2),
                        "value3" => ::std::option::Option::Some(__FieldIdent::value3),
                        _ => ::std::option::Option::None,
                    }
                }
            }
            impl Query {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __value1_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.value1(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __value2_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.value2(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __value3_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.value3(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
            }
            #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
            #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
            impl async_graphql::resolver_utils::ContainerType for Query {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let __field = __FieldIdent::from_name(
                                &ctx.item.node.name.node,
                            );
                            match __field {
                                ::std::option::Option::Some(__FieldIdent::value1) => {
                                    return __self.__value1_resolver(&ctx).await;
                                }
                                ::std::option::Option::Some(__FieldIdent::value2) => {
                                    return __self.__value2_resolver(&ctx).await;
                                }
                                ::std::option::Option::Some(__FieldIdent::value3) => {
                                    return __self.__value3_resolver(&ctx).await;
                                }
                                None => {}
                            }
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                    params: &'life3 async_graphql::Value,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let params = match params {
                                async_graphql::Value::Object(params) => params,
                                _ => {
                                    return ::std::result::Result::Ok(
                                        ::std::option::Option::None,
                                    );
                                }
                            };
                            let typename = if let ::std::option::Option::Some(
                                async_graphql::Value::String(typename),
                            ) = params.get("__typename")
                            {
                                typename
                            } else {
                                return ::std::result::Result::Err(
                                    async_graphql::ServerError::new(
                                        r#""__typename" must be an existing string."#,
                                        ::std::option::Option::Some(ctx.item.pos),
                                    ),
                                );
                            };
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputTypeMarker for Query {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed("Query")
                }
                fn create_type_info(
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    let ty = registry
                        .create_output_type::<
                            Self,
                            _,
                        >(
                            async_graphql::registry::MetaTypeId::Object,
                            |registry| async_graphql::registry::MetaType::Object {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("Query"),
                                ),
                                description: ::std::option::Option::None,
                                fields: {
                                    let mut fields = async_graphql::indexmap::IndexMap::new();
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("value1"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("value1"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <MyUnion as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("value2"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("value2"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <MyUnion as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("value3"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("value3"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <InnerUnion1 as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                },
                                cache_control: async_graphql::CacheControl {
                                    public: true,
                                    max_age: 0i32,
                                },
                                extends: false,
                                shareable: false,
                                resolvable: true,
                                inaccessible: false,
                                interface_object: false,
                                tags: ::alloc::vec::Vec::new(),
                                keys: ::std::option::Option::None,
                                visible: ::std::option::Option::None,
                                is_subscription: false,
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::alloc::vec::Vec::new(),
                                requires_scopes: ::alloc::vec::Vec::new(),
                            },
                        );
                    ty
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputType for Query {
                fn type_name(
                    &self,
                ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    <Self as async_graphql::OutputTypeMarker>::type_name()
                }
                fn create_type_info(
                    &self,
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                    _field: &'life3 async_graphql::Positioned<
                        async_graphql::parser::types::Field,
                    >,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<async_graphql::Value>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<async_graphql::Value>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                            async_graphql::resolver_utils::resolve_container(
                                    ctx,
                                    &__self as &dyn async_graphql::resolver_utils::ContainerType,
                                )
                                .await
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            impl async_graphql::ObjectType for Query {}
        };
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        let query = r#"
    {
        value1 {
            ... on MyObj1 {
                value1
            }
        }
        value2 {
            ... on MyObj2 {
                value2
            }
        }
        value3 {
            ... on MyObj1 {
                value1
            }
        }
    }"#;
        match (
            &schema.execute(query).await.into_result().unwrap().data,
            &::async_graphql_value::ConstValue::Object({
                let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                let _ = object
                    .insert(
                        ::async_graphql_value::Name::new("value1"),
                        ::async_graphql_value::ConstValue::Object({
                            let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                            let _ = object
                                .insert(
                                    ::async_graphql_value::Name::new("value1"),
                                    ::async_graphql_value::to_value(&99).unwrap(),
                                );
                            object
                        }),
                    );
                let _ = object
                    .insert(
                        ::async_graphql_value::Name::new("value2"),
                        ::async_graphql_value::ConstValue::Object({
                            let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                            let _ = object
                                .insert(
                                    ::async_graphql_value::Name::new("value2"),
                                    ::async_graphql_value::to_value(&88).unwrap(),
                                );
                            object
                        }),
                    );
                let _ = object
                    .insert(
                        ::async_graphql_value::Name::new("value3"),
                        ::async_graphql_value::ConstValue::Object({
                            let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                            let _ = object
                                .insert(
                                    ::async_graphql_value::Name::new("value1"),
                                    ::async_graphql_value::to_value(&77).unwrap(),
                                );
                            object
                        }),
                    );
                object
            }),
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    };
    let mut body = body;
    #[allow(unused_mut)]
    let mut body = unsafe { ::tokio::macros::support::Pin::new_unchecked(&mut body) };
    let body: ::core::pin::Pin<&mut dyn ::core::future::Future<Output = ()>> = body;
    #[allow(
        clippy::expect_used,
        clippy::diverging_sub_expression,
        clippy::needless_return
    )]
    {
        return tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
extern crate test;
#[rustc_test_marker = "test_trait_object_in_union"]
#[doc(hidden)]
pub const test_trait_object_in_union: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("test_trait_object_in_union"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/union.rs",
        start_line: 372usize,
        start_col: 14usize,
        end_line: 372usize,
        end_col: 40usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(test_trait_object_in_union()),
    ),
};
pub fn test_trait_object_in_union() {
    let body = async {
        pub trait ProductTrait: Send + Sync {
            fn id(&self) -> &str;
        }
        impl dyn ProductTrait {
            async fn gql_id(&self, _ctx: &Context<'_>) -> async_graphql::Result<&str> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: &str = { self.id() };
                            value
                        }
                            .await,
                    )
                }
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __FieldIdent {
                gql_id,
            }
            impl __FieldIdent {
                fn from_name(
                    __name: &async_graphql::Name,
                ) -> ::std::option::Option<__FieldIdent> {
                    match __name.as_str() {
                        "id" => ::std::option::Option::Some(__FieldIdent::gql_id),
                        _ => ::std::option::Option::None,
                    }
                }
            }
            impl dyn ProductTrait {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __gql_id_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.gql_id(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
            }
            #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
            #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
            impl async_graphql::resolver_utils::ContainerType for dyn ProductTrait {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let __field = __FieldIdent::from_name(
                                &ctx.item.node.name.node,
                            );
                            match __field {
                                ::std::option::Option::Some(__FieldIdent::gql_id) => {
                                    return __self.__gql_id_resolver(&ctx).await;
                                }
                                None => {}
                            }
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                    params: &'life3 async_graphql::Value,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let params = match params {
                                async_graphql::Value::Object(params) => params,
                                _ => {
                                    return ::std::result::Result::Ok(
                                        ::std::option::Option::None,
                                    );
                                }
                            };
                            let typename = if let ::std::option::Option::Some(
                                async_graphql::Value::String(typename),
                            ) = params.get("__typename")
                            {
                                typename
                            } else {
                                return ::std::result::Result::Err(
                                    async_graphql::ServerError::new(
                                        r#""__typename" must be an existing string."#,
                                        ::std::option::Option::Some(ctx.item.pos),
                                    ),
                                );
                            };
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputTypeMarker for dyn ProductTrait {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed("ProductTrait")
                }
                fn create_type_info(
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    let ty = registry
                        .create_output_type::<
                            Self,
                            _,
                        >(
                            async_graphql::registry::MetaTypeId::Object,
                            |registry| async_graphql::registry::MetaType::Object {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("ProductTrait"),
                                ),
                                description: ::std::option::Option::None,
                                fields: {
                                    let mut fields = async_graphql::indexmap::IndexMap::new();
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("id"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("id"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <&str as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                },
                                cache_control: async_graphql::CacheControl {
                                    public: true,
                                    max_age: 0i32,
                                },
                                extends: false,
                                shareable: false,
                                resolvable: true,
                                inaccessible: false,
                                interface_object: false,
                                tags: ::alloc::vec::Vec::new(),
                                keys: ::std::option::Option::None,
                                visible: ::std::option::Option::None,
                                is_subscription: false,
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::alloc::vec::Vec::new(),
                                requires_scopes: ::alloc::vec::Vec::new(),
                            },
                        );
                    ty
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputType for dyn ProductTrait {
                fn type_name(
                    &self,
                ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    <Self as async_graphql::OutputTypeMarker>::type_name()
                }
                fn create_type_info(
                    &self,
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                    _field: &'life3 async_graphql::Positioned<
                        async_graphql::parser::types::Field,
                    >,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<async_graphql::Value>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<async_graphql::Value>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                            async_graphql::resolver_utils::resolve_container(
                                    ctx,
                                    &__self as &dyn async_graphql::resolver_utils::ContainerType,
                                )
                                .await
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            impl async_graphql::ObjectType for dyn ProductTrait {}
        };
        struct MyProduct;
        impl ProductTrait for MyProduct {
            fn id(&self) -> &str {
                "abc"
            }
        }
        pub enum Content {
            Product(Box<dyn ProductTrait>),
        }
        const _: () = {
            fn assert_impl() {
                let _: ::static_assertions_next::True = {
                    #[allow(unused_imports)]
                    use ::static_assertions_next::{
                        _bool::{True, False},
                        _core::{marker::PhantomData, ops::Deref},
                    };
                    trait DoesntImpl {
                        const DOES_IMPL: False = False;
                    }
                    impl<T: ?Sized> DoesntImpl for T {}
                    *{
                        struct Wrapper<T: ?Sized>(PhantomData<T>);
                        #[allow(dead_code)]
                        impl<T: ?Sized + async_graphql::ObjectType> Wrapper<T> {
                            const DOES_IMPL: True = True;
                        }
                        &<Wrapper<Box<dyn ProductTrait>>>::DOES_IMPL
                    }
                };
            }
        };
        #[allow(clippy::all, clippy::pedantic)]
        impl ::std::convert::From<Box<dyn ProductTrait>> for Content {
            fn from(obj: Box<dyn ProductTrait>) -> Self {
                Content::Product(obj)
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::resolver_utils::ContainerType for Content {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = { ::std::result::Result::Ok(::std::option::Option::None) };
                    #[allow(unreachable_code)] __ret
                })
            }
            fn collect_all_fields<'__life>(
                &'__life self,
                ctx: &async_graphql::ContextSelectionSet<'__life>,
                fields: &mut async_graphql::resolver_utils::Fields<'__life>,
            ) -> async_graphql::ServerResult<()> {
                match self {
                    Content::Product(obj) => obj.collect_all_fields(ctx, fields),
                }
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputTypeMarker for Content {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("Content")
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                match self {
                    Content::Product(obj) => {
                        <Box<
                            dyn ProductTrait,
                        > as async_graphql::OutputTypeMarker>::type_name()
                    }
                }
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Union,
                        |registry| {
                            <Box<
                                dyn ProductTrait,
                            > as async_graphql::OutputTypeMarker>::create_type_info(
                                registry,
                            );
                            async_graphql::registry::MetaType::Union {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("Content"),
                                ),
                                description: ::std::option::Option::None,
                                possible_types: {
                                    let mut possible_types = async_graphql::indexmap::IndexSet::new();
                                    possible_types
                                        .insert(
                                            <Box<
                                                dyn ProductTrait,
                                            > as async_graphql::OutputTypeMarker>::type_name()
                                                .into_owned(),
                                        );
                                    possible_types
                                },
                                visible: ::std::option::Option::None,
                                inaccessible: false,
                                tags: ::alloc::vec::Vec::new(),
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::std::vec::Vec::new(),
                            }
                        },
                    )
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputType for Content {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::type_name()
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::introspection_type_name(&self)
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                __self as &dyn async_graphql::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::UnionType for Content {}
        struct Query;
        impl Query {
            async fn value(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<Content> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: Content = {
                                Content::Product(Box::new(MyProduct))
                            };
                            value
                        }
                            .await,
                    )
                }
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __FieldIdent {
                value,
            }
            impl __FieldIdent {
                fn from_name(
                    __name: &async_graphql::Name,
                ) -> ::std::option::Option<__FieldIdent> {
                    match __name.as_str() {
                        "value" => ::std::option::Option::Some(__FieldIdent::value),
                        _ => ::std::option::Option::None,
                    }
                }
            }
            impl Query {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __value_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.value(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
            }
            #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
            #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
            impl async_graphql::resolver_utils::ContainerType for Query {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let __field = __FieldIdent::from_name(
                                &ctx.item.node.name.node,
                            );
                            match __field {
                                ::std::option::Option::Some(__FieldIdent::value) => {
                                    return __self.__value_resolver(&ctx).await;
                                }
                                None => {}
                            }
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                    params: &'life3 async_graphql::Value,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let params = match params {
                                async_graphql::Value::Object(params) => params,
                                _ => {
                                    return ::std::result::Result::Ok(
                                        ::std::option::Option::None,
                                    );
                                }
                            };
                            let typename = if let ::std::option::Option::Some(
                                async_graphql::Value::String(typename),
                            ) = params.get("__typename")
                            {
                                typename
                            } else {
                                return ::std::result::Result::Err(
                                    async_graphql::ServerError::new(
                                        r#""__typename" must be an existing string."#,
                                        ::std::option::Option::Some(ctx.item.pos),
                                    ),
                                );
                            };
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputTypeMarker for Query {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed("Query")
                }
                fn create_type_info(
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    let ty = registry
                        .create_output_type::<
                            Self,
                            _,
                        >(
                            async_graphql::registry::MetaTypeId::Object,
                            |registry| async_graphql::registry::MetaType::Object {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("Query"),
                                ),
                                description: ::std::option::Option::None,
                                fields: {
                                    let mut fields = async_graphql::indexmap::IndexMap::new();
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("value"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("value"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <Content as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                },
                                cache_control: async_graphql::CacheControl {
                                    public: true,
                                    max_age: 0i32,
                                },
                                extends: false,
                                shareable: false,
                                resolvable: true,
                                inaccessible: false,
                                interface_object: false,
                                tags: ::alloc::vec::Vec::new(),
                                keys: ::std::option::Option::None,
                                visible: ::std::option::Option::None,
                                is_subscription: false,
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::alloc::vec::Vec::new(),
                                requires_scopes: ::alloc::vec::Vec::new(),
                            },
                        );
                    ty
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputType for Query {
                fn type_name(
                    &self,
                ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    <Self as async_graphql::OutputTypeMarker>::type_name()
                }
                fn create_type_info(
                    &self,
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                    _field: &'life3 async_graphql::Positioned<
                        async_graphql::parser::types::Field,
                    >,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<async_graphql::Value>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<async_graphql::Value>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                            async_graphql::resolver_utils::resolve_container(
                                    ctx,
                                    &__self as &dyn async_graphql::resolver_utils::ContainerType,
                                )
                                .await
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            impl async_graphql::ObjectType for Query {}
        };
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        match (
            &schema
                .execute("{ value { ... on ProductTrait { id } } }")
                .await
                .into_result()
                .unwrap()
                .data,
            &::async_graphql_value::ConstValue::Object({
                let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                let _ = object
                    .insert(
                        ::async_graphql_value::Name::new("value"),
                        ::async_graphql_value::ConstValue::Object({
                            let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                            let _ = object
                                .insert(
                                    ::async_graphql_value::Name::new("id"),
                                    ::async_graphql_value::to_value(&"abc").unwrap(),
                                );
                            object
                        }),
                    );
                object
            }),
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    };
    let mut body = body;
    #[allow(unused_mut)]
    let mut body = unsafe { ::tokio::macros::support::Pin::new_unchecked(&mut body) };
    let body: ::core::pin::Pin<&mut dyn ::core::future::Future<Output = ()>> = body;
    #[allow(
        clippy::expect_used,
        clippy::diverging_sub_expression,
        clippy::needless_return
    )]
    {
        return tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
extern crate test;
#[rustc_test_marker = "test_macro_generated_union"]
#[doc(hidden)]
pub const test_macro_generated_union: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("test_macro_generated_union"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/union.rs",
        start_line: 433usize,
        start_col: 8usize,
        end_line: 433usize,
        end_col: 34usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(test_macro_generated_union()),
    ),
};
pub fn test_macro_generated_union() {
    pub struct IntObj {
        pub val: i32,
    }
    #[allow(clippy::all, clippy::pedantic)]
    impl IntObj {
        #[inline]
        #[allow(missing_docs)]
        pub async fn val(
            &self,
            ctx: &async_graphql::Context<'_>,
        ) -> async_graphql::Result<&i32> {
            ::std::result::Result::Ok(&self.val)
        }
    }
    #[allow(clippy::all, clippy::pedantic)]
    impl async_graphql::resolver_utils::ContainerType for IntObj {
        #[allow(
            elided_named_lifetimes,
            clippy::async_yields_async,
            clippy::diverging_sub_expression,
            clippy::let_unit_value,
            clippy::needless_arbitrary_self_type,
            clippy::no_effect_underscore_binding,
            clippy::shadow_same,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds,
            clippy::used_underscore_binding
        )]
        fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
            &'life0 self,
            ctx: &'life1 async_graphql::Context<'life2>,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<
                    Output = async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    >,
                > + ::core::marker::Send + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            'life2: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                    async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    >,
                > {
                    #[allow(unreachable_code)] return __ret;
                }
                let __self = self;
                let __ret: async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > = {
                    if ctx.item.node.name.node == "val" {
                        let f = async move {
                            __self
                                .val(ctx)
                                .await
                                .map_err(|err| err.into_server_error(ctx.item.pos))
                        };
                        let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                        let ctx_obj = ctx
                            .with_selection_set(&ctx.item.node.selection_set);
                        return async_graphql::OutputType::resolve(
                                &obj,
                                &ctx_obj,
                                ctx.item,
                            )
                            .await
                            .map(::std::option::Option::Some);
                    }
                    ::std::result::Result::Ok(::std::option::Option::None)
                };
                #[allow(unreachable_code)] __ret
            })
        }
    }
    #[allow(clippy::all, clippy::pedantic)]
    impl async_graphql::OutputTypeMarker for IntObj {
        fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
            ::std::borrow::Cow::Borrowed("IntObj")
        }
        fn create_type_info(
            registry: &mut async_graphql::registry::Registry,
        ) -> ::std::string::String {
            registry
                .create_output_type::<
                    Self,
                    _,
                >(
                    async_graphql::registry::MetaTypeId::Object,
                    |registry| async_graphql::registry::MetaType::Object {
                        name: ::std::borrow::Cow::into_owned(
                            ::std::borrow::Cow::Borrowed("IntObj"),
                        ),
                        description: ::std::option::Option::None,
                        fields: {
                            let mut fields = async_graphql::indexmap::IndexMap::new();
                            fields
                                .insert(
                                    ::std::borrow::ToOwned::to_owned("val"),
                                    async_graphql::registry::MetaField {
                                        name: ::std::borrow::ToOwned::to_owned("val"),
                                        description: ::std::option::Option::None,
                                        args: ::std::default::Default::default(),
                                        ty: <i32 as async_graphql::OutputTypeMarker>::create_type_info(
                                            registry,
                                        ),
                                        deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                        cache_control: async_graphql::CacheControl {
                                            public: true,
                                            max_age: 0i32,
                                        },
                                        external: false,
                                        provides: ::std::option::Option::None,
                                        requires: ::std::option::Option::None,
                                        shareable: false,
                                        inaccessible: false,
                                        tags: ::alloc::vec::Vec::new(),
                                        override_from: ::std::option::Option::None,
                                        visible: ::std::option::Option::None,
                                        compute_complexity: ::std::option::Option::None,
                                        directive_invocations: ::alloc::vec::Vec::new(),
                                        requires_scopes: ::alloc::vec::Vec::new(),
                                    },
                                );
                            fields
                        },
                        cache_control: async_graphql::CacheControl {
                            public: true,
                            max_age: 0i32,
                        },
                        extends: false,
                        shareable: false,
                        resolvable: true,
                        inaccessible: false,
                        interface_object: false,
                        tags: ::alloc::vec::Vec::new(),
                        keys: ::std::option::Option::None,
                        visible: ::std::option::Option::None,
                        is_subscription: false,
                        rust_typename: ::std::option::Option::Some(
                            ::std::any::type_name::<Self>(),
                        ),
                        directive_invocations: ::alloc::vec::Vec::new(),
                        requires_scopes: ::alloc::vec::Vec::new(),
                    },
                )
        }
    }
    #[allow(clippy::all, clippy::pedantic)]
    impl async_graphql::OutputType for IntObj {
        fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
            <Self as async_graphql::OutputTypeMarker>::type_name()
        }
        fn create_type_info(
            &self,
            registry: &mut async_graphql::registry::Registry,
        ) -> ::std::string::String {
            <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
        }
        #[allow(
            elided_named_lifetimes,
            clippy::async_yields_async,
            clippy::diverging_sub_expression,
            clippy::let_unit_value,
            clippy::needless_arbitrary_self_type,
            clippy::no_effect_underscore_binding,
            clippy::shadow_same,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds,
            clippy::used_underscore_binding
        )]
        fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
            &'life0 self,
            ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
            _field: &'life3 async_graphql::Positioned<
                async_graphql::parser::types::Field,
            >,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<
                    Output = async_graphql::ServerResult<async_graphql::Value>,
                > + ::core::marker::Send + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            'life2: 'async_trait,
            'life3: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                    async_graphql::ServerResult<async_graphql::Value>,
                > {
                    #[allow(unreachable_code)] return __ret;
                }
                let __self = self;
                let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                    async_graphql::resolver_utils::resolve_container(
                            ctx,
                            &__self as &dyn async_graphql::resolver_utils::ContainerType,
                        )
                        .await
                };
                #[allow(unreachable_code)] __ret
            })
        }
    }
    impl async_graphql::ObjectType for IntObj {}
    pub enum MyEnum {
        Val(IntObj),
    }
    const _: () = {
        fn assert_impl() {
            let _: ::static_assertions_next::True = {
                #[allow(unused_imports)]
                use ::static_assertions_next::{
                    _bool::{True, False},
                    _core::{marker::PhantomData, ops::Deref},
                };
                trait DoesntImpl {
                    const DOES_IMPL: False = False;
                }
                impl<T: ?Sized> DoesntImpl for T {}
                *{
                    struct Wrapper<T: ?Sized>(PhantomData<T>);
                    #[allow(dead_code)]
                    impl<T: ?Sized + async_graphql::ObjectType> Wrapper<T> {
                        const DOES_IMPL: True = True;
                    }
                    &<Wrapper<IntObj>>::DOES_IMPL
                }
            };
        }
    };
    #[allow(clippy::all, clippy::pedantic)]
    impl ::std::convert::From<IntObj> for MyEnum {
        fn from(obj: IntObj) -> Self {
            MyEnum::Val(obj)
        }
    }
    #[allow(clippy::all, clippy::pedantic)]
    impl async_graphql::resolver_utils::ContainerType for MyEnum {
        #[allow(
            elided_named_lifetimes,
            clippy::async_yields_async,
            clippy::diverging_sub_expression,
            clippy::let_unit_value,
            clippy::needless_arbitrary_self_type,
            clippy::no_effect_underscore_binding,
            clippy::shadow_same,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds,
            clippy::used_underscore_binding
        )]
        fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
            &'life0 self,
            ctx: &'life1 async_graphql::Context<'life2>,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<
                    Output = async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    >,
                > + ::core::marker::Send + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            'life2: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                    async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    >,
                > {
                    #[allow(unreachable_code)] return __ret;
                }
                let __self = self;
                let __ret: async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > = { ::std::result::Result::Ok(::std::option::Option::None) };
                #[allow(unreachable_code)] __ret
            })
        }
        fn collect_all_fields<'__life>(
            &'__life self,
            ctx: &async_graphql::ContextSelectionSet<'__life>,
            fields: &mut async_graphql::resolver_utils::Fields<'__life>,
        ) -> async_graphql::ServerResult<()> {
            match self {
                MyEnum::Val(obj) => obj.collect_all_fields(ctx, fields),
            }
        }
    }
    #[allow(clippy::all, clippy::pedantic)]
    impl async_graphql::OutputTypeMarker for MyEnum {
        fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
            ::std::borrow::Cow::Borrowed("MyEnum")
        }
        fn introspection_type_name(
            &self,
        ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
            match self {
                MyEnum::Val(obj) => {
                    <IntObj as async_graphql::OutputTypeMarker>::type_name()
                }
            }
        }
        fn create_type_info(
            registry: &mut async_graphql::registry::Registry,
        ) -> ::std::string::String {
            registry
                .create_output_type::<
                    Self,
                    _,
                >(
                    async_graphql::registry::MetaTypeId::Union,
                    |registry| {
                        <IntObj as async_graphql::OutputTypeMarker>::create_type_info(
                            registry,
                        );
                        async_graphql::registry::MetaType::Union {
                            name: ::std::borrow::Cow::into_owned(
                                ::std::borrow::Cow::Borrowed("MyEnum"),
                            ),
                            description: ::std::option::Option::None,
                            possible_types: {
                                let mut possible_types = async_graphql::indexmap::IndexSet::new();
                                possible_types
                                    .insert(
                                        <IntObj as async_graphql::OutputTypeMarker>::type_name()
                                            .into_owned(),
                                    );
                                possible_types
                            },
                            visible: ::std::option::Option::None,
                            inaccessible: false,
                            tags: ::alloc::vec::Vec::new(),
                            rust_typename: ::std::option::Option::Some(
                                ::std::any::type_name::<Self>(),
                            ),
                            directive_invocations: ::std::vec::Vec::new(),
                        }
                    },
                )
        }
    }
    #[allow(clippy::all, clippy::pedantic)]
    impl async_graphql::OutputType for MyEnum {
        fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
            <Self as async_graphql::OutputTypeMarker>::type_name()
        }
        fn introspection_type_name(
            &self,
        ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
            <Self as async_graphql::OutputTypeMarker>::introspection_type_name(&self)
        }
        fn create_type_info(
            &self,
            registry: &mut async_graphql::registry::Registry,
        ) -> ::std::string::String {
            <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
        }
        #[allow(
            elided_named_lifetimes,
            clippy::async_yields_async,
            clippy::diverging_sub_expression,
            clippy::let_unit_value,
            clippy::needless_arbitrary_self_type,
            clippy::no_effect_underscore_binding,
            clippy::shadow_same,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds,
            clippy::used_underscore_binding
        )]
        fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
            &'life0 self,
            ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
            _field: &'life3 async_graphql::Positioned<
                async_graphql::parser::types::Field,
            >,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<
                    Output = async_graphql::ServerResult<async_graphql::Value>,
                > + ::core::marker::Send + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            'life2: 'async_trait,
            'life3: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                    async_graphql::ServerResult<async_graphql::Value>,
                > {
                    #[allow(unreachable_code)] return __ret;
                }
                let __self = self;
                let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                    async_graphql::resolver_utils::resolve_container(
                            ctx,
                            __self as &dyn async_graphql::ContainerType,
                        )
                        .await
                };
                #[allow(unreachable_code)] __ret
            })
        }
    }
    impl async_graphql::UnionType for MyEnum {}
    let _ = MyEnum::Val(IntObj { val: 1 });
}
extern crate test;
#[rustc_test_marker = "test_union_with_oneof_object"]
#[doc(hidden)]
pub const test_union_with_oneof_object: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("test_union_with_oneof_object"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/union.rs",
        start_line: 445usize,
        start_col: 14usize,
        end_line: 445usize,
        end_col: 42usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(test_union_with_oneof_object()),
    ),
};
pub fn test_union_with_oneof_object() {
    let body = async {
        #[graphql(input_name = "MyObjInput")]
        struct MyObj {
            id: i32,
            title: String,
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl MyObj {
            #[inline]
            #[allow(missing_docs)]
            async fn id(
                &self,
                ctx: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<&i32> {
                ::std::result::Result::Ok(&self.id)
            }
            #[inline]
            #[allow(missing_docs)]
            async fn title(
                &self,
                ctx: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<&String> {
                ::std::result::Result::Ok(&self.title)
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::resolver_utils::ContainerType for MyObj {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = {
                        if ctx.item.node.name.node == "id" {
                            let f = async move {
                                __self
                                    .id(ctx)
                                    .await
                                    .map_err(|err| err.into_server_error(ctx.item.pos))
                            };
                            let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                            let ctx_obj = ctx
                                .with_selection_set(&ctx.item.node.selection_set);
                            return async_graphql::OutputType::resolve(
                                    &obj,
                                    &ctx_obj,
                                    ctx.item,
                                )
                                .await
                                .map(::std::option::Option::Some);
                        }
                        if ctx.item.node.name.node == "title" {
                            let f = async move {
                                __self
                                    .title(ctx)
                                    .await
                                    .map_err(|err| err.into_server_error(ctx.item.pos))
                            };
                            let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                            let ctx_obj = ctx
                                .with_selection_set(&ctx.item.node.selection_set);
                            return async_graphql::OutputType::resolve(
                                    &obj,
                                    &ctx_obj,
                                    ctx.item,
                                )
                                .await
                                .map(::std::option::Option::Some);
                        }
                        ::std::result::Result::Ok(::std::option::Option::None)
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputTypeMarker for MyObj {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("MyObj")
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Object,
                        |registry| async_graphql::registry::MetaType::Object {
                            name: ::std::borrow::Cow::into_owned(
                                ::std::borrow::Cow::Borrowed("MyObj"),
                            ),
                            description: ::std::option::Option::None,
                            fields: {
                                let mut fields = async_graphql::indexmap::IndexMap::new();
                                fields
                                    .insert(
                                        ::std::borrow::ToOwned::to_owned("id"),
                                        async_graphql::registry::MetaField {
                                            name: ::std::borrow::ToOwned::to_owned("id"),
                                            description: ::std::option::Option::None,
                                            args: ::std::default::Default::default(),
                                            ty: <i32 as async_graphql::OutputTypeMarker>::create_type_info(
                                                registry,
                                            ),
                                            deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                            cache_control: async_graphql::CacheControl {
                                                public: true,
                                                max_age: 0i32,
                                            },
                                            external: false,
                                            provides: ::std::option::Option::None,
                                            requires: ::std::option::Option::None,
                                            shareable: false,
                                            inaccessible: false,
                                            tags: ::alloc::vec::Vec::new(),
                                            override_from: ::std::option::Option::None,
                                            visible: ::std::option::Option::None,
                                            compute_complexity: ::std::option::Option::None,
                                            directive_invocations: ::alloc::vec::Vec::new(),
                                            requires_scopes: ::alloc::vec::Vec::new(),
                                        },
                                    );
                                fields
                                    .insert(
                                        ::std::borrow::ToOwned::to_owned("title"),
                                        async_graphql::registry::MetaField {
                                            name: ::std::borrow::ToOwned::to_owned("title"),
                                            description: ::std::option::Option::None,
                                            args: ::std::default::Default::default(),
                                            ty: <String as async_graphql::OutputTypeMarker>::create_type_info(
                                                registry,
                                            ),
                                            deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                            cache_control: async_graphql::CacheControl {
                                                public: true,
                                                max_age: 0i32,
                                            },
                                            external: false,
                                            provides: ::std::option::Option::None,
                                            requires: ::std::option::Option::None,
                                            shareable: false,
                                            inaccessible: false,
                                            tags: ::alloc::vec::Vec::new(),
                                            override_from: ::std::option::Option::None,
                                            visible: ::std::option::Option::None,
                                            compute_complexity: ::std::option::Option::None,
                                            directive_invocations: ::alloc::vec::Vec::new(),
                                            requires_scopes: ::alloc::vec::Vec::new(),
                                        },
                                    );
                                fields
                            },
                            cache_control: async_graphql::CacheControl {
                                public: true,
                                max_age: 0i32,
                            },
                            extends: false,
                            shareable: false,
                            resolvable: true,
                            inaccessible: false,
                            interface_object: false,
                            tags: ::alloc::vec::Vec::new(),
                            keys: ::std::option::Option::None,
                            visible: ::std::option::Option::None,
                            is_subscription: false,
                            rust_typename: ::std::option::Option::Some(
                                ::std::any::type_name::<Self>(),
                            ),
                            directive_invocations: ::alloc::vec::Vec::new(),
                            requires_scopes: ::alloc::vec::Vec::new(),
                        },
                    )
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputType for MyObj {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::type_name()
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                &__self as &dyn async_graphql::resolver_utils::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::ObjectType for MyObj {}
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::InputType for MyObj {
            type RawValueType = Self;
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("MyObjInput")
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_input_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::InputObject,
                        |registry| async_graphql::registry::MetaType::InputObject {
                            name: ::std::borrow::Cow::into_owned(
                                ::std::borrow::Cow::Borrowed("MyObjInput"),
                            ),
                            description: ::std::option::Option::None,
                            input_fields: {
                                let mut fields = async_graphql::indexmap::IndexMap::new();
                                fields
                                    .insert(
                                        ::std::borrow::ToOwned::to_owned("id"),
                                        async_graphql::registry::MetaInputValue {
                                            name: ::std::string::ToString::to_string("id"),
                                            description: ::std::option::Option::None,
                                            ty: <i32 as async_graphql::InputType>::create_type_info(
                                                registry,
                                            ),
                                            deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                            default_value: ::std::option::Option::None,
                                            visible: ::std::option::Option::None,
                                            inaccessible: false,
                                            tags: ::alloc::vec::Vec::new(),
                                            is_secret: false,
                                            directive_invocations: ::alloc::vec::Vec::new(),
                                        },
                                    );
                                fields
                                    .insert(
                                        ::std::borrow::ToOwned::to_owned("title"),
                                        async_graphql::registry::MetaInputValue {
                                            name: ::std::string::ToString::to_string("title"),
                                            description: ::std::option::Option::None,
                                            ty: <String as async_graphql::InputType>::create_type_info(
                                                registry,
                                            ),
                                            deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                            default_value: ::std::option::Option::None,
                                            visible: ::std::option::Option::None,
                                            inaccessible: false,
                                            tags: ::alloc::vec::Vec::new(),
                                            is_secret: false,
                                            directive_invocations: ::alloc::vec::Vec::new(),
                                        },
                                    );
                                fields
                            },
                            visible: ::std::option::Option::None,
                            inaccessible: false,
                            tags: ::alloc::vec::Vec::new(),
                            rust_typename: ::std::option::Option::Some(
                                ::std::any::type_name::<Self>(),
                            ),
                            oneof: false,
                            directive_invocations: ::alloc::vec::Vec::new(),
                        },
                    )
            }
            fn parse(
                value: ::std::option::Option<async_graphql::Value>,
            ) -> async_graphql::InputValueResult<Self> {
                if let ::std::option::Option::Some(async_graphql::Value::Object(obj)) = value {
                    #[allow(non_snake_case, unused_mut)]
                    let mut id: i32 = async_graphql::InputType::parse(
                            obj.get("id").cloned(),
                        )
                        .map_err(async_graphql::InputValueError::propagate)?;
                    #[allow(non_snake_case, unused_mut)]
                    let mut title: String = async_graphql::InputType::parse(
                            obj.get("title").cloned(),
                        )
                        .map_err(async_graphql::InputValueError::propagate)?;
                    let obj = Self { id, title };
                    ::std::result::Result::Ok(obj)
                } else {
                    ::std::result::Result::Err(
                        async_graphql::InputValueError::expected_type(
                            value.unwrap_or_default(),
                        ),
                    )
                }
            }
            fn to_value(&self) -> async_graphql::Value {
                let mut map = async_graphql::indexmap::IndexMap::new();
                map.insert(
                    async_graphql::Name::new("id"),
                    async_graphql::InputType::to_value(&self.id),
                );
                map.insert(
                    async_graphql::Name::new("title"),
                    async_graphql::InputType::to_value(&self.title),
                );
                async_graphql::Value::Object(map)
            }
            fn federation_fields() -> ::std::option::Option<::std::string::String> {
                let mut res = ::std::vec::Vec::new();
                if let ::std::option::Option::Some(fields) = <i32 as async_graphql::InputType>::federation_fields() {
                    res.push(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(format_args!("{0} {1}", "id", fields))
                        }),
                    );
                } else {
                    res.push(::std::string::ToString::to_string("id"));
                }
                if let ::std::option::Option::Some(fields) = <String as async_graphql::InputType>::federation_fields() {
                    res.push(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!("{0} {1}", "title", fields),
                            )
                        }),
                    );
                } else {
                    res.push(::std::string::ToString::to_string("title"));
                }
                ::std::option::Option::Some(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(format_args!("{{ {0} }}", res.join(" ")))
                    }),
                )
            }
            fn as_raw_value(&self) -> ::std::option::Option<&Self::RawValueType> {
                ::std::option::Option::Some(self)
            }
        }
        impl async_graphql::InputObjectType for MyObj {}
        #[graphql(input_name = "NodeInput")]
        enum Node {
            MyObj(MyObj),
        }
        impl async_graphql::InputType for Node {
            type RawValueType = Self;
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("NodeInput")
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_input_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::InputObject,
                        |registry| async_graphql::registry::MetaType::InputObject {
                            name: ::std::borrow::Cow::into_owned(
                                ::std::borrow::Cow::Borrowed("NodeInput"),
                            ),
                            description: ::std::option::Option::None,
                            input_fields: {
                                let mut fields = async_graphql::indexmap::IndexMap::new();
                                fields
                                    .insert(
                                        ::std::borrow::ToOwned::to_owned("myObj"),
                                        async_graphql::registry::MetaInputValue {
                                            name: ::std::string::ToString::to_string("myObj"),
                                            description: ::std::option::Option::None,
                                            ty: <::std::option::Option<
                                                MyObj,
                                            > as async_graphql::InputType>::create_type_info(registry),
                                            deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                            default_value: ::std::option::Option::None,
                                            visible: ::std::option::Option::None,
                                            inaccessible: false,
                                            tags: ::alloc::vec::Vec::new(),
                                            is_secret: false,
                                            directive_invocations: ::alloc::vec::Vec::new(),
                                        },
                                    );
                                fields
                            },
                            visible: ::std::option::Option::None,
                            inaccessible: false,
                            tags: ::alloc::vec::Vec::new(),
                            rust_typename: ::std::option::Option::Some(
                                ::std::any::type_name::<Self>(),
                            ),
                            oneof: true,
                            directive_invocations: ::alloc::vec::Vec::new(),
                        },
                    )
            }
            fn parse(
                value: ::std::option::Option<async_graphql::Value>,
            ) -> async_graphql::InputValueResult<Self> {
                if let ::std::option::Option::Some(
                    async_graphql::Value::Object(mut obj),
                ) = value {
                    if obj.contains_key("myObj") && obj.len() == 1 {
                        let value = async_graphql::InputType::parse(obj.remove("myObj"))
                            .map_err(async_graphql::InputValueError::propagate)?;
                        return ::std::result::Result::Ok(Self::MyObj(value));
                    }
                    ::std::result::Result::Err(
                        async_graphql::InputValueError::expected_type(
                            async_graphql::Value::Object(obj),
                        ),
                    )
                } else {
                    ::std::result::Result::Err(
                        async_graphql::InputValueError::expected_type(
                            value.unwrap_or_default(),
                        ),
                    )
                }
            }
            fn to_value(&self) -> async_graphql::Value {
                let mut map = async_graphql::indexmap::IndexMap::new();
                match self {
                    Self::MyObj(value) => {
                        map.insert(
                            async_graphql::Name::new("myObj"),
                            async_graphql::InputType::to_value(value),
                        );
                    }
                }
                async_graphql::Value::Object(map)
            }
            fn federation_fields() -> ::std::option::Option<::std::string::String> {
                ::std::option::Option::None
            }
            fn as_raw_value(&self) -> ::std::option::Option<&Self::RawValueType> {
                ::std::option::Option::Some(self)
            }
        }
        impl async_graphql::InputObjectType for Node {}
        impl async_graphql::OneofObjectType for Node {}
        const _: () = {
            fn assert_impl() {
                let _: ::static_assertions_next::True = {
                    #[allow(unused_imports)]
                    use ::static_assertions_next::{
                        _bool::{True, False},
                        _core::{marker::PhantomData, ops::Deref},
                    };
                    trait DoesntImpl {
                        const DOES_IMPL: False = False;
                    }
                    impl<T: ?Sized> DoesntImpl for T {}
                    *{
                        struct Wrapper<T: ?Sized>(PhantomData<T>);
                        #[allow(dead_code)]
                        impl<T: ?Sized + async_graphql::ObjectType> Wrapper<T> {
                            const DOES_IMPL: True = True;
                        }
                        &<Wrapper<MyObj>>::DOES_IMPL
                    }
                };
            }
        };
        #[allow(clippy::all, clippy::pedantic)]
        impl ::std::convert::From<MyObj> for Node {
            fn from(obj: MyObj) -> Self {
                Node::MyObj(obj)
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::resolver_utils::ContainerType for Node {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = { ::std::result::Result::Ok(::std::option::Option::None) };
                    #[allow(unreachable_code)] __ret
                })
            }
            fn collect_all_fields<'__life>(
                &'__life self,
                ctx: &async_graphql::ContextSelectionSet<'__life>,
                fields: &mut async_graphql::resolver_utils::Fields<'__life>,
            ) -> async_graphql::ServerResult<()> {
                match self {
                    Node::MyObj(obj) => obj.collect_all_fields(ctx, fields),
                }
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputTypeMarker for Node {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("Node")
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                match self {
                    Node::MyObj(obj) => {
                        <MyObj as async_graphql::OutputTypeMarker>::type_name()
                    }
                }
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Union,
                        |registry| {
                            <MyObj as async_graphql::OutputTypeMarker>::create_type_info(
                                registry,
                            );
                            async_graphql::registry::MetaType::Union {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("Node"),
                                ),
                                description: ::std::option::Option::None,
                                possible_types: {
                                    let mut possible_types = async_graphql::indexmap::IndexSet::new();
                                    possible_types
                                        .insert(
                                            <MyObj as async_graphql::OutputTypeMarker>::type_name()
                                                .into_owned(),
                                        );
                                    possible_types
                                },
                                visible: ::std::option::Option::None,
                                inaccessible: false,
                                tags: ::alloc::vec::Vec::new(),
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::std::vec::Vec::new(),
                            }
                        },
                    )
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputType for Node {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::type_name()
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::introspection_type_name(&self)
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                __self as &dyn async_graphql::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::UnionType for Node {}
        struct Query;
        impl Query {
            async fn node(
                &self,
                _: &async_graphql::Context<'_>,
                input: Node,
            ) -> async_graphql::Result<Node> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: Node = { input };
                            value
                        }
                            .await,
                    )
                }
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __FieldIdent {
                node,
            }
            impl __FieldIdent {
                fn from_name(
                    __name: &async_graphql::Name,
                ) -> ::std::option::Option<__FieldIdent> {
                    match __name.as_str() {
                        "node" => ::std::option::Option::Some(__FieldIdent::node),
                        _ => ::std::option::Option::None,
                    }
                }
            }
            impl Query {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __node_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        #[allow(non_snake_case, unused_variables, unused_mut)]
                        let (__pos, mut input) = ctx
                            .param_value::<Node>("input", ::std::option::Option::None)?;
                        #[allow(non_snake_case, unused_variables)]
                        let input = input;
                        let res = self.node(ctx, input).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
            }
            #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
            #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
            impl async_graphql::resolver_utils::ContainerType for Query {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let __field = __FieldIdent::from_name(
                                &ctx.item.node.name.node,
                            );
                            match __field {
                                ::std::option::Option::Some(__FieldIdent::node) => {
                                    return __self.__node_resolver(&ctx).await;
                                }
                                None => {}
                            }
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                    params: &'life3 async_graphql::Value,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let params = match params {
                                async_graphql::Value::Object(params) => params,
                                _ => {
                                    return ::std::result::Result::Ok(
                                        ::std::option::Option::None,
                                    );
                                }
                            };
                            let typename = if let ::std::option::Option::Some(
                                async_graphql::Value::String(typename),
                            ) = params.get("__typename")
                            {
                                typename
                            } else {
                                return ::std::result::Result::Err(
                                    async_graphql::ServerError::new(
                                        r#""__typename" must be an existing string."#,
                                        ::std::option::Option::Some(ctx.item.pos),
                                    ),
                                );
                            };
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputTypeMarker for Query {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed("Query")
                }
                fn create_type_info(
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    let ty = registry
                        .create_output_type::<
                            Self,
                            _,
                        >(
                            async_graphql::registry::MetaTypeId::Object,
                            |registry| async_graphql::registry::MetaType::Object {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("Query"),
                                ),
                                description: ::std::option::Option::None,
                                fields: {
                                    let mut fields = async_graphql::indexmap::IndexMap::new();
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("node"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("node"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args.insert(
                                                        ::std::borrow::ToOwned::to_owned("input"),
                                                        async_graphql::registry::MetaInputValue {
                                                            name: ::std::string::ToString::to_string("input"),
                                                            description: ::std::option::Option::None,
                                                            ty: <Node as async_graphql::InputType>::create_type_info(
                                                                registry,
                                                            ),
                                                            deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                            default_value: ::std::option::Option::None,
                                                            visible: ::std::option::Option::None,
                                                            inaccessible: false,
                                                            tags: ::alloc::vec::Vec::new(),
                                                            is_secret: false,
                                                            directive_invocations: ::alloc::vec::Vec::new(),
                                                        },
                                                    );
                                                    args
                                                },
                                                ty: <Node as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                },
                                cache_control: async_graphql::CacheControl {
                                    public: true,
                                    max_age: 0i32,
                                },
                                extends: false,
                                shareable: false,
                                resolvable: true,
                                inaccessible: false,
                                interface_object: false,
                                tags: ::alloc::vec::Vec::new(),
                                keys: ::std::option::Option::None,
                                visible: ::std::option::Option::None,
                                is_subscription: false,
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::alloc::vec::Vec::new(),
                                requires_scopes: ::alloc::vec::Vec::new(),
                            },
                        );
                    ty
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputType for Query {
                fn type_name(
                    &self,
                ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    <Self as async_graphql::OutputTypeMarker>::type_name()
                }
                fn create_type_info(
                    &self,
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                    _field: &'life3 async_graphql::Positioned<
                        async_graphql::parser::types::Field,
                    >,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<async_graphql::Value>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<async_graphql::Value>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                            async_graphql::resolver_utils::resolve_container(
                                    ctx,
                                    &__self as &dyn async_graphql::resolver_utils::ContainerType,
                                )
                                .await
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            impl async_graphql::ObjectType for Query {}
        };
        let query = r#"{
            node(input: { myObj: { id: 10, title: "abc" } }) {
                ... on MyObj {
                    id title
                }
            }
        }"#;
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        match (
            &schema.execute(query).await.into_result().unwrap().data,
            &::async_graphql_value::ConstValue::Object({
                let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                let _ = object
                    .insert(
                        ::async_graphql_value::Name::new("node"),
                        ::async_graphql_value::ConstValue::Object({
                            let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                            let _ = object
                                .insert(
                                    ::async_graphql_value::Name::new("id"),
                                    ::async_graphql_value::to_value(&10).unwrap(),
                                );
                            let _ = object
                                .insert(
                                    ::async_graphql_value::Name::new("title"),
                                    ::async_graphql_value::to_value(&"abc").unwrap(),
                                );
                            object
                        }),
                    );
                object
            }),
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    };
    let mut body = body;
    #[allow(unused_mut)]
    let mut body = unsafe { ::tokio::macros::support::Pin::new_unchecked(&mut body) };
    let body: ::core::pin::Pin<&mut dyn ::core::future::Future<Output = ()>> = body;
    #[allow(
        clippy::expect_used,
        clippy::diverging_sub_expression,
        clippy::needless_return
    )]
    {
        return tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
extern crate test;
#[rustc_test_marker = "test_union_with_generic"]
#[doc(hidden)]
pub const test_union_with_generic: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("test_union_with_generic"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/union.rs",
        start_line: 488usize,
        start_col: 14usize,
        end_line: 488usize,
        end_col: 37usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(test_union_with_generic()),
    ),
};
pub fn test_union_with_generic() {
    let body = async {
        struct MyObj<T> {
            value: T,
        }
        impl<T> MyObj<T>
        where
            T: Send + Sync + async_graphql::OutputType + async_graphql::OutputTypeMarker,
            MyObj<T>: async_graphql::OutputTypeMarker,
        {
            async fn id(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<i32> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: i32 = { 10 };
                            value
                        }
                            .await,
                    )
                }
            }
            async fn title(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<String> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: String = { "abc".to_string() };
                            value
                        }
                            .await,
                    )
                }
            }
            async fn value(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<&T> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: &T = { &self.value };
                            value
                        }
                            .await,
                    )
                }
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __FieldIdent {
                id,
                title,
                value,
            }
            impl __FieldIdent {
                fn from_name(
                    __name: &async_graphql::Name,
                ) -> ::std::option::Option<__FieldIdent> {
                    match __name.as_str() {
                        "id" => ::std::option::Option::Some(__FieldIdent::id),
                        "title" => ::std::option::Option::Some(__FieldIdent::title),
                        "value" => ::std::option::Option::Some(__FieldIdent::value),
                        _ => ::std::option::Option::None,
                    }
                }
            }
            impl<T> MyObj<T>
            where
                T: Send + Sync + async_graphql::OutputType
                    + async_graphql::OutputTypeMarker,
                MyObj<T>: async_graphql::OutputTypeMarker,
            {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __id_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.id(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __title_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.title(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __value_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.value(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
                fn __internal_create_type_info(
                    registry: &mut async_graphql::registry::Registry,
                    name: &str,
                ) -> ::std::string::String
                where
                    Self: async_graphql::OutputType,
                {
                    let ty = registry
                        .create_output_type::<
                            Self,
                            _,
                        >(
                            async_graphql::registry::MetaTypeId::Object,
                            |registry| async_graphql::registry::MetaType::Object {
                                name: ::std::borrow::ToOwned::to_owned(name),
                                description: ::std::option::Option::None,
                                fields: {
                                    let mut fields = async_graphql::indexmap::IndexMap::new();
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("id"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("id"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <i32 as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("title"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("title"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <String as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("value"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("value"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <&T as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                },
                                cache_control: async_graphql::CacheControl {
                                    public: true,
                                    max_age: 0i32,
                                },
                                extends: false,
                                shareable: false,
                                resolvable: true,
                                inaccessible: false,
                                interface_object: false,
                                tags: ::alloc::vec::Vec::new(),
                                keys: ::std::option::Option::None,
                                visible: ::std::option::Option::None,
                                is_subscription: false,
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::alloc::vec::Vec::new(),
                                requires_scopes: ::alloc::vec::Vec::new(),
                            },
                        );
                    ty
                }
                async fn __internal_resolve_field(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                >
                where
                    Self: async_graphql::ContainerType,
                {
                    let __field = __FieldIdent::from_name(&ctx.item.node.name.node);
                    match __field {
                        ::std::option::Option::Some(__FieldIdent::id) => {
                            return self.__id_resolver(&ctx).await;
                        }
                        ::std::option::Option::Some(__FieldIdent::title) => {
                            return self.__title_resolver(&ctx).await;
                        }
                        ::std::option::Option::Some(__FieldIdent::value) => {
                            return self.__value_resolver(&ctx).await;
                        }
                        None => {}
                    }
                    ::std::result::Result::Ok(::std::option::Option::None)
                }
                async fn __internal_find_entity(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                    params: &async_graphql::Value,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let params = match params {
                        async_graphql::Value::Object(params) => params,
                        _ => {
                            return ::std::result::Result::Ok(::std::option::Option::None);
                        }
                    };
                    let typename = if let ::std::option::Option::Some(
                        async_graphql::Value::String(typename),
                    ) = params.get("__typename")
                    {
                        typename
                    } else {
                        return ::std::result::Result::Err(
                            async_graphql::ServerError::new(
                                r#""__typename" must be an existing string."#,
                                ::std::option::Option::Some(ctx.item.pos),
                            ),
                        );
                    };
                    ::std::result::Result::Ok(::std::option::Option::None)
                }
            }
        };
        impl async_graphql::resolver_utils::ContainerType for MyObj<String> {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = { __self.__internal_resolve_field(ctx).await };
                    #[allow(unreachable_code)] __ret
                })
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
                params: &'life3 async_graphql::Value,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = { __self.__internal_find_entity(ctx, params).await };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::OutputTypeMarker for MyObj<String> {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("MyObjString")
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                Self::__internal_create_type_info(registry, "MyObjString")
            }
        }
        impl async_graphql::OutputType for MyObj<String> {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::type_name()
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                &__self as &dyn async_graphql::resolver_utils::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::ObjectType for MyObj<String> {}
        impl async_graphql::resolver_utils::ContainerType for MyObj<i64> {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = { __self.__internal_resolve_field(ctx).await };
                    #[allow(unreachable_code)] __ret
                })
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
                params: &'life3 async_graphql::Value,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = { __self.__internal_find_entity(ctx, params).await };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::OutputTypeMarker for MyObj<i64> {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("MyObjInt")
            }
            fn create_type_info(
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                Self::__internal_create_type_info(registry, "MyObjInt")
            }
        }
        impl async_graphql::OutputType for MyObj<i64> {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                <Self as async_graphql::OutputTypeMarker>::type_name()
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                &__self as &dyn async_graphql::resolver_utils::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::ObjectType for MyObj<i64> {}
        #[graphql(concrete(name = "NodeInt", params(i64)))]
        #[graphql(concrete(name = "NodeString", params(String)))]
        enum Node<T>
        where
            T: Send + Sync + async_graphql::OutputType + async_graphql::OutputTypeMarker,
            Node<T>: async_graphql::OutputTypeMarker,
            MyObj<T>: async_graphql::OutputTypeMarker,
        {
            MyObj(MyObj<T>),
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::resolver_utils::ContainerType for Node<i64> {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = { ::std::result::Result::Ok(::std::option::Option::None) };
                    #[allow(unreachable_code)] __ret
                })
            }
            fn collect_all_fields<'__life>(
                &'__life self,
                ctx: &async_graphql::ContextSelectionSet<'__life>,
                fields: &mut async_graphql::resolver_utils::Fields<'__life>,
            ) -> async_graphql::ServerResult<()> {
                match self {
                    Node::MyObj(obj) => obj.collect_all_fields(ctx, fields),
                }
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputType for Node<i64> {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("NodeInt")
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                match self {
                    Node::MyObj(obj) => {
                        <MyObj<i64> as async_graphql::OutputTypeMarker>::type_name()
                    }
                }
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Union,
                        |registry| {
                            <MyObj<
                                i64,
                            > as async_graphql::OutputTypeMarker>::create_type_info(
                                registry,
                            );
                            async_graphql::registry::MetaType::Union {
                                name: ::std::borrow::ToOwned::to_owned("NodeInt"),
                                description: ::std::option::Option::None,
                                possible_types: {
                                    let mut possible_types = async_graphql::indexmap::IndexSet::new();
                                    possible_types
                                        .insert(
                                            <MyObj<i64> as async_graphql::OutputTypeMarker>::type_name()
                                                .into_owned(),
                                        );
                                    possible_types
                                },
                                visible: ::std::option::Option::None,
                                inaccessible: false,
                                tags: ::alloc::vec::Vec::new(),
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::std::vec::Vec::new(),
                            }
                        },
                    )
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                &__self as &dyn async_graphql::resolver_utils::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::ObjectType for Node<i64> {}
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::resolver_utils::ContainerType for Node<String> {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = { ::std::result::Result::Ok(::std::option::Option::None) };
                    #[allow(unreachable_code)] __ret
                })
            }
            fn collect_all_fields<'__life>(
                &'__life self,
                ctx: &async_graphql::ContextSelectionSet<'__life>,
                fields: &mut async_graphql::resolver_utils::Fields<'__life>,
            ) -> async_graphql::ServerResult<()> {
                match self {
                    Node::MyObj(obj) => obj.collect_all_fields(ctx, fields),
                }
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl async_graphql::OutputType for Node<String> {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("NodeString")
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                match self {
                    Node::MyObj(obj) => {
                        <MyObj<String> as async_graphql::OutputTypeMarker>::type_name()
                    }
                }
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Union,
                        |registry| {
                            <MyObj<
                                String,
                            > as async_graphql::OutputTypeMarker>::create_type_info(
                                registry,
                            );
                            async_graphql::registry::MetaType::Union {
                                name: ::std::borrow::ToOwned::to_owned("NodeString"),
                                description: ::std::option::Option::None,
                                possible_types: {
                                    let mut possible_types = async_graphql::indexmap::IndexSet::new();
                                    possible_types
                                        .insert(
                                            <MyObj<
                                                String,
                                            > as async_graphql::OutputTypeMarker>::type_name()
                                                .into_owned(),
                                        );
                                    possible_types
                                },
                                visible: ::std::option::Option::None,
                                inaccessible: false,
                                tags: ::alloc::vec::Vec::new(),
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::std::vec::Vec::new(),
                            }
                        },
                    )
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                &__self as &dyn async_graphql::resolver_utils::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl async_graphql::ObjectType for Node<String> {}
        struct Query;
        impl Query {
            async fn node_int(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<Node<i64>> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: Node<i64> = { Node::MyObj(MyObj { value: 10 }) };
                            value
                        }
                            .await,
                    )
                }
            }
            async fn node_str(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<Node<String>> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: Node<String> = {
                                Node::MyObj(MyObj { value: "abc".to_string() })
                            };
                            value
                        }
                            .await,
                    )
                }
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __FieldIdent {
                node_int,
                node_str,
            }
            impl __FieldIdent {
                fn from_name(
                    __name: &async_graphql::Name,
                ) -> ::std::option::Option<__FieldIdent> {
                    match __name.as_str() {
                        "nodeInt" => ::std::option::Option::Some(__FieldIdent::node_int),
                        "nodeStr" => ::std::option::Option::Some(__FieldIdent::node_str),
                        _ => ::std::option::Option::None,
                    }
                }
            }
            impl Query {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __node_int_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.node_int(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __node_str_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.node_str(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
            }
            #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
            #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
            impl async_graphql::resolver_utils::ContainerType for Query {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let __field = __FieldIdent::from_name(
                                &ctx.item.node.name.node,
                            );
                            match __field {
                                ::std::option::Option::Some(__FieldIdent::node_int) => {
                                    return __self.__node_int_resolver(&ctx).await;
                                }
                                ::std::option::Option::Some(__FieldIdent::node_str) => {
                                    return __self.__node_str_resolver(&ctx).await;
                                }
                                None => {}
                            }
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                    params: &'life3 async_graphql::Value,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let params = match params {
                                async_graphql::Value::Object(params) => params,
                                _ => {
                                    return ::std::result::Result::Ok(
                                        ::std::option::Option::None,
                                    );
                                }
                            };
                            let typename = if let ::std::option::Option::Some(
                                async_graphql::Value::String(typename),
                            ) = params.get("__typename")
                            {
                                typename
                            } else {
                                return ::std::result::Result::Err(
                                    async_graphql::ServerError::new(
                                        r#""__typename" must be an existing string."#,
                                        ::std::option::Option::Some(ctx.item.pos),
                                    ),
                                );
                            };
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputTypeMarker for Query {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed("Query")
                }
                fn create_type_info(
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    let ty = registry
                        .create_output_type::<
                            Self,
                            _,
                        >(
                            async_graphql::registry::MetaTypeId::Object,
                            |registry| async_graphql::registry::MetaType::Object {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("Query"),
                                ),
                                description: ::std::option::Option::None,
                                fields: {
                                    let mut fields = async_graphql::indexmap::IndexMap::new();
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("nodeInt"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("nodeInt"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <Node<
                                                    i64,
                                                > as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("nodeStr"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("nodeStr"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <Node<
                                                    String,
                                                > as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                },
                                cache_control: async_graphql::CacheControl {
                                    public: true,
                                    max_age: 0i32,
                                },
                                extends: false,
                                shareable: false,
                                resolvable: true,
                                inaccessible: false,
                                interface_object: false,
                                tags: ::alloc::vec::Vec::new(),
                                keys: ::std::option::Option::None,
                                visible: ::std::option::Option::None,
                                is_subscription: false,
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::alloc::vec::Vec::new(),
                                requires_scopes: ::alloc::vec::Vec::new(),
                            },
                        );
                    ty
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputType for Query {
                fn type_name(
                    &self,
                ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    <Self as async_graphql::OutputTypeMarker>::type_name()
                }
                fn create_type_info(
                    &self,
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                    _field: &'life3 async_graphql::Positioned<
                        async_graphql::parser::types::Field,
                    >,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<async_graphql::Value>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<async_graphql::Value>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                            async_graphql::resolver_utils::resolve_container(
                                    ctx,
                                    &__self as &dyn async_graphql::resolver_utils::ContainerType,
                                )
                                .await
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            impl async_graphql::ObjectType for Query {}
        };
        let query = r#"{
            nodeInt {
                ... on MyObjInt {
                    value
                }
            }
            nodeStr {
                ... on MyObjString {
                    value
                }
            }
        }"#;
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        match (
            &schema.execute(query).await.into_result().unwrap().data,
            &::async_graphql_value::ConstValue::Object({
                let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                let _ = object
                    .insert(
                        ::async_graphql_value::Name::new("nodeInt"),
                        ::async_graphql_value::ConstValue::Object({
                            let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                            let _ = object
                                .insert(
                                    ::async_graphql_value::Name::new("value"),
                                    ::async_graphql_value::to_value(&10).unwrap(),
                                );
                            object
                        }),
                    );
                let _ = object
                    .insert(
                        ::async_graphql_value::Name::new("nodeStr"),
                        ::async_graphql_value::ConstValue::Object({
                            let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                            let _ = object
                                .insert(
                                    ::async_graphql_value::Name::new("value"),
                                    ::async_graphql_value::to_value(&"abc").unwrap(),
                                );
                            object
                        }),
                    );
                object
            }),
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
        {
            ::std::io::_print(format_args!("{0}\n", schema.sdl()));
        };
    };
    let mut body = body;
    #[allow(unused_mut)]
    let mut body = unsafe { ::tokio::macros::support::Pin::new_unchecked(&mut body) };
    let body: ::core::pin::Pin<&mut dyn ::core::future::Future<Output = ()>> = body;
    #[allow(
        clippy::expect_used,
        clippy::diverging_sub_expression,
        clippy::needless_return
    )]
    {
        return tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
extern crate test;
#[rustc_test_marker = "test_union_with_sub_generic"]
#[doc(hidden)]
pub const test_union_with_sub_generic: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("test_union_with_sub_generic"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/union.rs",
        start_line: 571usize,
        start_col: 14usize,
        end_line: 571usize,
        end_col: 41usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(test_union_with_sub_generic()),
    ),
};
pub fn test_union_with_sub_generic() {
    let body = async {
        struct MyObj<G> {
            _marker: std::marker::PhantomData<G>,
        }
        impl<G: Send + Sync> MyObj<G> {
            async fn id(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<i32> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: i32 = { 10 };
                            value
                        }
                            .await,
                    )
                }
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __FieldIdent {
                id,
            }
            impl __FieldIdent {
                fn from_name(
                    __name: &async_graphql::Name,
                ) -> ::std::option::Option<__FieldIdent> {
                    match __name.as_str() {
                        "id" => ::std::option::Option::Some(__FieldIdent::id),
                        _ => ::std::option::Option::None,
                    }
                }
            }
            impl<G: Send + Sync> MyObj<G> {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __id_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.id(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
            }
            #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
            #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
            impl<G: Send + Sync> async_graphql::resolver_utils::ContainerType
            for MyObj<G> {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let __field = __FieldIdent::from_name(
                                &ctx.item.node.name.node,
                            );
                            match __field {
                                ::std::option::Option::Some(__FieldIdent::id) => {
                                    return __self.__id_resolver(&ctx).await;
                                }
                                None => {}
                            }
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                    params: &'life3 async_graphql::Value,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let params = match params {
                                async_graphql::Value::Object(params) => params,
                                _ => {
                                    return ::std::result::Result::Ok(
                                        ::std::option::Option::None,
                                    );
                                }
                            };
                            let typename = if let ::std::option::Option::Some(
                                async_graphql::Value::String(typename),
                            ) = params.get("__typename")
                            {
                                typename
                            } else {
                                return ::std::result::Result::Err(
                                    async_graphql::ServerError::new(
                                        r#""__typename" must be an existing string."#,
                                        ::std::option::Option::Some(ctx.item.pos),
                                    ),
                                );
                            };
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl<G: Send + Sync> async_graphql::OutputTypeMarker for MyObj<G> {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed("MyObj")
                }
                fn create_type_info(
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    let ty = registry
                        .create_output_type::<
                            Self,
                            _,
                        >(
                            async_graphql::registry::MetaTypeId::Object,
                            |registry| async_graphql::registry::MetaType::Object {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("MyObj"),
                                ),
                                description: ::std::option::Option::None,
                                fields: {
                                    let mut fields = async_graphql::indexmap::IndexMap::new();
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("id"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("id"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <i32 as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                },
                                cache_control: async_graphql::CacheControl {
                                    public: true,
                                    max_age: 0i32,
                                },
                                extends: false,
                                shareable: false,
                                resolvable: true,
                                inaccessible: false,
                                interface_object: false,
                                tags: ::alloc::vec::Vec::new(),
                                keys: ::std::option::Option::None,
                                visible: ::std::option::Option::None,
                                is_subscription: false,
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::alloc::vec::Vec::new(),
                                requires_scopes: ::alloc::vec::Vec::new(),
                            },
                        );
                    ty
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl<G: Send + Sync> async_graphql::OutputType for MyObj<G> {
                fn type_name(
                    &self,
                ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    <Self as async_graphql::OutputTypeMarker>::type_name()
                }
                fn create_type_info(
                    &self,
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                    _field: &'life3 async_graphql::Positioned<
                        async_graphql::parser::types::Field,
                    >,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<async_graphql::Value>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<async_graphql::Value>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                            async_graphql::resolver_utils::resolve_container(
                                    ctx,
                                    &__self as &dyn async_graphql::resolver_utils::ContainerType,
                                )
                                .await
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            impl<G: Send + Sync> async_graphql::ObjectType for MyObj<G> {}
        };
        struct MyObj2<G> {
            _marker: std::marker::PhantomData<G>,
        }
        impl<G: Send + Sync> MyObj2<G> {
            async fn id(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<i32> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: i32 = { 10 };
                            value
                        }
                            .await,
                    )
                }
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __FieldIdent {
                id,
            }
            impl __FieldIdent {
                fn from_name(
                    __name: &async_graphql::Name,
                ) -> ::std::option::Option<__FieldIdent> {
                    match __name.as_str() {
                        "id" => ::std::option::Option::Some(__FieldIdent::id),
                        _ => ::std::option::Option::None,
                    }
                }
            }
            impl<G: Send + Sync> MyObj2<G> {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __id_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.id(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
            }
            #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
            #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
            impl<G: Send + Sync> async_graphql::resolver_utils::ContainerType
            for MyObj2<G> {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let __field = __FieldIdent::from_name(
                                &ctx.item.node.name.node,
                            );
                            match __field {
                                ::std::option::Option::Some(__FieldIdent::id) => {
                                    return __self.__id_resolver(&ctx).await;
                                }
                                None => {}
                            }
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                    params: &'life3 async_graphql::Value,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let params = match params {
                                async_graphql::Value::Object(params) => params,
                                _ => {
                                    return ::std::result::Result::Ok(
                                        ::std::option::Option::None,
                                    );
                                }
                            };
                            let typename = if let ::std::option::Option::Some(
                                async_graphql::Value::String(typename),
                            ) = params.get("__typename")
                            {
                                typename
                            } else {
                                return ::std::result::Result::Err(
                                    async_graphql::ServerError::new(
                                        r#""__typename" must be an existing string."#,
                                        ::std::option::Option::Some(ctx.item.pos),
                                    ),
                                );
                            };
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl<G: Send + Sync> async_graphql::OutputTypeMarker for MyObj2<G> {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed("MyObj2")
                }
                fn create_type_info(
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    let ty = registry
                        .create_output_type::<
                            Self,
                            _,
                        >(
                            async_graphql::registry::MetaTypeId::Object,
                            |registry| async_graphql::registry::MetaType::Object {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("MyObj2"),
                                ),
                                description: ::std::option::Option::None,
                                fields: {
                                    let mut fields = async_graphql::indexmap::IndexMap::new();
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("id"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("id"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <i32 as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                },
                                cache_control: async_graphql::CacheControl {
                                    public: true,
                                    max_age: 0i32,
                                },
                                extends: false,
                                shareable: false,
                                resolvable: true,
                                inaccessible: false,
                                interface_object: false,
                                tags: ::alloc::vec::Vec::new(),
                                keys: ::std::option::Option::None,
                                visible: ::std::option::Option::None,
                                is_subscription: false,
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::alloc::vec::Vec::new(),
                                requires_scopes: ::alloc::vec::Vec::new(),
                            },
                        );
                    ty
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl<G: Send + Sync> async_graphql::OutputType for MyObj2<G> {
                fn type_name(
                    &self,
                ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    <Self as async_graphql::OutputTypeMarker>::type_name()
                }
                fn create_type_info(
                    &self,
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                    _field: &'life3 async_graphql::Positioned<
                        async_graphql::parser::types::Field,
                    >,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<async_graphql::Value>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<async_graphql::Value>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                            async_graphql::resolver_utils::resolve_container(
                                    ctx,
                                    &__self as &dyn async_graphql::resolver_utils::ContainerType,
                                )
                                .await
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            impl<G: Send + Sync> async_graphql::ObjectType for MyObj2<G> {}
        };
        #[graphql(
            concrete(name = "NodeMyObj", params("MyObj<G>"), bounds("G: Send + Sync"))
        )]
        enum Node<T>
        where
            T: Send + Sync + async_graphql::OutputType + async_graphql::OutputTypeMarker,
        {
            Nested(MyObj2<T>),
            NotNested(T),
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl<G: Send + Sync> async_graphql::resolver_utils::ContainerType
        for Node<MyObj<G>> {
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::Context<'life2>,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        >,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    > = { ::std::result::Result::Ok(::std::option::Option::None) };
                    #[allow(unreachable_code)] __ret
                })
            }
            fn collect_all_fields<'__life>(
                &'__life self,
                ctx: &async_graphql::ContextSelectionSet<'__life>,
                fields: &mut async_graphql::resolver_utils::Fields<'__life>,
            ) -> async_graphql::ServerResult<()> {
                match self {
                    Node::Nested(obj) => obj.collect_all_fields(ctx, fields),
                    Node::NotNested(obj) => obj.collect_all_fields(ctx, fields),
                }
            }
        }
        #[allow(clippy::all, clippy::pedantic)]
        impl<G: Send + Sync> async_graphql::OutputType for Node<MyObj<G>> {
            fn type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed("NodeMyObj")
            }
            fn introspection_type_name(
                &self,
            ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                match self {
                    Node::Nested(obj) => {
                        <MyObj2<
                            MyObj<G>,
                        > as async_graphql::OutputTypeMarker>::type_name()
                    }
                    Node::NotNested(obj) => {
                        <MyObj<G> as async_graphql::OutputTypeMarker>::type_name()
                    }
                }
            }
            fn create_type_info(
                &self,
                registry: &mut async_graphql::registry::Registry,
            ) -> ::std::string::String {
                registry
                    .create_output_type::<
                        Self,
                        _,
                    >(
                        async_graphql::registry::MetaTypeId::Union,
                        |registry| {
                            <MyObj2<
                                MyObj<G>,
                            > as async_graphql::OutputTypeMarker>::create_type_info(
                                registry,
                            );
                            <MyObj<
                                G,
                            > as async_graphql::OutputTypeMarker>::create_type_info(
                                registry,
                            );
                            async_graphql::registry::MetaType::Union {
                                name: ::std::borrow::ToOwned::to_owned("NodeMyObj"),
                                description: ::std::option::Option::None,
                                possible_types: {
                                    let mut possible_types = async_graphql::indexmap::IndexSet::new();
                                    possible_types
                                        .insert(
                                            <MyObj2<
                                                MyObj<G>,
                                            > as async_graphql::OutputTypeMarker>::type_name()
                                                .into_owned(),
                                        );
                                    possible_types
                                        .insert(
                                            <MyObj<G> as async_graphql::OutputTypeMarker>::type_name()
                                                .into_owned(),
                                        );
                                    possible_types
                                },
                                visible: ::std::option::Option::None,
                                inaccessible: false,
                                tags: ::alloc::vec::Vec::new(),
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::std::vec::Vec::new(),
                            }
                        },
                    )
            }
            #[allow(
                elided_named_lifetimes,
                clippy::async_yields_async,
                clippy::diverging_sub_expression,
                clippy::let_unit_value,
                clippy::needless_arbitrary_self_type,
                clippy::no_effect_underscore_binding,
                clippy::shadow_same,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds,
                clippy::used_underscore_binding
            )]
            fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                &'life0 self,
                ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                _field: &'life3 async_graphql::Positioned<
                    async_graphql::parser::types::Field,
                >,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = async_graphql::ServerResult<async_graphql::Value>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                'life2: 'async_trait,
                'life3: 'async_trait,
                Self: 'async_trait,
            {
                Box::pin(async move {
                    if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                        async_graphql::ServerResult<async_graphql::Value>,
                    > {
                        #[allow(unreachable_code)] return __ret;
                    }
                    let __self = self;
                    let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                        async_graphql::resolver_utils::resolve_container(
                                ctx,
                                &__self as &dyn async_graphql::resolver_utils::ContainerType,
                            )
                            .await
                    };
                    #[allow(unreachable_code)] __ret
                })
            }
        }
        impl<G: Send + Sync> async_graphql::ObjectType for Node<MyObj<G>> {}
        struct Query;
        impl Query {
            async fn nested(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<Node<MyObj<()>>> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: Node<MyObj<()>> = {
                                Node::Nested(MyObj2 {
                                    _marker: std::marker::PhantomData,
                                })
                            };
                            value
                        }
                            .await,
                    )
                }
            }
            async fn not_nested(
                &self,
                _: &async_graphql::Context<'_>,
            ) -> async_graphql::Result<Node<MyObj<()>>> {
                {
                    ::std::result::Result::Ok(
                        async move {
                            let value: Node<MyObj<()>> = {
                                Node::NotNested(MyObj {
                                    _marker: std::marker::PhantomData,
                                })
                            };
                            value
                        }
                            .await,
                    )
                }
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __FieldIdent {
                nested,
                not_nested,
            }
            impl __FieldIdent {
                fn from_name(
                    __name: &async_graphql::Name,
                ) -> ::std::option::Option<__FieldIdent> {
                    match __name.as_str() {
                        "nested" => ::std::option::Option::Some(__FieldIdent::nested),
                        "notNested" => {
                            ::std::option::Option::Some(__FieldIdent::not_nested)
                        }
                        _ => ::std::option::Option::None,
                    }
                }
            }
            impl Query {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __nested_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.nested(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                async fn __not_nested_resolver(
                    &self,
                    ctx: &async_graphql::Context<'_>,
                ) -> async_graphql::ServerResult<
                    ::std::option::Option<async_graphql::Value>,
                > {
                    let f = async {
                        let res = self.not_nested(ctx).await;
                        res.map_err(|err| {
                            ::std::convert::Into::<async_graphql::Error>::into(err)
                                .into_server_error(ctx.item.pos)
                        })
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
            }
            #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
            #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
            impl async_graphql::resolver_utils::ContainerType for Query {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let __field = __FieldIdent::from_name(
                                &ctx.item.node.name.node,
                            );
                            match __field {
                                ::std::option::Option::Some(__FieldIdent::nested) => {
                                    return __self.__nested_resolver(&ctx).await;
                                }
                                ::std::option::Option::Some(__FieldIdent::not_nested) => {
                                    return __self.__not_nested_resolver(&ctx).await;
                                }
                                None => {}
                            }
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::Context<'life2>,
                    params: &'life3 async_graphql::Value,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<
                                ::std::option::Option<async_graphql::Value>,
                            >,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<
                            ::std::option::Option<async_graphql::Value>,
                        > = {
                            let params = match params {
                                async_graphql::Value::Object(params) => params,
                                _ => {
                                    return ::std::result::Result::Ok(
                                        ::std::option::Option::None,
                                    );
                                }
                            };
                            let typename = if let ::std::option::Option::Some(
                                async_graphql::Value::String(typename),
                            ) = params.get("__typename")
                            {
                                typename
                            } else {
                                return ::std::result::Result::Err(
                                    async_graphql::ServerError::new(
                                        r#""__typename" must be an existing string."#,
                                        ::std::option::Option::Some(ctx.item.pos),
                                    ),
                                );
                            };
                            ::std::result::Result::Ok(::std::option::Option::None)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputTypeMarker for Query {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed("Query")
                }
                fn create_type_info(
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    let ty = registry
                        .create_output_type::<
                            Self,
                            _,
                        >(
                            async_graphql::registry::MetaTypeId::Object,
                            |registry| async_graphql::registry::MetaType::Object {
                                name: ::std::borrow::Cow::into_owned(
                                    ::std::borrow::Cow::Borrowed("Query"),
                                ),
                                description: ::std::option::Option::None,
                                fields: {
                                    let mut fields = async_graphql::indexmap::IndexMap::new();
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("nested"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("nested"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <Node<
                                                    MyObj<()>,
                                                > as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                        .insert(
                                            ::std::borrow::ToOwned::to_owned("notNested"),
                                            async_graphql::registry::MetaField {
                                                name: ::std::borrow::ToOwned::to_owned("notNested"),
                                                description: ::std::option::Option::None,
                                                args: {
                                                    let mut args = async_graphql::indexmap::IndexMap::new();
                                                    args
                                                },
                                                ty: <Node<
                                                    MyObj<()>,
                                                > as async_graphql::OutputTypeMarker>::create_type_info(
                                                    registry,
                                                ),
                                                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                                                cache_control: async_graphql::CacheControl {
                                                    public: true,
                                                    max_age: 0i32,
                                                },
                                                external: false,
                                                provides: ::std::option::Option::None,
                                                requires: ::std::option::Option::None,
                                                shareable: false,
                                                inaccessible: false,
                                                tags: ::alloc::vec::Vec::new(),
                                                override_from: ::std::option::Option::None,
                                                visible: ::std::option::Option::None,
                                                compute_complexity: ::std::option::Option::None,
                                                directive_invocations: ::alloc::vec::Vec::new(),
                                                requires_scopes: ::alloc::vec::Vec::new(),
                                            },
                                        );
                                    fields
                                },
                                cache_control: async_graphql::CacheControl {
                                    public: true,
                                    max_age: 0i32,
                                },
                                extends: false,
                                shareable: false,
                                resolvable: true,
                                inaccessible: false,
                                interface_object: false,
                                tags: ::alloc::vec::Vec::new(),
                                keys: ::std::option::Option::None,
                                visible: ::std::option::Option::None,
                                is_subscription: false,
                                rust_typename: ::std::option::Option::Some(
                                    ::std::any::type_name::<Self>(),
                                ),
                                directive_invocations: ::alloc::vec::Vec::new(),
                                requires_scopes: ::alloc::vec::Vec::new(),
                            },
                        );
                    ty
                }
            }
            #[allow(clippy::all, clippy::pedantic)]
            impl async_graphql::OutputType for Query {
                fn type_name(
                    &self,
                ) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    <Self as async_graphql::OutputTypeMarker>::type_name()
                }
                fn create_type_info(
                    &self,
                    registry: &mut async_graphql::registry::Registry,
                ) -> ::std::string::String {
                    <Self as async_graphql::OutputTypeMarker>::create_type_info(registry)
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
                    &'life0 self,
                    ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
                    _field: &'life3 async_graphql::Positioned<
                        async_graphql::parser::types::Field,
                    >,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = async_graphql::ServerResult<async_graphql::Value>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    'life3: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            async_graphql::ServerResult<async_graphql::Value>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: async_graphql::ServerResult<async_graphql::Value> = {
                            async_graphql::resolver_utils::resolve_container(
                                    ctx,
                                    &__self as &dyn async_graphql::resolver_utils::ContainerType,
                                )
                                .await
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            impl async_graphql::ObjectType for Query {}
        };
        let query = r#"{
            nested {
                ... on MyObj {
                    id
                }
                ... on MyObj2 {
                    id
                }
            }
        }"#;
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        match (
            &schema.execute(query).await.into_result().unwrap().data,
            &::async_graphql_value::ConstValue::Object({
                let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                let _ = object
                    .insert(
                        ::async_graphql_value::Name::new("nested"),
                        ::async_graphql_value::ConstValue::Object({
                            let mut object = ::async_graphql_value::indexmap::IndexMap::new();
                            let _ = object
                                .insert(
                                    ::async_graphql_value::Name::new("id"),
                                    ::async_graphql_value::to_value(&10).unwrap(),
                                );
                            object
                        }),
                    );
                object
            }),
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
        {
            ::std::io::_print(format_args!("{0}\n", schema.sdl()));
        };
    };
    let mut body = body;
    #[allow(unused_mut)]
    let mut body = unsafe { ::tokio::macros::support::Pin::new_unchecked(&mut body) };
    let body: ::core::pin::Pin<&mut dyn ::core::future::Future<Output = ()>> = body;
    #[allow(
        clippy::expect_used,
        clippy::diverging_sub_expression,
        clippy::needless_return
    )]
    {
        return tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
#[rustc_main]
#[coverage(off)]
#[doc(hidden)]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(
        &[
            &test_macro_generated_union,
            &test_multiple_objects_in_multiple_unions,
            &test_multiple_unions,
            &test_trait_object_in_union,
            &test_union_field_result,
            &test_union_flatten,
            &test_union_simple_object,
            &test_union_simple_object2,
            &test_union_with_generic,
            &test_union_with_oneof_object,
            &test_union_with_sub_generic,
        ],
    )
}
