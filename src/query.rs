use crate::{
    ContextBase, Data, ErrorWithPosition, GQLObject, QueryError, QueryParseError, Result, Variables,
};
use graphql_parser::parse_query;
use graphql_parser::query::{Definition, OperationDefinition};

pub struct QueryBuilder<'a, Query, Mutation> {
    query: Query,
    mutation: Mutation,
    query_source: &'a str,
    operation_name: Option<&'a str>,
    variables: Option<&'a Variables>,
    data: Option<&'a Data>,
}

impl<'a, Query, Mutation> QueryBuilder<'a, Query, Mutation> {
    pub fn new(query: Query, mutation: Mutation, query_source: &'a str) -> Self {
        Self {
            query,
            mutation,
            query_source,
            operation_name: None,
            variables: None,
            data: None,
        }
    }

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
        Query: GQLObject,
        Mutation: GQLObject,
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
