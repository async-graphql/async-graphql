use std::sync::Arc;

use futures_util::stream::BoxStream;
use futures_util::TryFutureExt;
use tracing_futures::Instrument;
use tracinglib::{span, Level};

use crate::extensions::{
    Extension, ExtensionContext, ExtensionFactory, NextExtension, ResolveInfo,
};
use crate::parser::types::ExecutableDocument;
use crate::{Response, ServerError, ServerResult, ValidationResult, Value, Variables};

/// Tracing extension
///
/// # References
///
/// <https://crates.io/crates/tracing>
///
/// # Examples
///
/// ```no_run
/// use async_graphql::*;
/// use async_graphql::extensions::Tracing;
///
/// #[derive(SimpleObject)]
/// struct Query {
///     value: i32,
/// }
///
/// let schema = Schema::build(Query { value: 100 }, EmptyMutation, EmptySubscription)
///     .extension(Tracing)
///     .finish();
///
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     schema.execute(Request::new("{ value }")).await;
/// });
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "tracing")))]
pub struct Tracing;

impl ExtensionFactory for Tracing {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(TracingExtension::default())
    }
}

#[derive(Default)]
struct TracingExtension;

#[async_trait::async_trait]
impl Extension for TracingExtension {
    async fn request(&self, ctx: &ExtensionContext<'_>, next: NextExtension<'_>) -> Response {
        next.request(ctx)
            .instrument(span!(
                target: "async_graphql::graphql",
                Level::INFO,
                "request",
            ))
            .await
    }

    fn subscribe<'s>(
        &self,
        ctx: &ExtensionContext<'_>,
        stream: BoxStream<'s, Response>,
        next: NextExtension<'_>,
    ) -> BoxStream<'s, Response> {
        Box::pin(next.subscribe(ctx, stream).instrument(span!(
            target: "async_graphql::graphql",
            Level::INFO,
            "subscribe",
        )))
    }

    async fn parse_query(
        &self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
        next: NextExtension<'_>,
    ) -> ServerResult<ExecutableDocument> {
        let span = span!(
            target: "async_graphql::graphql",
            Level::INFO,
            "parse",
            source = query,
            variables = %serde_json::to_string(&variables).unwrap(),
        );
        next.parse_query(ctx, query, variables)
            .instrument(span)
            .await
    }

    async fn validation(
        &self,
        ctx: &ExtensionContext<'_>,
        next: NextExtension<'_>,
    ) -> Result<ValidationResult, Vec<ServerError>> {
        let span = span!(
            target: "async_graphql::graphql",
            Level::INFO,
            "validation"
        );
        next.validation(ctx).instrument(span).await
    }

    async fn execute(&self, ctx: &ExtensionContext<'_>, next: NextExtension<'_>) -> Response {
        let span = span!(
            target: "async_graphql::graphql",
            Level::INFO,
            "execute"
        );
        next.execute(ctx).instrument(span).await
    }

    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextExtension<'_>,
    ) -> ServerResult<Option<Value>> {
        let span = span!(
            target: "async_graphql::graphql",
            Level::INFO,
            "field",
            path = %info.path_node,
            parent_type = %info.parent_type,
            return_type = %info.return_type,
        );
        next.resolve(ctx, info)
            .instrument(span)
            .map_err(|err| {
                tracinglib::error!(target: "async_graphql::graphql", error = %err.message);
                err
            })
            .await
    }
}
