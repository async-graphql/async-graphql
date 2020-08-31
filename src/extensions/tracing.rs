use crate::extensions::{Extension, ResolveInfo};
use crate::Variables;
use std::collections::BTreeMap;
use tracing::{event, span, Id, Level};
use uuid::Uuid;

/// Tracing extension
///
/// # References
///
/// https://crates.io/crates/tracing
#[derive(Default)]
pub struct Tracing {
    root_id: Option<Id>,
    fields: BTreeMap<usize, Id>,
}

impl Extension for Tracing {
    #[allow(clippy::deref_addrof)]
    fn parse_start(&mut self, query_source: &str, variables: &Variables) {
        let root_span: tracing::Span = span!(
            target: "async_graphql::graphql",
            parent:None,
            Level::INFO,
            "graphql",
            id = %Uuid::new_v4().to_string(),
        );

        if let Some(id) = root_span.id() {
            tracing::dispatcher::get_default(|d| d.enter(&id));
            self.root_id.replace(id);
        }

        event!(
            target: "async_graphql::query",
            Level::DEBUG,
            %variables,
            query = %query_source
        );
    }

    fn execution_end(&mut self) {
        if let Some(id) = self.root_id.take() {
            tracing::dispatcher::get_default(|d| d.exit(&id));
        }
    }

    fn resolve_start(&mut self, info: &ResolveInfo<'_>) {
        let parent_span = info
            .resolve_id
            .parent
            .and_then(|id| self.fields.get(&id))
            .or_else(|| self.root_id.as_ref())
            .cloned();
        let span = span!(
            target: "async_graphql::field",
            parent: parent_span,
            Level::INFO,
            "field",
            path = %info.path_node,
        );
        if let Some(id) = span.id() {
            tracing::dispatcher::get_default(|d| d.enter(&id));
            self.fields.insert(info.resolve_id.current, id);
        }
    }

    fn resolve_end(&mut self, info: &ResolveInfo<'_>) {
        if let Some(id) = self.fields.remove(&info.resolve_id.current) {
            tracing::dispatcher::get_default(|d| d.exit(&id));
        }
    }
}
