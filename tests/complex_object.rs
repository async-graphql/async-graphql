use async_graphql::*;

#[tokio::test]
pub async fn test_complex_object() {
    #[derive(SimpleObject)]
    #[graphql(complex)]
    struct MyObj {
        a: i32,
        b: i32,
    }

    #[ComplexObject]
    impl MyObj {
        async fn c(&self) -> i32 {
            self.a + self.b
        }
    }

    #[derive(Interface)]
    #[graphql(
        field(name = "a", type = "&i32"),
        field(name = "b", type = "&i32"),
        field(name = "c", type = "i32")
    )]
    enum ObjInterface {
        MyObj(MyObj),
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> MyObj {
            MyObj { a: 10, b: 20 }
        }

        async fn obj2(&self) -> ObjInterface {
            MyObj { a: 10, b: 20 }.into()
        }
    }

    let query = "{ obj { a b c } obj2 { a b c } }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "obj": {
                "a": 10,
                "b": 20,
                "c": 30,
            },
            "obj2": {
                "a": 10,
                "b": 20,
                "c": 30,
            }
        })
    );
}
