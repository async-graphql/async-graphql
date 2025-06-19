use std::sync::Arc;

use futures_util::{TryFutureExt, stream::BoxStream};
use tracing::{Level, span};
use tracing_futures::Instrument;

use crate::{
    Response, ServerError, ServerResult, ValidationResult, Value, Variables,
    extensions::{
        Extension, ExtensionContext, ExtensionFactory, NextExecute, NextParseQuery, NextRequest,
        NextResolve, NextSubscribe, NextValidation, ResolveInfo,
    },
    parser::types::ExecutableDocument,
};

/// Tracing extension
///
/// # References
///
/// <https://crates.io/crates/tracing>
///
/// # Examples
///
/// ```no_run
/// use async_graphql::{extensions::Tracing, *};
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
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// schema.execute(Request::new("{ value }")).await;
/// # });
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "tracing")))]
pub struct Tracing;

impl ExtensionFactory for Tracing {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(TracingExtension)
    }
}

struct TracingExtension;

#[async_trait::async_trait]
impl Extension for TracingExtension {
    async fn request(&self, ctx: &ExtensionContext<'_>, next: NextRequest<'_>) -> Response {
        next.run(ctx)
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
        next: NextSubscribe<'_>,
    ) -> BoxStream<'s, Response> {
        Box::pin(next.run(ctx, stream).instrument(span!(
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
        next: NextParseQuery<'_>,
    ) -> ServerResult<ExecutableDocument> {
        let span = span!(
            target: "async_graphql::graphql",
            Level::INFO,
            "parse",
            source = tracing::field::Empty
        );
        async move {
            let res = next.run(ctx, query, variables).await;
            if let Ok(doc) = &res {
                tracing::Span::current()
                    .record("source", ctx.stringify_execute_doc(doc, variables).as_str());
            }
            res
        }
        .instrument(span)
        .await
    }

    async fn validation(
        &self,
        ctx: &ExtensionContext<'_>,
        next: NextValidation<'_>,
    ) -> Result<ValidationResult, Vec<ServerError>> {
        let span = span!(
            target: "async_graphql::graphql",
            Level::INFO,
            "validation"
        );
        next.run(ctx).instrument(span).await
    }

    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        let span = span!(
            target: "async_graphql::graphql",
            Level::INFO,
            "execute"
        );
        next.run(ctx, operation_name).instrument(span).await
    }

    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextResolve<'_>,
    ) -> ServerResult<Option<Value>> {
        let span = if !info.is_for_introspection {
            Some(span!(
                target: "async_graphql::graphql",
                Level::INFO,
                "field",
                path = %info.path_node,
                parent_type = %info.parent_type,
                return_type = %info.return_type,
            ))
        } else {
            None
        };

        let fut = next.run(ctx, info).inspect_err(|err| {
            tracing::info!(
                target: "async_graphql::graphql",
                error = %err.message,
                "error",
            );
        });
        match span {
            Some(span) => fut.instrument(span).await,
            None => fut.await,
        }
    }
}
