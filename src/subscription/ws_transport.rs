use crate::context::Data;
use crate::http::{GQLError, GQLRequest, GQLResponse};
use crate::{
    ConnectionTransport, Error, FieldError, FieldResult, ObjectType, QueryBuilder, QueryError,
    QueryResponse, Result, Schema, SubscriptionStreams, SubscriptionType, Variables,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
struct OperationMessage {
    #[serde(rename = "type")]
    ty: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<serde_json::Value>,
}

/// WebSocket transport for subscription
#[derive(Default)]
pub struct WebSocketTransport {
    id_to_sid: HashMap<String, usize>,
    sid_to_id: HashMap<usize, String>,
    data: Arc<Data>,
    init_context_data: Option<Box<dyn Fn(serde_json::Value) -> FieldResult<Data> + Send + Sync>>,
}

impl WebSocketTransport {
    /// Creates a websocket transport and sets the function that converts the `payload` of the `connect_init` message to `Data`.
    pub fn new<F: Fn(serde_json::Value) -> FieldResult<Data> + Send + Sync + 'static>(
        init_context_data: F,
    ) -> Self {
        WebSocketTransport {
            init_context_data: Some(Box::new(init_context_data)),
            ..WebSocketTransport::default()
        }
    }
}

fn send_message<T: Serialize>(send_buf: &mut VecDeque<Vec<u8>>, msg: &T) {
    if let Ok(data) = serde_json::to_vec(msg) {
        send_buf.push_back(data);
    }
}

#[async_trait::async_trait]
impl ConnectionTransport for WebSocketTransport {
    type Error = FieldError;

    async fn handle_request<Query, Mutation, Subscription>(
        &mut self,
        schema: &Schema<Query, Mutation, Subscription>,
        streams: &mut SubscriptionStreams,
        request: Vec<u8>,
        send_buf: &mut VecDeque<Vec<u8>>,
    ) -> std::result::Result<(), Self::Error>
    where
        Query: ObjectType + Sync + Send + 'static,
        Mutation: ObjectType + Sync + Send + 'static,
        Subscription: SubscriptionType + Sync + Send + 'static,
    {
        match serde_json::from_slice::<OperationMessage>(&request) {
            Ok(msg) => match msg.ty.as_str() {
                "connection_init" => {
                    if let Some(payload) = msg.payload {
                        if let Some(init_context_data) = &self.init_context_data {
                            self.data = Arc::new(init_context_data(payload)?);
                        }
                    }
                    send_message(
                        send_buf,
                        &OperationMessage {
                            ty: "connection_ack".to_string(),
                            id: None,
                            payload: None,
                        },
                    );
                    Ok(())
                }
                "start" => {
                    if let (Some(id), Some(payload)) = (msg.id, msg.payload) {
                        if let Ok(request) = serde_json::from_value::<GQLRequest>(payload) {
                            let variables = request
                                .variables
                                .map(Variables::parse_from_json)
                                .unwrap_or_default();
                            match schema
                                .create_subscription_stream(
                                    &request.query,
                                    request.operation_name.as_deref(),
                                    variables.clone(),
                                    Some(self.data.clone()),
                                )
                                .await
                            {
                                Ok(stream) => {
                                    let stream_id = streams.add(stream);
                                    self.id_to_sid.insert(id.clone(), stream_id);
                                    self.sid_to_id.insert(stream_id, id);
                                }
                                Err(Error::Query { err, .. })
                                    if err == QueryError::NotSupported =>
                                {
                                    // Is query or mutation
                                    let mut builder =
                                        QueryBuilder::new(&request.query).variables(variables);
                                    if let Some(operation_name) = &request.operation_name {
                                        builder = builder.operation_name(operation_name);
                                    }

                                    match builder.execute(schema).await {
                                        Ok(resp) => {
                                            send_message(
                                                send_buf,
                                                &OperationMessage {
                                                    ty: "data".to_string(),
                                                    id: Some(id.clone()),
                                                    payload: Some(
                                                        serde_json::to_value(&GQLResponse(Ok(
                                                            resp,
                                                        )))
                                                        .unwrap(),
                                                    ),
                                                },
                                            );

                                            send_message(
                                                send_buf,
                                                &OperationMessage {
                                                    ty: "complete".to_string(),
                                                    id: Some(id),
                                                    payload: None,
                                                },
                                            );
                                        }
                                        Err(err) => {
                                            send_message(
                                                send_buf,
                                                &OperationMessage {
                                                    ty: "error".to_string(),
                                                    id: Some(id),
                                                    payload: Some(
                                                        serde_json::to_value(GQLError(&err))
                                                            .unwrap(),
                                                    ),
                                                },
                                            );
                                        }
                                    }
                                }
                                Err(err) => {
                                    send_message(
                                        send_buf,
                                        &OperationMessage {
                                            ty: "error".to_string(),
                                            id: Some(id),
                                            payload: Some(
                                                serde_json::to_value(GQLError(&err)).unwrap(),
                                            ),
                                        },
                                    );
                                }
                            }
                        }
                    }
                    Ok(())
                }
                "stop" => {
                    if let Some(id) = msg.id {
                        if let Some(sid) = self.id_to_sid.remove(&id) {
                            self.sid_to_id.remove(&sid);
                            streams.remove(sid);
                            send_message(
                                send_buf,
                                &OperationMessage {
                                    ty: "complete".to_string(),
                                    id: Some(id),
                                    payload: None,
                                },
                            );
                        }
                    }
                    Ok(())
                }
                "connection_terminate" => Err("connection_terminate".into()),
                _ => Err("Unknown op".into()),
            },
            Err(err) => Err(err.into()),
        }
    }

    fn handle_response(&mut self, id: usize, res: Result<serde_json::Value>) -> Option<Vec<u8>> {
        if let Some(id) = self.sid_to_id.get(&id) {
            match res {
                Ok(value) => Some(
                    serde_json::to_vec(&OperationMessage {
                        ty: "data".to_string(),
                        id: Some(id.clone()),
                        payload: Some(
                            serde_json::to_value(GQLResponse(Ok(QueryResponse {
                                data: value,
                                extensions: None,
                                cache_control: Default::default(),
                            })))
                            .unwrap(),
                        ),
                    })
                    .unwrap(),
                ),
                Err(err) => Some(
                    serde_json::to_vec(&OperationMessage {
                        ty: "error".to_string(),
                        id: Some(id.to_string()),
                        payload: Some(serde_json::to_value(GQLError(&err)).unwrap()),
                    })
                    .unwrap(),
                ),
            }
        } else {
            None
        }
    }
}
