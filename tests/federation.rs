#![allow(unreachable_code)]

use async_graphql::*;

#[async_std::test]
pub async fn test_nested_key() {
    #[derive(InputObject)]
    struct MyInputA {
        a: i32,
        b: i32,
        c: MyInputB,
    }

    #[derive(InputObject)]
    struct MyInputB {
        v: i32,
    }

    assert_eq!(MyInputB::federation_fields().as_deref(), Some("{ v }"));
    assert_eq!(
        MyInputA::federation_fields().as_deref(),
        Some("{ a b c { v } }")
    );

    struct QueryRoot;

    #[derive(SimpleObject)]
    struct MyObj {
        a: i32,
        b: i32,
        c: i32,
    }

    #[Object]
    impl QueryRoot {
        #[graphql(entity)]
        async fn find_obj(&self, input: MyInputA) -> MyObj {
            MyObj {
                a: input.a,
                b: input.b,
                c: input.c.v,
            }
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let query = r#"{
            _entities(representations: [{__typename: "MyObj", input: {a: 1, b: 2, c: { v: 3 }}}]) {
                __typename
                ... on MyObj {
                    a b c
                }
            }
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "_entities": [
                {"__typename": "MyObj", "a": 1, "b": 2, "c": 3},
            ]
        })
    );
}

#[async_std::test]
pub async fn test_federation() {
    struct User {
        id: ID,
    }

    #[Object(extends)]
    impl User {
        #[graphql(external)]
        async fn id(&self) -> &ID {
            &self.id
        }

        async fn reviews(&self) -> Vec<Review> {
            todo!()
        }
    }

    struct Review;

    #[Object]
    impl Review {
        async fn body(&self) -> String {
            todo!()
        }

        async fn author(&self) -> User {
            todo!()
        }

        async fn product(&self) -> Product {
            todo!()
        }
    }

    struct Product {
        upc: String,
    }

    #[Object(extends)]
    impl Product {
        #[graphql(external)]
        async fn upc(&self) -> &str {
            &self.upc
        }

        async fn reviews(&self) -> Vec<Review> {
            todo!()
        }
    }

    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        #[graphql(entity)]
        async fn find_user_by_id(&self, id: ID) -> User {
            User { id }
        }

        #[graphql(entity)]
        async fn find_product_by_upc(&self, upc: String) -> Product {
            Product { upc }
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let query = r#"{
            _entities(representations: [{__typename: "Product", upc: "B00005N5PF"}]) {
                __typename
                ... on Product {
                    upc
                }
            }
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "_entities": [
                {"__typename": "Product", "upc": "B00005N5PF"},
            ]
        })
    );
}
