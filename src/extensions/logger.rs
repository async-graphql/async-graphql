use std::fmt::{self, Display, Formatter};

use log::{error, info, trace};

use crate::extensions::{Extension, ExtensionContext, ExtensionFactory, ResolveInfo};
use crate::parser::types::{ExecutableDocument, OperationType, Selection};
use crate::{PathSegment, ServerError, Variables};

/// Logger extension
#[cfg_attr(docrs, doc(cfg(feature = "log")))]
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

    fn error(&mut self, _ctx: &ExtensionContext<'_>, err: &ServerError) {
        struct DisplayError<'a> {
            log: &'a LoggerExtension,
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
                write!(f, r#"query: "{}", "#, self.log.query)?;
                write!(f, "variables: {}", self.log.variables)?;
                write!(f, "{}", self.e.message)
            }
        }

        error!(
            target: "async-graphql",
            "{}",
            DisplayError {
                log: self,
                e: err,
            }
        );
    }
}
