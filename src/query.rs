use crate::context::{Data, DeferList, ResolveId};
use crate::error::ParseRequestError;
use crate::extensions::{BoxExtension, Extension};
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
pub trait IntoQueryBuilder: Sized {
    async fn into_query_builder(self) -> std::result::Result<QueryBuilder, ParseRequestError> {
        self.into_query_builder_opts(&Default::default()).await
    }

    async fn into_query_builder_opts(
        self,
        opts: &IntoQueryBuilderOpts,
    ) -> std::result::Result<QueryBuilder, ParseRequestError>;
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

/// Query builder
pub struct QueryBuilder {
    pub(crate) query_source: String,
    pub(crate) operation_name: Option<String>,
    pub(crate) variables: Variables,
    pub(crate) ctx_data: Option<Data>,
    extensions: Vec<Box<dyn Fn() -> BoxExtension + Send + Sync>>,
}

impl QueryBuilder {
    /// Create query builder with query source.
    pub fn new<T: Into<String>>(query_source: T) -> QueryBuilder {
        QueryBuilder {
            query_source: query_source.into(),
            operation_name: None,
            variables: Default::default(),
            ctx_data: None,
            extensions: Default::default(),
        }
    }

    /// Specify the operation name.
    pub fn operator_name<T: Into<String>>(self, name: T) -> Self {
        QueryBuilder {
            operation_name: Some(name.into()),
            ..self
        }
    }

    /// Specify the variables.
    pub fn variables(self, variables: Variables) -> Self {
        QueryBuilder { variables, ..self }
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

    /// Add a context data that can be accessed in the `Context`, you access it with `Context::data`.
    ///
    /// **This data is only valid for this query**
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

    /// Set uploaded file path
    pub fn set_upload(
        &mut self,
        var_path: &str,
        filename: String,
        content_type: Option<String>,
        content: File,
    ) {
        self.variables
            .set_upload(var_path, filename, content_type, content);
    }

    /// Execute the query, returns a stream, the first result being the query result,
    /// followed by the incremental result. Only when there are `@defer` and `@stream` directives
    /// in the query will there be subsequent incremental results.
    pub async fn execute_stream<Query, Mutation, Subscription>(
        self,
        schema: &Schema<Query, Mutation, Subscription>,
    ) -> StreamResponse
    where
        Query: ObjectType + Send + Sync + 'static,
        Mutation: ObjectType + Send + Sync + 'static,
        Subscription: SubscriptionType + Send + Sync + 'static,
    {
        let schema = schema.clone();
        match self.execute_first(&schema).await {
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
    ) -> Result<(QueryResponse, DeferList)>
    where
        Query: ObjectType + Send + Sync + 'static,
        Mutation: ObjectType + Send + Sync + 'static,
        Subscription: SubscriptionType + Send + Sync + 'static,
    {
        let (mut document, cache_control, extensions) =
            schema.prepare_query(&self.query_source, &self.extensions)?;

        // execute
        let inc_resolve_id = AtomicUsize::default();
        if !document.retain_operation(self.operation_name.as_deref()) {
            return extensions.log_error(if let Some(operation_name) = self.operation_name {
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
            });
        }

        let env = QueryEnv::new(
            extensions,
            self.variables,
            document,
            Arc::new(self.ctx_data.unwrap_or_default()),
        );
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

        env.extensions.execution_start();

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

        env.extensions.execution_end();

        let res = QueryResponse {
            label: None,
            path: None,
            data,
            extensions: env.extensions.result(),
            cache_control,
        };
        Ok((res, defer_list))
    }

    /// Execute the query, always return a complete result.
    pub async fn execute<Query, Mutation, Subscription>(
        self,
        schema: &Schema<Query, Mutation, Subscription>,
    ) -> Result<QueryResponse>
    where
        Query: ObjectType + Send + Sync + 'static,
        Mutation: ObjectType + Send + Sync + 'static,
        Subscription: SubscriptionType + Send + Sync + 'static,
    {
        let resp = self.execute_stream(schema).await;
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
