use crate::extensions::{Extension, ExtensionContext, ExtensionFactory, ResolveInfo};
use crate::{ServerError, Variables};
use async_graphql_parser::types::ExecutableDocument;
use std::collections::BTreeMap;
use tracing::{span, Level, Span};

/// Tracing extension
///
/// # References
///
/// <https://crates.io/crates/tracing>
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
    root: Option<Span>,
    parse: Option<Span>,
    validation: Option<Span>,
    execute: Option<Span>,
    fields: BTreeMap<usize, Span>,
}

impl Extension for TracingExtension {
    fn parse_start(
        &mut self,
        _ctx: &ExtensionContext<'_>,
        query_source: &str,
        _variables: &Variables,
    ) {
        let root_span = span!(
            target: "async_graphql::graphql",
            parent: None,
            Level::INFO,
            "query",
            source = %query_source
        );

        let parse_span = span!(
            target: "async_graphql::graphql",
            parent: &root_span,
            Level::INFO,
            "parse"
        );

        root_span.with_subscriber(|(id, d)| d.enter(id));
        self.root.replace(root_span);

        parse_span.with_subscriber(|(id, d)| d.enter(id));
        self.parse.replace(parse_span);
    }

    fn parse_end(&mut self, _ctx: &ExtensionContext<'_>, _document: &ExecutableDocument) {
        self.parse
            .take()
            .and_then(|span| span.with_subscriber(|(id, d)| d.exit(id)));
    }

    fn validation_start(&mut self, _ctx: &ExtensionContext<'_>) {
        if let Some(parent) = &self.root {
            let validation_span = span!(
                target: "async_graphql::graphql",
                parent: parent,
                Level::INFO,
                "validation"
            );
            validation_span.with_subscriber(|(id, d)| d.enter(id));
            self.validation.replace(validation_span);
        }
    }

    fn validation_end(&mut self, _ctx: &ExtensionContext<'_>) {
        self.validation
            .take()
            .and_then(|span| span.with_subscriber(|(id, d)| d.exit(id)));
    }

    fn execution_start(&mut self, _ctx: &ExtensionContext<'_>) {
        let execute_span = if let Some(parent) = &self.root {
            span!(
                target: "async_graphql::graphql",
                parent: parent,
                Level::INFO,
                "execute"
            )
        } else {
            // For every step of the subscription stream.
            span!(
                target: "async_graphql::graphql",
                parent: None,
                Level::INFO,
                "execute"
            )
        };

        execute_span.with_subscriber(|(id, d)| d.enter(id));
        self.execute.replace(execute_span);
    }

    fn execution_end(&mut self, _ctx: &ExtensionContext<'_>) {
        self.execute
            .take()
            .and_then(|span| span.with_subscriber(|(id, d)| d.exit(id)));

        self.root
            .take()
            .and_then(|span| span.with_subscriber(|(id, d)| d.exit(id)));
    }

    fn resolve_start(&mut self, _ctx: &ExtensionContext<'_>, info: &ResolveInfo<'_>) {
        let parent_span = match info.resolve_id.parent {
            Some(parent_id) if parent_id > 0 => self.fields.get(&parent_id),
            _ => self.execute.as_ref(),
        };

        if let Some(parent_span) = parent_span {
            let span = span!(
                target: "async_graphql::graphql",
                parent: parent_span,
                Level::INFO,
                "field",
                id = %info.resolve_id.current,
                path = %info.path_node,
                parent_type = %info.parent_type,
                return_type = %info.return_type,
            );
            span.with_subscriber(|(id, d)| d.enter(id));
            self.fields.insert(info.resolve_id.current, span);
        }
    }

    fn resolve_end(&mut self, _ctx: &ExtensionContext<'_>, info: &ResolveInfo<'_>) {
        if let Some(span) = self.fields.remove(&info.resolve_id.current) {
            span.with_subscriber(|(id, d)| d.exit(id));
        }
    }

    fn error(&mut self, _ctx: &ExtensionContext<'_>, err: &ServerError) {
        tracing::error!(target: "async_graphql::graphql", error = %err.message);

        for span in self.fields.values() {
            span.with_subscriber(|(id, d)| d.exit(id));
        }
        self.fields.clear();

        self.execute
            .take()
            .and_then(|span| span.with_subscriber(|(id, d)| d.exit(id)));
        self.validation
            .take()
            .and_then(|span| span.with_subscriber(|(id, d)| d.exit(id)));
        self.parse
            .take()
            .and_then(|span| span.with_subscriber(|(id, d)| d.exit(id)));
        self.root
            .take()
            .and_then(|span| span.with_subscriber(|(id, d)| d.exit(id)));
    }
}
