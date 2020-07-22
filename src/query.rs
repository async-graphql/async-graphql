use crate::context::{Data, ResolveId};
use crate::error::ParseRequestError;
use crate::extensions::{BoxExtension, ErrorLogger, Extension};
use crate::mutation_resolver::do_mutation_resolve;
use crate::registry::CacheControl;
use crate::{
    do_resolve, ContextBase, Error, ObjectType, Pos, QueryEnv, QueryError, Result, Schema,
    SubscriptionType, Variables,
};
use async_graphql_parser::query::OperationType;
use itertools::Itertools;
use std::any::Any;
use std::fs::File;
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
    async fn into_batch_query_definition(
        self,
    ) -> std::result::Result<BatchQueryDefinition, ParseRequestError> {
        self.into_batch_query_definition_opts(&Default::default())
            .await
    }

    async fn into_batch_query_definition_opts(
        self,
        opts: &IntoQueryBuilderOpts,
    ) -> std::result::Result<BatchQueryDefinition, ParseRequestError>;
}

/// Query response
#[derive(Debug)]
pub struct QueryResponse {
    /// Data of query result
    pub data: serde_json::Value,

    /// Extensions result
    pub extensions: Option<serde_json::Value>,

    /// Cache control value
    pub cache_control: CacheControl,
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

/// Query definition
pub struct QueryDefinitionPart {
    pub(crate) query_source: String,
    pub(crate) operation_name: Option<String>,
    pub(crate) variables: Variables,
    pub(crate) extensions: Vec<Box<dyn Fn() -> BoxExtension + Send + Sync>>,
}

pub enum QueryDefinitionTypes {
    /// Single query
    Single(QueryDefinitionPart),
    /// Batch query
    Batch(Vec<QueryDefinitionPart>),
}

/// Query definition for batch requests
pub struct BatchQueryDefinition {
    /// Concrete builder type
    pub definition: QueryDefinitionTypes,
    pub(crate) ctx_data: Option<Data>,
}

impl BatchQueryDefinition {
    pub(crate) fn set_upload(
        &mut self,
        var_path: &str,
        filename: String,
        content_type: Option<String>,
        content: File,
    ) -> std::result::Result<(), ParseRequestError> {
        match self.definition {
            QueryDefinitionTypes::Single(ref mut definition) => {
                Ok(definition.set_upload(var_path, filename, content_type, content))
            }
            QueryDefinitionTypes::Batch(ref mut definitions) => {
                let mut it = var_path.split('.').peekable();
                // First part of the name in a batch query with uploads is the index of the query
                // https://github.com/jaydenseric/graphql-multipart-request-spec
                let idx = it
                    .next()
                    .ok_or(ParseRequestError::BatchUploadIndexMissing)?
                    .parse::<usize>()
                    .or(Err(ParseRequestError::BatchUploadIndexMissing))?;
                Ok(definitions
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
        match self.definition {
            QueryDefinitionTypes::Single(definition) => BatchQueryResponse::Single(
                definition
                    .execute_with_ctx(schema, Arc::new(self.ctx_data.unwrap_or_default()))
                    .await,
            ),
            QueryDefinitionTypes::Batch(definitions) => {
                let ctx = Arc::new(self.ctx_data.unwrap_or_default());
                let futures = definitions
                    .into_iter()
                    .map(|definition| definition.execute_with_ctx(schema, Arc::clone(&ctx)));
                BatchQueryResponse::Batch(futures::future::join_all(futures).await)
            }
        }
    }
}

/// Query builder for a single-type query
pub struct SingleQueryBuilder {
    pub(crate) query_source: String,
    pub(crate) operation_name: Option<String>,
    pub(crate) variables: Variables,
    extensions: Vec<Box<dyn Fn() -> BoxExtension + Send + Sync>>,
}

impl SingleQueryBuilder {
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
        BatchQueryDefinition {
            definition: QueryDefinitionTypes::Single(self.into()),
            ctx_data: None,
        }
    }
}

impl From<SingleQueryBuilder> for QueryDefinitionPart {
    fn from(item: SingleQueryBuilder) -> Self {
        Self {
            query_source: item.query_source,
            operation_name: item.operation_name,
            variables: item.variables,
            extensions: item.extensions,
        }
    }
}

impl QueryDefinitionPart {
    /// Set uploaded file path
    fn set_upload(
        &mut self,
        var_path: &str,
        filename: String,
        content_type: Option<String>,
        content: File,
    ) {
        self.variables
            .set_upload(var_path, filename, content_type, content);
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
        let ctx = ContextBase {
            path_node: None,
            resolve_id: ResolveId::root(),
            inc_resolve_id: &inc_resolve_id,
            item: &env.document.current_operation().selection_set,
            schema_env: &schema.env,
            query_env: &env,
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
        let extensions = env.extensions.lock().result();
        Ok(QueryResponse {
            data,
            extensions,
            cache_control,
        })
    }
}

/// Main query builder type. You can use it to build either a single query, or a batch query
/// The difference between single and batch queries using `new_single` and `new_batch` methods
pub struct QueryBuilder {
    current_builder: SingleQueryBuilder,
    completed_builders: Vec<QueryDefinitionPart>,
}

impl QueryBuilder {
    /// Create query builder with query source for a single query.
    pub fn new_single<T: Into<String>>(query_source: T) -> SingleQueryBuilder {
        SingleQueryBuilder::new(query_source)
    }

    /// Create query builder with query source for a batch query.
    /// You need to supply the source of the first query to start the process, and then
    /// you can customize first query however you want.
    /// When you are done building first query in the batch, you need to call `next` with
    /// the source of the second query and so on, untill finishing the process with
    /// `finish` method, that will return you `BatchQueryDefinition`
    pub fn new_batch<T: Into<String>>(query_source: T) -> Self {
        Self {
            current_builder: SingleQueryBuilder::new(query_source),
            completed_builders: vec![],
        }
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
        self.current_builder = self.current_builder.extension(extension_factory);
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
        BatchQueryDefinition {
            definition: QueryDefinitionTypes::Batch(self.completed_builders),
            ctx_data: None,
        }
    }
}
