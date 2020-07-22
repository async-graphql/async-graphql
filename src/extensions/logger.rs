use crate::extensions::{Extension, ResolveInfo};
use crate::{Error, Variables};
use async_graphql_parser::query::{Definition, Document, OperationDefinition, Selection};
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

        info!(
            target = "async-graphql",
            "Query",
            id = self.id,
            source = self.query,
            variables = self.variables
        );
    }

    fn resolve_start(&mut self, info: &ResolveInfo<'_>) {
        if !self.enabled {
            return;
        }

        trace!(
            target = "async-graphql",
            "Resolve start",
            id = self.id,
            path = info.path_node
        );
    }

    fn resolve_end(&mut self, info: &ResolveInfo<'_>) {
        if !self.enabled {
            return;
        }

        trace!(
            target = "async-graphql",
            "Resolve end",
            id = self.id,
            path = info.path_node
        );
    }

    fn error(&mut self, err: &Error) {
        match err {
            Error::Parse(err) => {
                error!(
                    target = "async-graphql",
                    "Parse error",
                    id = self.id,
                    pos = err.pos,
                    query = self.query,
                    variables = self.variables,
                    message = err.message
                );
            }
            Error::Query { pos, path, err } => {
                error!(
                    target = "async-graphql",
                    "Query error",
                    id = self.id,
                    pos = pos,
                    path = path,
                    query = self.query,
                    variables = self.variables,
                    error = err,
                );
            }
            Error::Rule { errors } => {
                for err in errors {
                    error!(
                        target = "async-graphql",
                        "Validation error",
                        id = self.id,
                        pos = err.locations,
                        query = self.query,
                        variables = self.variables,
                        error = err.message,
                    );
                }
            }
        }
    }
}
