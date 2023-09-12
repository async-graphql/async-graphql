use std::{io::Error as IoError, str::FromStr, sync::Arc};

use async_graphql::{
    http::{WebSocketProtocols, WsMessage, ALL_WEBSOCKET_PROTOCOLS},
    Data, Executor, PerMessagePostHook, PerMessagePreHook,
};
use futures_util::{
    future::{self, BoxFuture, Ready},
    stream::{SplitSink, SplitStream},
    Future, Sink, SinkExt, Stream, StreamExt,
};
use poem::{
    http,
    http::StatusCode,
    web::websocket::{Message, WebSocket},
    Endpoint, Error, FromRequest, IntoResponse, Request, RequestBody, Response, Result,
};

/// A GraphQL protocol extractor.
///
/// It extract GraphQL protocol from `SEC_WEBSOCKET_PROTOCOL` header.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GraphQLProtocol(pub WebSocketProtocols);

#[poem::async_trait]
impl<'a> FromRequest<'a> for GraphQLProtocol {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> Result<Self> {
        req.headers()
            .get(http::header::SEC_WEBSOCKET_PROTOCOL)
            .and_then(|value| value.to_str().ok())
            .and_then(|protocols| {
                protocols
                    .split(',')
                    .find_map(|p| WebSocketProtocols::from_str(p.trim()).ok())
            })
            .map(Self)
            .ok_or_else(|| Error::from_status(StatusCode::BAD_REQUEST))
    }
}

/// A GraphQL subscription endpoint.
///
/// # Example
///
/// ```
/// use async_graphql::{EmptyMutation, Object, Schema, Subscription};
/// use async_graphql_poem::GraphQLSubscription;
/// use futures_util::{stream, Stream};
/// use poem::{get, Route};
///
/// struct Query;
///
/// #[Object]
/// impl Query {
///     async fn value(&self) -> i32 {
///         100
///     }
/// }
///
/// struct Subscription;
///
/// #[Subscription]
/// impl Subscription {
///     async fn values(&self) -> impl Stream<Item = i32> {
///         stream::iter(vec![1, 2, 3, 4, 5])
///     }
/// }
///
/// type MySchema = Schema<Query, EmptyMutation, Subscription>;
///
/// let schema = Schema::new(Query, EmptyMutation, Subscription);
/// let app = Route::new().at("/ws", get(GraphQLSubscription::new(schema)));
/// ```
pub struct GraphQLSubscription<E> {
    executor: E,
}

impl<E> GraphQLSubscription<E> {
    /// Create a GraphQL subscription endpoint.
    pub fn new(executor: E) -> Self {
        Self { executor }
    }
}

#[poem::async_trait]
impl<E> Endpoint for GraphQLSubscription<E>
where
    E: Executor,
{
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        let (req, mut body) = req.split();
        let websocket = WebSocket::from_request(&req, &mut body).await?;
        let protocol = GraphQLProtocol::from_request(&req, &mut body).await?;
        let executor = self.executor.clone();

        let resp = websocket
            .protocols(ALL_WEBSOCKET_PROTOCOLS)
            .on_upgrade(move |stream| GraphQLWebSocket::new(stream, executor, protocol).serve())
            .into_response();
        Ok(resp)
    }
}

type DefaultOnConnInitType = fn(serde_json::Value) -> Ready<async_graphql::Result<Data>>;

fn default_on_connection_init(_: serde_json::Value) -> Ready<async_graphql::Result<Data>> {
    futures_util::future::ready(Ok(Data::default()))
}

/// A Websocket connection for GraphQL subscription.
pub struct GraphQLWebSocket<Sink, Stream, E, OnConnInit> {
    sink: Sink,
    stream: Stream,
    executor: E,
    data: Data,
    on_connection_init: OnConnInit,
    protocol: GraphQLProtocol,
    per_message_pre_hook: Option<Arc<PerMessagePreHook>>,
    per_message_post_hook: Option<Arc<PerMessagePostHook>>,
}

impl<S, E> GraphQLWebSocket<SplitSink<S, Message>, SplitStream<S>, E, DefaultOnConnInitType>
where
    S: Stream<Item = Result<Message, IoError>> + Sink<Message>,
    E: Executor,
{
    /// Create a [`GraphQLWebSocket`] object.
    pub fn new(stream: S, executor: E, protocol: GraphQLProtocol) -> Self {
        let (sink, stream) = stream.split();
        GraphQLWebSocket::new_with_pair(sink, stream, executor, protocol)
    }
}

impl<Sink, Stream, E> GraphQLWebSocket<Sink, Stream, E, DefaultOnConnInitType>
where
    Sink: futures_util::sink::Sink<Message>,
    Stream: futures_util::stream::Stream<Item = Result<Message, IoError>>,
    E: Executor,
{
    /// Create a [`GraphQLWebSocket`] object with sink and stream objects.
    pub fn new_with_pair(
        sink: Sink,
        stream: Stream,
        executor: E,
        protocol: GraphQLProtocol,
    ) -> Self {
        GraphQLWebSocket {
            sink,
            stream,
            executor,
            data: Data::default(),
            on_connection_init: default_on_connection_init,
            protocol,
            per_message_pre_hook: None,
            per_message_post_hook: None,
        }
    }
}

impl<Sink, Stream, E, OnConnInit, OnConnInitFut> GraphQLWebSocket<Sink, Stream, E, OnConnInit>
where
    Sink: futures_util::sink::Sink<Message>,
    Stream: futures_util::stream::Stream<Item = Result<Message, IoError>>,
    E: Executor,
    OnConnInit: FnOnce(serde_json::Value) -> OnConnInitFut + Send + 'static,
    OnConnInitFut: Future<Output = async_graphql::Result<Data>> + Send + 'static,
{
    /// Specify the initial subscription context data, usually you can get
    /// something from the incoming request to create it.
    #[must_use]
    pub fn with_data(self, data: Data) -> Self {
        Self { data, ..self }
    }

    /// Specify a callback function to be called when the connection is
    /// initialized.
    ///
    /// You can get something from the payload of [`GQL_CONNECTION_INIT` message](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md#gql_connection_init) to create [`Data`].
    /// The data returned by this callback function will be merged with the data
    /// specified by [`with_data`].
    pub fn on_connection_init<OnConnInit2, Fut>(
        self,
        callback: OnConnInit2,
    ) -> GraphQLWebSocket<Sink, Stream, E, OnConnInit2>
    where
        OnConnInit2: FnOnce(serde_json::Value) -> Fut + Send + 'static,
        Fut: Future<Output = async_graphql::Result<Data>> + Send + 'static,
    {
        GraphQLWebSocket {
            sink: self.sink,
            stream: self.stream,
            executor: self.executor,
            data: self.data,
            on_connection_init: callback,
            protocol: self.protocol,
            per_message_pre_hook: self.per_message_pre_hook,
            per_message_post_hook: self.per_message_post_hook,
        }
    }

    /// Specify a per-message pre-hook.
    ///
    /// This hook will run for each message that the subscription stream emits, before running
    /// the resolvers. It can be used for starting a transaction, that all resolvers will use.
    #[must_use]
    pub fn per_message_pre_hook<F, Fut>(self, hook: F) -> Self
    where
        Fut: Future<Output = async_graphql::Result<Option<Data>>> + Send + 'static,
        F: Fn() -> Fut + Send + Sync + 'static,
    {
        GraphQLWebSocket {
            sink: self.sink,
            stream: self.stream,
            executor: self.executor,
            data: self.data,
            on_connection_init: self.on_connection_init,
            protocol: self.protocol,
            per_message_pre_hook: Some(Arc::new(move || Box::pin(hook()))),
            per_message_post_hook: self.per_message_post_hook,
        }
    }

    /// Specify a per-message post-hook.
    ///
    /// This hook will run for each message that the subscription stream emits, after running
    /// the resolvers. It can be used for committing a transaction, that all resolvers will use.
    #[must_use]
    pub fn per_message_post_hook<F>(self, hook: F) -> Self
    where
        F: for<'a> Fn(&'a Data) -> BoxFuture<'a, async_graphql::Result<()>> + Send + Sync + 'static,
    {
        GraphQLWebSocket {
            sink: self.sink,
            stream: self.stream,
            executor: self.executor,
            data: self.data,
            on_connection_init: self.on_connection_init,
            protocol: self.protocol,
            per_message_pre_hook: self.per_message_pre_hook,
            per_message_post_hook: Some(Arc::new(hook)),
        }
    }

    /// Processing subscription requests.
    pub async fn serve(self) {
        let stream = self
            .stream
            .take_while(|res| future::ready(res.is_ok()))
            .map(Result::unwrap)
            .filter_map(|msg| {
                if msg.is_text() || msg.is_binary() {
                    future::ready(Some(msg))
                } else {
                    future::ready(None)
                }
            })
            .map(Message::into_bytes);

        let stream =
            async_graphql::http::WebSocket::new(self.executor.clone(), stream, self.protocol.0)
                .connection_data(self.data)
                .on_connection_init(self.on_connection_init)
                .per_message_pre_hook(self.per_message_pre_hook)
                .per_message_post_hook(self.per_message_post_hook)
                .map(|msg| match msg {
                    WsMessage::Text(text) => Message::text(text),
                    WsMessage::Close(code, status) => Message::close_with(code, status),
                });

        let sink = self.sink;
        futures_util::pin_mut!(stream, sink);

        while let Some(item) = stream.next().await {
            let _ = sink.send(item).await;
        }
    }
}
