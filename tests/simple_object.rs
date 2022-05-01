use async_graphql::*;

#[tokio::test]
async fn test_flatten() {
    #[derive(SimpleObject)]
    struct A {
        a: i32,
        b: i32,
    }

    #[derive(SimpleObject)]
    struct B {
        #[graphql(flatten)]
        a: A,
        c: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> B {
            B {
                a: A { a: 100, b: 200 },
                c: 300,
            }
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
async fn recursive_fragment_definition() {
    #[derive(SimpleObject)]
    struct Hello {
        world: String,
    }

    struct Query;

    // this setup is actually completely irrelevant we just need to be able ot
    // execute a query
    #[Object]
    impl Query {
        async fn obj(&self) -> Hello {
            Hello {
                world: "Hello World".into(),
            }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "fragment f on Query {...f} { __typename }";
    assert!(schema.execute(query).await.into_result().is_err());
}

#[tokio::test]
async fn recursive_fragment_definition_nested() {
    #[derive(SimpleObject)]
    struct Hello {
        world: String,
    }

    struct Query;

    // this setup is actually completely irrelevant we just need to be able ot
    // execute a query
    #[Object]
    impl Query {
        async fn obj(&self) -> Hello {
            Hello {
                world: "Hello World".into(),
            }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "fragment f on Query { a { ...f a { ...f } } } { __typename }";
    assert!(schema.execute(query).await.into_result().is_err());
}
