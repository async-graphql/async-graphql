//! Extensions for schema

#[cfg(feature = "apollo_tracing")]
mod apollo_tracing;
#[cfg(feature = "log")]
mod logger;
#[cfg(feature = "tracing")]
mod tracing;

use crate::context::{QueryPathNode, ResolveId};
use crate::{Data, FieldResult, Result, Variables};

#[cfg(feature = "apollo_tracing")]
pub use self::apollo_tracing::ApolloTracing;
#[cfg(feature = "log")]
pub use self::logger::Logger;
#[cfg(feature = "tracing")]
pub use self::tracing::Tracing;
use crate::parser::types::ExecutableDocument;
use crate::Error;
use serde_json::Value;
use std::any::{Any, TypeId};

pub(crate) type BoxExtension = Box<dyn Extension>;

#[doc(hidden)]
pub struct Extensions(pub(crate) Vec<BoxExtension>);

/// Context for extension
pub struct ExtensionContext<'a> {
    #[doc(hidden)]
    pub schema_data: &'a Data,

    #[doc(hidden)]
    pub query_data: &'a Data,
}

impl<'a> ExtensionContext<'a> {
    /// Gets the global data defined in the `Context` or `Schema`.
    ///
    /// If both `Schema` and `Query` have the same data type, the data in the `Query` is obtained.
    ///
    /// # Errors
    ///
    /// Returns a `FieldError` if the specified type data does not exist.
    pub fn data<D: Any + Send + Sync>(&self) -> FieldResult<&D> {
        self.data_opt::<D>()
            .ok_or_else(|| format!("Data `{}` does not exist.", std::any::type_name::<D>()).into())
    }

    /// Gets the global data defined in the `Context` or `Schema`.
    ///
    /// # Panics
    ///
    /// It will panic if the specified data type does not exist.
    pub fn data_unchecked<D: Any + Send + Sync>(&self) -> &D {
        self.data_opt::<D>()
            .unwrap_or_else(|| panic!("Data `{}` does not exist.", std::any::type_name::<D>()))
    }

    /// Gets the global data defined in the `Context` or `Schema` or `None` if the specified type data does not exist.
    pub fn data_opt<D: Any + Send + Sync>(&self) -> Option<&D> {
        self.query_data
            .get(&TypeId::of::<D>())
            .or_else(|| self.schema_data.get(&TypeId::of::<D>()))
            .and_then(|d| d.downcast_ref::<D>())
    }
}

/// Parameters for `Extension::resolve_field_start`
pub struct ResolveInfo<'a> {
    /// Because resolver is concurrent, `Extension::resolve_field_start` and `Extension::resolve_field_end` are
    /// not strictly ordered, so each pair is identified by an id.
    pub resolve_id: ResolveId,

    /// Current path node, You can go through the entire path.
    pub path_node: &'a QueryPathNode<'a>,

    /// Parent type
    pub parent_type: &'a str,

    /// Current return type, is qualified name.
    pub return_type: &'a str,
}

/// Represents a GraphQL extension
#[allow(unused_variables)]
pub trait Extension: Sync + Send + 'static {
    /// If this extension needs to output data to query results, you need to specify a name.
    fn name(&self) -> Option<&'static str> {
        None
    }

    /// Called at the begin of the parse.
    fn parse_start(
        &mut self,
        ctx: &ExtensionContext<'_>,
        query_source: &str,
        variables: &Variables,
    ) {
    }

    /// Called at the end of the parse.
    fn parse_end(&mut self, ctx: &ExtensionContext<'_>, document: &ExecutableDocument) {}

    /// Called at the begin of the validation.
    fn validation_start(&mut self, ctx: &ExtensionContext<'_>) {}

    /// Called at the end of the validation.
    fn validation_end(&mut self, ctx: &ExtensionContext<'_>) {}

    /// Called at the begin of the execution.
    fn execution_start(&mut self, ctx: &ExtensionContext<'_>) {}

    /// Called at the end of the execution.
    fn execution_end(&mut self, ctx: &ExtensionContext<'_>) {}

    /// Called at the begin of the resolve field.
    fn resolve_start(&mut self, ctx: &ExtensionContext<'_>, info: &ResolveInfo<'_>) {}

    /// Called at the end of the resolve field.
    fn resolve_end(&mut self, ctx: &ExtensionContext<'_>, info: &ResolveInfo<'_>) {}

    /// Called when an error occurs.
    fn error(&mut self, ctx: &ExtensionContext<'_>, err: &Error) {}

    /// Get the results
    fn result(&mut self, ctx: &ExtensionContext<'_>) -> Option<serde_json::Value> {
        None
    }
}

pub(crate) trait ErrorLogger {
    fn log_error(self, ctx: &ExtensionContext<'_>, extensions: &spin::Mutex<Extensions>) -> Self;
}

impl<T> ErrorLogger for Result<T> {
    fn log_error(self, ctx: &ExtensionContext<'_>, extensions: &spin::Mutex<Extensions>) -> Self {
        if let Err(err) = &self {
            extensions.lock().error(ctx, err);
        }
        self
    }
}

impl Extension for Extensions {
    fn parse_start(
        &mut self,
        ctx: &ExtensionContext<'_>,
        query_source: &str,
        variables: &Variables,
    ) {
        self.0
            .iter_mut()
            .for_each(|e| e.parse_start(ctx, query_source, variables));
    }

    fn parse_end(&mut self, ctx: &ExtensionContext<'_>, document: &ExecutableDocument) {
        self.0.iter_mut().for_each(|e| e.parse_end(ctx, document));
    }

    fn validation_start(&mut self, ctx: &ExtensionContext<'_>) {
        self.0.iter_mut().for_each(|e| e.validation_start(ctx));
    }

    fn validation_end(&mut self, ctx: &ExtensionContext<'_>) {
        self.0.iter_mut().for_each(|e| e.validation_end(ctx));
    }

    fn execution_start(&mut self, ctx: &ExtensionContext<'_>) {
        self.0.iter_mut().for_each(|e| e.execution_start(ctx));
    }

    fn execution_end(&mut self, ctx: &ExtensionContext<'_>) {
        self.0.iter_mut().for_each(|e| e.execution_end(ctx));
    }

    fn resolve_start(&mut self, ctx: &ExtensionContext<'_>, info: &ResolveInfo<'_>) {
        self.0.iter_mut().for_each(|e| e.resolve_start(ctx, info));
    }

    fn resolve_end(&mut self, ctx: &ExtensionContext<'_>, resolve_id: &ResolveInfo<'_>) {
        self.0
            .iter_mut()
            .for_each(|e| e.resolve_end(ctx, resolve_id));
    }

    fn error(&mut self, ctx: &ExtensionContext<'_>, err: &Error) {
        self.0.iter_mut().for_each(|e| e.error(ctx, err));
    }

    fn result(&mut self, ctx: &ExtensionContext<'_>) -> Option<Value> {
        if !self.0.is_empty() {
            let value = self
                .0
                .iter_mut()
                .filter_map(|e| {
                    if let Some(name) = e.name() {
                        e.result(ctx).map(|res| (name.to_string(), res))
                    } else {
                        None
                    }
                })
                .collect::<serde_json::Map<_, _>>();
            if value.is_empty() {
                None
            } else {
                Some(value.into())
            }
        } else {
            None
        }
    }
}

/// Extension factory
///
/// Used to create an extension instance.
pub trait ExtensionFactory: Send + Sync + 'static {
    /// Create an extended instance.
    fn create(&self) -> Box<dyn Extension>;
}
