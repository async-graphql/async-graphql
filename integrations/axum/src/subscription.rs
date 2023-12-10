use std::{borrow::Cow, convert::Infallible, future::Future, str::FromStr};

use async_graphql::{
    futures_util::task::{Context, Poll},
    http::{WebSocketProtocols, WsMessage, ALL_WEBSOCKET_PROTOCOLS},
    Data, Executor, Result,
};
use axum::{
    body::{Body, HttpBody},
    extract::{
        ws::{CloseFrame, Message},
        FromRequestParts, WebSocketUpgrade,
    },
    http::{self, request::Parts, Request, Response, StatusCode},
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
impl<S> FromRequestParts<S> for GraphQLProtocol
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .headers
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
pub struct GraphQLSubscription<E> {
    executor: E,
}

impl<E> Clone for GraphQLSubscription<E>
where
    E: Executor,
{
    fn clone(&self) -> Self {
        Self {
            executor: self.executor.clone(),
        }
    }
}

impl<E> GraphQLSubscription<E>
where
    E: Executor,
{
    /// Create a GraphQL subscription service.
    pub fn new(executor: E) -> Self {
        Self { executor }
    }
}

impl<B, E> Service<Request<B>> for GraphQLSubscription<E>
where
    B: HttpBody + Send + 'static,
    E: Executor,
{
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let executor = self.executor.clone();

        Box::pin(async move {
            let (mut parts, _body) = req.into_parts();

            let protocol = match GraphQLProtocol::from_request_parts(&mut parts, &()).await {
                Ok(protocol) => protocol,
                Err(err) => return Ok(err.into_response()),
            };
            let upgrade = match WebSocketUpgrade::from_request_parts(&mut parts, &()).await {
                Ok(protocol) => protocol,
                Err(err) => return Ok(err.into_response()),
            };

            let executor = executor.clone();

            let resp = upgrade
                .protocols(ALL_WEBSOCKET_PROTOCOLS)
                .on_upgrade(move |stream| {
                    GraphQLWebSocket::new(stream, executor, protocol).serve()
                });
            Ok(resp.into_response())
        })
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
}

impl<S, E> GraphQLWebSocket<SplitSink<S, Message>, SplitStream<S>, E, DefaultOnConnInitType>
where
    S: Stream<Item = Result<Message, Error>> + Sink<Message>,
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
    Stream: futures_util::stream::Stream<Item = Result<Message, Error>>,
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
        }
    }
}

impl<Sink, Stream, E, OnConnInit, OnConnInitFut> GraphQLWebSocket<Sink, Stream, E, OnConnInit>
where
    Sink: futures_util::sink::Sink<Message>,
    Stream: futures_util::stream::Stream<Item = Result<Message, Error>>,
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
            async_graphql::http::WebSocket::new(self.executor.clone(), input, self.protocol.0)
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
