use crate::context::{Data, ResolveId};
use crate::error::ParseRequestError;
use crate::extensions::{BoxExtension, ErrorLogger, Extension};
use crate::mutation_resolver::do_mutation_resolve;
use crate::parser::types::{OperationType, UploadValue};
use crate::registry::CacheControl;
use crate::{
    do_resolve, ContextBase, Error, ObjectType, Pos, QueryEnv, QueryError, Result, Schema,
    SubscriptionType, Value, Variables,
};
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
    /// Data of query result
    pub data: serde_json::Value,

    /// Extensions result
    pub extensions: Option<serde_json::Value>,

    /// Cache control value
    pub cache_control: CacheControl,
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
    pub fn operation_name<T: Into<String>>(self, name: T) -> Self {
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
        let variable = match self.variables.variable_path(var_path) {
            Some(variable) => variable,
            None => return,
        };
        *variable = Value::Upload(UploadValue {
            filename,
            content_type,
            content,
        });
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
        let (document, cache_control, extensions) =
            schema.prepare_query(&self.query_source, &self.variables, &self.extensions)?;

        // execute
        let inc_resolve_id = AtomicUsize::default();
        let document = match document.into_data(self.operation_name.as_deref()) {
            Some(document) => document,
            None => {
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
                .log_error(&extensions)
            }
        };

        let env = QueryEnv::new(
            extensions,
            self.variables,
            document,
            Arc::new(self.ctx_data.unwrap_or_default()),
        );
        let ctx = ContextBase {
            path_node: None,
            resolve_id: ResolveId::root(),
            inc_resolve_id: &inc_resolve_id,
            item: &env.document.operation.node.selection_set,
            schema_env: &schema.env,
            query_env: &env,
        };

        env.extensions.lock().execution_start();
        let data = match &env.document.operation.node.ty {
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
        let resp = QueryResponse {
            data,
            extensions: env.extensions.lock().result(),
            cache_control,
        };
        Ok(resp)
    }

    /// Get query source
    #[inline]
    pub fn query_source(&self) -> &str {
        &self.query_source
    }
}
