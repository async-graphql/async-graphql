#![allow(dead_code)]

#[tokio::test]
pub async fn test_object() {
    macro_rules! test_data {
        ($test_name:ident) => {
            #[derive(Debug, Clone)]
            pub struct $test_name {
                value: i32,
            }

            #[async_graphql::Object]
            impl $test_name {
                async fn value(&self) -> i32 {
                    self.value
                }
            }
        };
    }

    test_data!(A);
}

#[tokio::test]
pub async fn test_object_with_context_arg() {
    use async_graphql::{Context, EmptyMutation, EmptySubscription, Result, Schema};

    macro_rules! test_data {
        ($test_name:ident { $($arg:ident: $arg_ty:ty),* $(,)? } -> $ret_ty:ty) => {
            #[derive(Debug, Clone)]
            pub struct $test_name;

            #[async_graphql::Object]
            impl $test_name {
                async fn echo(&self, $($arg: $arg_ty),*) -> $ret_ty {
                    echo($($arg),*)
                }
            }
        };
    }

    fn echo(_ctx: &Context<'_>, value: i32) -> Result<i32> {
        Ok(value)
    }

    test_data!(Query { ctx: &Context<'_>, value: i32 } -> Result<i32>);

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let sdl = schema.sdl();

    assert!(sdl.contains("echo(value: Int!): Int!"), "sdl was:\n{sdl}");
    assert_eq!(
        schema
            .execute("{ echo(value: 42) }")
            .await
            .into_result()
            .unwrap()
            .data,
        async_graphql::value!({ "echo": 42 }),
    );
}

#[tokio::test]
pub async fn test_subscription() {
    macro_rules! test_data {
        ($test_name:ident) => {
            #[derive(Debug, Clone)]
            pub struct $test_name {
                value: i32,
            }

            #[async_graphql::Subscription]
            impl $test_name {
                async fn value(&self) -> impl futures_util::stream::Stream<Item = i32> + 'static {
                    let value = self.value;
                    futures_util::stream::once(async move { value })
                }
            }
        };
    }

    test_data!(A);
}

#[tokio::test]
pub async fn test_scalar() {
    macro_rules! test_data {
        ($test_name:ident) => {
            #[derive(Debug, Clone)]
            pub struct $test_name(i64);

            #[async_graphql::Scalar]
            impl async_graphql::ScalarType for $test_name {
                fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
                    match value {
                        async_graphql::Value::Number(n) if n.is_i64() => {
                            Ok($test_name(n.as_i64().unwrap()))
                        }
                        _ => Err(async_graphql::InputValueError::expected_type(value)),
                    }
                }

                fn to_value(&self) -> async_graphql::Value {
                    self.0.to_value()
                }
            }
        };
    }

    test_data!(A);
}

#[tokio::test]
pub async fn test_oneof_object_type() {
    macro_rules! test_data {
        ($test_name:ident, $type1:ty, $type2:ty) => {
            #[derive(async_graphql::OneofObject)]
            enum $test_name {
                Type1($type1),
                Type2($type2),
            }
        };
    }

    #[derive(async_graphql::InputObject)]
    struct A {
        a: i32,
    }

    #[derive(async_graphql::InputObject)]
    struct B {
        b: i32,
    }

    test_data!(C, A, B);
}
