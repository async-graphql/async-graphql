use std::sync::Arc;

use async_graphql::{
    Response, ServerError, ServerResult, ValidationResult, Value,
    extensions::{
        Extension, ExtensionContext, ExtensionFactory, NextExecute, NextParseQuery, NextRequest,
        NextResolve, NextSubscribe, NextValidation, ResolveInfo,
    },
    registry::MetaTypeName,
};
use async_graphql_parser::types::ExecutableDocument;
use async_graphql_value::Variables;
use futures_util::{TryFutureExt, stream::BoxStream};
use opentelemetry::{
    Context as OpenTelemetryContext, Key, KeyValue,
    trace::{FutureExt, SpanKind, TraceContextExt, Tracer},
};

const KEY_SOURCE: Key = Key::from_static_str("graphql.source");
const KEY_VARIABLES: Key = Key::from_static_str("graphql.variables");
const KEY_PARENT_TYPE: Key = Key::from_static_str("graphql.parentType");
const KEY_RETURN_TYPE: Key = Key::from_static_str("graphql.returnType");
const KEY_ERROR: Key = Key::from_static_str("graphql.error");
const KEY_COMPLEXITY: Key = Key::from_static_str("graphql.complexity");
const KEY_DEPTH: Key = Key::from_static_str("graphql.depth");

/// OpenTelemetry extension
///
/// # Example
///
/// ```ignore
/// use async_graphql::extensions::OpenTelemetry;
/// use async_graphql_extras::OpenTelemetry as ExtrasOpenTelemetry;
///
/// let tracer = todo!("create your OpenTelemetry tracer");
///
/// let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
///     .extension(ExtrasOpenTelemetry::new(tracer))
///     .finish();
/// ```
pub struct OpenTelemetry<T> {
    tracer: Arc<T>,
    trace_scalars: bool,
}

impl<T> OpenTelemetry<T> {
    /// Use `tracer` to create an OpenTelemetry extension.
    pub fn new(tracer: T) -> OpenTelemetry<T>
    where
        T: Tracer + Send + Sync + 'static,
        <T as Tracer>::Span: Sync + Send,
    {
        Self {
            tracer: Arc::new(tracer),
            trace_scalars: false,
        }
    }

    /// Enable or disable tracing for scalar and enum field resolutions.
    ///
    /// When `false` (the default), spans are not created for fields that return
    /// scalar or enum types. This significantly reduces the number of spans
    /// generated, preventing span explosion in queries with many scalar fields.
    ///
    /// When `true`, spans are created for all field resolutions, including
    /// scalars and enums.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use async_graphql::extensions::OpenTelemetry;
    /// use async_graphql_extras::OpenTelemetry as ExtrasOpenTelemetry;
    ///
    /// let tracer = todo!("create your OpenTelemetry tracer");
    ///
    /// // Trace all fields including scalars
    /// let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
    ///     .extension(ExtrasOpenTelemetry::new(tracer).with_trace_scalars(true))
    ///     .finish();
    /// ```
    pub fn with_trace_scalars(mut self, trace_scalars: bool) -> Self {
        self.trace_scalars = trace_scalars;
        self
    }
}

impl<T> ExtensionFactory for OpenTelemetry<T>
where
    T: Tracer + Send + Sync + 'static,
    <T as Tracer>::Span: Sync + Send,
{
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(OpenTelemetryExtension {
            tracer: self.tracer.clone(),
            trace_scalars: self.trace_scalars,
        })
    }
}

struct OpenTelemetryExtension<T> {
    tracer: Arc<T>,
    trace_scalars: bool,
}

#[async_trait::async_trait]
impl<T> Extension for OpenTelemetryExtension<T>
where
    T: Tracer + Send + Sync + 'static,
    <T as Tracer>::Span: Sync + Send,
{
    async fn request(&self, ctx: &ExtensionContext<'_>, next: NextRequest<'_>) -> Response {
        next.run(ctx)
            .with_context(OpenTelemetryContext::current_with_span(
                self.tracer
                    .span_builder("request")
                    .with_kind(SpanKind::Server)
                    .start(&*self.tracer),
            ))
            .await
    }

    fn subscribe<'s>(
        &self,
        ctx: &ExtensionContext<'_>,
        stream: BoxStream<'s, Response>,
        next: NextSubscribe<'_>,
    ) -> BoxStream<'s, Response> {
        Box::pin(
            next.run(ctx, stream)
                .with_context(OpenTelemetryContext::current_with_span(
                    self.tracer
                        .span_builder("subscribe")
                        .with_kind(SpanKind::Server)
                        .start(&*self.tracer),
                )),
        )
    }

    async fn parse_query(
        &self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
        next: NextParseQuery<'_>,
    ) -> ServerResult<ExecutableDocument> {
        let attributes = vec![
            KeyValue::new(KEY_SOURCE, query.to_string()),
            KeyValue::new(KEY_VARIABLES, serde_json::to_string(variables).unwrap()),
        ];
        let span = self
            .tracer
            .span_builder("parse")
            .with_kind(SpanKind::Server)
            .with_attributes(attributes)
            .start(&*self.tracer);

        async move {
            let res = next.run(ctx, query, variables).await;
            if let Ok(doc) = &res {
                OpenTelemetryContext::current()
                    .span()
                    .set_attribute(KeyValue::new(
                        KEY_SOURCE,
                        ctx.stringify_execute_doc(doc, variables),
                    ));
            }
            res
        }
        .with_context(OpenTelemetryContext::current_with_span(span))
        .await
    }

    async fn validation(
        &self,
        ctx: &ExtensionContext<'_>,
        next: NextValidation<'_>,
    ) -> Result<ValidationResult, Vec<ServerError>> {
        let span = self
            .tracer
            .span_builder("validation")
            .with_kind(SpanKind::Server)
            .start(&*self.tracer);
        next.run(ctx)
            .with_context(OpenTelemetryContext::current_with_span(span))
            .map_ok(|res| {
                let current_cx = OpenTelemetryContext::current();
                let span = current_cx.span();
                span.set_attribute(KeyValue::new(KEY_COMPLEXITY, res.complexity as i64));
                span.set_attribute(KeyValue::new(KEY_DEPTH, res.depth as i64));
                res
            })
            .await
    }

    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        let span = self
            .tracer
            .span_builder("execute")
            .with_kind(SpanKind::Server)
            .start(&*self.tracer);
        next.run(ctx, operation_name)
            .with_context(OpenTelemetryContext::current_with_span(span))
            .await
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
            let attributes = vec![
                KeyValue::new(KEY_PARENT_TYPE, info.parent_type.to_string()),
                KeyValue::new(KEY_RETURN_TYPE, info.return_type.to_string()),
            ];
            Some(
                self.tracer
                    .span_builder(info.path_node.to_string())
                    .with_kind(SpanKind::Server)
                    .with_attributes(attributes)
                    .start(&*self.tracer),
            )
        } else {
            None
        };

        let fut = next.run(ctx, info).inspect_err(|err| {
            let current_cx = OpenTelemetryContext::current();
            current_cx.span().add_event(
                "error".to_string(),
                vec![KeyValue::new(KEY_ERROR, err.to_string())],
            );
        });

        match span {
            Some(span) => {
                fut.with_context(OpenTelemetryContext::current_with_span(span))
                    .await
            }
            None => fut.await,
        }
    }
}
