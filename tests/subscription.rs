use async_graphql::*;
use futures_util::{
    stream::{Stream, StreamExt, TryStreamExt},
    FutureExt,
};
use std::sync::Arc;
use tokio::sync::Mutex;

struct Query;

#[Object]
impl Query {
    async fn value(&self) -> i32 {
        10
    }
}

#[tokio::test]
pub async fn test_subscription() {
    #[derive(SimpleObject)]
    struct Event {
        a: i32,
        b: i32,
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn values(&self, start: i32, end: i32) -> impl Stream<Item = i32> {
            futures_util::stream::iter(start..end)
        }

        async fn events(&self, start: i32, end: i32) -> impl Stream<Item = Event> {
            futures_util::stream::iter((start..end).map(|n| Event { a: n, b: n * 10 }))
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);

    {
        let mut stream = schema
            .execute_stream("subscription { values(start: 10, end: 20) }")
            .map(|resp| resp.into_result().unwrap().data);
        for i in 10..20 {
            assert_eq!(value!({ "values": i }), stream.next().await.unwrap());
        }
        assert!(stream.next().await.is_none());
    }

    {
        let mut stream = schema
            .execute_stream("subscription { events(start: 10, end: 20) { a b } }")
            .map(|resp| resp.into_result().unwrap().data);
        for i in 10..20 {
            assert_eq!(
                value!({ "events": {"a": i, "b": i * 10} }),
                stream.next().await.unwrap()
            );
        }
        assert!(stream.next().await.is_none());
    }
}

#[tokio::test]
pub async fn test_subscription_with_ctx_data() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            10
        }
    }

    struct MyObject;

    #[Object]
    impl MyObject {
        async fn value(&self, ctx: &Context<'_>) -> i32 {
            *ctx.data_unchecked::<i32>()
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn values(&self, ctx: &Context<'_>) -> impl Stream<Item = i32> {
            let value = *ctx.data_unchecked::<i32>();
            futures_util::stream::once(async move { value })
        }

        async fn objects(&self) -> impl Stream<Item = MyObject> {
            futures_util::stream::once(async move { MyObject })
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);

    {
        let mut stream = schema
            .execute_stream(Request::new("subscription { values objects { value } }").data(100i32))
            .map(|resp| resp.data);
        assert_eq!(value!({ "values": 100 }), stream.next().await.unwrap());
        assert_eq!(
            value!({ "objects": { "value": 100 } }),
            stream.next().await.unwrap()
        );
        assert!(stream.next().await.is_none());
    }
}

#[tokio::test]
pub async fn test_subscription_with_token() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            10
        }
    }

    struct Subscription;

    struct Token(String);

    #[Subscription]
    impl Subscription {
        async fn values(&self, ctx: &Context<'_>) -> Result<impl Stream<Item = i32>> {
            if ctx.data_unchecked::<Token>().0 != "123456" {
                return Err("forbidden".into());
            }
            Ok(futures_util::stream::once(async move { 100 }))
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);

    {
        let mut stream = schema
            .execute_stream(
                Request::new("subscription { values }").data(Token("123456".to_string())),
            )
            .map(|resp| resp.into_result().unwrap().data);
        assert_eq!(value!({ "values": 100 }), stream.next().await.unwrap());
        assert!(stream.next().await.is_none());
    }

    {
        assert!(schema
            .execute_stream(
                Request::new("subscription { values }").data(Token("654321".to_string()))
            )
            .next()
            .await
            .unwrap()
            .is_err());
    }
}

#[tokio::test]
pub async fn test_subscription_inline_fragment() {
    #[derive(SimpleObject)]
    struct Event {
        a: i32,
        b: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> i32 {
            10
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn events(&self, start: i32, end: i32) -> impl Stream<Item = Event> {
            futures_util::stream::iter((start..end).map(|n| Event { a: n, b: n * 10 }))
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);
    let mut stream = schema
        .execute_stream(
            r#"
            subscription {
                events(start: 10, end: 20) {
                    a
                    ... {
                        b
                    }
                }
            }
            "#,
        )
        .map(|resp| resp.data);
    for i in 10..20 {
        assert_eq!(
            value!({ "events": {"a": i, "b": i * 10} }),
            stream.next().await.unwrap()
        );
    }
    assert!(stream.next().await.is_none());
}

#[tokio::test]
pub async fn test_subscription_fragment() {
    #[derive(SimpleObject)]
    struct Event {
        a: i32,
        b: i32,
    }

    #[derive(Interface)]
    #[graphql(field(name = "a", ty = "&i32"))]
    enum MyInterface {
        Event(Event),
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn events(&self, start: i32, end: i32) -> impl Stream<Item = Event> {
            futures_util::stream::iter((start..end).map(|n| Event { a: n, b: n * 10 }))
        }
    }

    let schema = Schema::build(Query, EmptyMutation, Subscription)
        .register_output_type::<MyInterface>()
        .finish();
    let mut stream = schema
        .execute_stream(
            r#"
            subscription s {
                events(start: 10, end: 20) {
                    ... on MyInterface {
                        a
                    }
                    b
                }
            }
            "#,
        )
        .map(|resp| resp.data);
    for i in 10i32..20 {
        assert_eq!(
            value!({ "events": {"a": i, "b": i * 10} }),
            stream.next().await.unwrap()
        );
    }
    assert!(stream.next().await.is_none());
}

#[tokio::test]
pub async fn test_subscription_fragment2() {
    #[derive(SimpleObject)]
    struct Event {
        a: i32,
        b: i32,
    }

    #[derive(Interface)]
    #[graphql(field(name = "a", ty = "&i32"))]
    enum MyInterface {
        Event(Event),
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn events(&self, start: i32, end: i32) -> impl Stream<Item = Event> {
            futures_util::stream::iter((start..end).map(|n| Event { a: n, b: n * 10 }))
        }
    }

    let schema = Schema::build(Query, EmptyMutation, Subscription)
        .register_output_type::<MyInterface>()
        .finish();
    let mut stream = schema
        .execute_stream(
            r#"
            subscription s {
                events(start: 10, end: 20) {
                    ... Frag
                }
            }

            fragment Frag on Event {
                a b
            }
            "#,
        )
        .map(|resp| resp.data);
    for i in 10..20 {
        assert_eq!(
            value!({ "events": {"a": i, "b": i * 10} }),
            stream.next().await.unwrap()
        );
    }
    assert!(stream.next().await.is_none());
}

#[tokio::test]
pub async fn test_subscription_error() {
    struct Event {
        value: i32,
    }

    #[Object]
    impl Event {
        async fn value(&self) -> Result<i32> {
            if self.value != 5 {
                Ok(self.value)
            } else {
                Err("TestError".into())
            }
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn events(&self) -> impl Stream<Item = Event> {
            futures_util::stream::iter((0..10).map(|n| Event { value: n }))
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);
    let mut stream = schema
        .execute_stream("subscription { events { value } }")
        .map(|resp| resp.into_result())
        .map_ok(|resp| resp.data);

    for i in 0i32..5 {
        assert_eq!(
            value!({ "events": { "value": i } }),
            stream.next().await.unwrap().unwrap()
        );
    }

    assert_eq!(
        stream.next().await,
        Some(Err(vec![ServerError {
            message: "TestError".to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 25
            }],
            path: vec![
                PathSegment::Field("events".to_owned()),
                PathSegment::Field("value".to_owned())
            ],
            extensions: None,
        }]))
    );

    for i in 6i32..10 {
        assert_eq!(
            value!({ "events": { "value": i } }),
            stream.next().await.unwrap().unwrap()
        );
    }

    assert!(stream.next().await.is_none());
}

#[tokio::test]
pub async fn test_subscription_fieldresult() {
    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn values(&self) -> impl Stream<Item = Result<i32>> {
            futures_util::stream::iter(0..5)
                .map(Result::Ok)
                .chain(futures_util::stream::once(async move {
                    Err("StreamErr".into())
                }))
                .chain(futures_util::stream::iter(5..10).map(Result::Ok))
        }
    }

    let schema = Schema::new(Query, EmptyMutation, Subscription);
    let mut stream = schema.execute_stream("subscription { values }");

    for i in 0i32..5 {
        assert_eq!(
            Response::new(value!({ "values": i })),
            stream.next().await.unwrap()
        );
    }

    let resp = stream.next().await.unwrap();
    assert_eq!(
        resp.errors,
        vec![ServerError {
            message: "StreamErr".to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 16
            }],
            path: vec![PathSegment::Field("values".to_owned())],
            extensions: None,
        }]
    );

    for i in 5i32..10 {
        assert_eq!(
            Response::new(value!({ "values": i })),
            stream.next().await.unwrap()
        );
    }

    assert!(stream.next().await.is_none());
}

#[tokio::test]
pub async fn subscription_per_message_hooks() {
    #[derive(Debug, Eq, PartialEq)]
    enum LogElement {
        PreHook(i32),
        OuterAccess(i32),
        InnerAccess(i32),
        PostHook(i32),
    }

    type Log = Arc<Mutex<Vec<LogElement>>>;
    let log: Log = Arc::new(Mutex::new(vec![]));
    let message_counter = Arc::new(Mutex::new(0));

    #[derive(Clone, Copy)]
    struct Inner(i32);

    #[Object]
    impl Inner {
        async fn value(&self, ctx: &Context<'_>) -> i32 {
            if let Some(log) = ctx.data_opt::<Log>() {
                log.lock().await.push(LogElement::InnerAccess(self.0));
            }
            self.0
        }
    }

    #[derive(Clone, Copy)]
    struct Outer(Inner);

    #[Object]
    impl Outer {
        async fn inner(&self, ctx: &Context<'_>) -> Inner {
            if let Some(log) = ctx.data_opt::<Log>() {
                log.lock().await.push(LogElement::OuterAccess(self.0 .0));
            }
            self.0
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn outers(&self, ctx: &Context<'_>) -> impl Stream<Item = Outer> {
            assert!(
                ctx.data_opt::<Log>().is_none(),
                "Pre-hook ran before the first message"
            );
            futures_util::stream::iter(10..13).map(Inner).map(Outer)
        }
    }

    let per_message_pre_hook = {
        let log: Arc<Mutex<Vec<LogElement>>> = Arc::clone(&log);
        let message_counter = Arc::clone(&message_counter);
        Arc::new(move || {
            let log = Arc::clone(&log);
            let message_counter = Arc::clone(&message_counter);
            async move {
                let mut message_counter = message_counter.lock().await;
                let mut data = Data::default();
                log.lock().await.push(LogElement::PreHook(*message_counter));
                data.insert(log);
                data.insert(*message_counter);
                *message_counter += 1;
                Ok(Some(data))
            }
            .boxed()
        })
    };
    let per_message_post_hook: Arc<PerMessagePostHook> = {
        let log = Arc::clone(&log);
        Arc::new(move |data| {
            let message_counter = *data.get_data::<i32>().unwrap();
            let log = Arc::clone(&log);
            async move {
                log.lock().await.push(LogElement::PostHook(message_counter));
                Ok(())
            }
            .boxed()
        })
    };

    let schema = Schema::new(Query, EmptyMutation, Subscription);
    let mut stream = schema.execute_stream_with_session_data(
        "subscription { outers { inner { value } } }",
        Arc::new(Data::default()),
        Some(per_message_pre_hook),
        Some(per_message_post_hook),
    );

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
        let log = log.lock().await;
        assert_eq!(
            *log,
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
