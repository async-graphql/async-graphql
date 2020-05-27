use crate::extensions::{Extension, ResolveInfo};
use crate::Error;
use async_graphql_parser::query::{Definition, Document, OperationDefinition, Selection};
use itertools::Itertools;
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;

/// Logger extension
pub struct Logger {
    id: Uuid,
    enabled: AtomicBool,
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            enabled: AtomicBool::new(true),
        }
    }
}

impl Extension for Logger {
    fn parse_end(&self, query_source: &str, document: &Document) {
        let mut is_schema = false;

        for definition in document.definitions() {
            if let Definition::Operation(operation) = &definition.node {
                let selection_set = match &operation.node {
                    OperationDefinition::Query(query) => &query.selection_set,
                    OperationDefinition::SelectionSet(selection_set) => &selection_set.node,
                    _ => continue,
                };
                is_schema = selection_set.items.iter().any(|selection| {
                    if let Selection::Field(field) = &selection.node {
                        if field.name.as_str() == "__schema" {
                            return true;
                        }
                    }
                    false
                });
                if is_schema {
                    break;
                }
            }
        }

        if is_schema {
            self.enabled.store(false, Ordering::Relaxed);
            return;
        }

        info!(target: "async-graphql", "query, id: {}, source: \"{}\"", self.id, query_source);
    }

    fn resolve_start(&self, info: &ResolveInfo<'_>) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }
        trace!(target: "async-graphql", "resolve start, id: {}, path: \"{}\"", self.id, info.path_node);
    }

    fn resolve_end(&self, info: &ResolveInfo<'_>) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }
        trace!(target: "async-graphql", "resolve end, id: {}, path: \"{}\"", self.id, info.path_node);
    }

    fn error(&self, err: &Error) {
        match err {
            Error::Parse(err) => {
                error!(target: "async-graphql", "parse error, id: {}, [{}:{}] {}", self.id, err.pos.line, err.pos.column, err)
            }
            Error::Query { pos, path, err } => {
                if let Some(path) = path {
                    error!(target: "async-graphql", "query error, id: {}, path: \"{}\", [{}:{}] {}", self.id, path, pos.line, pos.column, err)
                } else {
                    error!(target: "async-graphql", "query error, id: {}, [{}:{}] {}", self.id, pos.line, pos.column, err)
                }
            }
            Error::Rule { errors } => {
                for error in errors {
                    let locations = error
                        .locations
                        .iter()
                        .map(|pos| format!("{}:{}", pos.line, pos.column))
                        .join(", ");
                    error!(target: "async-graphql", "validation error, id: {}, [{}] {}", self.id, locations, error.message)
                }
            }
        }
    }
}
