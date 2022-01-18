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
