use async_graphql::*;

struct User {
    id: ID,
}

#[Object(extends)]
impl User {
    #[field(external)]
    async fn id(&self) -> &ID {
        &self.id
    }

    #[field]
    async fn reviews(&self) -> Vec<Review> {
        todo!()
    }
}

struct Review;

#[Object]
impl Review {
    #[field]
    async fn body(&self) -> String {
        todo!()
    }

    #[field(provides = "username")]
    async fn author(&self) -> User {
        todo!()
    }

    #[field]
    async fn product(&self) -> Product {
        todo!()
    }
}

struct Product {
    upc: String,
}

#[Object(extends)]
impl Product {
    #[field(external)]
    async fn upc(&self) -> &str {
        &self.upc
    }

    #[field]
    async fn reviews(&self) -> Vec<Review> {
        todo!()
    }
}

struct QueryRoot;

#[Object]
impl QueryRoot {
    #[entity]
    async fn find_user_by_id(&self, id: ID) -> User {
        User { id }
    }

    #[entity]
    async fn find_product_by_upc(&self, upc: String) -> Product {
        Product { upc }
    }
}

#[async_std::test]
pub async fn test_federation() {
    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let query = format!(
        r#"{{
            _entities(representations: [{{__typename: "Product", upc: "B00005N5PF"}}]) {{
                __typename
                ... on Product {{
                    upc
                }}
            }}
        }}"#
    );
    assert_eq!(
        schema.execute(&query).await.unwrap().data,
        serde_json::json!({
            "_entities": [
                {"__typename": "Product", "upc": "B00005N5PF"},
            ]
        })
    );
}
