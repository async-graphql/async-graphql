use crate::extensions::{Extension, ResolveInfo};
use crate::parser::types::{ExecutableDocument, OperationType, Selection};
use crate::{Error, Variables};
use itertools::Itertools;
use log::{error, info, trace};
use std::borrow::Cow;
use uuid::Uuid;

/// Logger extension
#[cfg_attr(feature = "nightly", doc(cfg(feature = "log")))]
pub struct Logger {
    id: Uuid,
    enabled: bool,
    query: String,
    variables: Variables,
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            enabled: true,
            query: String::new(),
            variables: Default::default(),
        }
    }
}

impl Extension for Logger {
    fn parse_start(&mut self, query_source: &str, variables: &Variables) {
        self.query = query_source.replace(char::is_whitespace, "");
        self.variables = variables.clone();
    }

    fn parse_end(&mut self, document: &ExecutableDocument) {
        let is_schema = document
            .operations
            .iter()
            .filter(|(_, operation)| operation.node.ty == OperationType::Query)
            .any(|(_, operation)| operation.node.selection_set.node.items.iter().any(|selection| matches!(&selection.node, Selection::Field(field) if field.node.name.node == "__schema")));

        if is_schema {
            self.enabled = false;
            return;
        }

        info!(target: "async-graphql", "[Query] id: \"{}\", query: \"{}\", variables: {}", self.id, &self.query, self.variables);
    }

    fn resolve_start(&mut self, info: &ResolveInfo<'_>) {
        if !self.enabled {
            return;
        }
        trace!(target: "async-graphql", "[ResolveStart] id: \"{}\", path: \"{}\"", self.id, info.path_node);
    }

    fn resolve_end(&mut self, info: &ResolveInfo<'_>) {
        if !self.enabled {
            return;
        }
        trace!(target: "async-graphql", "[ResolveEnd] id: \"{}\", path: \"{}\"", self.id, info.path_node);
    }

    fn error(&mut self, err: &Error) {
        match err {
            Error::Parse(err) => {
                error!(
                    target: "async-graphql", "[ParseError] id: \"{}\", {}query: \"{}\", variables: {}, {}",
                    self.id,
                    if let Some(pos) = err.positions().next() {
                        // TODO: Make this more efficient
                        format!("pos: [{}:{}], ", pos.line, pos.column)
                    } else {
                        String::new()
                    },
                    self.query,
                    self.variables,
                    err
                )
            }
            Error::Query { pos, path, err } => {
                if let Some(path) = path {
                    let path = if let serde_json::Value::Array(values) = path {
                        values
                            .iter()
                            .filter_map(|value| match value {
                                serde_json::Value::String(s) => Some(Cow::Borrowed(s.as_str())),
                                serde_json::Value::Number(n) => Some(Cow::Owned(n.to_string())),
                                _ => None,
                            })
                            .join(".")
                    } else {
                        String::new()
                    };
                    error!(target: "async-graphql", "[QueryError] id: \"{}\", path: \"{}\", pos: [{}:{}], query: \"{}\", variables: {}, {}", self.id, path, pos.line, pos.column, self.query, self.variables, err)
                } else {
                    error!(target: "async-graphql", "[QueryError] id: \"{}\", pos: [{}:{}], query: \"{}\", variables: {}, {}", self.id, pos.line, pos.column, self.query, self.variables, err)
                }
            }
            Error::Rule { errors } => {
                for error in errors {
                    let locations = error
                        .locations
                        .iter()
                        .map(|pos| format!("{}:{}", pos.line, pos.column))
                        .join(", ");
                    error!(target: "async-graphql", "[ValidationError] id: \"{}\", pos: [{}], query: \"{}\", variables: {}, {}", self.id, locations, self.query, self.variables, error.message)
                }
            }
        }
    }
}
