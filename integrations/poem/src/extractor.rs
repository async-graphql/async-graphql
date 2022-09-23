use async_graphql::http::MultipartOptions;
use poem::{
    async_trait,
    error::BadRequest,
    http::{header, Method},
    web::Query,
    FromRequest, Request, RequestBody, Result,
};
use tokio_util::compat::TokioAsyncReadCompatExt;

/// An extractor for GraphQL request.
///
/// You can just use the extractor as in the example below, but I would
/// recommend using the [`GraphQL`](crate::GraphQL) endpoint because it is
/// easier to integrate.
///
/// # Example
///
/// ```
/// use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
/// use async_graphql_poem::GraphQLRequest;
/// use poem::{
///     handler,
///     middleware::AddData,
///     post,
///     web::{Data, Json},
///     EndpointExt, Route,
/// };
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

            let content_encoding = req
                .headers()
                .get(header::CONTENT_ENCODING)
                .and_then(|value| value.to_str().ok())
                .map(ToString::to_string);

            Ok(Self(
                async_graphql::http::receive_batch_body(
                    content_type,
                    content_encoding,
                    body.take()?.into_async_read().compat(),
                    MultipartOptions::default(),
                )
                .await
                .map_err(BadRequest)?,
            ))
        }
    }
}
