use axum::body::Body;
use axum::response::IntoResponse;
use headers::HeaderName;
use http::{HeaderValue, Response};

/// Responder for a GraphQL response.
///
/// This contains a batch response, but since regular responses are a type of batch response it
/// works for both.
pub struct GraphQLResponse(pub async_graphql::BatchResponse);

impl From<async_graphql::Response> for GraphQLResponse {
    fn from(resp: async_graphql::Response) -> Self {
        Self(resp.into())
    }
}

impl From<async_graphql::BatchResponse> for GraphQLResponse {
    fn from(resp: async_graphql::BatchResponse) -> Self {
        Self(resp)
    }
}

impl IntoResponse for GraphQLResponse {
    type Body = Body;
    type BodyError = <Self::Body as axum::body::HttpBody>::Error;

    fn into_response(self) -> Response<Body> {
        let mut resp = Response::new(serde_json::to_string(&self.0).unwrap().into());
        resp.headers_mut().insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        if self.0.is_ok() {
            if let Some(cache_control) = self.0.cache_control().value() {
                if let Ok(value) = HeaderValue::from_str(&cache_control) {
                    resp.headers_mut()
                        .insert(http::header::CACHE_CONTROL, value);
                }
            }
        }
        for (name, value) in self.0.http_headers() {
            if let (Ok(name), Ok(value)) = (
                HeaderName::try_from(name.as_bytes()),
                HeaderValue::from_str(value),
            ) {
                resp.headers_mut().insert(name, value);
            }
        }
        resp
    }
}
