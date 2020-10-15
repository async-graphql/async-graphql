use std::error::Error;
use std::fmt::{self, Display, Formatter};

use async_graphql::ParseRequestError;
use warp::http::{Response, StatusCode};
use warp::hyper::Body;
use warp::reject::Reject;
use warp::Reply;

/// Bad request error.
///
/// It's a wrapper of `async_graphql::ParseRequestError`. It is also a `Reply` - by default it just
/// returns a response containing the error message in plain text.
#[derive(Debug)]
pub struct BadRequest(pub ParseRequestError);

impl BadRequest {
    /// Get the appropriate status code of the error.
    #[must_use]
    pub fn status(&self) -> StatusCode {
        match self.0 {
            ParseRequestError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

impl Display for BadRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for BadRequest {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl Reject for BadRequest {}

impl Reply for BadRequest {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .status(self.status())
            .body(Body::from(self.0.to_string()))
            .unwrap()
    }
}

impl From<ParseRequestError> for BadRequest {
    fn from(e: ParseRequestError) -> Self {
        Self(e)
    }
}
impl From<BadRequest> for ParseRequestError {
    fn from(e: BadRequest) -> Self {
        e.0
    }
}
