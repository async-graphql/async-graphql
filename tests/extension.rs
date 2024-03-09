use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};

use async_graphql::{
    extensions::{
        Extension, ExtensionContext, ExtensionFactory, NextExecute, NextParseQuery,
        NextPrepareRequest, NextRequest, NextResolve, NextSubscribe, NextValidation, ResolveInfo,
    },
    futures_util::stream::BoxStream,
    parser::types::ExecutableDocument,
    *,
};
use async_graphql_value::ConstValue;
use futures_util::{lock::Mutex, stream::Stream, StreamExt};

#[tokio::test]
pub async fn test_extension_ctx() {
    #[derive(Default, Clone)]
    struct MyData(Arc<Mutex<i32>>);

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self, ctx: &Context<'_>) -> i32 {
            *ctx.data_unchecked::<MyData>().0.lock().await
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn value(&self, ctx: &Context<'_>) -> impl Stream<Item = i32> {
            let data = *ctx.data_unchecked::<MyData>().0.lock().await;
            futures_util::stream::once(async move { data })
        }
    }

    struct MyExtensionImpl;

    #[async_trait::async_trait]
    impl Extension for MyExtensionImpl {
        async fn parse_query(
            &self,
            ctx: &ExtensionContext<'_>,
            query: &str,
            variables: &Variables,
            next: NextParseQuery<'_>,
        ) -> ServerResult<ExecutableDocument> {
            if let Ok(data) = ctx.data::<MyData>() {
                *data.0.lock().await = 100;
            }
            next.run(ctx, query, variables).await
        }
    }

    struct MyExtension;

    impl ExtensionFactory for MyExtension {
        fn create(&self) -> Arc<dyn Extension> {
            Arc::new(MyExtensionImpl)
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
        let mut stream = schema.execute_stream_with_session_data(
            Request::new("subscription { value }"),
            Arc::new(data),
        );
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
        async fn request(&self, ctx: &ExtensionContext<'_>, next: NextRequest<'_>) -> Response {
            self.calls.lock().await.push("request_start");
            let res = next.run(ctx).await;
            self.calls.lock().await.push("request_end");
            res
        }

        fn subscribe<'s>(
            &self,
            ctx: &ExtensionContext<'_>,
            mut stream: BoxStream<'s, Response>,
            next: NextSubscribe<'_>,
        ) -> BoxStream<'s, Response> {
            let calls = self.calls.clone();
            next.run(
                ctx,
                Box::pin(async_stream::stream! {
                    calls.lock().await.push("subscribe_start");
                    while let Some(item) = stream.next().await {
                        yield item;
                    }
                    calls.lock().await.push("subscribe_end");
                }),
            )
        }

        async fn prepare_request(
            &self,
            ctx: &ExtensionContext<'_>,
            request: Request,
            next: NextPrepareRequest<'_>,
        ) -> ServerResult<Request> {
            self.calls.lock().await.push("prepare_request_start");
            let res = next.run(ctx, request).await;
            self.calls.lock().await.push("prepare_request_end");
            res
        }

        async fn parse_query(
            &self,
            ctx: &ExtensionContext<'_>,
            query: &str,
            variables: &Variables,
            next: NextParseQuery<'_>,
        ) -> ServerResult<ExecutableDocument> {
            self.calls.lock().await.push("parse_query_start");
            let res = next.run(ctx, query, variables).await;
            self.calls.lock().await.push("parse_query_end");
            res
        }

        async fn validation(
            &self,
            ctx: &ExtensionContext<'_>,
            next: NextValidation<'_>,
        ) -> Result<ValidationResult, Vec<ServerError>> {
            self.calls.lock().await.push("validation_start");
            let res = next.run(ctx).await;
            self.calls.lock().await.push("validation_end");
            res
        }

        async fn execute(
            &self,
            ctx: &ExtensionContext<'_>,
            operation_name: Option<&str>,
            next: NextExecute<'_>,
        ) -> Response {
            assert_eq!(operation_name, Some("Abc"));
            self.calls.lock().await.push("execute_start");
            let res = next.run(ctx, operation_name).await;
            self.calls.lock().await.push("execute_end");
            res
        }

        async fn resolve(
            &self,
            ctx: &ExtensionContext<'_>,
            info: ResolveInfo<'_>,
            next: NextResolve<'_>,
        ) -> ServerResult<Option<ConstValue>> {
            self.calls.lock().await.push("resolve_start");
            let res = next.run(ctx, info).await;
            self.calls.lock().await.push("resolve_end");
            res
        }
    }

    struct MyExtension {
        calls: Arc<Mutex<Vec<&'static str>>>,
    }

    impl ExtensionFactory for MyExtension {
        fn create(&self) -> Arc<dyn Extension> {
            Arc::new(MyExtensionImpl {
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
            .execute("query Abc { value1 value2 }")
            .await
            .into_result()
            .unwrap();
        let calls = calls.lock().await;
        assert_eq!(
            &*calls,
            &vec![
                "request_start",
                "prepare_request_start",
                "prepare_request_end",
                "parse_query_start",
                "parse_query_end",
                "validation_start",
                "validation_end",
                "execute_start",
                "resolve_start",
                "resolve_end",
                "resolve_start",
                "resolve_end",
                "execute_end",
                "request_end",
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
        let mut stream = schema.execute_stream("subscription Abc { value }");
        while stream.next().await.is_some() {}
        let calls = calls.lock().await;
        assert_eq!(
            &*calls,
            &vec![
                "subscribe_start",
                "prepare_request_start",
                "prepare_request_end",
                "parse_query_start",
                "parse_query_end",
                "validation_start",
                "validation_end",
                // push 1
                "execute_start",
                "resolve_start",
                "resolve_end",
                "execute_end",
                // push 2
                "execute_start",
                "resolve_start",
                "resolve_end",
                "execute_end",
                // push 3
                "execute_start",
                "resolve_start",
                "resolve_end",
                "execute_end",
                // end
                "subscribe_end",
            ]
        );
    }
}

#[tokio::test]
pub async fn query_execute_with_data() {
    struct MyExtensionImpl<T>(T);

    #[async_trait::async_trait]
    impl<T> Extension for MyExtensionImpl<T>
    where
        T: Copy + Sync + Send + 'static,
    {
        async fn execute(
            &self,
            ctx: &ExtensionContext<'_>,
            operation_name: Option<&str>,
            next: NextExecute<'_>,
        ) -> Response {
            let mut data = Data::default();
            data.insert(self.0);
            next.run_with_data(ctx, operation_name, data).await
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self, ctx: &Context<'_>) -> Result<i64> {
            Ok(*ctx.data::<i32>()? as i64 + ctx.data::<i64>()?)
        }
    }

    struct MyExtension<T>(T);

    impl<T> ExtensionFactory for MyExtension<T>
    where
        T: Copy + Sync + Send + 'static,
    {
        fn create(&self) -> Arc<dyn Extension> {
            Arc::new(MyExtensionImpl(self.0))
        }
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .extension(MyExtension(100i32))
        .extension(MyExtension(200i64))
        .finish();
    let query = "{ value }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "value": 300
        })
    );
}

#[tokio::test]
pub async fn subscription_execute_with_data() {
    type Logs = Arc<Mutex<Vec<LogElement>>>;

    struct MyExtensionImpl {
        counter: Arc<AtomicI32>,
    }

    impl MyExtensionImpl {
        async fn append_log(&self, ctx: &ExtensionContext<'_>, element: LogElement) {
            ctx.data::<Logs>().unwrap().lock().await.push(element);
        }
    }

    #[async_trait::async_trait]
    impl Extension for MyExtensionImpl {
        async fn execute(
            &self,
            ctx: &ExtensionContext<'_>,
            operation_name: Option<&str>,
            next: NextExecute<'_>,
        ) -> Response {
            let mut data = Data::default();

            let current_counter = self.counter.fetch_add(1, Ordering::SeqCst);
            data.insert(current_counter);
            self.append_log(ctx, LogElement::PreHook(current_counter))
                .await;
            let resp = next.run_with_data(ctx, operation_name, data).await;
            self.append_log(ctx, LogElement::PostHook(current_counter))
                .await;
            resp
        }
    }

    struct MyExtension {
        counter: Arc<AtomicI32>,
    }

    impl ExtensionFactory for MyExtension {
        fn create(&self) -> Arc<dyn Extension> {
            Arc::new(MyExtensionImpl {
                counter: self.counter.clone(),
            })
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    enum LogElement {
        PreHook(i32),
        OuterAccess(i32),
        InnerAccess(i32),
        PostHook(i32),
    }

    let logs = Logs::default();
    let message_counter = Arc::new(AtomicI32::new(0));

    #[derive(Clone, Copy)]
    struct Inner(i32);

    #[Object]
    impl Inner {
        async fn value(&self, ctx: &Context<'_>) -> i32 {
            if let Some(logs) = ctx.data_opt::<Logs>() {
                logs.lock().await.push(LogElement::InnerAccess(self.0));
            }
            self.0
        }
    }

    #[derive(Clone, Copy)]
    struct Outer(Inner);

    #[Object]
    impl Outer {
        async fn inner(&self, ctx: &Context<'_>) -> Inner {
            if let Some(logs) = ctx.data_opt::<Logs>() {
                logs.lock().await.push(LogElement::OuterAccess(self.0 .0));
            }
            self.0
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i64 {
            0
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn outers(&self) -> impl Stream<Item = Outer> {
            futures_util::stream::iter(10..13).map(Inner).map(Outer)
        }
    }

    let schema: Schema<Query, EmptyMutation, Subscription> =
        Schema::build(Query, EmptyMutation, Subscription)
            .data(logs.clone())
            .extension(MyExtension {
                counter: message_counter.clone(),
            })
            .finish();
    let mut stream = schema.execute_stream("subscription { outers { inner { value } } }");

    for i in 10i32..13 {
        assert_eq!(
            Response::new(value!({
                "outers": {
                    "inner": {
                        "value": i
                    }
                }
            })),
            stream.next().await.unwrap()
        );
    }

    {
        let logs = logs.lock().await;
        assert_eq!(
            *logs,
            vec![
                LogElement::PreHook(0),
                LogElement::OuterAccess(10),
                LogElement::InnerAccess(10),
                LogElement::PostHook(0),
                LogElement::PreHook(1),
                LogElement::OuterAccess(11),
                LogElement::InnerAccess(11),
                LogElement::PostHook(1),
                LogElement::PreHook(2),
                LogElement::OuterAccess(12),
                LogElement::InnerAccess(12),
                LogElement::PostHook(2),
            ],
            "Log mismatch"
        );
    }
}
