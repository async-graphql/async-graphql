use async_graphql::*;
use futures::lock::Mutex;
use std::sync::Arc;
use std::time::Duration;

#[async_std::test]
pub async fn test_mutation_execution_order() {
    type List = Arc<Mutex<Vec<i32>>>;

    #[SimpleObject]
    struct QueryRoot;

    struct MutationRoot;

    #[Object]
    impl MutationRoot {
        async fn append1(&self, ctx: &Context<'_>) -> bool {
            async_std::task::sleep(Duration::from_secs(1)).await;
            ctx.data_unchecked::<List>().lock().await.push(1);
            true
        }

        async fn append2(&self, ctx: &Context<'_>) -> bool {
            async_std::task::sleep(Duration::from_millis(500)).await;
            ctx.data_unchecked::<List>().lock().await.push(2);
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

#[async_std::test]
pub async fn test_mutation_fragment() {
    #[SimpleObject]
    struct QueryRoot;

    struct MutationRoot;

    #[Object]
    impl MutationRoot {
        async fn action(&self) -> bool {
            true
        }
    }

    let schema = Schema::new(QueryRoot, MutationRoot, EmptySubscription);
    let resp = schema
        .execute(
            r#"
        mutation {
            ... {
                actionInUnnamedFragment: action
            }
            ... on MutationRoot {
                actionInNamedFragment: action
            }
        }"#,
        )
        .await
        .unwrap();
    assert_eq!(
        resp.data,
        serde_json::json!({
            "actionInUnnamedFragment": true,
            "actionInNamedFragment": true,
        })
    );
}
