use crate::extensions::{Extension, ExtensionContext, ExtensionFactory, ResolveInfo};
use crate::parser::types::{ExecutableDocument, OperationType, Selection};
use crate::{Error, Variables};
use itertools::Itertools;
use log::{error, info, trace};
use std::borrow::Cow;

/// Logger extension
#[cfg_attr(feature = "nightly", doc(cfg(feature = "log")))]
pub struct Logger;

impl ExtensionFactory for Logger {
    fn create(&self) -> Box<dyn Extension> {
        Box::new(LoggerExtension {
            enabled: true,
            query: String::new(),
            variables: Default::default(),
        })
    }
}

struct LoggerExtension {
    enabled: bool,
    query: String,
    variables: Variables,
}

impl Extension for LoggerExtension {
    fn parse_start(
        &mut self,
        _ctx: &ExtensionContext<'_>,
        query_source: &str,
        variables: &Variables,
    ) {
        self.query = query_source.replace(char::is_whitespace, "");
        self.variables = variables.clone();
    }

    fn parse_end(&mut self, _ctx: &ExtensionContext<'_>, document: &ExecutableDocument) {
        let is_schema = document
            .operations
            .iter()
            .filter(|(_, operation)| operation.node.ty == OperationType::Query)
            .any(|(_, operation)| operation.node.selection_set.node.items.iter().any(|selection| matches!(&selection.node, Selection::Field(field) if field.node.name.node == "__schema")));

        if is_schema {
            self.enabled = false;
            return;
        }

        info!(target: "async-graphql", "[Query] query: \"{}\", variables: {}", &self.query, self.variables);
    }

    fn resolve_start(&mut self, _ctx: &ExtensionContext<'_>, info: &ResolveInfo<'_>) {
        if !self.enabled {
            return;
        }
        trace!(target: "async-graphql", "[ResolveStart] path: \"{}\"", info.path_node);
    }

    fn resolve_end(&mut self, _ctx: &ExtensionContext<'_>, info: &ResolveInfo<'_>) {
        if !self.enabled {
            return;
        }
        trace!(target: "async-graphql", "[ResolveEnd] path: \"{}\"", info.path_node);
    }

    fn error(&mut self, _ctx: &ExtensionContext<'_>, err: &Error) {
        match err {
            Error::Parse(err) => {
                error!(
                    target: "async-graphql", "[ParseError] {}query: \"{}\", variables: {}, {}",
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
                    error!(target: "async-graphql", "[QueryError] path: \"{}\", pos: [{}:{}], query: \"{}\", variables: {}, {}", path, pos.line, pos.column, self.query, self.variables, err)
                } else {
                    error!(target: "async-graphql", "[QueryError] pos: [{}:{}], query: \"{}\", variables: {}, {}", pos.line, pos.column, self.query, self.variables, err)
                }
            }
            Error::Rule { errors } => {
                for error in errors.iter() {
                    let locations = error
                        .locations
                        .iter()
                        .map(|pos| format!("{}:{}", pos.line, pos.column))
                        .join(", ");
                    error!(target: "async-graphql", "[ValidationError] pos: [{}], query: \"{}\", variables: {}, {}", locations, self.query, self.variables, error.message)
                }
            }
        }
    }
}
