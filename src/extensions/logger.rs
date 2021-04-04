use std::fmt::{self, Display, Formatter};
use std::sync::Arc;

use futures_util::lock::Mutex;

use crate::extensions::{
    Extension, ExtensionContext, ExtensionFactory, NextExtension, ResolveInfo,
};
use crate::parser::types::{ExecutableDocument, OperationType, Selection};
use crate::{PathSegment, ServerError, ServerResult, Value, Variables};

/// Logger extension
#[cfg_attr(docsrs, doc(cfg(feature = "log")))]
pub struct Logger;

impl ExtensionFactory for Logger {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(LoggerExtension {
            inner: Mutex::new(Inner {
                enabled: true,
                query: String::new(),
                variables: Default::default(),
            }),
        })
    }
}

struct Inner {
    enabled: bool,
    query: String,
    variables: Variables,
}

struct LoggerExtension {
    inner: Mutex<Inner>,
}

#[async_trait::async_trait]
impl Extension for LoggerExtension {
    async fn parse_query(
        &self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
        next: NextExtension<'_>,
    ) -> ServerResult<ExecutableDocument> {
        let mut inner = self.inner.lock().await;
        inner.query = query.replace(char::is_whitespace, "");
        inner.variables = variables.clone();
        let document = next.parse_query(ctx, query, variables).await?;
        let is_schema = document
            .operations
            .iter()
            .filter(|(_, operation)| operation.node.ty == OperationType::Query)
            .any(|(_, operation)| operation.node.selection_set.node.items.iter().any(|selection| matches!(&selection.node, Selection::Field(field) if field.node.name.node == "__schema")));
        inner.enabled = !is_schema;
        Ok(document)
    }

    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextExtension<'_>,
    ) -> ServerResult<Option<Value>> {
        let enabled = self.inner.lock().await.enabled;
        if enabled {
            let path = info.path_node.to_string();
            log::trace!(target: "async-graphql", "[ResolveStart] path: \"{}\"", path);
            let res = next.resolve(ctx, info).await;
            if let Err(err) = &res {
                let inner = self.inner.lock().await;
                log::error!(
                    target: "async-graphql",
                    "{}",
                    DisplayError { query:&inner.query,variables:&inner.variables, e: &err }
                );
            }
            log::trace!(target: "async-graphql", "[ResolveEnd] path: \"{}\"", path);
            res
        } else {
            next.resolve(ctx, info).await
        }
    }
}

struct DisplayError<'a> {
    query: &'a str,
    variables: &'a Variables,
    e: &'a ServerError,
}
impl<'a> Display for DisplayError<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[Error] ")?;

        if !self.e.path.is_empty() {
            write!(f, "path: ")?;
            for (i, segment) in self.e.path.iter().enumerate() {
                if i != 0 {
                    write!(f, ".")?;
                }

                match segment {
                    PathSegment::Field(field) => write!(f, "{}", field),
                    PathSegment::Index(i) => write!(f, "{}", i),
                }?;
            }
            write!(f, ", ")?;
        }
        if !self.e.locations.is_empty() {
            write!(f, "pos: [")?;
            for (i, location) in self.e.locations.iter().enumerate() {
                if i != 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}:{}", location.line, location.column)?;
            }
            write!(f, "], ")?;
        }
        write!(f, r#"query: "{}", "#, self.query)?;
        write!(f, "variables: {}", self.variables)?;
        write!(f, "{}", self.e.message)
    }
}
