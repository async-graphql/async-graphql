use crate::extensions::{Extension, ResolveInfo};
use crate::Error;
use itertools::Itertools;
use uuid::Uuid;

/// Logger extension
pub struct Logger(Uuid);

impl Default for Logger {
    fn default() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Extension for Logger {
    fn parse_start(&self, query_source: &str) {
        info!(target: "async-graphql", "query start, id: {}, source: \"{}\"", self.0, query_source);
    }

    fn resolve_start(&self, info: &ResolveInfo<'_>) {
        trace!(target: "async-graphql", "resolve start, id: {}, path: \"{}\"", self.0, info.path_node);
    }

    fn resolve_end(&self, info: &ResolveInfo<'_>) {
        trace!(target: "async-graphql", "resolve end, id: {}, path: \"{}\"", self.0, info.path_node);
    }

    fn error(&self, err: &Error) {
        match err {
            Error::Parse(err) => {
                error!(target: "async-graphql", "parse error, id: {}, [{}:{}] {}", self.0, err.pos.line, err.pos.column, err)
            }
            Error::Query { pos, path, err } => {
                if let Some(path) = path {
                    error!(target: "async-graphql", "query error, id: {}, path: \"{}\", [{}:{}] {}", self.0, path, pos.line, pos.column, err)
                } else {
                    error!(target: "async-graphql", "query error, id: {}, [{}:{}] {}", self.0, pos.line, pos.column, err)
                }
            }
            Error::Rule { errors } => {
                for error in errors {
                    let locations = error
                        .locations
                        .iter()
                        .map(|pos| format!("{}:{}", pos.line, pos.column))
                        .join(", ");
                    error!(target: "async-graphql", "validation error, id: {}, [{}] {}", self.0, locations, error.message)
                }
            }
        }
    }
}
