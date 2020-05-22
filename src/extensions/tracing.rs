use crate::extensions::{Extension, ResolveInfo};
use crate::QueryPathSegment;
use parking_lot::Mutex;
use std::collections::BTreeMap;
use tracing::{span, Id, Level};

#[derive(Default)]
struct Inner {
    root_id: Option<Id>,
    fields: BTreeMap<usize, Id>,
}

/// Tracing extension
///
/// # References
///
/// https://crates.io/crates/tracing
pub struct Tracing {
    inner: Mutex<Inner>,
}

impl Default for Tracing {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl Extension for Tracing {
    fn parse_start(&self, query_source: &str) {
        let root_span = span!(target: "async-graphql", parent:None, Level::INFO, "query", source = query_source);
        if let Some(id) = root_span.id() {
            tracing::dispatcher::get_default(|d| d.enter(&id));
            self.inner.lock().root_id.replace(id);
        }
    }

    fn execution_end(&self) {
        if let Some(id) = self.inner.lock().root_id.take() {
            tracing::dispatcher::get_default(|d| d.exit(&id));
        }
    }

    fn resolve_start(&self, info: &ResolveInfo<'_>) {
        let mut inner = self.inner.lock();
        let parent_span = info
            .resolve_id
            .parent
            .and_then(|id| inner.fields.get(&id))
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
            inner.fields.insert(info.resolve_id.current, id);
        }
    }

    fn resolve_end(&self, info: &ResolveInfo<'_>) {
        if let Some(id) = self.inner.lock().fields.remove(&info.resolve_id.current) {
            tracing::dispatcher::get_default(|d| d.exit(&id));
        }
    }
}
