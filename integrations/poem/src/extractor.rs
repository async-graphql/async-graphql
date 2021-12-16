use async_graphql::http::MultipartOptions;
use poem::error::BadRequest;
use poem::http::{header, Method};
use poem::web::Query;
use poem::{async_trait, FromRequest, Request, RequestBody, Result};
use tokio_util::compat::TokioAsyncReadCompatExt;

/// An extractor for GraphQL request.
///
/// You can just use the extractor as in the example below, but I would recommend using
/// the [`GraphQL`](crate::GraphQL) endpoint because it is easier to integrate.
///
/// # Example
///
/// ```
/// use poem::{handler, Route, post, EndpointExt};
/// use poem::web::{Json, Data};
/// use poem::middleware::AddData;
/// use async_graphql_poem::GraphQLRequest;
/// use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
///
/// struct Query;
///
/// #[Object]
/// impl Query {
///     async fn value(&self) -> i32 {
///         100
///     }
/// }
///
/// type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;
///
/// #[handler]
/// async fn index(req: GraphQLRequest, schema: Data<&MySchema>) -> Json<async_graphql::Response> {
///     Json(schema.execute(req.0).await)
/// }
///
/// let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
/// let app = Route::new().at("/", post(index.with(AddData::new(schema))));
/// ```
pub struct GraphQLRequest(pub async_graphql::Request);

#[async_trait]
impl<'a> FromRequest<'a> for GraphQLRequest {
    async fn from_request(req: &'a Request, body: &mut RequestBody) -> Result<Self> {
        Ok(GraphQLRequest(
            GraphQLBatchRequest::from_request(req, body)
                .await?
                .0
                .into_single()
                .map_err(BadRequest)?,
        ))
    }
}

/// An extractor for GraphQL batch request.
pub struct GraphQLBatchRequest(pub async_graphql::BatchRequest);

#[async_trait]
impl<'a> FromRequest<'a> for GraphQLBatchRequest {
    async fn from_request(req: &'a Request, body: &mut RequestBody) -> Result<Self> {
        if req.method() == Method::GET {
            let req = Query::from_request(req, body).await?.0;
            Ok(Self(async_graphql::BatchRequest::Single(req)))
        } else {
            let content_type = req
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .map(ToString::to_string);
            Ok(Self(
                async_graphql::http::receive_batch_body(
                    content_type,
                    body.take()?.into_async_read().compat(),
                    MultipartOptions::default(),
                )
                .await
                .map_err(BadRequest)?,
            ))
        }
    }
}
