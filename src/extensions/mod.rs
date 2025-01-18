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

use std::{
    any::{Any, TypeId},
    future::Future,
    sync::Arc,
};

use futures_util::{future::BoxFuture, stream::BoxStream, FutureExt};

pub use self::analyzer::Analyzer;
#[cfg(feature = "apollo_tracing")]
pub use self::apollo_tracing::ApolloTracing;
#[cfg(feature = "log")]
pub use self::logger::Logger;
#[cfg(feature = "opentelemetry")]
pub use self::opentelemetry::OpenTelemetry;
#[cfg(feature = "tracing")]
pub use self::tracing::Tracing;
use crate::{
    parser::types::{ExecutableDocument, Field},
    Data, DataContext, Error, QueryPathNode, Request, Response, Result, SDLExportOptions,
    SchemaEnv, ServerError, ServerResult, ValidationResult, Value, Variables,
};

/// Context for extension
pub struct ExtensionContext<'a> {
    #[doc(hidden)]
    pub schema_env: &'a SchemaEnv,

    #[doc(hidden)]
    pub session_data: &'a Data,

    #[doc(hidden)]
    pub query_data: Option<&'a Data>,
}

impl<'a> DataContext<'a> for ExtensionContext<'a> {
    fn data<D: Any + Send + Sync>(&self) -> Result<&'a D> {
        ExtensionContext::data::<D>(self)
    }

    fn data_unchecked<D: Any + Send + Sync>(&self) -> &'a D {
        ExtensionContext::data_unchecked::<D>(self)
    }

    fn data_opt<D: Any + Send + Sync>(&self) -> Option<&'a D> {
        ExtensionContext::data_opt::<D>(self)
    }
}

impl<'a> ExtensionContext<'a> {
    /// Convert the specified [ExecutableDocument] into a query string.
    ///
    /// Usually used for log extension, it can hide secret arguments.
    pub fn stringify_execute_doc(&self, doc: &ExecutableDocument, variables: &Variables) -> String {
        self.schema_env
            .registry
            .stringify_exec_doc(variables, doc)
            .unwrap_or_default()
    }

    /// Returns SDL(Schema Definition Language) of this schema.
    pub fn sdl(&self) -> String {
        self.schema_env.registry.export_sdl(Default::default())
    }

    /// Returns SDL(Schema Definition Language) of this schema with options.
    pub fn sdl_with_options(&self, options: SDLExportOptions) -> String {
        self.schema_env.registry.export_sdl(options)
    }

    /// Gets the global data defined in the `Context` or `Schema`.
    ///
    /// If both `Schema` and `Query` have the same data type, the data in the
    /// `Query` is obtained.
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

    /// Gets the global data defined in the `Context` or `Schema` or `None` if
    /// the specified type data does not exist.
    pub fn data_opt<D: Any + Send + Sync>(&self) -> Option<&'a D> {
        self.query_data
            .and_then(|query_data| query_data.get(&TypeId::of::<D>()))
            .or_else(|| self.session_data.get(&TypeId::of::<D>()))
            .or_else(|| self.schema_env.data.get(&TypeId::of::<D>()))
            .and_then(|d| d.downcast_ref::<D>())
    }
}

/// Parameters for `Extension::resolve_field_start`
pub struct ResolveInfo<'a> {
    /// Current path node, You can go through the entire path.
    pub path_node: &'a QueryPathNode<'a>,

    /// Parent type
    pub parent_type: &'a str,

    /// Current return type, is qualified name.
    pub return_type: &'a str,

    /// Current field name
    pub name: &'a str,

    /// Current field alias
    pub alias: Option<&'a str>,

    /// If `true` means the current field is for introspection.
    pub is_for_introspection: bool,

    /// Current field
    pub field: &'a Field,
}

type RequestFut<'a> = &'a mut (dyn Future<Output = Response> + Send + Unpin);

type ParseFut<'a> = &'a mut (dyn Future<Output = ServerResult<ExecutableDocument>> + Send + Unpin);

type ValidationFut<'a> =
    &'a mut (dyn Future<Output = Result<ValidationResult, Vec<ServerError>>> + Send + Unpin);

type ExecuteFutFactory<'a> = Box<dyn FnOnce(Option<Data>) -> BoxFuture<'a, Response> + Send + 'a>;

/// A future type used to resolve the field
pub type ResolveFut<'a> = &'a mut (dyn Future<Output = ServerResult<Option<Value>>> + Send + Unpin);

/// The remainder of a extension chain for request.
pub struct NextRequest<'a> {
    chain: &'a [Arc<dyn Extension>],
    request_fut: RequestFut<'a>,
}

impl NextRequest<'_> {
    /// Call the [Extension::request] function of next extension.
    pub async fn run(self, ctx: &ExtensionContext<'_>) -> Response {
        if let Some((first, next)) = self.chain.split_first() {
            first
                .request(
                    ctx,
                    NextRequest {
                        chain: next,
                        request_fut: self.request_fut,
                    },
                )
                .await
        } else {
            self.request_fut.await
        }
    }
}

/// The remainder of a extension chain for subscribe.
pub struct NextSubscribe<'a> {
    chain: &'a [Arc<dyn Extension>],
}

impl NextSubscribe<'_> {
    /// Call the [Extension::subscribe] function of next extension.
    pub fn run<'s>(
        self,
        ctx: &ExtensionContext<'_>,
        stream: BoxStream<'s, Response>,
    ) -> BoxStream<'s, Response> {
        if let Some((first, next)) = self.chain.split_first() {
            first.subscribe(ctx, stream, NextSubscribe { chain: next })
        } else {
            stream
        }
    }
}

/// The remainder of a extension chain for subscribe.
pub struct NextPrepareRequest<'a> {
    chain: &'a [Arc<dyn Extension>],
}

impl NextPrepareRequest<'_> {
    /// Call the [Extension::prepare_request] function of next extension.
    pub async fn run(self, ctx: &ExtensionContext<'_>, request: Request) -> ServerResult<Request> {
        if let Some((first, next)) = self.chain.split_first() {
            first
                .prepare_request(ctx, request, NextPrepareRequest { chain: next })
                .await
        } else {
            Ok(request)
        }
    }
}

/// The remainder of a extension chain for parse query.
pub struct NextParseQuery<'a> {
    chain: &'a [Arc<dyn Extension>],
    parse_query_fut: ParseFut<'a>,
}

impl NextParseQuery<'_> {
    /// Call the [Extension::parse_query] function of next extension.
    pub async fn run(
        self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
    ) -> ServerResult<ExecutableDocument> {
        if let Some((first, next)) = self.chain.split_first() {
            first
                .parse_query(
                    ctx,
                    query,
                    variables,
                    NextParseQuery {
                        chain: next,
                        parse_query_fut: self.parse_query_fut,
                    },
                )
                .await
        } else {
            self.parse_query_fut.await
        }
    }
}

/// The remainder of a extension chain for validation.
pub struct NextValidation<'a> {
    chain: &'a [Arc<dyn Extension>],
    validation_fut: ValidationFut<'a>,
}

impl NextValidation<'_> {
    /// Call the [Extension::validation] function of next extension.
    pub async fn run(
        self,
        ctx: &ExtensionContext<'_>,
    ) -> Result<ValidationResult, Vec<ServerError>> {
        if let Some((first, next)) = self.chain.split_first() {
            first
                .validation(
                    ctx,
                    NextValidation {
                        chain: next,
                        validation_fut: self.validation_fut,
                    },
                )
                .await
        } else {
            self.validation_fut.await
        }
    }
}

/// The remainder of a extension chain for execute.
pub struct NextExecute<'a> {
    chain: &'a [Arc<dyn Extension>],
    execute_fut_factory: ExecuteFutFactory<'a>,
    execute_data: Option<Data>,
}

impl NextExecute<'_> {
    async fn internal_run(
        self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        data: Option<Data>,
    ) -> Response {
        let execute_data = match (self.execute_data, data) {
            (Some(mut data1), Some(data2)) => {
                data1.merge(data2);
                Some(data1)
            }
            (Some(data), None) => Some(data),
            (None, Some(data)) => Some(data),
            (None, None) => None,
        };

        if let Some((first, next)) = self.chain.split_first() {
            first
                .execute(
                    ctx,
                    operation_name,
                    NextExecute {
                        chain: next,
                        execute_fut_factory: self.execute_fut_factory,
                        execute_data,
                    },
                )
                .await
        } else {
            (self.execute_fut_factory)(execute_data).await
        }
    }

    /// Call the [Extension::execute] function of next extension.
    pub async fn run(self, ctx: &ExtensionContext<'_>, operation_name: Option<&str>) -> Response {
        self.internal_run(ctx, operation_name, None).await
    }

    /// Call the [Extension::execute] function of next extension with context
    /// data.
    pub async fn run_with_data(
        self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        data: Data,
    ) -> Response {
        self.internal_run(ctx, operation_name, Some(data)).await
    }
}

/// The remainder of a extension chain for resolve.
pub struct NextResolve<'a> {
    chain: &'a [Arc<dyn Extension>],
    resolve_fut: ResolveFut<'a>,
}

impl NextResolve<'_> {
    /// Call the [Extension::resolve] function of next extension.
    pub async fn run(
        self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
    ) -> ServerResult<Option<Value>> {
        if let Some((first, next)) = self.chain.split_first() {
            first
                .resolve(
                    ctx,
                    info,
                    NextResolve {
                        chain: next,
                        resolve_fut: self.resolve_fut,
                    },
                )
                .await
        } else {
            self.resolve_fut.await
        }
    }
}

/// Represents a GraphQL extension
#[async_trait::async_trait]
pub trait Extension: Sync + Send + 'static {
    /// Called at start query/mutation request.
    async fn request(&self, ctx: &ExtensionContext<'_>, next: NextRequest<'_>) -> Response {
        next.run(ctx).await
    }

    /// Called at subscribe request.
    fn subscribe<'s>(
        &self,
        ctx: &ExtensionContext<'_>,
        stream: BoxStream<'s, Response>,
        next: NextSubscribe<'_>,
    ) -> BoxStream<'s, Response> {
        next.run(ctx, stream)
    }

    /// Called at prepare request.
    async fn prepare_request(
        &self,
        ctx: &ExtensionContext<'_>,
        request: Request,
        next: NextPrepareRequest<'_>,
    ) -> ServerResult<Request> {
        next.run(ctx, request).await
    }

    /// Called at parse query.
    async fn parse_query(
        &self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
        next: NextParseQuery<'_>,
    ) -> ServerResult<ExecutableDocument> {
        next.run(ctx, query, variables).await
    }

    /// Called at validation query.
    async fn validation(
        &self,
        ctx: &ExtensionContext<'_>,
        next: NextValidation<'_>,
    ) -> Result<ValidationResult, Vec<ServerError>> {
        next.run(ctx).await
    }

    /// Called at execute query.
    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        next.run(ctx, operation_name).await
    }

    /// Called at resolve field.
    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextResolve<'_>,
    ) -> ServerResult<Option<Value>> {
        next.run(ctx, info).await
    }
}

/// Extension factory
///
/// Used to create an extension instance.
pub trait ExtensionFactory: Send + Sync + 'static {
    /// Create an extended instance.
    fn create(&self) -> Arc<dyn Extension>;
}

#[derive(Clone)]
#[doc(hidden)]
pub struct Extensions {
    extensions: Vec<Arc<dyn Extension>>,
    schema_env: SchemaEnv,
    session_data: Arc<Data>,
    query_data: Option<Arc<Data>>,
}

#[doc(hidden)]
impl Extensions {
    pub(crate) fn new(
        extensions: impl IntoIterator<Item = Arc<dyn Extension>>,
        schema_env: SchemaEnv,
        session_data: Arc<Data>,
    ) -> Self {
        Extensions {
            extensions: extensions.into_iter().collect(),
            schema_env,
            session_data,
            query_data: None,
        }
    }

    #[inline]
    pub(crate) fn attach_query_data(&mut self, data: Arc<Data>) {
        self.query_data = Some(data);
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.extensions.is_empty()
    }

    #[inline]
    fn create_context(&self) -> ExtensionContext {
        ExtensionContext {
            schema_env: &self.schema_env,
            session_data: &self.session_data,
            query_data: self.query_data.as_deref(),
        }
    }

    pub async fn request(&self, request_fut: RequestFut<'_>) -> Response {
        let next = NextRequest {
            chain: &self.extensions,
            request_fut,
        };
        next.run(&self.create_context()).await
    }

    pub fn subscribe<'s>(&self, stream: BoxStream<'s, Response>) -> BoxStream<'s, Response> {
        let next = NextSubscribe {
            chain: &self.extensions,
        };
        next.run(&self.create_context(), stream)
    }

    pub async fn prepare_request(&self, request: Request) -> ServerResult<Request> {
        let next = NextPrepareRequest {
            chain: &self.extensions,
        };
        next.run(&self.create_context(), request).await
    }

    pub async fn parse_query(
        &self,
        query: &str,
        variables: &Variables,
        parse_query_fut: ParseFut<'_>,
    ) -> ServerResult<ExecutableDocument> {
        let next = NextParseQuery {
            chain: &self.extensions,
            parse_query_fut,
        };
        next.run(&self.create_context(), query, variables).await
    }

    pub async fn validation(
        &self,
        validation_fut: ValidationFut<'_>,
    ) -> Result<ValidationResult, Vec<ServerError>> {
        let next = NextValidation {
            chain: &self.extensions,
            validation_fut,
        };
        next.run(&self.create_context()).await
    }

    pub async fn execute<'a, 'b, F, T>(
        &'a self,
        operation_name: Option<&str>,
        execute_fut_factory: F,
    ) -> Response
    where
        F: FnOnce(Option<Data>) -> T + Send + 'a,
        T: Future<Output = Response> + Send + 'a,
    {
        let next = NextExecute {
            chain: &self.extensions,
            execute_fut_factory: Box::new(|data| execute_fut_factory(data).boxed()),
            execute_data: None,
        };
        next.run(&self.create_context(), operation_name).await
    }

    pub async fn resolve(
        &self,
        info: ResolveInfo<'_>,
        resolve_fut: ResolveFut<'_>,
    ) -> ServerResult<Option<Value>> {
        let next = NextResolve {
            chain: &self.extensions,
            resolve_fut,
        };
        next.run(&self.create_context(), info).await
    }
}
