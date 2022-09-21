use std::{borrow::Cow, convert::Infallible, future::Future, str::FromStr};

use async_graphql::{
    futures_util::task::{Context, Poll},
    http::{WebSocketProtocols, WsMessage, ALL_WEBSOCKET_PROTOCOLS},
    Data, ObjectType, Result, Schema, SubscriptionType,
};
use axum::{
    body::{boxed, BoxBody, HttpBody},
    extract::{
        ws::{CloseFrame, Message},
        FromRequest, RequestParts, WebSocketUpgrade,
    },
    http::{self, Request, Response, StatusCode},
    response::IntoResponse,
    Error,
};
use futures_util::{
    future,
    future::{BoxFuture, Ready},
    stream::{SplitSink, SplitStream},
    Sink, SinkExt, Stream, StreamExt,
};
use tower_service::Service;

/// A GraphQL protocol extractor.
///
/// It extract GraphQL protocol from `SEC_WEBSOCKET_PROTOCOL` header.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GraphQLProtocol(WebSocketProtocols);

#[async_trait::async_trait]
impl<B: Send> FromRequest<B> for GraphQLProtocol {
    type Rejection = StatusCode;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        req.headers()
            .get(http::header::SEC_WEBSOCKET_PROTOCOL)
            .and_then(|value| value.to_str().ok())
            .and_then(|protocols| {
                protocols
                    .split(',')
                    .find_map(|p| WebSocketProtocols::from_str(p.trim()).ok())
            })
            .map(Self)
            .ok_or(StatusCode::BAD_REQUEST)
    }
}

/// A GraphQL subscription service.
pub struct GraphQLSubscription<Query, Mutation, Subscription> {
    schema: Schema<Query, Mutation, Subscription>,
}

impl<Query, Mutation, Subscription> Clone for GraphQLSubscription<Query, Mutation, Subscription>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
{
    fn clone(&self) -> Self {
        Self {
            schema: self.schema.clone(),
        }
    }
}

impl<Query, Mutation, Subscription> GraphQLSubscription<Query, Mutation, Subscription>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
{
    /// Create a GraphQL subscription service.
    pub fn new(schema: Schema<Query, Mutation, Subscription>) -> Self {
        Self { schema }
    }
}

impl<B, Query, Mutation, Subscription> Service<Request<B>>
    for GraphQLSubscription<Query, Mutation, Subscription>
where
    B: HttpBody + Send + 'static,
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
{
    type Response = Response<BoxBody>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let schema = self.schema.clone();

        Box::pin(async move {
            let mut parts = RequestParts::new(req);
            let protocol = match GraphQLProtocol::from_request(&mut parts).await {
                Ok(protocol) => protocol,
                Err(err) => return Ok(err.into_response().map(boxed)),
            };
            let upgrade = match WebSocketUpgrade::from_request(&mut parts).await {
                Ok(protocol) => protocol,
                Err(err) => return Ok(err.into_response().map(boxed)),
            };

            let schema = schema.clone();

            let resp = upgrade
                .protocols(ALL_WEBSOCKET_PROTOCOLS)
                .on_upgrade(move |stream| GraphQLWebSocket::new(stream, schema, protocol).serve());
            Ok(resp.into_response().map(boxed))
        })
    }
}

type DefaultOnConnInitType = fn(serde_json::Value) -> Ready<async_graphql::Result<Data>>;

fn default_on_connection_init(_: serde_json::Value) -> Ready<async_graphql::Result<Data>> {
    futures_util::future::ready(Ok(Data::default()))
}

/// A Websocket connection for GraphQL subscription.
pub struct GraphQLWebSocket<Sink, Stream, Query, Mutation, Subscription, OnConnInit> {
    sink: Sink,
    stream: Stream,
    schema: Schema<Query, Mutation, Subscription>,
    data: Data,
    on_connection_init: OnConnInit,
    protocol: GraphQLProtocol,
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
        stream: S,
        schema: Schema<Query, Mutation, Subscription>,
        protocol: GraphQLProtocol,
    ) -> Self {
        let (sink, stream) = stream.split();
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
        protocol: GraphQLProtocol,
    ) -> Self {
        GraphQLWebSocket {
            sink,
            stream,
            schema,
            data: Data::default(),
            on_connection_init: default_on_connection_init,
            protocol,
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
    ) -> GraphQLWebSocket<Sink, Stream, Query, Mutation, Subscription, OnConnInit2>
    where
        OnConnInit2: FnOnce(serde_json::Value) -> Fut + Send + 'static,
        Fut: Future<Output = async_graphql::Result<Data>> + Send + 'static,
    {
        GraphQLWebSocket {
            sink: self.sink,
            stream: self.stream,
            schema: self.schema,
            data: self.data,
            on_connection_init: callback,
            protocol: self.protocol,
        }
    }

    /// Processing subscription requests.
    pub async fn serve(self) {
        let input = self
            .stream
            .take_while(|res| future::ready(res.is_ok()))
            .map(Result::unwrap)
            .filter_map(|msg| {
                if let Message::Text(_) | Message::Binary(_) = msg {
                    future::ready(Some(msg))
                } else {
                    future::ready(None)
                }
            })
            .map(Message::into_data);

        let stream =
            async_graphql::http::WebSocket::new(self.schema.clone(), input, self.protocol.0)
                .connection_data(self.data)
                .on_connection_init(self.on_connection_init)
                .map(|msg| match msg {
                    WsMessage::Text(text) => Message::Text(text),
                    WsMessage::Close(code, status) => Message::Close(Some(CloseFrame {
                        code,
                        reason: Cow::from(status),
                    })),
                });

        let sink = self.sink;
        futures_util::pin_mut!(stream, sink);

        while let Some(item) = stream.next().await {
            let _ = sink.send(item).await;
        }
    }
}
