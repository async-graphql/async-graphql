use crate::extensions::{Extension, ResolveInfo};
use crate::{Error, Variables};
use async_graphql_parser::query::{Definition, Document, OperationDefinition, Selection};
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

    fn parse_end(&mut self, document: &Document) {
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
