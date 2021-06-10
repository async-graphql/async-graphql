use std::sync::Arc;

use chrono::{DateTime, Utc};
use futures_util::lock::Mutex;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

use crate::extensions::{
    Extension, ExtensionContext, ExtensionFactory, NextExecute, NextResolve, ResolveInfo,
};
use crate::{value, Response, ServerResult, Value};

struct ResolveState {
    path: Vec<String>,
    field_name: String,
    parent_type: String,
    return_type: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    start_offset: i64,
}

impl Serialize for ResolveState {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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
#[cfg_attr(docsrs, doc(cfg(feature = "apollo_tracing")))]
pub struct ApolloTracing;

impl ExtensionFactory for ApolloTracing {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(ApolloTracingExtension {
            inner: Mutex::new(Inner {
                start_time: Utc::now(),
                end_time: Utc::now(),
                resolves: Default::default(),
            }),
        })
    }
}

struct Inner {
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    resolves: Vec<ResolveState>,
}

struct ApolloTracingExtension {
    inner: Mutex<Inner>,
}

#[async_trait::async_trait]
impl Extension for ApolloTracingExtension {
    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        self.inner.lock().await.start_time = Utc::now();
        let resp = next.run(ctx, operation_name).await;

        let mut inner = self.inner.lock().await;
        inner.end_time = Utc::now();
        inner
            .resolves
            .sort_by(|a, b| a.start_offset.cmp(&b.start_offset));
        resp.extension(
            "tracing",
            value!({
                "version": 1,
                "startTime": inner.start_time.to_rfc3339(),
                "endTime": inner.end_time.to_rfc3339(),
                "duration": (inner.end_time - inner.start_time).num_nanoseconds(),
                "execution": {
                    "resolvers": inner.resolves
                }
            }),
        )
    }

    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextResolve<'_>,
    ) -> ServerResult<Option<Value>> {
        let path = info.path_node.to_string_vec();
        let field_name = info.path_node.field_name().to_string();
        let parent_type = info.parent_type.to_string();
        let return_type = info.return_type.to_string();
        let start_time = Utc::now();
        let start_offset = (start_time - self.inner.lock().await.start_time)
            .num_nanoseconds()
            .unwrap();

        let res = next.run(ctx, info).await;
        let end_time = Utc::now();

        self.inner.lock().await.resolves.push(ResolveState {
            path,
            field_name,
            parent_type,
            return_type,
            start_time,
            end_time,
            start_offset,
        });
        res
    }
}
