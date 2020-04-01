use crate::http::{GQLError, GQLRequest, GQLResponse};
use crate::{
    ObjectType, QueryResult, Result, Schema, SubscriptionStubs, SubscriptionTransport,
    SubscriptionType, Variables,
};
use bytes::Bytes;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct OperationMessage {
    #[serde(rename = "type")]
    ty: String,
    id: Option<String>,
    payload: Option<serde_json::Value>,
}

/// WebSocket transport
#[derive(Default)]
pub struct WebSocketTransport {
    id_to_sid: HashMap<String, usize>,
    sid_to_id: HashMap<usize, String>,
}

impl SubscriptionTransport for WebSocketTransport {
    fn handle_request<Query, Mutation, Subscription>(
        &mut self,
        schema: &Schema<Query, Mutation, Subscription>,
        stubs: &mut SubscriptionStubs<Query, Mutation, Subscription>,
        data: Bytes,
    ) -> Result<Option<Bytes>>
    where
        Query: ObjectType + Sync + Send + 'static,
        Mutation: ObjectType + Sync + Send + 'static,
        Subscription: SubscriptionType + Sync + Send + 'static,
    {
        match serde_json::from_slice::<OperationMessage>(&data) {
            Ok(msg) => match msg.ty.as_str() {
                "connection_init" => Ok(Some(
                    serde_json::to_vec(&OperationMessage {
                        ty: "connection_ack".to_string(),
                        id: None,
                        payload: None,
                    })
                    .unwrap()
                    .into(),
                )),
                "start" => {
                    if let (Some(id), Some(payload)) = (msg.id, msg.payload) {
                        if let Ok(request) = serde_json::from_value::<GQLRequest>(payload) {
                            let variables = request
                                .variables
                                .map(|value| Variables::parse_from_json(value).ok())
                                .flatten()
                                .unwrap_or_default();

                            match schema.create_subscription_stub(
                                &request.query,
                                request.operation_name.as_deref(),
                                variables,
                            ) {
                                Ok(stub) => {
                                    let stub_id = stubs.add(stub);
                                    self.id_to_sid.insert(id.clone(), stub_id);
                                    self.sid_to_id.insert(stub_id, id);
                                    Ok(None)
                                }
                                Err(err) => Ok(Some(
                                    serde_json::to_vec(&OperationMessage {
                                        ty: "error".to_string(),
                                        id: Some(id),
                                        payload: Some(
                                            serde_json::to_value(GQLError(&err)).unwrap(),
                                        ),
                                    })
                                    .unwrap()
                                    .into(),
                                )),
                            }
                        } else {
                            Ok(None)
                        }
                    } else {
                        Ok(None)
                    }
                }
                "stop" => {
                    if let Some(id) = msg.id {
                        if let Some(id) = self.id_to_sid.remove(&id) {
                            self.sid_to_id.remove(&id);
                            stubs.remove(id);
                        }
                    }
                    Ok(None)
                }
                "connection_terminate" => Err(anyhow::anyhow!("connection_terminate")),
                _ => Err(anyhow::anyhow!("unknown op")),
            },
            Err(err) => Err(err.into()),
        }
    }

    fn handle_response(&mut self, id: usize, result: Result<serde_json::Value>) -> Option<Bytes> {
        if let Some(id) = self.sid_to_id.get(&id) {
            Some(
                serde_json::to_vec(&OperationMessage {
                    ty: "data".to_string(),
                    id: Some(id.clone()),
                    payload: Some(
                        serde_json::to_value(GQLResponse(result.map(|data| QueryResult {
                            data,
                            extensions: None,
                        })))
                        .unwrap(),
                    ),
                })
                .unwrap()
                .into(),
            )
        } else {
            None
        }
    }
}
