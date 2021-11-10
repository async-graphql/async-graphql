use std::str::FromStr;

use async_graphql::http::{WebSocketProtocols, WsMessage, ALL_WEBSOCKET_PROTOCOLS};
use async_graphql::{Data, ObjectType, Schema, SubscriptionType};
use futures_util::future::{self, Ready};
use futures_util::{Future, SinkExt, StreamExt};
use poem::http::StatusCode;
use poem::web::websocket::{Message, WebSocket};
use poem::{http, Endpoint, Error, FromRequest, IntoResponse, Request, Response, Result};

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
pub struct GraphQLSubscription<Query, Mutation, Subscription, OnCreate, OnInit> {
    schema: Schema<Query, Mutation, Subscription>,
    on_connection_create: OnCreate,
    on_connection_init: OnInit,
}

type DefaultOnConnCreateType = fn(&Request) -> Ready<Result<Data>>;

fn default_on_connection_create(_: &Request) -> Ready<Result<Data>> {
    futures_util::future::ready(Ok(Data::default()))
}

type DefaultOnConnInitType = fn(serde_json::Value) -> Ready<Result<Data>>;

fn default_on_connection_init(_: serde_json::Value) -> Ready<Result<Data>> {
    futures_util::future::ready(Ok(Data::default()))
}

impl<Query, Mutation, Subscription>
    GraphQLSubscription<
        Query,
        Mutation,
        Subscription,
        DefaultOnConnCreateType,
        DefaultOnConnInitType,
    >
{
    /// Create a GraphQL subscription endpoint.
    pub fn new(schema: Schema<Query, Mutation, Subscription>) -> Self {
        Self {
            schema,
            on_connection_create: default_on_connection_create,
            on_connection_init: default_on_connection_init,
        }
    }
}

impl<Query, Mutation, Subscription, OnCreate, OnInit>
    GraphQLSubscription<Query, Mutation, Subscription, OnCreate, OnInit>
{
    /// Specify the callback function to be called when the connection is created.
    ///
    /// You can get something from the incoming request to create [`Data`].
    pub fn on_connection_create<OnCreate2, Fut>(
        self,
        callback: OnCreate2,
    ) -> GraphQLSubscription<Query, Mutation, Subscription, OnCreate2, OnInit>
    where
        OnCreate2: Fn(&Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Data>> + Send + 'static,
    {
        GraphQLSubscription {
            schema: self.schema,
            on_connection_create: callback,
            on_connection_init: self.on_connection_init,
        }
    }

    /// Specify a callback function to be called when the connection is initialized.
    ///
    /// You can get something from the payload of [`GQL_CONNECTION_INIT` message](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md#gql_connection_init) to create [`Data`].
    pub fn on_connection_init<OnInit2, Fut>(
        self,
        callback: OnInit2,
    ) -> GraphQLSubscription<Query, Mutation, Subscription, OnCreate, OnInit2>
    where
        OnInit2: FnOnce(serde_json::Value) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<Data>> + Send + 'static,
    {
        GraphQLSubscription {
            schema: self.schema,
            on_connection_create: self.on_connection_create,
            on_connection_init: callback,
        }
    }
}

#[poem::async_trait]
impl<Query, Mutation, Subscription, OnCreate, OnCreateFut, OnInit, OnInitFut> Endpoint
    for GraphQLSubscription<Query, Mutation, Subscription, OnCreate, OnInit>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
    OnCreate: Fn(&Request) -> OnCreateFut + Send + Sync + 'static,
    OnCreateFut: Future<Output = async_graphql::Result<Data>> + Send + 'static,
    OnInit: FnOnce(serde_json::Value) -> OnInitFut + Clone + Send + Sync + 'static,
    OnInitFut: Future<Output = async_graphql::Result<Data>> + Send + 'static,
{
    type Output = Result<Response>;

    async fn call(&self, req: Request) -> Self::Output {
        let data = (self.on_connection_create)(&req)
            .await
            .map_err(|_| Error::new(StatusCode::BAD_REQUEST))?;

        let (req, mut body) = req.split();
        let websocket = WebSocket::from_request(&req, &mut body).await?;
        let protocol = req
            .headers()
            .get(http::header::SEC_WEBSOCKET_PROTOCOL)
            .and_then(|value| value.to_str().ok())
            .and_then(|protocols| {
                protocols
                    .split(',')
                    .find_map(|p| WebSocketProtocols::from_str(p.trim()).ok())
            })
            .unwrap_or(WebSocketProtocols::SubscriptionsTransportWS);
        let schema = self.schema.clone();
        let on_connection_init = self.on_connection_init.clone();

        let resp = websocket
            .protocols(ALL_WEBSOCKET_PROTOCOLS)
            .on_upgrade(move |socket| async move {
                let (mut sink, stream) = socket.split();

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

                let mut stream = async_graphql::http::WebSocket::new(schema, stream, protocol)
                    .connection_data(data)
                    .on_connection_init(on_connection_init)
                    .map(|msg| match msg {
                        WsMessage::Text(text) => Message::text(text),
                        WsMessage::Close(code, status) => Message::close_with(code, status),
                    });

                while let Some(item) = stream.next().await {
                    let _ = sink.send(item).await;
                }
            })
            .into_response();

        Ok(resp)
    }
}
