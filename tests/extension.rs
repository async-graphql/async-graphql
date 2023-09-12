use std::sync::Arc;

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
            None,
            None,
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
