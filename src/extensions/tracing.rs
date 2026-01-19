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
    registry::MetaTypeName,
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

impl Tracing {
    /// Create a configurable tracing extension.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use async_graphql::{extensions::Tracing, *};
    ///
    /// #[derive(SimpleObject)]
    /// struct Query {
    ///     value: i32,
    /// }
    ///
    /// // Trace all fields including scalars
    /// let schema = Schema::build(Query { value: 100 }, EmptyMutation, EmptySubscription)
    ///     .extension(Tracing::config().with_trace_scalars(true))
    ///     .finish();
    /// ```
    pub fn config() -> TracingConfig {
        TracingConfig::default()
    }
}

impl ExtensionFactory for Tracing {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(TracingExtension {
            trace_scalars: false,
        })
    }
}

/// Configuration for the [`Tracing`] extension.
#[cfg_attr(docsrs, doc(cfg(feature = "tracing")))]
#[derive(Clone, Copy, Debug, Default)]
pub struct TracingConfig {
    trace_scalars: bool,
}

impl TracingConfig {
    /// Enable or disable tracing for scalar and enum field resolutions.
    ///
    /// When `false` (the default), spans are not created for fields that return
    /// scalar or enum types. This significantly reduces the number of spans
    /// generated, preventing span explosion in queries with many scalar fields.
    ///
    /// When `true`, spans are created for all field resolutions, including
    /// scalars and enums.
    pub fn with_trace_scalars(mut self, trace_scalars: bool) -> Self {
        self.trace_scalars = trace_scalars;
        self
    }
}

impl ExtensionFactory for TracingConfig {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(TracingExtension {
            trace_scalars: self.trace_scalars,
        })
    }
}

struct TracingExtension {
    trace_scalars: bool,
}

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
        // Check if we should skip tracing for this field
        let should_trace = if info.is_for_introspection {
            false
        } else if !self.trace_scalars {
            // Check if the return type is a scalar or enum (leaf type)
            let concrete_type = MetaTypeName::concrete_typename(info.return_type);
            !ctx.schema_env
                .registry
                .types
                .get(concrete_type)
                .map(|ty| ty.is_leaf())
                .unwrap_or(false)
        } else {
            true
        };

        let span = if should_trace {
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
