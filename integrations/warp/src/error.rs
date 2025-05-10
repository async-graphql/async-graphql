use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use async_graphql::ParseRequestError;
use warp::{
    Reply,
    http::{Response, StatusCode},
    hyper::Body,
    reject::Reject,
};

/// Bad request error.
///
/// It's a wrapper of `async_graphql::ParseRequestError`. It is also a `Reply` -
/// by default it just returns a response containing the error message in plain
/// text.
#[derive(Debug)]
pub struct GraphQLBadRequest(pub ParseRequestError);

impl GraphQLBadRequest {
    /// Get the appropriate status code of the error.
    #[must_use]
    pub fn status(&self) -> StatusCode {
        match self.0 {
            ParseRequestError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

impl Display for GraphQLBadRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for GraphQLBadRequest {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl Reject for GraphQLBadRequest {}

impl Reply for GraphQLBadRequest {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .status(self.status())
            .body(Body::from(self.0.to_string()))
            .unwrap()
    }
}

impl From<ParseRequestError> for GraphQLBadRequest {
    fn from(e: ParseRequestError) -> Self {
        Self(e)
    }
}

impl From<GraphQLBadRequest> for ParseRequestError {
    fn from(e: GraphQLBadRequest) -> Self {
        e.0
    }
}
