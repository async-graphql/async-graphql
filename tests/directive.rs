use async_graphql::*;
use serde::{Deserialize, Serialize};

#[tokio::test]
pub async fn test_directive_skip() {
    struct Query;

    #[Object]
    impl Query {
        pub async fn value(&self) -> i32 {
            10
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let data = schema
        .execute(
            r#"
            fragment A on Query {
                value5: value @skip(if: true)
                value6: value @skip(if: false)
            }

            query {
                value1: value @skip(if: true)
                value2: value @skip(if: false)
                ... @skip(if: true) {
                    value3: value
                }
                ... @skip(if: false) {
                    value4: value
                }
                ... A
            }
        "#,
        )
        .await
        .into_result()
        .unwrap()
        .data;
    assert_eq!(
        data,
        value!({
            "value2": 10,
            "value4": 10,
            "value6": 10,
        })
    );
}

#[tokio::test]
pub async fn test_directive_include() {
    struct Query;

    #[Object]
    impl Query {
        pub async fn value(&self) -> i32 {
            10
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let resp = schema
        .execute(
            r#"
            {
                value1: value @include(if: true)
                value2: value @include(if: false)
            }
        "#,
        )
        .await;
    assert_eq!(
        resp.data,
        value!({
            "value1": 10,
        })
    );
}

#[tokio::test]
pub async fn test_custom_directive() {
    struct Concat {
        prefix: String,
        suffix: String,
    }

    #[async_trait::async_trait]
    impl CustomDirective for Concat {
        async fn resolve_field(
            &self,
            _ctx: &Context<'_>,
            resolve: ResolveFut<'_>,
        ) -> ServerResult<Option<Value>> {
            resolve.await.map(|value| {
                value.map(|value| match value {
                    Value::String(str) => Value::String(self.prefix.clone() + &str + &self.suffix),
                    _ => value,
                })
            })
        }
    }

    #[Directive(location = "Field")]
    fn concat(prefix: String, suffix: String) -> impl CustomDirective {
        Concat { prefix, suffix }
    }

    struct Query;

    #[Object]
    impl Query {
        pub async fn value(&self) -> &'static str {
            "abc"
        }
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .directive(concat)
        .finish();
    assert_eq!(
        schema
            .execute(r#"{ value @concat(prefix: "&", suffix: "*") }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "value": "&abc*" })
    );
}

#[tokio::test]
pub async fn test_no_unused_directives() {
    struct Query;

    #[Object]
    impl Query {
        pub async fn a(&self) -> String {
            "a".into()
        }
    }

    let sdl = Schema::new(Query, EmptyMutation, EmptySubscription).sdl();

    assert!(!sdl.contains("directive @deprecated"));
    assert!(!sdl.contains("directive @specifiedBy"));
    assert!(!sdl.contains("directive @oneOf"));
}

#[tokio::test]
pub async fn test_includes_deprecated_directive() {
    #[derive(SimpleObject)]
    struct A {
        #[graphql(deprecation = "Use `Foo` instead")]
        a: String,
    }

    struct Query;

    #[Object]
    impl Query {
        pub async fn a(&self) -> A {
            A { a: "a".into() }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    assert!(schema.sdl().contains(
        r#"
"""
Marks an element of a GraphQL schema as no longer supported.
"""
directive @deprecated(reason: String = "No longer supported") on FIELD_DEFINITION | ARGUMENT_DEFINITION | INPUT_FIELD_DEFINITION | ENUM_VALUE"#,
    ))
}

#[tokio::test]
pub async fn test_includes_specified_by_directive() {
    #[derive(Serialize, Deserialize)]
    struct A {
        a: String,
    }

    scalar!(
        A,
        "A",
        "This is A",
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
    );

    struct Query;

    #[Object]
    impl Query {
        pub async fn a(&self) -> A {
            A { a: "a".into() }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    assert!(
        schema
            .sdl()
            .contains(r#"directive @specifiedBy(url: String!) on SCALAR"#)
    )
}

#[tokio::test]
pub async fn test_includes_one_of_directive() {
    #[derive(OneofObject)]
    enum AB {
        A(String),
        B(i64),
    }

    struct Query;

    #[Object]
    impl Query {
        pub async fn ab(&self, _input: AB) -> bool {
            true
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    assert!(schema.sdl().contains(r#"directive @oneOf on INPUT_OBJECT"#))
}
