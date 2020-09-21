use warp::reject::Reject;

/// Bad request error
///
/// It's a wrapper of `async_graphql::ParseRequestError`.
pub struct BadRequest(pub anyhow::Error);

impl std::fmt::Debug for BadRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Reject for BadRequest {}
