use crate::context::Data;
use crate::registry::{CacheControl, Registry};
use crate::types::QueryRoot;
use crate::validation::check_rules;
use crate::{ContextBase, JsonWriter, OutputValueType, Result};
use crate::{ObjectType, QueryError, QueryParseError, Variables};
use bytes::Bytes;
use graphql_parser::parse_query;
use graphql_parser::query::{
    Definition, FragmentDefinition, OperationDefinition, SelectionSet, VariableDefinition,
};
use std::collections::HashMap;

enum Root<'a, Query, Mutation> {
    Query(&'a QueryRoot<Query>),
    Mutation(&'a Mutation),
}

/// Query builder
pub struct QueryBuilder<'a, Query, Mutation> {
    pub(crate) query: &'a QueryRoot<Query>,
    pub(crate) mutation: &'a Mutation,
    pub(crate) registry: &'a Registry,
    pub(crate) source: &'a str,
    pub(crate) operation_name: Option<&'a str>,
    pub(crate) variables: Option<Variables>,
    pub(crate) data: &'a Data,
}

impl<'a, Query, Mutation> QueryBuilder<'a, Query, Mutation> {
    /// Specify the operation name.
    pub fn operator_name(self, name: &'a str) -> Self {
        QueryBuilder {
            operation_name: Some(name),
            ..self
        }
    }

    /// Specify the variables.
    pub fn variables(self, vars: Variables) -> Self {
        QueryBuilder {
            variables: Some(vars),
            ..self
        }
    }

    /// Prepare query
    pub fn prepare(self) -> Result<PreparedQuery<'a, Query, Mutation>> {
        let document = parse_query(self.source).map_err(|err| QueryParseError(err.to_string()))?;
        let cache_control = check_rules(self.registry, &document)?;

        let mut fragments = HashMap::new();
        let mut selection_set = None;
        let mut variable_definitions = None;
        let mut root = None;

        for definition in document.definitions {
            match definition {
                Definition::Operation(operation_definition) => match operation_definition {
                    OperationDefinition::SelectionSet(s) => {
                        selection_set = Some(s);
                        root = Some(Root::Query(self.query));
                    }
                    OperationDefinition::Query(query)
                        if query.name.is_none() || query.name.as_deref() == self.operation_name =>
                    {
                        selection_set = Some(query.selection_set);
                        variable_definitions = Some(query.variable_definitions);
                        root = Some(Root::Query(self.query));
                    }
                    OperationDefinition::Mutation(mutation)
                        if mutation.name.is_none()
                            || mutation.name.as_deref() == self.operation_name =>
                    {
                        selection_set = Some(mutation.selection_set);
                        variable_definitions = Some(mutation.variable_definitions);
                        root = Some(Root::Mutation(self.mutation));
                    }
                    OperationDefinition::Subscription(subscription)
                        if subscription.name.is_none()
                            || subscription.name.as_deref() == self.operation_name =>
                    {
                        return Err(QueryError::NotSupported.into());
                    }
                    _ => {}
                },
                Definition::Fragment(fragment) => {
                    fragments.insert(fragment.name.clone(), fragment);
                }
            }
        }

        Ok(PreparedQuery {
            registry: self.registry,
            variables: self.variables.unwrap_or_default(),
            data: self.data,
            fragments,
            selection_set: selection_set.ok_or({
                if let Some(name) = self.operation_name {
                    QueryError::UnknownOperationNamed {
                        name: name.to_string(),
                    }
                } else {
                    QueryError::MissingOperation
                }
            })?,
            root: root.unwrap(),
            variable_definitions,
            cache_control,
        })
    }

    /// Execute the query.
    pub async fn execute(self) -> Result<String>
    where
        Query: ObjectType + Send + Sync,
        Mutation: ObjectType + Send + Sync,
    {
        self.prepare()?.execute().await
    }
}

/// Prepared query object
pub struct PreparedQuery<'a, Query, Mutation> {
    root: Root<'a, Query, Mutation>,
    registry: &'a Registry,
    variables: Variables,
    data: &'a Data,
    fragments: HashMap<String, FragmentDefinition>,
    selection_set: SelectionSet,
    variable_definitions: Option<Vec<VariableDefinition>>,
    cache_control: CacheControl,
}

impl<'a, Query, Mutation> PreparedQuery<'a, Query, Mutation> {
    /// Detects whether any parameter contains the Upload type
    pub fn is_upload(&self) -> bool {
        if let Some(variable_definitions) = &self.variable_definitions {
            for d in variable_definitions {
                if let Some(ty) = self.registry.basic_type_by_parsed_type(&d.var_type) {
                    if ty.name() == "Upload" {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Set upload files
    pub fn set_upload(
        &mut self,
        var_path: &str,
        filename: &str,
        content_type: Option<&str>,
        content: Bytes,
    ) {
        self.variables
            .set_upload(var_path, filename, content_type, content);
    }

    /// Execute the query.
    pub async fn execute(self) -> Result<String>
    where
        Query: ObjectType + Send + Sync,
        Mutation: ObjectType + Send + Sync,
    {
        let ctx = ContextBase {
            item: &self.selection_set,
            variables: &self.variables,
            variable_definitions: self.variable_definitions.as_deref(),
            registry: self.registry,
            data: self.data,
            fragments: &self.fragments,
        };
        let mut w = JsonWriter::default();

        match self.root {
            Root::Query(query) => OutputValueType::resolve(query, &ctx, &mut w).await?,
            Root::Mutation(mutation) => OutputValueType::resolve(mutation, &ctx, &mut w).await?,
        }
        Ok(w.into_string())
    }

    /// Get cache control value
    pub fn cache_control(&self) -> CacheControl {
        self.cache_control
    }
}
