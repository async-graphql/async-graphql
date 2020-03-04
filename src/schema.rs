use crate::registry::Registry;
use crate::types::QueryRoot;
use crate::{
    ContextBase, Data, ErrorWithPosition, GQLObject, GQLOutputValue, QueryError, QueryParseError,
    Result, Variables,
};
use graphql_parser::parse_query;
use graphql_parser::query::{Definition, OperationDefinition};
use serde::export::PhantomData;

pub struct Schema<Query, Mutation> {
    mark_query: PhantomData<Query>,
    mark_mutation: PhantomData<Mutation>,
    registry: Registry,
}

impl<Query: GQLObject, Mutation: GQLObject> Schema<Query, Mutation> {
    pub fn new() -> Self {
        let mut registry = Default::default();
        Query::create_type_info(&mut registry);
        Mutation::create_type_info(&mut registry);
        Self {
            mark_query: PhantomData,
            mark_mutation: PhantomData,
            registry,
        }
    }

    pub fn query<'a>(
        &'a self,
        query: Query,
        mutation: Mutation,
        query_source: &'a str,
    ) -> QueryBuilder<'a, Query, Mutation> {
        QueryBuilder {
            query: QueryRoot {
                inner: query,
                query_type: Query::type_name().to_string(),
                mutation_type: Mutation::type_name().to_string(),
            },
            mutation,
            registry: &self.registry,
            query_source,
            operation_name: None,
            variables: None,
            data: None,
        }
    }
}

pub struct QueryBuilder<'a, Query, Mutation> {
    query: QueryRoot<Query>,
    mutation: Mutation,
    registry: &'a Registry,
    query_source: &'a str,
    operation_name: Option<&'a str>,
    variables: Option<&'a Variables>,
    data: Option<&'a Data>,
}

impl<'a, Query, Mutation> QueryBuilder<'a, Query, Mutation> {
    pub fn operator_name(self, name: &'a str) -> Self {
        QueryBuilder {
            operation_name: Some(name),
            ..self
        }
    }

    pub fn variables(self, vars: &'a Variables) -> Self {
        QueryBuilder {
            variables: Some(vars),
            ..self
        }
    }

    pub fn data(self, data: &'a Data) -> Self {
        QueryBuilder {
            data: Some(data),
            ..self
        }
    }

    pub async fn execute(self) -> Result<serde_json::Value>
    where
        Query: GQLObject + Send + Sync,
        Mutation: GQLObject + Send + Sync,
    {
        let document =
            parse_query(self.query_source).map_err(|err| QueryParseError(err.to_string()))?;

        for definition in &document.definitions {
            match definition {
                Definition::Operation(OperationDefinition::SelectionSet(selection_set)) => {
                    if self.operation_name.is_none() {
                        let ctx = ContextBase {
                            item: selection_set,
                            data: self.data.as_deref(),
                            variables: self.variables.as_deref(),
                            variable_definitions: None,
                            registry: &self.registry,
                        };
                        return self.query.resolve(&ctx).await;
                    }
                }
                Definition::Operation(OperationDefinition::Query(query)) => {
                    if self.operation_name.is_none()
                        || self.operation_name == query.name.as_ref().map(|s| s.as_str())
                    {
                        let ctx = ContextBase {
                            item: &query.selection_set,
                            data: self.data.as_deref(),
                            variables: self.variables.as_deref(),
                            variable_definitions: Some(&query.variable_definitions),
                            registry: self.registry.clone(),
                        };
                        return self.query.resolve(&ctx).await;
                    }
                }
                Definition::Operation(OperationDefinition::Mutation(mutation)) => {
                    if self.operation_name.is_none()
                        || self.operation_name == mutation.name.as_ref().map(|s| s.as_str())
                    {
                        let ctx = ContextBase {
                            item: &mutation.selection_set,
                            data: self.data.as_deref(),
                            variables: self.variables.as_deref(),
                            variable_definitions: Some(&mutation.variable_definitions),
                            registry: self.registry.clone(),
                        };
                        return self.mutation.resolve(&ctx).await;
                    }
                }
                Definition::Operation(OperationDefinition::Subscription(subscription)) => {
                    anyhow::bail!(QueryError::NotSupported.with_position(subscription.position));
                }
                Definition::Fragment(fragment) => {
                    anyhow::bail!(QueryError::NotSupported.with_position(fragment.position));
                }
            }
        }

        if let Some(operation_name) = self.operation_name {
            anyhow::bail!(QueryError::UnknownOperationNamed {
                name: operation_name.to_string()
            });
        }

        Ok(serde_json::Value::Null)
    }
}
