#![allow(unreachable_code)]

use std::collections::HashMap;
use std::convert::Infallible;

use async_graphql::dataloader::{DataLoader, Loader};
use async_graphql::*;

#[tokio::test]
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

#[tokio::test]
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

#[tokio::test]
pub async fn test_find_entity_with_context() {
    struct MyLoader;

    #[async_trait::async_trait]
    impl Loader<ID> for MyLoader {
        type Value = MyObj;
        type Error = Infallible;

        async fn load(&self, keys: &[ID]) -> Result<HashMap<ID, Self::Value>, Self::Error> {
            Ok(keys
                .iter()
                .filter(|id| id.as_str() != "999")
                .map(|id| {
                    (
                        id.clone(),
                        MyObj {
                            id: id.clone(),
                            value: 999,
                        },
                    )
                })
                .collect())
        }
    }

    #[derive(Clone, SimpleObject)]
    struct MyObj {
        id: ID,
        value: i32,
    }

    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        #[graphql(entity)]
        async fn find_user_by_id(&self, ctx: &Context<'_>, id: ID) -> FieldResult<MyObj> {
            let loader = ctx.data_unchecked::<DataLoader<MyLoader>>();
            loader
                .load_one(id)
                .await
                .unwrap()
                .ok_or_else(|| "Not found".into())
        }
    }

    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(DataLoader::new(MyLoader))
        .finish();
    let query = r#"{
            _entities(representations: [
                {__typename: "MyObj", id: "1"},
                {__typename: "MyObj", id: "2"},
                {__typename: "MyObj", id: "3"},
                {__typename: "MyObj", id: "4"}
            ]) {
                __typename
                ... on MyObj {
                    id
                    value
                }
            }
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "_entities": [
                {"__typename": "MyObj", "id": "1", "value": 999 },
                {"__typename": "MyObj", "id": "2", "value": 999 },
                {"__typename": "MyObj", "id": "3", "value": 999 },
                {"__typename": "MyObj", "id": "4", "value": 999 },
            ]
        })
    );

    let query = r#"{
            _entities(representations: [
                {__typename: "MyObj", id: "999"}
            ]) {
                __typename
                ... on MyObj {
                    id
                    value
                }
            }
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap_err(),
        vec![ServerError {
            message: "Not found".to_string(),
            source: None,
            locations: vec![Pos {
                line: 2,
                column: 13
            }],
            path: vec![PathSegment::Field("_entities".to_owned())],
            extensions: None,
        }]
    );
}

#[tokio::test]
pub async fn test_entity_union() {
    #[derive(SimpleObject)]
    struct MyObj {
        a: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        #[graphql(entity)]
        async fn find_obj(&self, id: i32) -> MyObj {
            todo!()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = r#"{
            __type(name: "_Entity") { possibleTypes { name } }
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "__type": {
                "possibleTypes": [
                    {"name": "MyObj"},
                ]
            }
        })
    );
}
