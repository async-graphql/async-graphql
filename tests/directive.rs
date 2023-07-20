use async_graphql::*;

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
