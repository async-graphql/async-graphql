use async_graphql::*;

#[tokio::test]
async fn test_flatten() {
    #[derive(SimpleObject)]
    struct A {
        a: i32,
        b: i32,
    }

    struct B;

    #[Object]
    impl B {
        #[graphql(flatten)]
        async fn a(&self) -> A {
            A { a: 100, b: 200 }
        }

        async fn c(&self) -> i32 {
            300
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> B {
            B
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ __type(name: \"B\") { fields { name } } }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "__type": {
                "fields": [
                    {"name": "a"},
                    {"name": "b"},
                    {"name": "c"}
                ]
            }
        })
    );

    let query = "{ obj { a b c } }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "obj": {
                "a": 100,
                "b": 200,
                "c": 300,
            }
        })
    );
}

#[tokio::test]
async fn test_flatten_with_context() {
    #[derive(SimpleObject)]
    struct A {
        a: i32,
        b: i32,
    }

    struct B;

    #[Object]
    impl B {
        #[graphql(flatten)]
        async fn a(&self, _ctx: &Context<'_>) -> A {
            A { a: 100, b: 200 }
        }

        async fn c(&self) -> i32 {
            300
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> B {
            B
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ __type(name: \"B\") { fields { name } } }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "__type": {
                "fields": [
                    {"name": "a"},
                    {"name": "b"},
                    {"name": "c"}
                ]
            }
        })
    );

    let query = "{ obj { a b c } }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "obj": {
                "a": 100,
                "b": 200,
                "c": 300,
            }
        })
    );
}

#[tokio::test]
async fn test_object_process_with_field() {
    struct Query;

    #[Object]
    impl Query {
        async fn test(
            &self,
            #[graphql(process_with = "str::make_ascii_uppercase")] processed_arg: String,
        ) -> String {
            processed_arg
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ test(processedArg: \"smol\") }";
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "test": "SMOL"
        })
    );
}
