use std::str::FromStr;

use async_graphql::http::{WebSocketProtocols, WsMessage, ALL_WEBSOCKET_PROTOCOLS};
use async_graphql::{Data, ObjectType, Schema, SubscriptionType};
use futures_util::future::{self, Ready};
use futures_util::{Future, SinkExt, StreamExt};
use poem::http::StatusCode;
use poem::web::websocket::{Message, WebSocket, WebSocketStream};
use poem::{http, Endpoint, FromRequest, IntoResponse, Request, RequestBody, Response, Result};

/// A GraphQL protocol extractor.
///
/// It extract GraphQL protocol from `SEC_WEBSOCKET_PROTOCOL` header.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GraphQLProtocol(WebSocketProtocols);

#[poem::async_trait]
impl<'a> FromRequest<'a> for GraphQLProtocol {
    type Error = StatusCode;

    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> Result<Self, Self::Error> {
        req.headers()
            .get(http::header::SEC_WEBSOCKET_PROTOCOL)
            .and_then(|value| value.to_str().ok())
            .and_then(|protocols| {
                protocols
                    .split(',')
                    .find_map(|p| WebSocketProtocols::from_str(p.trim()).ok())
            })
            .map(Self)
            .ok_or_else(|| StatusCode::BAD_REQUEST)
    }
}

/// A GraphQL subscription endpoint.
///
/// # Example
///
/// ```
/// use poem::{Route, get};
/// use async_graphql_poem::GraphQLSubscription;
/// use async_graphql::{EmptyMutation, Object, Schema, Subscription};
/// use futures_util::{Stream, stream};
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
pub struct GraphQLSubscription<Query, Mutation, Subscription> {
    schema: Schema<Query, Mutation, Subscription>,
}

impl<Query, Mutation, Subscription> GraphQLSubscription<Query, Mutation, Subscription> {
    /// Create a GraphQL subscription endpoint.
    pub fn new(schema: Schema<Query, Mutation, Subscription>) -> Self {
        Self { schema }
    }
}

#[poem::async_trait]
impl<Query, Mutation, Subscription> Endpoint for GraphQLSubscription<Query, Mutation, Subscription>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
{
    type Output = Result<Response>;

    async fn call(&self, req: Request) -> Self::Output {
        let (req, mut body) = req.split();
        let websocket = WebSocket::from_request(&req, &mut body).await?;
        let protocol = GraphQLProtocol::from_request(&req, &mut body).await?;
        let schema = self.schema.clone();

        let resp = websocket
            .protocols(ALL_WEBSOCKET_PROTOCOLS)
            .on_upgrade(move |stream| GraphQLWebSocket::new(stream, schema, protocol).serve())
            .into_response();
        Ok(resp)
    }
}

type DefaultOnConnInitType = fn(serde_json::Value) -> Ready<async_graphql::Result<Data>>;

fn default_on_connection_init(_: serde_json::Value) -> Ready<async_graphql::Result<Data>> {
    futures_util::future::ready(Ok(Data::default()))
}

/// A Websocket connection for GraphQL subscription.
pub struct GraphQLWebSocket<Query, Mutation, Subscription, OnConnInit> {
    schema: Schema<Query, Mutation, Subscription>,
    stream: WebSocketStream,
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
        stream: WebSocketStream,
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

        let stream = stream
            .take_while(|res| future::ready(res.is_ok()))
            .map(Result::unwrap)
            .filter_map(|msg| {
                if msg.is_text() || msg.is_binary() {
                    future::ready(Some(msg))
                } else {
                    future::ready(None)
                }
            })
            .map(Message::into_bytes)
            .boxed();

        let mut stream =
            async_graphql::http::WebSocket::new(self.schema.clone(), stream, self.protocol.0)
                .connection_data(self.data)
                .on_connection_init(self.on_connection_init)
                .map(|msg| match msg {
                    WsMessage::Text(text) => Message::text(text),
                    WsMessage::Close(code, status) => Message::close_with(code, status),
                });

        while let Some(item) = stream.next().await {
            let _ = sink.send(item).await;
        }
    }
}
