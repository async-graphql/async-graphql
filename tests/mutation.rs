use async_graphql::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[tokio::test]
pub async fn test_root_mutation_execution_order() {
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
    assert_eq!(&*list.lock().await, &[1, 2]);
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

#[tokio::test]
pub async fn test_serial_object() {
    type List = Arc<Mutex<Vec<i32>>>;

    struct MyObj;

    #[Object(serial)]
    impl MyObj {
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
        async fn obj(&self) -> MyObj {
            MyObj
        }
    }

    let list = List::default();
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(list.clone())
        .finish();
    schema.execute("mutation { obj { append1 append2 } }").await;
    assert_eq!(&*list.lock().await, &[1, 2]);
}

#[tokio::test]
pub async fn test_serial_simple_object() {
    type List = Arc<Mutex<Vec<i32>>>;

    #[derive(SimpleObject)]
    #[graphql(complex, serial)]
    struct MyObj {
        value: i32,
    }

    #[ComplexObject]
    impl MyObj {
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
        async fn obj(&self) -> MyObj {
            MyObj { value: 10 }
        }
    }

    let list = List::default();
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(list.clone())
        .finish();
    schema.execute("mutation { obj { append1 append2 } }").await;
    assert_eq!(&*list.lock().await, &[1, 2]);
}

#[tokio::test]
pub async fn test_serial_merged_object() {
    type List = Arc<Mutex<Vec<i32>>>;

    #[derive(MergedObject)]
    #[graphql(serial)]
    struct MyObj(MyObj1, MyObj2);

    struct MyObj1;

    #[Object]
    impl MyObj1 {
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

    struct MyObj2;

    #[Object]
    impl MyObj2 {
        async fn append3(&self, ctx: &Context<'_>) -> bool {
            tokio::time::sleep(Duration::from_millis(200)).await;
            ctx.data_unchecked::<List>().lock().await.push(3);
            true
        }

        async fn append4(&self, ctx: &Context<'_>) -> bool {
            tokio::time::sleep(Duration::from_millis(700)).await;
            ctx.data_unchecked::<List>().lock().await.push(4);
            true
        }
    }

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
        async fn obj(&self) -> MyObj {
            MyObj(MyObj1, MyObj2)
        }
    }

    let list = List::default();
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(list.clone())
        .finish();
    schema
        .execute("mutation { obj { append1 append2 append3 append4 } }")
        .await;
    assert_eq!(&*list.lock().await, &[1, 2, 3, 4]);
}
