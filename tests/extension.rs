use std::sync::Arc;

use async_graphql::extensions::{Extension, ExtensionContext, ExtensionFactory, ResolveInfo};
use async_graphql::parser::types::ExecutableDocument;
use async_graphql::*;
use async_graphql_value::ConstValue;
use futures_util::stream::Stream;
use futures_util::StreamExt;
use spin::Mutex;

#[tokio::test]
pub async fn test_extension_ctx() {
    #[derive(Default, Clone)]
    struct MyData(Arc<Mutex<i32>>);

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self, ctx: &Context<'_>) -> i32 {
            *ctx.data_unchecked::<MyData>().0.lock()
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn value(&self, ctx: &Context<'_>) -> impl Stream<Item = i32> {
            let data = *ctx.data_unchecked::<MyData>().0.lock();
            futures_util::stream::once(async move { data })
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
            if let Ok(data) = ctx.data::<MyData>() {
                *data.0.lock() = 100;
            }
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
        assert_eq!(
            schema
                .execute("{ value }")
                .await
                .into_result()
                .unwrap()
                .data,
            value! ({
                "value": 100
            })
        );
    }

    // data in request
    {
        let data = MyData::default();
        let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
            .extension(MyExtension)
            .finish();

        assert_eq!(
            schema
                .execute(Request::new("{ value }").data(data.clone()))
                .await
                .into_result()
                .unwrap()
                .data,
            value! ({
                "value": 100
            })
        );
    }

    // data in session
    {
        let schema = Schema::build(Query, EmptyMutation, Subscription)
            .extension(MyExtension)
            .finish();

        let mut data = Data::default();
        data.insert(MyData::default());
        let mut stream = schema
            .execute_stream_with_session_data(
                Request::new("subscription { value }"),
                Arc::new(data),
            )
            .boxed();
        assert_eq!(
            stream.next().await.unwrap().into_result().unwrap().data,
            value! ({
                "value": 100
            })
        );
    }
}

#[tokio::test]
pub async fn test_extension_call_order() {
    struct MyExtensionImpl {
        calls: Arc<Mutex<Vec<&'static str>>>,
    }

    #[async_trait::async_trait]
    #[allow(unused_variables)]
    impl Extension for MyExtensionImpl {
        fn name(&self) -> Option<&'static str> {
            Some("test")
        }

        fn start(&mut self, ctx: &ExtensionContext<'_>) {
            self.calls.lock().push("start");
        }

        fn end(&mut self, ctx: &ExtensionContext<'_>) {
            self.calls.lock().push("end");
        }

        async fn prepare_request(
            &mut self,
            ctx: &ExtensionContext<'_>,
            request: Request,
        ) -> ServerResult<Request> {
            self.calls.lock().push("prepare_request");
            Ok(request)
        }

        fn parse_start(
            &mut self,
            ctx: &ExtensionContext<'_>,
            query_source: &str,
            variables: &Variables,
        ) {
            self.calls.lock().push("parse_start");
        }

        fn parse_end(&mut self, ctx: &ExtensionContext<'_>, document: &ExecutableDocument) {
            self.calls.lock().push("parse_end");
        }

        fn validation_start(&mut self, ctx: &ExtensionContext<'_>) {
            self.calls.lock().push("validation_start");
        }

        fn validation_end(&mut self, ctx: &ExtensionContext<'_>, result: &ValidationResult) {
            self.calls.lock().push("validation_end");
        }

        fn execution_start(&mut self, ctx: &ExtensionContext<'_>) {
            self.calls.lock().push("execution_start");
        }

        fn execution_end(&mut self, ctx: &ExtensionContext<'_>) {
            self.calls.lock().push("execution_end");
        }

        fn resolve_start(&mut self, ctx: &ExtensionContext<'_>, info: &ResolveInfo<'_>) {
            self.calls.lock().push("resolve_start");
        }

        fn resolve_end(&mut self, ctx: &ExtensionContext<'_>, info: &ResolveInfo<'_>) {
            self.calls.lock().push("resolve_end");
        }

        fn result(&mut self, ctx: &ExtensionContext<'_>) -> Option<ConstValue> {
            self.calls.lock().push("result");
            None
        }
    }

    struct MyExtension {
        calls: Arc<Mutex<Vec<&'static str>>>,
    }

    impl ExtensionFactory for MyExtension {
        fn create(&self) -> Box<dyn Extension> {
            Box::new(MyExtensionImpl {
                calls: self.calls.clone(),
            })
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn value1(&self) -> i32 {
            10
        }

        async fn value2(&self) -> i32 {
            10
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn value(&self) -> impl Stream<Item = i32> {
            futures_util::stream::iter(vec![1, 2, 3])
        }
    }

    {
        let calls: Arc<Mutex<Vec<&'static str>>> = Default::default();
        let schema = Schema::build(Query, EmptyMutation, Subscription)
            .extension(MyExtension {
                calls: calls.clone(),
            })
            .finish();
        let _ = schema
            .execute("{ value1 value2 }")
            .await
            .into_result()
            .unwrap();
        let calls = calls.lock();
        assert_eq!(
            &*calls,
            &vec![
                "start",
                "prepare_request",
                "parse_start",
                "parse_end",
                "validation_start",
                "validation_end",
                "execution_start",
                "resolve_start",
                "resolve_end",
                "resolve_start",
                "resolve_end",
                "execution_end",
                "result",
                "end",
            ]
        );
    }

    {
        let calls: Arc<Mutex<Vec<&'static str>>> = Default::default();
        let schema = Schema::build(Query, EmptyMutation, Subscription)
            .extension(MyExtension {
                calls: calls.clone(),
            })
            .finish();
        let mut stream = schema.execute_stream("subscription { value }").boxed();
        while let Some(_) = stream.next().await {}
        let calls = calls.lock();
        assert_eq!(
            &*calls,
            &vec![
                "start",
                "prepare_request",
                "parse_start",
                "parse_end",
                "validation_start",
                "validation_end",
                "execution_start",
                "resolve_start",
                "resolve_end",
                "execution_end",
                "result",
                "execution_start",
                "resolve_start",
                "resolve_end",
                "execution_end",
                "result",
                "execution_start",
                "resolve_start",
                "resolve_end",
                "execution_end",
                "result",
                "end",
            ]
        );
    }
}
