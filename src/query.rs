use crate::context::{Data, DeferList, ResolveId};
use crate::error::ParseRequestError;
use crate::extensions::{BoxExtension, ErrorLogger, Extension};
use crate::mutation_resolver::do_mutation_resolve;
use crate::registry::CacheControl;
use crate::{
    do_resolve, ContextBase, Error, ObjectType, Pos, QueryEnv, QueryError, Result, Schema,
    SubscriptionType, Variables,
};
use async_graphql_parser::query::OperationType;
use futures::{Stream, StreamExt};
use itertools::Itertools;
use std::any::Any;
use std::borrow::Cow;
use std::fs::File;
use std::pin::Pin;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

/// IntoQueryBuilder options
#[derive(Default, Clone)]
pub struct IntoQueryBuilderOpts {
    /// Maximum file size.
    pub max_file_size: Option<usize>,

    /// Maximum number of files.
    pub max_num_files: Option<usize>,
}

#[allow(missing_docs)]
#[async_trait::async_trait]
pub trait IntoBatchQueryDefinition: Sized {
    async fn into_batch_query_builder(
        self,
    ) -> std::result::Result<BatchQueryBuilder, ParseRequestError> {
        self.into_batch_query_builder_opts(&Default::default())
            .await
    }

    async fn into_batch_query_builder_opts(
        self,
        opts: &IntoQueryBuilderOpts,
    ) -> std::result::Result<BatchQueryBuilder, ParseRequestError>;
}

/// Query response
#[derive(Debug)]
pub struct QueryResponse {
    /// Label for RelayModernQueryExecutor
    ///
    /// https://github.com/facebook/relay/blob/2859aa8df4df7d4d6d9eef4c9dc1134286773710/packages/relay-runtime/store/RelayModernQueryExecutor.js#L1267
    pub label: Option<String>,

    /// Path for subsequent response
    pub path: Option<Vec<serde_json::Value>>,

    /// Data of query result
    pub data: serde_json::Value,

    /// Extensions result
    pub extensions: Option<serde_json::Value>,

    /// Cache control value
    pub cache_control: CacheControl,
}

impl QueryResponse {
    pub(crate) fn apply_path_prefix(mut self, mut prefix: Vec<serde_json::Value>) -> Self {
        if let Some(path) = &mut self.path {
            prefix.extend(path.drain(..));
            *path = prefix;
        } else {
            self.path = Some(prefix);
        }

        self.label = self.path.as_ref().map(|path| {
            path.iter()
                .map(|value| {
                    if let serde_json::Value::String(s) = value {
                        Cow::Borrowed(s.as_str())
                    } else {
                        Cow::Owned(value.to_string())
                    }
                })
                .join("$")
        });

        self
    }

    pub(crate) fn merge(&mut self, resp: QueryResponse) {
        let mut p = &mut self.data;
        for item in resp.path.unwrap_or_default() {
            match item {
                serde_json::Value::String(name) => {
                    if let serde_json::Value::Object(obj) = p {
                        if let Some(next) = obj.get_mut(&name) {
                            p = next;
                            continue;
                        }
                    }
                    return;
                }
                serde_json::Value::Number(idx) => {
                    if let serde_json::Value::Array(array) = p {
                        let idx = idx.as_i64().unwrap() as usize;
                        while array.len() <= idx {
                            array.push(serde_json::Value::Null);
                        }
                        p = array.get_mut(idx as usize).unwrap();
                        continue;
                    }
                    return;
                }
                _ => {}
            }
        }
        *p = resp.data;
    }
}

/// Batch Query response
pub enum BatchQueryResponse {
    /// Respond with a single object
    Single(Result<QueryResponse>),
    /// Respond with an array of responses
    Batch(Vec<Result<QueryResponse>>),
}

impl BatchQueryResponse {
    /// return response's cache-control if single response,
    /// smallest cache-control if batch
    pub fn cache_control(&self) -> Option<CacheControl> {
        match self {
            BatchQueryResponse::Single(response_result) => response_result
                .as_ref()
                .ok()
                .map(|ref query_response| query_response.cache_control),
            BatchQueryResponse::Batch(responses) => {
                responses.iter().fold(None, |prev, response_result| {
                    let this_cache = response_result
                        .as_ref()
                        .ok()
                        .map(|ref query_response| query_response.cache_control);
                    match this_cache {
                        None => prev,
                        Some(cache) => match prev {
                            None => Some(cache),
                            Some(prev_cache) => {
                                if prev_cache.max_age < cache.max_age {
                                    Some(prev_cache)
                                } else {
                                    Some(cache)
                                }
                            }
                        },
                    }
                })
            }
        }
    }

    /// Either extracts `Single` variant from the enum, or panics
    pub fn unwrap_single(self) -> Result<QueryResponse> {
        match self {
            BatchQueryResponse::Single(resp) => resp,
            _ => panic!(),
        }
    }

    /// Either extracts `Batch` variant from the enum, or panics
    pub fn unwrap_batch(self) -> Vec<Result<QueryResponse>> {
        match self {
            BatchQueryResponse::Batch(resp) => resp,
            _ => panic!(),
        }
    }
}

/// Response for `Schema::execute_stream` and `QueryBuilder::execute_stream`
pub enum StreamResponse {
    /// There is no `@defer` or `@stream` directive in the query, this is the final result.
    Single(Result<QueryResponse>),

    /// Streaming responses.
    Stream(Pin<Box<dyn Stream<Item = Result<QueryResponse>> + Send + 'static>>),
}

impl StreamResponse {
    /// Convert to a stream.
    pub fn into_stream(self) -> impl Stream<Item = Result<QueryResponse>> + Send + 'static {
        match self {
            StreamResponse::Single(resp) => Box::pin(futures::stream::once(async move { resp })),
            StreamResponse::Stream(stream) => stream,
        }
    }
}

/// Query definition
struct QueryDefinition {
    pub(crate) query_source: String,
    pub(crate) operation_name: Option<String>,
    pub(crate) variables: Variables,
    extensions: Vec<Box<dyn Fn() -> BoxExtension + Send + Sync>>,
}

pub enum QueryDefinitionTypes {
    /// Single query
    Single(QueryDefinition),
    /// Batch query
    Batch(Vec<QueryDefinition>),
}

/// Query definition for batch requests
pub struct BatchQueryDefinition {
    /// Concrete builder type
    pub definition: QueryBuilderTypes,
    pub(crate) ctx_data: Option<Data>,
}

impl BatchQueryDefinition{
    pub(crate) fn set_upload(
        &mut self,
        var_path: &str,
        filename: String,
        content_type: Option<String>,
        content: File,
    ) -> std::result::Result<(), ParseRequestError> {
        match self.builder {
            QueryBuilderTypes::Single(ref mut builder) => {
                Ok(builder.set_upload(var_path, filename, content_type, content))
            }
            QueryBuilderTypes::Batch(ref mut builders) => {
                let mut it = var_path.split('.').peekable();
                // First part of the name in a batch query with uploads is the index of the query
                // https://github.com/jaydenseric/graphql-multipart-request-spec
                let idx = it
                    .next()
                    .ok_or(ParseRequestError::BatchUploadIndexMissing)?
                    .parse::<usize>()
                    .or(Err(ParseRequestError::BatchUploadIndexMissing))?;
                Ok(builders
                    .get_mut(idx)
                    .ok_or(ParseRequestError::BatchUploadIndexIncorrect)?
                    .set_upload(it.join("").as_str(), filename, content_type, content))
            }
        }
    }

    /// Add a context data that can be accessed in the `Context`, you access it with `Context::data`.
    ///
    /// **This data is valid for all queries in the batch**
    pub fn data<D: Any + Send + Sync>(mut self, data: D) -> Self {
        if let Some(ctx_data) = &mut self.ctx_data {
            ctx_data.insert(data);
        } else {
            let mut ctx_data = Data::default();
            ctx_data.insert(data);
            self.ctx_data = Some(ctx_data);
        }
        self
    }

    /// Execute the query, always return a complete result.
    pub async fn execute<Query, Mutation, Subscription>(
        self,
        schema: &Schema<Query, Mutation, Subscription>,
    ) -> BatchQueryResponse
        where
            Query: ObjectType + Send + Sync + 'static,
            Mutation: ObjectType + Send + Sync + 'static,
            Subscription: SubscriptionType + Send + Sync + 'static,
    {
        match self.builder {
            QueryBuilderTypes::Single(builder) => BatchQueryResponse::Single(
                builder
                    .execute_with_ctx(schema, Arc::new(self.ctx_data.unwrap_or_default()))
                    .await,
            ),
            QueryBuilderTypes::Batch(builders) => {
                let ctx = Arc::new(self.ctx_data.unwrap_or_default());
                let futures = builders
                    .into_iter()
                    .map(|builder| builder.execute_with_ctx(schema, Arc::clone(&ctx)));
                BatchQueryResponse::Batch(futures::future::join_all(futures).await)
            }
        }
    }
}

struct SingleQueryBuilder{
    pub(crate) query_source: String,
    pub(crate) operation_name: Option<String>,
    pub(crate) variables: Variables,
    extensions: Vec<Box<dyn Fn() -> BoxExtension + Send + Sync>>,
}

impl SingleQueryBuilder{
    /// Create query builder with query source.
    fn new<T: Into<String>>(query_source: T) -> Self {
        Self {
            query_source: query_source.into(),
            operation_name: None,
            variables: Default::default(),
            extensions: Default::default(),
        }
    }

    /// Specify the operation name.
    pub fn operation_name<T: Into<String>>(self, name: T) -> Self {
        Self {
            operation_name: Some(name.into()),
            ..self
        }
    }

    /// Specify the variables.
    pub fn variables(self, variables: Variables) -> Self {
        Self { variables, ..self }
    }

    /// Add an extension
    pub fn extension<F: Fn() -> E + Send + Sync + 'static, E: Extension>(
        mut self,
        extension_factory: F,
    ) -> Self {
        self.extensions
            .push(Box::new(move || Box::new(extension_factory())));
        self
    }

    pub fn finish(self) -> BatchQueryDefinition {
        BatchQueryDefinition{
            definition: QueryDefinitionTypes::Single(self.into()),
            ctx_data: None
        }
    }
}

impl From<SingleQueryBuilder> for QueryDefinition{
    fn from(item: SingleQueryBuilder) -> Self {
        Self{
            query_source: item.query_source,
            operation_name: item.operation_name,
            variables: item.variables,
            extensions: item.extensions
        }
    }
}

impl QueryDefinition{
    async fn execute_stream_with_ctx<Query, Mutation, Subscription>(
        self,
        schema: &Schema<Query, Mutation, Subscription>,
        ctx_data: Arc<Data>,
    ) -> StreamResponse
        where
            Query: ObjectType + Send + Sync + 'static,
            Mutation: ObjectType + Send + Sync + 'static,
            Subscription: SubscriptionType + Send + Sync + 'static,
    {
        let schema = schema.clone();
        match self.execute_first(&schema, ctx_data).await {
            Ok((first_resp, defer_list)) if defer_list.futures.lock().is_empty() => {
                StreamResponse::Single(Ok(first_resp))
            }
            Err(err) => StreamResponse::Single(Err(err)),
            Ok((first_resp, defer_list)) => {
                let stream = async_stream::try_stream! {
                    yield first_resp;

                    let mut current_defer_list = Vec::new();
                    for fut in defer_list.futures.into_inner() {
                        current_defer_list.push((defer_list.path_prefix.clone(), fut));
                    }

                    loop {
                        let mut next_defer_list = Vec::new();
                        for (path_prefix, defer) in current_defer_list {
                            let (res, mut defer_list) = defer.await?;
                            for fut in defer_list.futures.into_inner() {
                                let mut next_path_prefix = path_prefix.clone();
                                next_path_prefix.extend(defer_list.path_prefix.clone());
                                next_defer_list.push((next_path_prefix, fut));
                            }
                            let mut new_res = res.apply_path_prefix(path_prefix);
                            new_res.label = new_res.path.as_ref().map(|path| path.iter().map(|value| {
                                if let serde_json::Value::String(s) = value {
                                    s.to_string()
                                } else {
                                    value.to_string()
                                }
                            }).join("$"));
                            yield new_res;
                        }
                        if next_defer_list.is_empty() {
                            break;
                        }
                        current_defer_list = next_defer_list;
                    }
                };
                StreamResponse::Stream(Box::pin(stream))
            }
        }
    }

    async fn execute_first<'a, Query, Mutation, Subscription>(
        self,
        schema: &Schema<Query, Mutation, Subscription>,
        ctx_data: Arc<Data>,
    ) -> Result<(QueryResponse, DeferList)>
        where
            Query: ObjectType + Send + Sync + 'static,
            Mutation: ObjectType + Send + Sync + 'static,
            Subscription: SubscriptionType + Send + Sync + 'static,
    {
        let (mut document, cache_control, extensions) =
            schema.prepare_query(&self.query_source, &self.variables, &self.extensions)?;

        // execute
        let inc_resolve_id = AtomicUsize::default();
        if !document.retain_operation(self.operation_name.as_deref()) {
            return if let Some(operation_name) = self.operation_name {
                Err(Error::Query {
                    pos: Pos::default(),
                    path: None,
                    err: QueryError::UnknownOperationNamed {
                        name: operation_name,
                    },
                })
            } else {
                Err(Error::Query {
                    pos: Pos::default(),
                    path: None,
                    err: QueryError::MissingOperation,
                })
            }
                .log_error(&extensions);
        }

        let env = QueryEnv::new(extensions, self.variables, document, ctx_data);
        let defer_list = DeferList {
            path_prefix: Vec::new(),
            futures: Default::default(),
        };
        let ctx = ContextBase {
            path_node: None,
            resolve_id: ResolveId::root(),
            inc_resolve_id: &inc_resolve_id,
            item: &env.document.current_operation().selection_set,
            schema_env: &schema.env,
            query_env: &env,
            defer_list: Some(&defer_list),
        };

        env.extensions.lock().execution_start();
        let data = match &env.document.current_operation().ty {
            OperationType::Query => do_resolve(&ctx, &schema.query).await?,
            OperationType::Mutation => do_mutation_resolve(&ctx, &schema.mutation).await?,
            OperationType::Subscription => {
                return Err(Error::Query {
                    pos: Pos::default(),
                    path: None,
                    err: QueryError::NotSupported,
                })
            }
        };

        env.extensions.lock().execution_end();
        let res = QueryResponse {
            label: None,
            path: None,
            data,
            extensions: env.extensions.lock().result(),
            cache_control,
        };
        Ok((res, defer_list))
    }

    async fn execute_with_ctx<Query, Mutation, Subscription>(
        self,
        schema: &Schema<Query, Mutation, Subscription>,
        ctx_data: Arc<Data>,
    ) -> Result<QueryResponse>
        where
            Query: ObjectType + Send + Sync + 'static,
            Mutation: ObjectType + Send + Sync + 'static,
            Subscription: SubscriptionType + Send + Sync + 'static,
    {
        let resp = self.execute_stream_with_ctx(schema, ctx_data).await;
        match resp {
            StreamResponse::Single(res) => res,
            StreamResponse::Stream(mut stream) => {
                let mut resp = stream.next().await.unwrap()?;
                while let Some(resp_part) = stream.next().await.transpose()? {
                    resp.merge(resp_part);
                }
                Ok(resp)
            }
        }
    }
}

// TODO: rename
pub struct QueryBuilderReal{
    current_builder: SingleQueryBuilder,
    completed_builders: Vec<QueryDefinition>,
}

impl QueryBuilderReal {
    /// Create query builder with query source for a single query.
    fn new_single<T: Into<String>>(query_source: T) -> SingleQueryBuilder {
        SingleQueryBuilder::new(query_source)
    }

    /// Create query builder with query source for a batch query.
    fn new_batch<T: Into<String>>(query_source: T) -> Self {
        Self{ current_builder: SingleQueryBuilder::new(query_source), completed_builders: vec![] }
    }

    /// Specify the operation name.
    pub fn operation_name<T: Into<String>>(mut self, name: T) -> Self {
        self.current_builder = self.current_builder.operation_name(name);
        self
    }

    /// Specify the variables.
    pub fn variables(mut self, variables: Variables) -> Self {
        self.current_builder = self.current_builder.variables(variables);
        self
    }

    /// Add an extension
    pub fn extension<F: Fn() -> E + Send + Sync + 'static, E: Extension>(
        mut self,
        extension_factory: F,
    ) -> Self {
        self.current_builder = self.current_builder.extensions(extension_factory);
        self
    }

    /// Start building next query in the batch
    pub fn next<T: Into<String>>(mut self, query_source: T) -> Self {
        self.completed_builders.push(self.current_builder.into());
        self.current_builder = SingleQueryBuilder::new(query_source);
        self
    }

    /// Finish building a query, get back the definition
    pub fn finish(mut self) -> BatchQueryDefinition {
        self.completed_builders.push(self.current_builder.into());
        BatchQueryDefinition{ definition: QueryDefinitionTypes::Batch(self.completed_builders), ctx_data: None }
    }
}
