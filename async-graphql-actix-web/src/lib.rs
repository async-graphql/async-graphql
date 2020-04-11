mod request;
mod session;

use crate::request::RequestWrapper;
use crate::session::WsSession;
use actix_web::http::{header, Method};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use async_graphql::http::{playground_source, GQLHttpRequest, GQLResponse, IntoQueryBuilder};
use async_graphql::{ObjectType, QueryBuilder, Schema, SubscriptionType};
use futures::Future;
use std::pin::Pin;
use std::sync::Arc;

type BoxOnRequestFn<Query, Mutation, Subscription> = Arc<
    dyn for<'a> Fn(
        &HttpRequest,
        QueryBuilder<Query, Mutation, Subscription>,
    ) -> QueryBuilder<Query, Mutation, Subscription>,
>;

pub struct HandlerBuilder<Query, Mutation, Subscription> {
    schema: Schema<Query, Mutation, Subscription>,
    enable_subscription: bool,
    enable_ui: Option<(String, Option<String>)>,
    on_request: Option<BoxOnRequestFn<Query, Mutation, Subscription>>,
}

impl<Query, Mutation, Subscription> HandlerBuilder<Query, Mutation, Subscription>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    /// Create an HTTP handler builder
    pub fn new(schema: Schema<Query, Mutation, Subscription>) -> Self {
        Self {
            schema,
            enable_subscription: false,
            enable_ui: None,
            on_request: None,
        }
    }

    /// Enable GraphQL playground
    ///
    /// 'endpoint' is the endpoint of the GraphQL Request.
    /// 'subscription_endpoint' is the endpoint of the GraphQL Subscription.
    pub fn enable_ui(self, endpoint: &str, subscription_endpoint: Option<&str>) -> Self {
        Self {
            enable_ui: Some((
                endpoint.to_string(),
                subscription_endpoint.map(|s| s.to_string()),
            )),
            ..self
        }
    }

    /// Enable GraphQL Subscription.
    pub fn enable_subscription(self) -> Self {
        Self {
            enable_subscription: true,
            ..self
        }
    }

    /// When a new request arrives, you can use this closure to append your own data to the `QueryBuilder`.
    pub fn on_request<
        F: for<'a> Fn(
                &HttpRequest,
                QueryBuilder<Query, Mutation, Subscription>,
            ) -> QueryBuilder<Query, Mutation, Subscription>
            + 'static,
    >(
        self,
        f: F,
    ) -> Self {
        Self {
            on_request: Some(Arc::new(f)),
            ..self
        }
    }

    /// Create an HTTP handler.
    pub fn build(
        self,
    ) -> impl Fn(
        HttpRequest,
        web::Payload,
    ) -> Pin<Box<dyn Future<Output = actix_web::Result<HttpResponse>>>>
           + Clone
           + 'static {
        let schema = self.schema.clone();
        let enable_ui = self.enable_ui;
        let enable_subscription = self.enable_subscription;
        let on_request = self.on_request;

        move |req: HttpRequest, payload: web::Payload| {
            let schema = schema.clone();
            let enable_ui = enable_ui.clone();
            let on_request = on_request.clone();

            Box::pin(async move {
                if req.method() == Method::GET {
                    if enable_subscription {
                        if let Some(s) = req.headers().get(header::UPGRADE) {
                            if let Ok(s) = s.to_str() {
                                if s.to_ascii_lowercase().contains("websocket") {
                                    return ws::start_with_protocols(
                                        WsSession::new(schema.clone()),
                                        &["graphql-ws"],
                                        &req,
                                        payload,
                                    );
                                }
                            }
                        }
                    }

                    if let Some((endpoint, subscription_endpoint)) = &enable_ui {
                        return Ok(HttpResponse::Ok()
                            .content_type("text/html; charset=utf-8")
                            .body(playground_source(
                                endpoint,
                                subscription_endpoint.as_deref(),
                            )));
                    }
                }

                if req.method() == Method::POST {
                    let content_type = req
                        .headers()
                        .get(header::CONTENT_TYPE)
                        .and_then(|value| value.to_str().ok())
                        .map(|s| s.to_string());
                    let r = RequestWrapper(content_type, payload);
                    r.content_type();
                    let mut builder = match IntoQueryBuilder::into_query_builder(r, &schema).await {
                        Ok(builder) => builder,
                        Err(err) => {
                            return Ok(web::Json(GQLResponse(Err(err))).respond_to(&req).await?)
                        }
                    };

                    let cache_control = builder.cache_control().value();
                    if let Some(on_request) = &on_request {
                        builder = on_request(&req, builder);
                    }

                    let mut resp = web::Json(GQLResponse(builder.execute().await))
                        .respond_to(&req)
                        .await?;
                    if let Some(cache_control) = cache_control {
                        resp.headers_mut().insert(
                            header::CACHE_CONTROL,
                            header::HeaderValue::from_str(&cache_control).unwrap(),
                        );
                    }
                    Ok(resp)
                } else {
                    Ok(HttpResponse::MethodNotAllowed().finish())
                }
            })
        }
    }
}
