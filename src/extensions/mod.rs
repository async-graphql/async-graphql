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

pub use self::analyzer::Analyzer;
#[cfg(feature = "apollo_tracing")]
pub use self::apollo_tracing::ApolloTracing;
#[cfg(feature = "log")]
pub use self::logger::Logger;
#[cfg(feature = "opentelemetry")]
pub use self::opentelemetry::OpenTelemetry;
#[cfg(feature = "tracing")]
pub use self::tracing::Tracing;

use std::any::{Any, TypeId};
use std::future::Future;
use std::sync::Arc;

use futures_util::stream::BoxStream;
use futures_util::stream::StreamExt;

use crate::parser::types::ExecutableDocument;
use crate::{
    Data, Error, QueryPathNode, Request, Response, Result, SchemaEnv, ServerError, ServerResult,
    ValidationResult, Value, Variables,
};

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
    /// Current path node, You can go through the entire path.
    pub path_node: &'a QueryPathNode<'a>,

    /// Parent type
    pub parent_type: &'a str,

    /// Current return type, is qualified name.
    pub return_type: &'a str,
}

type RequestFut<'a> = &'a mut (dyn Future<Output = Response> + Send + Unpin);

type ParseFut<'a> = &'a mut (dyn Future<Output = ServerResult<ExecutableDocument>> + Send + Unpin);

type ValidationFut<'a> =
    &'a mut (dyn Future<Output = Result<ValidationResult, Vec<ServerError>>> + Send + Unpin);

type ExecuteFut<'a> = &'a mut (dyn Future<Output = Response> + Send + Unpin);

type ResolveFut<'a> = &'a mut (dyn Future<Output = ServerResult<Option<Value>>> + Send + Unpin);

/// The remainder of a extension chain.
pub struct NextExtension<'a> {
    chain: &'a [Arc<dyn Extension>],
    request_fut: Option<RequestFut<'a>>,
    parse_query_fut: Option<ParseFut<'a>>,
    validation_fut: Option<ValidationFut<'a>>,
    execute_fut: Option<ExecuteFut<'a>>,
    resolve_fut: Option<ResolveFut<'a>>,
}

impl<'a> NextExtension<'a> {
    #[inline]
    pub(crate) fn new(chain: &'a [Arc<dyn Extension>]) -> Self {
        Self {
            chain,
            request_fut: None,
            parse_query_fut: None,
            validation_fut: None,
            execute_fut: None,
            resolve_fut: None,
        }
    }

    #[inline]
    pub(crate) fn with_chain(self, chain: &'a [Arc<dyn Extension>]) -> Self {
        Self { chain, ..self }
    }

    #[inline]
    pub(crate) fn with_request(self, fut: RequestFut<'a>) -> Self {
        Self {
            request_fut: Some(fut),
            ..self
        }
    }

    #[inline]
    pub(crate) fn with_parse_query(self, fut: ParseFut<'a>) -> Self {
        Self {
            parse_query_fut: Some(fut),
            ..self
        }
    }

    #[inline]
    pub(crate) fn with_validation(self, fut: ValidationFut<'a>) -> Self {
        Self {
            validation_fut: Some(fut),
            ..self
        }
    }

    #[inline]
    pub(crate) fn with_execute(self, fut: ExecuteFut<'a>) -> Self {
        Self {
            execute_fut: Some(fut),
            ..self
        }
    }

    #[inline]
    pub(crate) fn with_resolve(self, fut: ResolveFut<'a>) -> Self {
        Self {
            resolve_fut: Some(fut),
            ..self
        }
    }

    /// Call the [Extension::request] function of next extension.
    pub async fn request(mut self, ctx: &ExtensionContext<'_>) -> Response {
        if let Some((first, next)) = self.chain.split_first() {
            first.request(ctx, self.with_chain(next)).await
        } else {
            self.request_fut
                .take()
                .expect("You definitely called the wrong function.")
                .await
        }
    }

    /// Call the [Extension::subscribe] function of next extension.
    pub fn subscribe<'s>(
        self,
        ctx: &ExtensionContext<'_>,
        stream: BoxStream<'s, Response>,
    ) -> BoxStream<'s, Response> {
        if let Some((first, next)) = self.chain.split_first() {
            first.subscribe(ctx, stream, self.with_chain(next))
        } else {
            stream
        }
    }

    /// Call the [Extension::prepare_request] function of next extension.
    pub async fn prepare_request(
        self,
        ctx: &ExtensionContext<'_>,
        request: Request,
    ) -> ServerResult<Request> {
        if let Some((first, next)) = self.chain.split_first() {
            first
                .prepare_request(ctx, request, self.with_chain(next))
                .await
        } else {
            Ok(request)
        }
    }

    /// Call the [Extension::parse_query] function of next extension.
    pub async fn parse_query(
        mut self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
    ) -> ServerResult<ExecutableDocument> {
        if let Some((first, next)) = self.chain.split_first() {
            first
                .parse_query(ctx, query, variables, self.with_chain(next))
                .await
        } else {
            self.parse_query_fut
                .take()
                .expect("You definitely called the wrong function.")
                .await
        }
    }

    /// Call the [Extension::validation] function of next extension.
    pub async fn validation(
        mut self,
        ctx: &ExtensionContext<'_>,
    ) -> Result<ValidationResult, Vec<ServerError>> {
        if let Some((first, next)) = self.chain.split_first() {
            first.validation(ctx, self.with_chain(next)).await
        } else {
            self.validation_fut
                .take()
                .expect("You definitely called the wrong function.")
                .await
        }
    }

    /// Call the [Extension::execute] function of next extension.
    pub async fn execute(mut self, ctx: &ExtensionContext<'_>) -> Response {
        if let Some((first, next)) = self.chain.split_first() {
            first.execute(ctx, self.with_chain(next)).await
        } else {
            self.execute_fut
                .take()
                .expect("You definitely called the wrong function.")
                .await
        }
    }

    /// Call the [Extension::resolve] function of next extension.
    pub async fn resolve(
        mut self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
    ) -> ServerResult<Option<Value>> {
        if let Some((first, next)) = self.chain.split_first() {
            first.resolve(ctx, info, self.with_chain(next)).await
        } else {
            self.resolve_fut
                .take()
                .expect("You definitely called the wrong function.")
                .await
        }
    }
}

/// Represents a GraphQL extension
#[async_trait::async_trait]
#[allow(unused_variables)]
pub trait Extension: Sync + Send + 'static {
    /// Called at start query/mutation request.
    async fn request(&self, ctx: &ExtensionContext<'_>, next: NextExtension<'_>) -> Response {
        next.request(ctx).await
    }

    /// Called at subscribe request.
    fn subscribe<'s>(
        &self,
        ctx: &ExtensionContext<'_>,
        stream: BoxStream<'s, Response>,
        next: NextExtension<'_>,
    ) -> BoxStream<'s, Response> {
        next.subscribe(ctx, stream)
    }

    /// Called at prepare request.
    async fn prepare_request(
        &self,
        ctx: &ExtensionContext<'_>,
        request: Request,
        next: NextExtension<'_>,
    ) -> ServerResult<Request> {
        next.prepare_request(ctx, request).await
    }

    /// Called at parse query.
    async fn parse_query(
        &self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
        next: NextExtension<'_>,
    ) -> ServerResult<ExecutableDocument> {
        next.parse_query(ctx, query, variables).await
    }

    /// Called at validation query.
    async fn validation(
        &self,
        ctx: &ExtensionContext<'_>,
        next: NextExtension<'_>,
    ) -> Result<ValidationResult, Vec<ServerError>> {
        next.validation(ctx).await
    }

    /// Called at execute query.
    async fn execute(&self, ctx: &ExtensionContext<'_>, next: NextExtension<'_>) -> Response {
        next.execute(ctx).await
    }

    /// Called at resolve field.
    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextExtension<'_>,
    ) -> ServerResult<Option<Value>> {
        next.resolve(ctx, info).await
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

    pub fn attach_query_data(&mut self, data: Arc<Data>) {
        self.query_data = Some(data);
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.extensions.is_empty()
    }

    #[inline]
    fn create_context(&self) -> ExtensionContext {
        ExtensionContext {
            schema_data: &self.schema_env.data,
            session_data: &self.session_data,
            query_data: self.query_data.as_deref(),
        }
    }

    pub async fn request(&self, request_fut: RequestFut<'_>) -> Response {
        if !self.extensions.is_empty() {
            let next = NextExtension::new(&self.extensions).with_request(request_fut);
            next.request(&self.create_context()).await
        } else {
            request_fut.await
        }
    }

    pub fn subscribe<'s>(&self, stream: BoxStream<'s, Response>) -> BoxStream<'s, Response> {
        if !self.extensions.is_empty() {
            let next = NextExtension::new(&self.extensions);
            next.subscribe(&self.create_context(), stream)
        } else {
            stream.boxed()
        }
    }

    pub async fn prepare_request(&self, request: Request) -> ServerResult<Request> {
        if !self.extensions.is_empty() {
            let next = NextExtension::new(&self.extensions);
            next.prepare_request(&self.create_context(), request).await
        } else {
            Ok(request)
        }
    }

    pub async fn parse_query(
        &self,
        query: &str,
        variables: &Variables,
        parse_query_fut: ParseFut<'_>,
    ) -> ServerResult<ExecutableDocument> {
        if !self.extensions.is_empty() {
            let next = NextExtension::new(&self.extensions).with_parse_query(parse_query_fut);
            next.parse_query(&self.create_context(), query, variables)
                .await
        } else {
            parse_query_fut.await
        }
    }

    pub async fn validation(
        &self,
        validation_fut: ValidationFut<'_>,
    ) -> Result<ValidationResult, Vec<ServerError>> {
        if !self.extensions.is_empty() {
            let next = NextExtension::new(&self.extensions).with_validation(validation_fut);
            next.validation(&self.create_context()).await
        } else {
            validation_fut.await
        }
    }

    pub async fn execute(&self, execute_fut: ExecuteFut<'_>) -> Response {
        if !self.extensions.is_empty() {
            let next = NextExtension::new(&self.extensions).with_execute(execute_fut);
            next.execute(&self.create_context()).await
        } else {
            execute_fut.await
        }
    }

    pub async fn resolve(
        &self,
        info: ResolveInfo<'_>,
        resolve_fut: ResolveFut<'_>,
    ) -> ServerResult<Option<Value>> {
        if !self.extensions.is_empty() {
            let next = NextExtension::new(&self.extensions).with_resolve(resolve_fut);
            next.resolve(&self.create_context(), info).await
        } else {
            resolve_fut.await
        }
    }
}
