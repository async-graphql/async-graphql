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

#[tokio::test]
pub async fn test_http_headers() {
    #[derive(Default)]
    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        async fn value(&self, ctx: &Context<'_>) -> i32 {
            ctx.insert_http_header("A", "1");
            10
        }

        async fn err(&self, ctx: &Context<'_>) -> FieldResult<i32> {
            ctx.insert_http_header("A", "1");
            Err("error".into())
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    let resp = schema.execute("{ value }").await;
    assert_eq!(resp.http_headers.get("A").map(|s| &**s), Some("1"));

    let resp = schema.execute("{ err }").await;
    assert_eq!(resp.http_headers.get("A").map(|s| &**s), Some("1"));
}
