use poem::http::header::HeaderName;
use poem::web::Json;
use poem::{IntoResponse, Response};

/// Response for `async_graphql::Request`.
pub struct GraphQLResponse(pub async_graphql::Response);

impl From<async_graphql::Response> for GraphQLResponse {
    fn from(resp: async_graphql::Response) -> Self {
        Self(resp)
    }
}

impl IntoResponse for GraphQLResponse {
    fn into_response(self) -> Response {
        GraphQLBatchResponse(self.0.into()).into_response()
    }
}

/// Response for `async_graphql::BatchRequest`.
pub struct GraphQLBatchResponse(pub async_graphql::BatchResponse);

impl From<async_graphql::BatchResponse> for GraphQLBatchResponse {
    fn from(resp: async_graphql::BatchResponse) -> Self {
        Self(resp)
    }
}

impl IntoResponse for GraphQLBatchResponse {
    fn into_response(self) -> Response {
        let mut resp = Json(&self.0).into_response();

        if self.0.is_ok() {
            if let Some(cache_control) = self.0.cache_control().value() {
                if let Ok(value) = cache_control.try_into() {
                    resp.headers_mut().insert("cache-control", value);
                }
            }
        }

        for (name, value) in self.0.http_headers() {
            if let (Ok(name), Ok(value)) = (TryInto::<HeaderName>::try_into(name), value.try_into())
            {
                resp.headers_mut().append(name, value);
            }
        }

        resp
    }
}
