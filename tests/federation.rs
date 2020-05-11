#![allow(unreachable_code)]

use async_graphql::prelude::*;
use async_graphql::{EmptyMutation, EmptySubscription};

struct User {
    id: GqlID,
}

#[GqlObject(extends)]
impl User {
    #[field(external)]
    async fn id(&self) -> &GqlID {
        &self.id
    }

    async fn reviews(&self) -> Vec<Review> {
        todo!()
    }
}

struct Review;

#[GqlObject]
impl Review {
    async fn body(&self) -> String {
        todo!()
    }

    #[field(provides = "username")]
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

#[GqlObject(extends)]
impl Product {
    #[field(external)]
    async fn upc(&self) -> &str {
        &self.upc
    }

    async fn reviews(&self) -> Vec<Review> {
        todo!()
    }
}

struct QueryRoot;

#[GqlObject]
impl QueryRoot {
    #[entity]
    async fn find_user_by_id(&self, id: GqlID) -> User {
        User { id }
    }

    #[entity]
    async fn find_product_by_upc(&self, upc: String) -> Product {
        Product { upc }
    }
}

#[async_std::test]
pub async fn test_federation() {
    let schema = GqlSchema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let query = r#"{
            _entities(representations: [{__typename: "Product", upc: "B00005N5PF"}]) {
                __typename
                ... on Product {
                    upc
                }
            }
        }"#;
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "_entities": [
                {"__typename": "Product", "upc": "B00005N5PF"},
            ]
        })
    );
}
