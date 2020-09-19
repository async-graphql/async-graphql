use async_graphql::*;
use futures::lock::Mutex;
use std::sync::Arc;

#[async_std::test]
pub async fn test_mutation_query() {
    type List = Arc<Mutex<Vec<i32>>>;

    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        async fn len(&self, ctx: &Context<'_>) -> i32 {
            ctx.data_unchecked::<List>().lock().await.len() as i32
        }

        async fn first(&self, ctx: &Context<'_>) -> i32 {
            *ctx.data_unchecked::<List>()
                .lock()
                .await
                .first()
                .expect("no element")
        }

        async fn last(&self, ctx: &Context<'_>) -> i32 {
            *ctx.data_unchecked::<List>()
                .lock()
                .await
                .last()
                .expect("no element")
        }
    }

    struct MutationRoot;

    #[Object]
    impl MutationRoot {
        async fn append1(&self, ctx: &Context<'_>) -> bool {
            ctx.data_unchecked::<List>().lock().await.push(1);
            true
        }
        async fn append2(&self, ctx: &Context<'_>) -> bool {
            ctx.data_unchecked::<List>().lock().await.push(2);
            true
        }
    }

    let list = List::default();
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(list.clone())
        .finish();
    schema.execute("mutation { append1 }").await;
    schema.execute("mutation { append2 }").await;
    assert_eq!(
        schema.execute("{ last }").await.data,
        serde_json::json!({ "last": 2 })
    );
    assert_eq!(
        schema.execute("{ first }").await.data,
        serde_json::json!({ "first": 1 })
    );
    assert_eq!(
        schema.execute("{ len }").await.data,
        serde_json::json!({ "len": 2 })
    );
}
