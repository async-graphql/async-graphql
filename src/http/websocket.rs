//! WebSocket transport for subscription

use crate::resolver_utils::ObjectType;
use crate::{Data, FieldResult, Request, Response, Schema, SubscriptionType};
use futures::channel::mpsc;
use futures::task::{Context, Poll};
use futures::{Future, Sink, SinkExt, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::pin::Pin;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
struct OperationMessage<'a, T> {
    #[serde(rename = "type")]
    ty: &'a str,

    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<T>,
}

type SubscriptionStreams = HashMap<String, Pin<Box<dyn Stream<Item = Response> + Send>>>;

type HandleRequestBoxFut<'a> =
    Pin<Box<dyn Future<Output = FieldResult<WSContext<'a>>> + Send + 'a>>;

type InitializerFn = Arc<dyn Fn(serde_json::Value) -> FieldResult<Data> + Send + Sync>;

/// A wrapper around an underlying raw stream which implements the WebSocket protocol.
///
/// Only Text messages can be transmitted. You can use `futures::stream::StreamExt::split` function
/// to splits this object into separate Sink and Stream objects.
pub struct WebSocketStream {
    tx: mpsc::UnboundedSender<String>,
    rx: Pin<Box<dyn Stream<Item = String> + Send>>,
}

impl Sink<String> for WebSocketStream {
    type Error = mpsc::SendError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.tx.poll_ready_unpin(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: String) -> Result<(), Self::Error> {
        self.tx.start_send(item)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.tx.poll_flush_unpin(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.tx.poll_close_unpin(cx)
    }
}

impl Stream for WebSocketStream {
    type Item = String;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_next_unpin(cx)
    }
}

/// Create a websocket transport.
pub fn create<Query, Mutation, Subscription>(
    schema: &Schema<Query, Mutation, Subscription>,
) -> WebSocketStream
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    create_with_initializer(schema, |_| Ok(Default::default()))
}

/// Create a websocket transport and specify a context initialization function.
pub fn create_with_initializer<Query, Mutation, Subscription>(
    schema: &Schema<Query, Mutation, Subscription>,
    initializer: impl Fn(serde_json::Value) -> FieldResult<Data> + Send + Sync + 'static,
) -> WebSocketStream
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    let schema = schema.clone();
    let (tx, rx) = mpsc::unbounded();
    let stream = async_stream::stream! {
        let mut streams = Default::default();
        let mut send_buf = Default::default();
        let mut data = Arc::new(Data::default());
        let mut inner_stream = SubscriptionStream {
            schema: &schema,
            initializer: Arc::new(initializer),
            rx_bytes: rx,
            handle_request_fut: None,
            ctx: Some(WSContext {
                streams: &mut streams,
                send_buf: &mut send_buf,
                ctx_data: &mut data,
            }),
        };
        while let Some(data) = inner_stream.next().await {
            yield data;
        }
    };
    WebSocketStream {
        tx,
        rx: Box::pin(stream),
    }
}

struct WSContext<'a> {
    streams: &'a mut SubscriptionStreams,
    send_buf: &'a mut VecDeque<String>,
    ctx_data: &'a mut Arc<Data>,
}

fn send_message<T: Serialize>(send_buf: &mut VecDeque<String>, msg: &T) {
    if let Ok(data) = serde_json::to_string(msg) {
        send_buf.push_back(data);
    }
}

#[allow(missing_docs)]
#[allow(clippy::type_complexity)]
struct SubscriptionStream<'a, Query, Mutation, Subscription> {
    schema: &'a Schema<Query, Mutation, Subscription>,
    initializer: InitializerFn,
    rx_bytes: mpsc::UnboundedReceiver<String>,
    handle_request_fut: Option<HandleRequestBoxFut<'a>>,
    ctx: Option<WSContext<'a>>,
}

impl<'a, Query, Mutation, Subscription> Stream
    for SubscriptionStream<'a, Query, Mutation, Subscription>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    type Item = String;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = &mut *self;

        loop {
            // receive bytes
            if let Some(ctx) = &mut this.ctx {
                if let Some(bytes) = ctx.send_buf.pop_front() {
                    return Poll::Ready(Some(bytes));
                }
            }

            if let Some(handle_request_fut) = &mut this.handle_request_fut {
                match handle_request_fut.as_mut().poll(cx) {
                    Poll::Ready(Ok(ctx)) => {
                        this.ctx = Some(ctx);
                        this.handle_request_fut = None;
                        continue;
                    }
                    Poll::Ready(Err(_)) => return Poll::Ready(None),
                    Poll::Pending => {}
                }
            } else {
                match Pin::new(&mut this.rx_bytes).poll_next(cx) {
                    Poll::Ready(Some(data)) => {
                        let ctx = this.ctx.take().unwrap();
                        this.handle_request_fut = Some(Box::pin(handle_request(
                            this.schema.clone(),
                            this.initializer.clone(),
                            ctx,
                            data,
                        )));
                        continue;
                    }
                    Poll::Ready(None) => return Poll::Ready(None),
                    Poll::Pending => {}
                }
            }

            // receive msg
            if let Some(ctx) = &mut this.ctx {
                let mut closed = Vec::new();

                for (id, incoming_stream) in ctx.streams.iter_mut() {
                    match incoming_stream.as_mut().poll_next(cx) {
                        Poll::Ready(Some(res)) => {
                            if let Some(err) = &res.error {
                                closed.push(id.to_string());
                                send_message(
                                    ctx.send_buf,
                                    &OperationMessage {
                                        ty: "error",
                                        id: Some(id.to_string()),
                                        payload: Some(err),
                                    },
                                );
                            } else {
                                send_message(
                                    ctx.send_buf,
                                    &OperationMessage {
                                        ty: "data",
                                        id: Some(id.to_string()),
                                        payload: Some(&res),
                                    },
                                );
                            }
                        }
                        Poll::Ready(None) => {
                            closed.push(id.to_string());
                            send_message(
                                ctx.send_buf,
                                &OperationMessage {
                                    ty: "complete",
                                    id: Some(id.to_string()),
                                    payload: Option::<serde_json::Value>::None,
                                },
                            );
                        }
                        Poll::Pending => {}
                    }
                }

                for id in closed {
                    ctx.streams.remove(&id);
                }

                if !ctx.send_buf.is_empty() {
                    continue;
                }
            }

            return Poll::Pending;
        }
    }
}

async fn handle_request<'a, Query, Mutation, Subscription>(
    schema: Schema<Query, Mutation, Subscription>,
    initializer: InitializerFn,
    ctx: WSContext<'a>,
    data: String,
) -> FieldResult<WSContext<'a>>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    match serde_json::from_str::<OperationMessage<serde_json::Value>>(&data) {
        Ok(msg) => match msg.ty {
            "connection_init" => {
                if let Some(payload) = msg.payload {
                    *ctx.ctx_data = Arc::new(initializer(payload)?);
                }
                send_message(
                    ctx.send_buf,
                    &OperationMessage {
                        ty: "connection_ack",
                        id: None,
                        payload: Option::<serde_json::Value>::None,
                    },
                );
            }
            "start" => {
                if let (Some(id), Some(payload)) = (msg.id, msg.payload) {
                    if let Ok(request) = serde_json::from_value::<Request>(payload) {
                        let stream = schema
                            .execute_stream_with_ctx_data(request, ctx.ctx_data.clone())
                            .boxed();
                        ctx.streams.insert(id, stream);
                    }
                }
            }
            "stop" => {
                if let Some(id) = msg.id {
                    if ctx.streams.remove(&id).is_some() {
                        send_message(
                            ctx.send_buf,
                            &OperationMessage {
                                ty: "complete",
                                id: Some(id),
                                payload: Option::<serde_json::Value>::None,
                            },
                        );
                    }
                }
            }
            "connection_terminate" => return Err("connection_terminate".into()),
            _ => return Err("Unknown op".into()),
        },
        Err(err) => return Err(err.into()),
    }

    Ok(ctx)
}
