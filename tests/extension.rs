use async_graphql::extensions::{Extension, ExtensionContext, ExtensionFactory};
use async_graphql::*;
use spin::Mutex;
use std::sync::Arc;

#[tokio::test]
pub async fn test_extension_ctx() {
    #[derive(Default, Clone)]
    struct MyData(Arc<Mutex<i32>>);

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> bool {
            true
        }
    }

    struct MyExtensionImpl;

    #[async_trait::async_trait]
    impl Extension for MyExtensionImpl {
        fn parse_start(
            &mut self,
            ctx: &ExtensionContext<'_>,
            _query_source: &str,
            _variables: &Variables,
        ) {
            *ctx.data_unchecked::<MyData>().0.lock() = 100;
        }
    }

    struct MyExtension;

    impl ExtensionFactory for MyExtension {
        fn create(&self) -> Box<dyn Extension> {
            Box::new(MyExtensionImpl)
        }
    }

    // data in schema
    {
        let data = MyData::default();
        let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
            .data(data.clone())
            .extension(MyExtension)
            .finish();

        schema.execute("{ value }").await.into_result().unwrap();
        assert_eq!(*data.0.lock(), 100);
    }

    // data in request
    {
        let data = MyData::default();
        let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
            .extension(MyExtension)
            .finish();

        schema
            .execute(Request::new("{ value }").data(data.clone()))
            .await
            .into_result()
            .unwrap();
        assert_eq!(*data.0.lock(), 100);
    }
}
