use async_graphql::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[tokio::test]
pub async fn test_mutation_execution_order() {
    type List = Arc<Mutex<Vec<i32>>>;

    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        async fn value(&self) -> i32 {
            10
        }
    }

    struct MutationRoot;

    #[Object]
    impl MutationRoot {
        async fn append1(&self, ctx: &Context<'_>) -> bool {
            tokio::time::sleep(Duration::from_secs(1)).await;
            ctx.data_unchecked::<List>().lock().await.push(1);
            true
        }

        async fn append2(&self, ctx: &Context<'_>) -> bool {
            tokio::time::sleep(Duration::from_millis(500)).await;
            ctx.data_unchecked::<List>().lock().await.push(2);
            true
        }
    }

    let list = List::default();
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(list.clone())
        .finish();
    schema.execute("mutation { append1 append2 }").await;
    assert_eq!(list.lock().await[0], 1);
    assert_eq!(list.lock().await[1], 2);
}

#[tokio::test]
pub async fn test_mutation_fragment() {
    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        async fn value(&self) -> i32 {
            10
        }
    }

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
        .await;
    assert_eq!(
        resp.data,
        value!({
            "actionInUnnamedFragment": true,
            "actionInNamedFragment": true,
        })
    );
}
