use crate::context::{GqlData, ResolveId};
use crate::error::ParseRequestError;
use crate::mutation_resolver::do_mutation_resolve;
use crate::parser::ast::{
    Definition, Document, OperationDefinition, SelectionSet, VariableDefinition,
};
use crate::parser::parse_query;
use crate::registry::CacheControl;
use crate::validation::{check_rules, CheckResult};
use crate::{
    do_resolve, GqlContextBase, GqlError, GqlResult, GqlSchema, GqlVariables, ObjectType, Pos,
    Positioned, QueryError,
};
use itertools::Itertools;
use std::any::Any;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;

/// IntoGqlQueryBuilder options
#[derive(Default, Clone)]
pub struct IntoGqlQueryBuilderOpts {
    /// A temporary path to store the contents of all files.
    ///
    /// If None, the system temporary path is used.
    pub temp_dir: Option<PathBuf>,

    /// Maximum file size.
    pub max_file_size: Option<usize>,

    /// Maximum number of files.
    pub max_num_files: Option<usize>,
}

#[allow(missing_docs)]
#[async_trait::async_trait]
pub trait IntoGqlQueryBuilder: Sized {
    async fn into_query_builder(self) -> Result<GqlQueryBuilder, ParseRequestError> {
        self.into_query_builder_opts(&Default::default()).await
    }

    async fn into_query_builder_opts(
        self,
        opts: &IntoGqlQueryBuilderOpts,
    ) -> std::result::Result<GqlQueryBuilder, ParseRequestError>;
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
pub struct GqlQueryBuilder {
    pub(crate) query_source: String,
    pub(crate) operation_name: Option<String>,
    pub(crate) variables: GqlVariables,
    pub(crate) ctx_data: Option<GqlData>,
}

impl GqlQueryBuilder {
    /// Create query builder with query source.
    pub fn new<T: Into<String>>(query_source: T) -> GqlQueryBuilder {
        GqlQueryBuilder {
            query_source: query_source.into(),
            operation_name: None,
            variables: Default::default(),
            ctx_data: None,
        }
    }

    /// Specify the operation name.
    pub fn operator_name<T: Into<String>>(self, name: T) -> Self {
        GqlQueryBuilder {
            operation_name: Some(name.into()),
            ..self
        }
    }

    /// Specify the variables.
    pub fn variables(self, variables: GqlVariables) -> Self {
        GqlQueryBuilder { variables, ..self }
    }

    /// Add a context data that can be accessed in the `GqlContext`, you access it with `GqlContext::data`.
    ///
    /// **This data is only valid for this query**
    pub fn data<D: Any + Send + Sync>(mut self, data: D) -> Self {
        if let Some(ctx_data) = &mut self.ctx_data {
            ctx_data.insert(data);
        } else {
            let mut ctx_data = GqlData::default();
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
        schema: &GqlSchema<Query, Mutation, Subscription>,
    ) -> GqlResult<QueryResponse>
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
        let document = parse_query(&self.query_source).map_err(Into::<GqlError>::into)?;
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
        let mut fragments = HashMap::new();
        let (selection_set, variable_definitions, is_query) =
            current_operation(&document, self.operation_name.as_deref()).ok_or_else(|| {
                GqlError::Query {
                    pos: Pos::default(),
                    path: None,
                    err: QueryError::MissingOperation,
                }
            })?;

        for definition in &document.definitions {
            if let Definition::Fragment(fragment) = &definition.node {
                fragments.insert(fragment.name.clone_inner(), fragment.clone_inner());
            }
        }

        let ctx = GqlContextBase {
            path_node: None,
            resolve_id: ResolveId::root(),
            inc_resolve_id: &inc_resolve_id,
            extensions: &extensions,
            item: selection_set,
            variables: &self.variables,
            variable_definitions,
            registry: &schema.0.registry,
            data: &schema.0.data,
            ctx_data: self.ctx_data.as_ref(),
            fragments: &fragments,
        };

        extensions.iter().for_each(|e| e.execution_start());
        let data = if is_query {
            do_resolve(&ctx, &schema.0.query).await?
        } else {
            do_mutation_resolve(&ctx, &schema.0.mutation).await?
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

#[allow(clippy::type_complexity)]
fn current_operation<'a>(
    document: &'a Document,
    operation_name: Option<&str>,
) -> Option<(
    &'a Positioned<SelectionSet>,
    &'a [Positioned<VariableDefinition>],
    bool,
)> {
    for definition in &document.definitions {
        match &definition.node {
            Definition::Operation(operation_definition) => match &operation_definition.node {
                OperationDefinition::SelectionSet(s) => {
                    return Some((s, &[], true));
                }
                OperationDefinition::Query(query)
                    if query.name.is_none()
                        || operation_name.is_none()
                        || query.name.as_ref().map(|name| name.as_str())
                            == operation_name.as_deref() =>
                {
                    return Some((&query.selection_set, &query.variable_definitions, true));
                }
                OperationDefinition::Mutation(mutation)
                    if mutation.name.is_none()
                        || operation_name.is_none()
                        || mutation.name.as_ref().map(|name| name.as_str())
                            == operation_name.as_deref() =>
                {
                    return Some((
                        &mutation.selection_set,
                        &mutation.variable_definitions,
                        false,
                    ));
                }
                OperationDefinition::Subscription(subscription)
                    if subscription.name.is_none()
                        || operation_name.is_none()
                        || subscription.name.as_ref().map(|name| name.as_str())
                            == operation_name.as_deref() =>
                {
                    return None;
                }
                _ => {}
            },
            Definition::Fragment(_) => {}
        }
    }
    None
}
