use std::time::Duration;

use actix_http::StatusCode;
use actix_web::{Handler, HttpRequest, HttpResponse, Responder};
use async_graphql::{
    http::{create_multipart_mixed_stream, is_accept_multipart_mixed},
    Executor,
};
use futures_util::{future::LocalBoxFuture, FutureExt, StreamExt};

use crate::{GraphQLRequest, GraphQLResponse};

/// A GraphQL handler.
#[derive(Clone)]
pub struct GraphQL<E> {
    executor: E,
}

impl<E> GraphQL<E> {
    /// Create a GraphQL handler.
    pub fn new(executor: E) -> Self {
        Self { executor }
    }
}

impl<E: Executor> Handler<(HttpRequest, GraphQLRequest)> for GraphQL<E> {
    type Output = HttpResponse;
    type Future = LocalBoxFuture<'static, Self::Output>;

    fn call(&self, (http_req, graphql_req): (HttpRequest, GraphQLRequest)) -> Self::Future {
        let executor = self.executor.clone();
        async move {
            let is_accept_multipart_mixed = http_req
                .headers()
                .get("accept")
                .and_then(|value| value.to_str().ok())
                .map(is_accept_multipart_mixed)
                .unwrap_or_default();

            if is_accept_multipart_mixed {
                let stream = executor.execute_stream(graphql_req.0, None);
                let interval = Box::pin(async_stream::stream! {
                    let mut interval = actix_web::rt::time::interval(Duration::from_secs(30));
                    loop {
                        interval.tick().await;
                        yield ();
                    }
                });
                HttpResponse::build(StatusCode::OK)
                    .insert_header(("content-type", "multipart/mixed; boundary=graphql"))
                    .streaming(
                        create_multipart_mixed_stream(stream, interval)
                            .map(Ok::<_, actix_web::Error>),
                    )
            } else {
                GraphQLResponse(executor.execute(graphql_req.into_inner()).await.into())
                    .respond_to(&http_req)
            }
        }
        .boxed_local()
    }
}
