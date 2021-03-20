use std::collections::HashMap;

use tracing::{span, Level, Span};

use crate::extensions::{Extension, ExtensionContext, ExtensionFactory, ResolveInfo};
use crate::parser::types::ExecutableDocument;
use crate::{ServerError, ValidationResult, Variables};

/// Tracing extension configuration for each request.
#[derive(Default)]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "tracing")))]
pub struct TracingConfig {
    /// Use a span as the parent node of the entire query.
    parent: Option<Span>,
}

impl TracingConfig {
    /// Use a span as the parent node of the entire query.
    pub fn parent_span(mut self, span: Span) -> Self {
        self.parent = Some(span);
        self
    }
}

const REQUEST_CTX: usize = 0;
const PARSE_CTX: usize = 1;
const VALIDATION_CTX: usize = 2;
const EXECUTE_CTX: usize = 3;

#[inline]
fn resolve_span_id(resolver_id: usize) -> usize {
    resolver_id + 10
}

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
/// use async_graphql::extensions::{Tracing, TracingConfig};
/// use tracing::{span, Level};
///
/// #[derive(SimpleObject)]
/// struct Query {
///     value: i32,
/// }
///
/// let schema = Schema::build(Query { value: 100 }, EmptyMutation, EmptySubscription).
///     extension(Tracing::default())
///     .finish();
///
/// let root_span = span!(
///     parent: None,
///     Level::INFO,
///     "span root"
/// );
///
/// tokio::runtime::Runtime::new().unwrap().block_on(async move {
///     let request = Request::new("{ value }")
///         .data(TracingConfig::default().parent_span(root_span));
///     schema.execute(request).await;
/// });
/// ```
#[derive(Default)]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "tracing")))]
pub struct Tracing;

impl ExtensionFactory for Tracing {
    fn create(&self) -> Box<dyn Extension> {
        Box::new(TracingExtension::default())
    }
}

#[derive(Default)]
struct TracingExtension {
    spans: HashMap<usize, Span>,
}

impl TracingExtension {
    fn enter_span(&mut self, id: usize, span: Span) -> &Span {
        let _ = span.enter();
        self.spans.insert(id, span);
        self.spans.get(&id).unwrap()
    }

    fn exit_span(&mut self, id: usize) {
        if let Some(span) = self.spans.remove(&id) {
            let _ = span.enter();
        }
    }
}

impl Extension for TracingExtension {
    fn parse_start(
        &mut self,
        ctx: &ExtensionContext<'_>,
        query_source: &str,
        variables: &Variables,
    ) {
        let request_span = ctx
            .data_opt::<TracingConfig>()
            .and_then(|cfg| cfg.parent.as_ref())
            .cloned()
            .unwrap_or_else(|| {
                span!(
                    target: "async_graphql::graphql",
                    parent: None,
                    Level::INFO,
                    "request",
                )
            });

        let variables = serde_json::to_string(&variables).unwrap();
        let parse_span = span!(
            target: "async_graphql::graphql",
            parent: &request_span,
            Level::INFO,
            "parse",
            source = query_source,
            variables = %variables,
        );

        self.enter_span(REQUEST_CTX, request_span);
        self.enter_span(PARSE_CTX, parse_span);
    }

    fn parse_end(&mut self, _ctx: &ExtensionContext<'_>, _document: &ExecutableDocument) {
        self.exit_span(PARSE_CTX);
    }

    fn validation_start(&mut self, _ctx: &ExtensionContext<'_>) {
        if let Some(parent) = self.spans.get(&REQUEST_CTX) {
            let span = span!(
                target: "async_graphql::graphql",
                parent: parent,
                Level::INFO,
                "validation"
            );
            self.enter_span(VALIDATION_CTX, span);
        }
    }

    fn validation_end(&mut self, _ctx: &ExtensionContext<'_>, _result: &ValidationResult) {
        self.exit_span(VALIDATION_CTX);
    }

    fn execution_start(&mut self, _ctx: &ExtensionContext<'_>) {
        let span = match self.spans.get(&REQUEST_CTX) {
            Some(parent) => span!(
                target: "async_graphql::graphql",
                parent: parent,
                Level::INFO,
                "execute"
            ),
            None => span!(
                target: "async_graphql::graphql",
                parent: None,
                Level::INFO,
                "execute"
            ),
        };

        self.enter_span(EXECUTE_CTX, span);
    }

    fn execution_end(&mut self, ctx: &ExtensionContext<'_>) {
        self.exit_span(EXECUTE_CTX);
        if ctx.data_opt::<TracingConfig>().is_some() {
            self.exit_span(REQUEST_CTX);
        }
    }

    fn resolve_start(&mut self, _ctx: &ExtensionContext<'_>, info: &ResolveInfo<'_>) {
        let parent = match info.resolve_id.parent {
            Some(parent_id) if parent_id > 0 => self.spans.get(&resolve_span_id(parent_id)),
            _ => self.spans.get(&EXECUTE_CTX),
        };

        if let Some(parent) = parent {
            let span = span!(
                target: "async_graphql::graphql",
                parent: parent,
                Level::INFO,
                "field",
                id = %info.resolve_id.current,
                path = %info.path_node,
                parent_type = %info.parent_type,
                return_type = %info.return_type,
            );
            self.enter_span(resolve_span_id(info.resolve_id.current), span);
        }
    }

    fn resolve_end(&mut self, _ctx: &ExtensionContext<'_>, info: &ResolveInfo<'_>) {
        self.exit_span(resolve_span_id(info.resolve_id.current));
    }

    fn error(&mut self, _ctx: &ExtensionContext<'_>, err: &ServerError) {
        tracing::error!(target: "async_graphql::graphql", error = %err.message);
    }
}
