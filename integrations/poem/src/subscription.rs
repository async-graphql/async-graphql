use std::str::FromStr;

use async_graphql::http::{WebSocketProtocols, WsMessage, ALL_WEBSOCKET_PROTOCOLS};
use async_graphql::{Data, ObjectType, Schema, SubscriptionType};
use futures_util::future::{self, Ready};
use futures_util::{Future, SinkExt, StreamExt};
use poem::web::websocket::{Message, WebSocket};
use poem::{http, Endpoint, FromRequest, IntoResponse, Request, Response, Result};

/// A GraphQL subscription endpoint.
///
/// # Example
///
/// ```
/// use poem::{route, RouteMethod};
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
/// let app = route().at("/ws", RouteMethod::new().get(GraphQLSubscription::new(schema)));
/// ```
pub struct GraphQLSubscription<Query, Mutation, Subscription, F> {
    schema: Schema<Query, Mutation, Subscription>,
    initializer: F,
}

impl<Query, Mutation, Subscription>
    GraphQLSubscription<
        Query,
        Mutation,
        Subscription,
        fn(serde_json::Value) -> Ready<async_graphql::Result<Data>>,
    >
{
    /// Create a GraphQL subscription endpoint.
    pub fn new(schema: Schema<Query, Mutation, Subscription>) -> Self {
        Self {
            schema,
            initializer: |_| futures_util::future::ready(Ok(Default::default())),
        }
    }
}

impl<Query, Mutation, Subscription, F> GraphQLSubscription<Query, Mutation, Subscription, F> {
    /// With a data initialization function.
    pub fn with_initializer<F2, R>(
        self,
        initializer: F2,
    ) -> GraphQLSubscription<Query, Mutation, Subscription, F2>
    where
        F2: FnOnce(serde_json::Value) -> R + Clone + Send + Sync + 'static,
        R: Future<Output = Result<Data>> + Send + 'static,
    {
        GraphQLSubscription {
            schema: self.schema,
            initializer,
        }
    }
}

#[poem::async_trait]
impl<Query, Mutation, Subscription, F, R> Endpoint
    for GraphQLSubscription<Query, Mutation, Subscription, F>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
    F: FnOnce(serde_json::Value) -> R + Clone + Send + Sync + 'static,
    R: Future<Output = async_graphql::Result<Data>> + Send + 'static,
{
    type Output = Result<Response>;

    async fn call(&self, req: Request) -> Self::Output {
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
        let initializer = self.initializer.clone();

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

                let mut stream = async_graphql::http::WebSocket::with_data(
                    schema,
                    stream,
                    initializer,
                    protocol,
                )
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
