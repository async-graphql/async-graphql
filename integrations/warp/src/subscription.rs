use std::{future::Future, str::FromStr};

use async_graphql::{
    http::{WebSocketProtocols, WsMessage},
    Data, ObjectType, Result, Schema, SubscriptionType,
};
use futures_util::{
    future,
    future::Ready,
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
pub fn graphql_subscription<Query, Mutation, Subscription>(
    schema: Schema<Query, Mutation, Subscription>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone
where
    Query: ObjectType + Sync + Send + 'static,
    Mutation: ObjectType + Sync + Send + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    warp::ws()
        .and(graphql_protocol())
        .map(move |ws: ws::Ws, protocol| {
            let schema = schema.clone();

            let reply = ws.on_upgrade(move |socket| {
                GraphQLWebSocket::new(socket, schema, protocol)
                    .on_connection_init(default_on_connection_init)
                    .serve()
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

type DefaultOnConnInitType = fn(serde_json::Value) -> Ready<async_graphql::Result<Data>>;

fn default_on_connection_init(_: serde_json::Value) -> Ready<async_graphql::Result<Data>> {
    futures_util::future::ready(Ok(Data::default()))
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
pub struct GraphQLWebSocket<Sink, Stream, Query, Mutation, Subscription, OnInit> {
    sink: Sink,
    stream: Stream,
    protocol: WebSocketProtocols,
    schema: Schema<Query, Mutation, Subscription>,
    data: Data,
    on_init: OnInit,
}

impl<S, Query, Mutation, Subscription>
    GraphQLWebSocket<
        SplitSink<S, Message>,
        SplitStream<S>,
        Query,
        Mutation,
        Subscription,
        DefaultOnConnInitType,
    >
where
    S: Stream<Item = Result<Message, Error>> + Sink<Message>,
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
{
    /// Create a [`GraphQLWebSocket`] object.
    pub fn new(
        socket: S,
        schema: Schema<Query, Mutation, Subscription>,
        protocol: WebSocketProtocols,
    ) -> Self {
        let (sink, stream) = socket.split();
        GraphQLWebSocket::new_with_pair(sink, stream, schema, protocol)
    }
}

impl<Sink, Stream, Query, Mutation, Subscription>
    GraphQLWebSocket<Sink, Stream, Query, Mutation, Subscription, DefaultOnConnInitType>
where
    Sink: futures_util::sink::Sink<Message>,
    Stream: futures_util::stream::Stream<Item = Result<Message, Error>>,
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
{
    /// Create a [`GraphQLWebSocket`] object with sink and stream objects.
    pub fn new_with_pair(
        sink: Sink,
        stream: Stream,
        schema: Schema<Query, Mutation, Subscription>,
        protocol: WebSocketProtocols,
    ) -> Self {
        GraphQLWebSocket {
            sink,
            stream,
            protocol,
            schema,
            data: Data::default(),
            on_init: default_on_connection_init,
        }
    }
}

impl<Sink, Stream, Query, Mutation, Subscription, OnConnInit, OnConnInitFut>
    GraphQLWebSocket<Sink, Stream, Query, Mutation, Subscription, OnConnInit>
where
    Sink: futures_util::sink::Sink<Message>,
    Stream: futures_util::stream::Stream<Item = Result<Message, Error>>,
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
    OnConnInit: Fn(serde_json::Value) -> OnConnInitFut + Send + Sync + 'static,
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
    ) -> GraphQLWebSocket<Sink, Stream, Query, Mutation, Subscription, OnConnInit2>
    where
        OnConnInit2: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = async_graphql::Result<Data>> + Send + 'static,
    {
        GraphQLWebSocket {
            sink: self.sink,
            stream: self.stream,
            schema: self.schema,
            data: self.data,
            on_init: callback,
            protocol: self.protocol,
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

        let _ = async_graphql::http::WebSocket::new(self.schema.clone(), stream, self.protocol)
            .connection_data(self.data)
            .on_connection_init(self.on_init)
            .map(|msg| match msg {
                WsMessage::Text(text) => ws::Message::text(text),
                WsMessage::Close(code, status) => ws::Message::close_with(code, status),
            })
            .map(Ok)
            .forward(self.sink)
            .await;
    }
}
