#![cfg(feature = "uuid")]

use async_graphql::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
struct AssetId(pub uuid::Uuid);

async_graphql::scalar!(AssetId);

#[tokio::test]
/// Test case for
/// https://github.com/async-graphql/async-graphql/issues/603
pub async fn test_serialize_uuid() {
    let generated_uuid = uuid::Uuid::new_v4();

    struct Query {
        data: AssetId,
    }

    #[Object]
    impl Query {
        async fn data(&self) -> &AssetId {
            &self.data
        }
    }

    let schema = Schema::new(
        Query {
            data: AssetId(generated_uuid),
        },
        EmptyMutation,
        EmptySubscription,
    );
    let query = r#"{ data }"#;
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "data": generated_uuid.to_string(),
        })
    );
}
