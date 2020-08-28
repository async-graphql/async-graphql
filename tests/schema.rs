use async_graphql::*;

#[async_std::test]
pub async fn test_schema_default() {
    #[derive(GQLSimpleObject, Default)]
    struct Query;

    type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;

    let _schema = MySchema::default();
}
