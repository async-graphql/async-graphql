use std::borrow::Cow;
use std::convert::Infallible;
use std::future::Future;
use std::str::FromStr;

use async_graphql::futures_util::task::{Context, Poll};
use async_graphql::http::{WebSocketProtocols, WsMessage, ALL_WEBSOCKET_PROTOCOLS};
use async_graphql::{Data, ObjectType, Result, Schema, SubscriptionType};
use axum::body::{box_body, BoxBody, HttpBody};
use axum::extract::ws::{CloseFrame, Message, WebSocket};
use axum::extract::{FromRequest, RequestParts, WebSocketUpgrade};
use axum::http::{self, Request, Response, StatusCode};
use axum::response::IntoResponse;
use futures_util::future::{BoxFuture, Ready};
use futures_util::{future, SinkExt, StreamExt};
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
            .and_then(|headers| headers.get(http::header::SEC_WEBSOCKET_PROTOCOL))
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

impl<Query, Mutation, Subscription> GraphQLSubscription<Query, Mutation, Subscription> {
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
                Err(err) => return Ok(err.into_response().map(box_body)),
            };
            let upgrade = match WebSocketUpgrade::from_request(&mut parts).await {
                Ok(protocol) => protocol,
                Err(err) => return Ok(err.into_response().map(box_body)),
            };

            let schema = schema.clone();

            let resp = upgrade
                .protocols(ALL_WEBSOCKET_PROTOCOLS)
                .on_upgrade(move |stream| GraphQLWebSocket::new(stream, schema, protocol).serve());
            Ok(resp.into_response().map(box_body))
        })
    }
}

type DefaultOnConnInitType = fn(serde_json::Value) -> Ready<async_graphql::Result<Data>>;

fn default_on_connection_init(_: serde_json::Value) -> Ready<async_graphql::Result<Data>> {
    futures_util::future::ready(Ok(Data::default()))
}

/// A Websocket connection for GraphQL subscription.
pub struct GraphQLWebSocket<Query, Mutation, Subscription, OnConnInit> {
    schema: Schema<Query, Mutation, Subscription>,
    stream: WebSocket,
    data: Data,
    on_connection_init: OnConnInit,
    protocol: GraphQLProtocol,
}

impl<Query, Mutation, Subscription>
    GraphQLWebSocket<Query, Mutation, Subscription, DefaultOnConnInitType>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
{
    /// Create a [`GraphQLWebSocket`] object.
    pub fn new(
        stream: WebSocket,
        schema: Schema<Query, Mutation, Subscription>,
        protocol: GraphQLProtocol,
    ) -> Self {
        GraphQLWebSocket {
            schema,
            stream,
            data: Data::default(),
            on_connection_init: default_on_connection_init,
            protocol,
        }
    }
}

impl<Query, Mutation, Subscription, OnConnInit, OnConnInitFut>
    GraphQLWebSocket<Query, Mutation, Subscription, OnConnInit>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
    OnConnInit: Fn(serde_json::Value) -> OnConnInitFut + Send + Sync + 'static,
    OnConnInitFut: Future<Output = async_graphql::Result<Data>> + Send + 'static,
{
    /// Specify the initial subscription context data, usually you can get something from the
    /// incoming request to create it.
    pub fn with_data(self, data: Data) -> Self {
        Self { data, ..self }
    }

    /// Specify a callback function to be called when the connection is initialized.
    ///
    /// You can get something from the payload of [`GQL_CONNECTION_INIT` message](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md#gql_connection_init) to create [`Data`].
    /// The data returned by this callback function will be merged with the data specified by [`with_data`].
    pub fn on_connection_init<OnConnInit2, Fut>(
        self,
        callback: OnConnInit2,
    ) -> GraphQLWebSocket<Query, Mutation, Subscription, OnConnInit2>
    where
        OnConnInit2: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = async_graphql::Result<Data>> + Send + 'static,
    {
        GraphQLWebSocket {
            schema: self.schema,
            stream: self.stream,
            data: self.data,
            on_connection_init: callback,
            protocol: self.protocol,
        }
    }

    /// Processing subscription requests.
    pub async fn serve(self) {
        let (mut sink, stream) = self.stream.split();

        let input = stream
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

        let mut stream =
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

        while let Some(item) = stream.next().await {
            let _ = sink.send(item).await;
        }
    }
}
