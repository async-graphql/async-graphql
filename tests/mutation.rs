use async_graphql::*;
use futures::lock::Mutex;
use std::sync::Arc;
use std::time::Duration;

#[async_std::test]
pub async fn test_list_type() {
    struct QueryRoot;

    type List = Arc<Mutex<Vec<i32>>>;

    #[Object]
    impl QueryRoot {}

    struct MutationRoot;

    #[Object]
    impl MutationRoot {
        #[field]
        async fn append1(&self, ctx: &Context<'_>) -> bool {
            async_std::task::sleep(Duration::from_secs(1)).await;
            ctx.data::<List>().lock().await.push(1);
            true
        }

        #[field]
        async fn append2(&self, ctx: &Context<'_>) -> bool {
            async_std::task::sleep(Duration::from_millis(500)).await;
            ctx.data::<List>().lock().await.push(2);
            true
        }
    }

    let list = List::default();
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(list.clone())
        .finish();
    schema
        .execute("mutation { append1 append2 }")
        .await
        .unwrap();
    assert_eq!(list.lock().await[0], 1);
    assert_eq!(list.lock().await[1], 2);
}
