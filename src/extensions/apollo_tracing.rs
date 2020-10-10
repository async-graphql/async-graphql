use crate::extensions::{Extension, ExtensionContext, ExtensionFactory, ResolveInfo};
use crate::{Value, Variables};
use chrono::{DateTime, Utc};
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::ops::Deref;

struct PendingResolve {
    path: serde_json::Value,
    field_name: String,
    parent_type: String,
    return_type: String,
    start_time: DateTime<Utc>,
}

struct ResolveStat {
    pending_resolve: PendingResolve,
    end_time: DateTime<Utc>,
    start_offset: i64,
}

impl Deref for ResolveStat {
    type Target = PendingResolve;

    fn deref(&self) -> &Self::Target {
        &self.pending_resolve
    }
}

impl Serialize for ResolveStat {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("path", &self.path)?;
        map.serialize_entry("fieldName", &self.field_name)?;
        map.serialize_entry("parentType", &self.parent_type)?;
        map.serialize_entry("returnType", &self.return_type)?;
        map.serialize_entry("startOffset", &self.start_offset)?;
        map.serialize_entry(
            "duration",
            &(self.end_time - self.start_time).num_nanoseconds(),
        )?;
        map.end()
    }
}

/// Apollo tracing extension for performance tracing
///
/// Apollo Tracing works by including data in the extensions field of the GraphQL response, which is
/// reserved by the GraphQL spec for extra information that a server wants to return. That way, you
/// have access to performance traces alongside the data returned by your query.
/// It's already supported by `Apollo Engine`, and we're excited to see what other kinds of
/// integrations people can build on top of this format.
#[cfg_attr(feature = "nightly", doc(cfg(feature = "apollo_tracing")))]
pub struct ApolloTracing;

impl ExtensionFactory for ApolloTracing {
    fn create(&self) -> Box<dyn Extension> {
        Box::new(ApolloTracingExtension {
            start_time: Utc::now(),
            end_time: Utc::now(),
            pending_resolves: Default::default(),
            resolves: Default::default(),
        })
    }
}

struct ApolloTracingExtension {
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    pending_resolves: BTreeMap<usize, PendingResolve>,
    resolves: Vec<ResolveStat>,
}

impl Extension for ApolloTracingExtension {
    fn name(&self) -> Option<&'static str> {
        Some("tracing")
    }

    fn parse_start(
        &mut self,
        _ctx: &ExtensionContext<'_>,
        _query_source: &str,
        _variables: &Variables,
    ) {
        self.start_time = Utc::now();
    }

    fn execution_end(&mut self, _ctx: &ExtensionContext<'_>) {
        self.end_time = Utc::now();
    }

    fn resolve_start(&mut self, _ctx: &ExtensionContext<'_>, info: &ResolveInfo<'_>) {
        self.pending_resolves.insert(
            info.resolve_id.current,
            PendingResolve {
                path: serde_json::to_value(info.path_node).unwrap(),
                field_name: info.path_node.field_name().to_string(),
                parent_type: info.parent_type.to_string(),
                return_type: info.return_type.to_string(),
                start_time: Utc::now(),
            },
        );
    }

    fn resolve_end(&mut self, _ctx: &ExtensionContext<'_>, info: &ResolveInfo<'_>) {
        if let Some(pending_resolve) = self.pending_resolves.remove(&info.resolve_id.current) {
            let start_offset = (pending_resolve.start_time - self.start_time)
                .num_nanoseconds()
                .unwrap();
            self.resolves.push(ResolveStat {
                pending_resolve,
                start_offset,
                end_time: Utc::now(),
            });
        }
    }

    fn result(&mut self, _ctx: &ExtensionContext<'_>) -> Option<Value> {
        self.resolves
            .sort_by(|a, b| a.start_offset.cmp(&b.start_offset));

        serde_json::json!({
            "version": 1,
            "startTime": self.start_time.to_rfc3339(),
            "endTime": self.end_time.to_rfc3339(),
            "duration": (self.end_time - self.start_time).num_nanoseconds(),
            "execution": {
                "resolvers": self.resolves
            }
        })
        .try_into()
        .ok()
    }
}
