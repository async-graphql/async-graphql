use crate::extensions::{Extension, ResolveInfo};
use crate::{QueryPathSegment, Variables};
use std::collections::BTreeMap;
use tracing::{span, Id, Level};

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
    fn parse_start(&mut self, query_source: &str, _variables: &Variables) {
        let root_span = span!(target: "async-graphql", parent:None, Level::INFO, "query", source = query_source);
        if let Some(id) = root_span.id() {
            tracing::dispatcher::get_default(|d| d.enter(&id));
            self.root_id.replace(id);
        }
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
            .cloned();
        let span = match &info.path_node.segment {
            QueryPathSegment::Index(idx) => span!(
                target: "async-graphql",
                parent: parent_span,
                Level::INFO,
                "field",
                index = *idx,
                parent_type = info.parent_type,
                return_type = info.return_type
            ),
            QueryPathSegment::Name(name) => span!(
                target: "async-graphql",
                parent: parent_span,
                Level::INFO,
                "field",
                name = name,
                parent_type = info.parent_type,
                return_type = info.return_type
            ),
        };
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
