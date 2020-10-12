#![allow(unreachable_code)]

use async_graphql::*;

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

    #[graphql(provides = "username")]
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

#[async_std::test]
pub async fn test_federation() {
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
