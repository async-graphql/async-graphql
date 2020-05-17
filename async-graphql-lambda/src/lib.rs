use async_graphql::{IntoQueryBuilder, IntoQueryBuilderOpts, ParseRequestError, QueryBuilder};
use futures::io::AllowStdIo;
use lambda_http::Request;
use std::io::Cursor;

/// Lambda request extension
///
#[async_trait::async_trait]
pub trait GQLRequestExt {
    /// Convert a query to `async_graphql::QueryBuilder`.
    async fn graphql(&self) -> Result<QueryBuilder, ParseRequestError> {
        self.graphql_opts(Default::default()).await
    }

    /// Similar to graphql, but you can set the options `IntoQueryBuilderOpts`.
    async fn graphql_opts(
        &self,
        opts: IntoQueryBuilderOpts,
    ) -> Result<QueryBuilder, ParseRequestError>;
}

#[async_trait::async_trait]
impl GQLRequestExt for Request {
    async fn graphql_opts(
        &self,
        opts: IntoQueryBuilderOpts,
    ) -> Result<QueryBuilder, ParseRequestError> {
        let ct = self
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok());
        (ct, AllowStdIo::new(Cursor::new(self.body().to_vec())))
            .into_query_builder_opts(&opts)
            .await
    }
}
