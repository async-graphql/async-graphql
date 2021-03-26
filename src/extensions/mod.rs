//! Extensions for schema

mod analyzer;
#[cfg(feature = "apollo_persisted_queries")]
pub mod apollo_persisted_queries;
#[cfg(feature = "apollo_tracing")]
mod apollo_tracing;
#[cfg(feature = "log")]
mod logger;
#[cfg(feature = "opentelemetry")]
mod opentelemetry;
#[cfg(feature = "tracing")]
mod tracing;

use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::context::{QueryPathNode, ResolveId};
use crate::parser::types::ExecutableDocument;
use crate::{
    Data, Request, Result, SchemaEnv, ServerError, ServerResult, ValidationResult, Variables,
};
use crate::{Error, Name, Value};

pub use self::analyzer::Analyzer;
#[cfg(feature = "apollo_tracing")]
pub use self::apollo_tracing::ApolloTracing;
#[cfg(feature = "log")]
pub use self::logger::Logger;
#[cfg(feature = "opentelemetry")]
pub use self::opentelemetry::{OpenTelemetry, OpenTelemetryConfig};
#[cfg(feature = "tracing")]
pub use self::tracing::{Tracing, TracingConfig};

pub(crate) type BoxExtension = Box<dyn Extension>;

/// Context for extension
pub struct ExtensionContext<'a> {
    #[doc(hidden)]
    pub schema_data: &'a Data,

    #[doc(hidden)]
    pub session_data: &'a Data,

    #[doc(hidden)]
    pub query_data: Option<&'a Data>,
}

impl<'a> ExtensionContext<'a> {
    /// Gets the global data defined in the `Context` or `Schema`.
    ///
    /// If both `Schema` and `Query` have the same data type, the data in the `Query` is obtained.
    ///
    /// # Errors
    ///
    /// Returns a `Error` if the specified type data does not exist.
    pub fn data<D: Any + Send + Sync>(&self) -> Result<&'a D> {
        self.data_opt::<D>().ok_or_else(|| {
            Error::new(format!(
                "Data `{}` does not exist.",
                std::any::type_name::<D>()
            ))
        })
    }

    /// Gets the global data defined in the `Context` or `Schema`.
    ///
    /// # Panics
    ///
    /// It will panic if the specified data type does not exist.
    pub fn data_unchecked<D: Any + Send + Sync>(&self) -> &'a D {
        self.data_opt::<D>()
            .unwrap_or_else(|| panic!("Data `{}` does not exist.", std::any::type_name::<D>()))
    }

    /// Gets the global data defined in the `Context` or `Schema` or `None` if the specified type data does not exist.
    pub fn data_opt<D: Any + Send + Sync>(&self) -> Option<&'a D> {
        self.query_data
            .and_then(|query_data| query_data.get(&TypeId::of::<D>()))
            .or_else(|| self.session_data.get(&TypeId::of::<D>()))
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
///
/// # Call order for query and mutation
///
/// - start
///     - prepare_request
///     - parse_start
///     - parse_end
///     - validation_start
///     - validation_end
///     - execution_start
///         - resolve_start
///         - resolve_end
///     - result
///     - execution_end
/// - end
///     
/// # Call order for subscription
///
/// - start
/// - prepare_request
/// - parse_start
/// - parse_end
/// - validation_start
/// - validation_end
///     - execution_start
///         - resolve_start
///         - resolve_end
///     - execution_end
///     - result
/// ```
#[async_trait::async_trait]
#[allow(unused_variables)]
pub trait Extension: Sync + Send + 'static {
    /// If this extension needs to output data to query results, you need to specify a name.
    fn name(&self) -> Option<&'static str> {
        None
    }

    /// Called at the beginning of query.
    fn start(&mut self, ctx: &ExtensionContext<'_>) {}

    /// Called at the beginning of query.
    fn end(&mut self, ctx: &ExtensionContext<'_>) {}

    /// Called at prepare request.
    async fn prepare_request(
        &mut self,
        ctx: &ExtensionContext<'_>,
        request: Request,
    ) -> ServerResult<Request> {
        Ok(request)
    }

    /// Called at the beginning of parse query source.
    fn parse_start(
        &mut self,
        ctx: &ExtensionContext<'_>,
        query_source: &str,
        variables: &Variables,
    ) {
    }

    /// Called at the end of parse query source.
    fn parse_end(&mut self, ctx: &ExtensionContext<'_>, document: &ExecutableDocument) {}

    /// Called at the beginning of the validation.
    fn validation_start(&mut self, ctx: &ExtensionContext<'_>) {}

    /// Called at the end of the validation.
    fn validation_end(&mut self, ctx: &ExtensionContext<'_>, result: &ValidationResult) {}

    /// Called at the beginning of execute a query.
    fn execution_start(&mut self, ctx: &ExtensionContext<'_>) {}

    /// Called at the end of execute a query.
    fn execution_end(&mut self, ctx: &ExtensionContext<'_>) {}

    /// Called at the beginning of resolve a field.
    fn resolve_start(&mut self, ctx: &ExtensionContext<'_>, info: &ResolveInfo<'_>) {}

    /// Called at the end of resolve a field.
    fn resolve_end(&mut self, ctx: &ExtensionContext<'_>, info: &ResolveInfo<'_>) {}

    /// Called when an error occurs.
    fn error(&mut self, ctx: &ExtensionContext<'_>, err: &ServerError) {}

    /// Get the results.
    fn result(&mut self, ctx: &ExtensionContext<'_>) -> Option<Value> {
        None
    }
}

pub(crate) trait ErrorLogger {
    fn log_error(self, extensions: &Extensions) -> Self;
}

impl<T> ErrorLogger for ServerResult<T> {
    fn log_error(self, extensions: &Extensions) -> Self {
        if let Err(err) = &self {
            extensions.error(err);
        }
        self
    }
}

impl<T> ErrorLogger for Result<T, Vec<ServerError>> {
    fn log_error(self, extensions: &Extensions) -> Self {
        if let Err(errors) = &self {
            for error in errors {
                extensions.error(error);
            }
        }
        self
    }
}

/// Extension factory
///
/// Used to create an extension instance.
pub trait ExtensionFactory: Send + Sync + 'static {
    /// Create an extended instance.
    fn create(&self) -> Box<dyn Extension>;
}

#[doc(hidden)]
pub struct Extensions {
    extensions: Option<spin::Mutex<Vec<BoxExtension>>>,
    schema_env: SchemaEnv,
    session_data: Arc<Data>,
    query_data: Option<Arc<Data>>,
}

#[doc(hidden)]
impl Extensions {
    pub fn new(
        extensions: Vec<BoxExtension>,
        schema_env: SchemaEnv,
        session_data: Arc<Data>,
    ) -> Self {
        Extensions {
            extensions: if extensions.is_empty() {
                None
            } else {
                Some(spin::Mutex::new(extensions))
            },
            schema_env,
            session_data,
            query_data: None,
        }
    }

    pub fn attach_query_data(&mut self, data: Arc<Data>) {
        self.query_data = Some(data);
    }
}

impl Drop for Extensions {
    fn drop(&mut self) {
        self.end();
    }
}

#[doc(hidden)]
impl Extensions {
    #[inline]
    fn context(&self) -> ExtensionContext<'_> {
        ExtensionContext {
            schema_data: &self.schema_env.data,
            session_data: &self.session_data,
            query_data: self.query_data.as_deref(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.extensions.is_none()
    }

    pub fn start(&self) {
        if let Some(e) = &self.extensions {
            e.lock().iter_mut().for_each(|e| e.start(&self.context()));
        }
    }

    pub fn end(&self) {
        if let Some(e) = &self.extensions {
            e.lock().iter_mut().for_each(|e| e.end(&self.context()));
        }
    }

    pub async fn prepare_request(&self, request: Request) -> ServerResult<Request> {
        let mut request = request;
        if let Some(e) = &self.extensions {
            for e in e.lock().iter_mut() {
                request = e.prepare_request(&self.context(), request).await?;
            }
        }
        Ok(request)
    }

    pub fn parse_start(&self, query_source: &str, variables: &Variables) {
        if let Some(e) = &self.extensions {
            e.lock()
                .iter_mut()
                .for_each(|e| e.parse_start(&self.context(), query_source, variables));
        }
    }

    pub fn parse_end(&self, document: &ExecutableDocument) {
        if let Some(e) = &self.extensions {
            e.lock()
                .iter_mut()
                .for_each(|e| e.parse_end(&self.context(), document));
        }
    }

    pub fn validation_start(&self) {
        if let Some(e) = &self.extensions {
            e.lock()
                .iter_mut()
                .for_each(|e| e.validation_start(&self.context()));
        }
    }

    pub fn validation_end(&self, result: &ValidationResult) {
        if let Some(e) = &self.extensions {
            e.lock()
                .iter_mut()
                .for_each(|e| e.validation_end(&self.context(), result));
        }
    }

    pub fn execution_start(&self) {
        if let Some(e) = &self.extensions {
            e.lock()
                .iter_mut()
                .for_each(|e| e.execution_start(&self.context()));
        }
    }

    pub fn execution_end(&self) {
        if let Some(e) = &self.extensions {
            e.lock()
                .iter_mut()
                .for_each(|e| e.execution_end(&self.context()));
        }
    }

    pub fn resolve_start(&self, info: &ResolveInfo<'_>) {
        if let Some(e) = &self.extensions {
            e.lock()
                .iter_mut()
                .for_each(|e| e.resolve_start(&self.context(), info));
        }
    }

    pub fn resolve_end(&self, resolve_id: &ResolveInfo<'_>) {
        if let Some(e) = &self.extensions {
            e.lock()
                .iter_mut()
                .for_each(|e| e.resolve_end(&self.context(), resolve_id));
        }
    }

    pub fn error(&self, err: &ServerError) {
        if let Some(e) = &self.extensions {
            e.lock()
                .iter_mut()
                .for_each(|e| e.error(&self.context(), err));
        }
    }

    pub fn result(&self) -> Option<Value> {
        if let Some(e) = &self.extensions {
            let value = e
                .lock()
                .iter_mut()
                .filter_map(|e| {
                    if let Some(name) = e.name() {
                        e.result(&self.context()).map(|res| (Name::new(name), res))
                    } else {
                        None
                    }
                })
                .collect::<BTreeMap<_, _>>();
            if value.is_empty() {
                None
            } else {
                Some(Value::Object(value))
            }
        } else {
            None
        }
    }
}
