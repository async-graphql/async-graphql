use crate::extensions::{Extension, ResolveInfo};
use crate::parser::types::{ExecutableDefinition, ExecutableDocument, OperationType, Selection};
use crate::{Error, Variables};
use itertools::Itertools;
use log::{error, info, trace};
use std::borrow::Cow;
use uuid::Uuid;

/// Logger extension
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
            .definitions
            .iter()
            .filter_map(|definition| match definition {
                ExecutableDefinition::Operation(operation) if operation.node.ty == OperationType::Query => Some(operation),
                _ => None,
            })
            .any(|operation| operation.node.selection_set.node.items.iter().any(|selection| matches!(&selection.node, Selection::Field(field) if field.node.name.node == "__schema")));

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
                error!(target: "async-graphql", "[ParseError] id: \"{}\", pos: [{}:{}], query: \"{}\", variables: {}, {}", self.id, err.pos.line, err.pos.column, self.query, self.variables, err)
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
