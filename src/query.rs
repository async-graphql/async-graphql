use crate::context::{Data, ResolveId};
use crate::error::ParseRequestError;
use crate::mutation_resolver::do_mutation_resolve;
use crate::parser::parse_query;
use crate::registry::CacheControl;
use crate::validation::{check_rules, CheckResult};
use crate::{
    do_resolve, ContextBase, Error, ObjectType, Pos, QueryError, Result, Schema, Variables,
};
use async_graphql_parser::query::OperationType;
use itertools::Itertools;
use std::any::Any;
use std::fs::File;
use std::sync::atomic::AtomicUsize;

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
    pub extensions: Option<serde_json::Map<String, serde_json::Value>>,

    /// Cache control value
    pub cache_control: CacheControl,
}

/// Query builder
pub struct QueryBuilder {
    pub(crate) query_source: String,
    pub(crate) operation_name: Option<String>,
    pub(crate) variables: Variables,
    pub(crate) ctx_data: Option<Data>,
}

impl QueryBuilder {
    /// Create query builder with query source.
    pub fn new<T: Into<String>>(query_source: T) -> QueryBuilder {
        QueryBuilder {
            query_source: query_source.into(),
            operation_name: None,
            variables: Default::default(),
            ctx_data: None,
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

    /// Execute the query.
    pub async fn execute<Query, Mutation, Subscription>(
        self,
        schema: &Schema<Query, Mutation, Subscription>,
    ) -> Result<QueryResponse>
    where
        Query: ObjectType + Send + Sync,
        Mutation: ObjectType + Send + Sync,
    {
        // create extension instances
        let extensions = schema
            .0
            .extensions
            .iter()
            .map(|factory| factory())
            .collect_vec();

        // parse query source
        extensions
            .iter()
            .for_each(|e| e.parse_start(&self.query_source));
        let mut document = parse_query(&self.query_source).map_err(Into::<Error>::into)?;
        extensions.iter().for_each(|e| e.parse_end());

        // check rules
        extensions.iter().for_each(|e| e.validation_start());
        let CheckResult {
            cache_control,
            complexity,
            depth,
        } = check_rules(&schema.0.registry, &document, schema.0.validation_mode)?;
        extensions.iter().for_each(|e| e.validation_end());

        // check limit
        if let Some(limit_complexity) = schema.0.complexity {
            if complexity > limit_complexity {
                return Err(QueryError::TooComplex.into_error(Pos::default()));
            }
        }

        if let Some(limit_depth) = schema.0.depth {
            if depth > limit_depth {
                return Err(QueryError::TooDeep.into_error(Pos::default()));
            }
        }

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
            };
        }

        let ctx = ContextBase {
            path_node: None,
            resolve_id: ResolveId::root(),
            inc_resolve_id: &inc_resolve_id,
            extensions: &extensions,
            item: &document.current_operation().selection_set,
            variables: &self.variables,
            registry: &schema.0.registry,
            data: &schema.0.data,
            ctx_data: self.ctx_data.as_ref(),
            document: &document,
        };

        extensions.iter().for_each(|e| e.execution_start());

        let data = match document.current_operation().ty {
            OperationType::Query => do_resolve(&ctx, &schema.0.query).await?,
            OperationType::Mutation => do_mutation_resolve(&ctx, &schema.0.mutation).await?,
            OperationType::Subscription => {
                return Err(Error::Query {
                    pos: Pos::default(),
                    path: None,
                    err: QueryError::NotSupported,
                })
            }
        };

        extensions.iter().for_each(|e| e.execution_end());

        let res = QueryResponse {
            data,
            extensions: if !extensions.is_empty() {
                Some(
                    extensions
                        .iter()
                        .filter_map(|e| {
                            if let Some(name) = e.name() {
                                e.result().map(|res| (name.to_string(), res))
                            } else {
                                None
                            }
                        })
                        .collect::<serde_json::Map<_, _>>(),
                )
            } else {
                None
            },
            cache_control,
        };
        Ok(res)
    }
}
