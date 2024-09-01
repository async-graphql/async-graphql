use std::{future::Future, str::FromStr, time::Duration};

use async_graphql::{
    http::{
        default_on_connection_init, default_on_ping, DefaultOnConnInitType, DefaultOnPingType,
        WebSocketProtocols, WsMessage,
    },
    Data, Executor, Result,
};
use futures_util::{
    future,
    stream::{SplitSink, SplitStream},
    Sink, Stream, StreamExt,
};
use warp::{filters::ws, ws::Message, Error, Filter, Rejection, Reply};

/// GraphQL subscription filter
///
/// # Examples
///
/// ```no_run
/// use std::time::Duration;
///
/// use async_graphql::*;
/// use async_graphql_warp::*;
/// use futures_util::stream::{Stream, StreamExt};
/// use warp::Filter;
///
/// struct QueryRoot;
///
/// #[Object]
/// impl QueryRoot {
///     async fn value(&self) -> i32 {
///         // A GraphQL Object type must define one or more fields.
///         100
///     }
/// }
///
/// struct SubscriptionRoot;
///
/// #[Subscription]
/// impl SubscriptionRoot {
///     async fn tick(&self) -> impl Stream<Item = String> {
///         async_stream::stream! {
///             let mut interval = tokio::time::interval(Duration::from_secs(1));
///             loop {
///                 let n = interval.tick().await;
///                 yield format!("{}", n.elapsed().as_secs_f32());
///             }
///         }
///     }
/// }
///
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
/// let filter =
///     async_graphql_warp::graphql_subscription(schema).or(warp::any().map(|| "Hello, World!"));
/// warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
/// # });
/// ```
pub fn graphql_subscription<E>(
    executor: E,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone
where
    E: Executor,
{
    warp::ws()
        .and(graphql_protocol())
        .map(move |ws: ws::Ws, protocol| {
            let executor = executor.clone();

            let reply = ws.on_upgrade(move |socket| {
                GraphQLWebSocket::new(socket, executor, protocol).serve()
            });

            warp::reply::with_header(
                reply,
                "Sec-WebSocket-Protocol",
                protocol.sec_websocket_protocol(),
            )
        })
}

/// Create a `Filter` that parse [WebSocketProtocols] from
/// `sec-websocket-protocol` header.
pub fn graphql_protocol() -> impl Filter<Extract = (WebSocketProtocols,), Error = Rejection> + Clone
{
    warp::header::optional::<String>("sec-websocket-protocol").map(|protocols: Option<String>| {
        protocols
            .and_then(|protocols| {
                protocols
                    .split(',')
                    .find_map(|p| WebSocketProtocols::from_str(p.trim()).ok())
            })
            .unwrap_or(WebSocketProtocols::SubscriptionsTransportWS)
    })
}

/// A Websocket connection for GraphQL subscription.
///
/// # Examples
///
/// ```no_run
/// use std::time::Duration;
///
/// use async_graphql::*;
/// use async_graphql_warp::*;
/// use futures_util::stream::{Stream, StreamExt};
/// use warp::{ws, Filter};
///
/// struct QueryRoot;
///
/// #[Object]
/// impl QueryRoot {
///     async fn value(&self) -> i32 {
///         // A GraphQL Object type must define one or more fields.
///         100
///     }
/// }
///
/// struct SubscriptionRoot;
///
/// #[Subscription]
/// impl SubscriptionRoot {
///     async fn tick(&self) -> impl Stream<Item = String> {
///         async_stream::stream! {
///             let mut interval = tokio::time::interval(Duration::from_secs(1));
///             loop {
///                 let n = interval.tick().await;
///                 yield format!("{}", n.elapsed().as_secs_f32());
///             }
///         }
///     }
/// }
///
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
///
/// let filter = warp::ws()
///     .and(graphql_protocol())
///     .map(move |ws: ws::Ws, protocol| {
///         let schema = schema.clone();
///
///         let reply = ws
///             .on_upgrade(move |socket| GraphQLWebSocket::new(socket, schema, protocol).serve());
///
///         warp::reply::with_header(
///             reply,
///             "Sec-WebSocket-Protocol",
///             protocol.sec_websocket_protocol(),
///         )
///     });
///
/// warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
/// # });
/// ```
pub struct GraphQLWebSocket<Sink, Stream, E, OnInit, OnPing> {
    sink: Sink,
    stream: Stream,
    protocol: WebSocketProtocols,
    executor: E,
    data: Data,
    on_init: OnInit,
    on_ping: OnPing,
    keepalive_timeout: Option<Duration>,
}

impl<S, E>
    GraphQLWebSocket<
        SplitSink<S, Message>,
        SplitStream<S>,
        E,
        DefaultOnConnInitType,
        DefaultOnPingType,
    >
where
    S: Stream<Item = Result<Message, Error>> + Sink<Message>,
    E: Executor,
{
    /// Create a [`GraphQLWebSocket`] object.
    pub fn new(socket: S, executor: E, protocol: WebSocketProtocols) -> Self {
        let (sink, stream) = socket.split();
        GraphQLWebSocket::new_with_pair(sink, stream, executor, protocol)
    }
}

impl<Sink, Stream, E> GraphQLWebSocket<Sink, Stream, E, DefaultOnConnInitType, DefaultOnPingType>
where
    Sink: futures_util::sink::Sink<Message>,
    Stream: futures_util::stream::Stream<Item = Result<Message, Error>>,
    E: Executor,
{
    /// Create a [`GraphQLWebSocket`] object with sink and stream objects.
    pub fn new_with_pair(
        sink: Sink,
        stream: Stream,
        executor: E,
        protocol: WebSocketProtocols,
    ) -> Self {
        GraphQLWebSocket {
            sink,
            stream,
            protocol,
            executor,
            data: Data::default(),
            on_init: default_on_connection_init,
            on_ping: default_on_ping,
            keepalive_timeout: None,
        }
    }
}

impl<Sink, Stream, E, OnConnInit, OnConnInitFut, OnPing, OnPingFut>
    GraphQLWebSocket<Sink, Stream, E, OnConnInit, OnPing>
where
    Sink: futures_util::sink::Sink<Message>,
    Stream: futures_util::stream::Stream<Item = Result<Message, Error>>,
    E: Executor,
    OnConnInit: FnOnce(serde_json::Value) -> OnConnInitFut + Send + 'static,
    OnConnInitFut: Future<Output = async_graphql::Result<Data>> + Send + 'static,
    OnPing: FnOnce(Option<&Data>, Option<serde_json::Value>) -> OnPingFut + Clone + Send + 'static,
    OnPingFut: Future<Output = async_graphql::Result<Option<serde_json::Value>>> + Send + 'static,
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
    #[must_use]
    pub fn on_connection_init<F, R>(
        self,
        callback: F,
    ) -> GraphQLWebSocket<Sink, Stream, E, F, OnPing>
    where
        F: FnOnce(serde_json::Value) -> R + Send + 'static,
        R: Future<Output = async_graphql::Result<Data>> + Send + 'static,
    {
        GraphQLWebSocket {
            sink: self.sink,
            stream: self.stream,
            executor: self.executor,
            data: self.data,
            on_init: callback,
            on_ping: self.on_ping,
            protocol: self.protocol,
            keepalive_timeout: self.keepalive_timeout,
        }
    }

    /// Specify a ping callback function.
    ///
    /// This function if present, will be called with the data sent by the
    /// client in the [`Ping` message](https://github.com/enisdenjo/graphql-ws/blob/master/PROTOCOL.md#ping).
    ///
    /// The function should return the data to be sent in the [`Pong` message](https://github.com/enisdenjo/graphql-ws/blob/master/PROTOCOL.md#pong).
    ///
    /// NOTE: Only used for the `graphql-ws` protocol.
    #[must_use]
    pub fn on_ping<F, R>(self, callback: F) -> GraphQLWebSocket<Sink, Stream, E, OnConnInit, F>
    where
        F: FnOnce(Option<&Data>, Option<serde_json::Value>) -> R + Send + Clone + 'static,
        R: Future<Output = Result<Option<serde_json::Value>>> + Send + 'static,
    {
        GraphQLWebSocket {
            sink: self.sink,
            stream: self.stream,
            executor: self.executor,
            data: self.data,
            on_init: self.on_init,
            on_ping: callback,
            protocol: self.protocol,
            keepalive_timeout: self.keepalive_timeout,
        }
    }

    /// Sets a timeout for receiving an acknowledgement of the keep-alive ping.
    ///
    /// If the ping is not acknowledged within the timeout, the connection will
    /// be closed.
    ///
    /// NOTE: Only used for the `graphql-ws` protocol.
    #[must_use]
    pub fn keepalive_timeout(self, timeout: impl Into<Option<Duration>>) -> Self {
        Self {
            keepalive_timeout: timeout.into(),
            ..self
        }
    }

    /// Processing subscription requests.
    pub async fn serve(self) {
        let stream = self
            .stream
            .take_while(|msg| future::ready(msg.is_ok()))
            .map(Result::unwrap)
            .filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
            .map(ws::Message::into_bytes);

        let _ = async_graphql::http::WebSocket::new(self.executor.clone(), stream, self.protocol)
            .connection_data(self.data)
            .on_connection_init(self.on_init)
            .on_ping(self.on_ping)
            .keepalive_timeout(self.keepalive_timeout)
            .map(|msg| match msg {
                WsMessage::Text(text) => ws::Message::text(text),
                WsMessage::Close(code, status) => ws::Message::close_with(code, status),
            })
            .map(Ok)
            .forward(self.sink)
            .await;
    }
}
