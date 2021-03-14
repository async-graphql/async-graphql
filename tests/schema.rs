use async_graphql::*;

#[tokio::test]
pub async fn test_schema_default() {
    #[derive(Default)]
    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        async fn value(&self) -> i32 {
            10
        }
    }

    type MySchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

    let _schema = MySchema::default();
}
