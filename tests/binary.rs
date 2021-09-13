use async_graphql::*;
use bytes::Bytes;

#[tokio::test]
pub async fn test_batch_request() {
    struct Query;

    #[Object]
    impl Query {
        async fn data(&self) -> Bytes {
            Bytes::from_static(b"abcdef")
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute("{ data }").await.into_result().unwrap().data,
        value!({
            "data": Bytes::from_static(b"abcdef"),
        })
    );
}
