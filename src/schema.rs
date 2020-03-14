use crate::context::Data;
use crate::model::__DirectiveLocation;
use crate::registry::{Directive, InputValue, Registry};
use crate::types::QueryRoot;
use crate::validation::check_rules;
use crate::{
    ContextBase, GQLObject, GQLOutputValue, GQLType, QueryError, QueryParseError, Result, Variables,
};
use graphql_parser::parse_query;
use graphql_parser::query::{
    Definition, FragmentDefinition, OperationDefinition, SelectionSet, VariableDefinition,
};
use std::any::Any;
use std::collections::HashMap;

/// GraphQL schema
pub struct Schema<Query, Mutation> {
    query: QueryRoot<Query>,
    mutation: Mutation,
    registry: Registry,
    data: Data,
}

impl<Query: GQLObject, Mutation: GQLObject> Schema<Query, Mutation> {
    /// Create a schema.
    ///
    /// The root object for the query and Mutation needs to be specified.
    /// If there is no mutation, you can use `GQLEmptyMutation`.
    pub fn new(query: Query, mutation: Mutation) -> Self {
        let mut registry = Registry {
            types: Default::default(),
            directives: Default::default(),
            implements: Default::default(),
            query_type: Query::type_name().to_string(),
            mutation_type: if Mutation::is_empty() {
                None
            } else {
                Some(Mutation::type_name().to_string())
            },
        };

        registry.add_directive(Directive {
            name: "include",
            description: Some("Directs the executor to include this field or fragment only when the `if` argument is true."),
            locations: vec![
                __DirectiveLocation::FIELD,
                __DirectiveLocation::FRAGMENT_SPREAD,
                __DirectiveLocation::INLINE_FRAGMENT
            ],
            args: {
                let mut args = HashMap::new();
                args.insert("if", InputValue {
                    name: "if",
                    description: Some("Included when true."),
                    ty: "Boolean!".to_string(),
                    default_value: None
                });
                args
            }
        });

        registry.add_directive(Directive {
            name: "skip",
            description: Some("Directs the executor to skip this field or fragment when the `if` argument is true."),
            locations: vec![
                __DirectiveLocation::FIELD,
                __DirectiveLocation::FRAGMENT_SPREAD,
                __DirectiveLocation::INLINE_FRAGMENT
            ],
            args: {
                let mut args = HashMap::new();
                args.insert("if", InputValue {
                    name: "if",
                    description: Some("Skipped when true."),
                    ty: "Boolean!".to_string(),
                    default_value: None
                });
                args
            }
        });

        // register scalars
        bool::create_type_info(&mut registry);
        i32::create_type_info(&mut registry);
        f32::create_type_info(&mut registry);
        String::create_type_info(&mut registry);

        QueryRoot::<Query>::create_type_info(&mut registry);
        if !Mutation::is_empty() {
            Mutation::create_type_info(&mut registry);
        }

        Self {
            query: QueryRoot { inner: query },
            mutation,
            registry,
            data: Default::default(),
        }
    }

    /// Add a global data that can be accessed in the `Context`.
    pub fn data<D: Any + Send + Sync>(mut self, data: D) -> Self {
        self.data.insert(data);
        self
    }

    /// Start a query and return `QueryBuilder`.
    pub fn query<'a>(&'a self, query_source: &'a str) -> QueryBuilder<'a, Query, Mutation> {
        QueryBuilder {
            query: &self.query,
            mutation: &self.mutation,
            registry: &self.registry,
            query_source,
            operation_name: None,
            variables: None,
            data: &self.data,
        }
    }
}

enum Root<'a, Query, Mutation> {
    Query(&'a QueryRoot<Query>),
    Mutation(&'a Mutation),
}

/// Query builder
pub struct QueryBuilder<'a, Query, Mutation> {
    query: &'a QueryRoot<Query>,
    mutation: &'a Mutation,
    registry: &'a Registry,
    query_source: &'a str,
    operation_name: Option<&'a str>,
    variables: Option<Variables>,
    data: &'a Data,
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
        let document =
            parse_query(self.query_source).map_err(|err| QueryParseError(err.to_string()))?;

        check_rules(self.registry, &document)?;

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
                        break;
                    }
                    OperationDefinition::Query(query)
                        if query.name.is_none() || query.name.as_deref() == self.operation_name =>
                    {
                        selection_set = Some(query.selection_set);
                        variable_definitions = Some(query.variable_definitions);
                        root = Some(Root::Query(self.query));
                        break;
                    }
                    OperationDefinition::Mutation(mutation)
                        if mutation.name.is_none()
                            || mutation.name.as_deref() == self.operation_name =>
                    {
                        selection_set = Some(mutation.selection_set);
                        variable_definitions = Some(mutation.variable_definitions);
                        root = Some(Root::Mutation(self.mutation));
                        break;
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
        })
    }

    /// Execute the query.
    pub async fn execute(self) -> Result<serde_json::Value>
    where
        Query: GQLObject + Send + Sync,
        Mutation: GQLObject + Send + Sync,
    {
        self.prepare()?.execute().await
    }
}

pub struct PreparedQuery<'a, Query, Mutation> {
    root: Root<'a, Query, Mutation>,
    registry: &'a Registry,
    variables: Variables,
    data: &'a Data,
    fragments: HashMap<String, FragmentDefinition>,
    selection_set: SelectionSet,
    variable_definitions: Option<Vec<VariableDefinition>>,
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
        content: Vec<u8>,
    ) {
        self.variables
            .set_upload(var_path, filename, content_type, content);
    }

    /// Execute the query.
    pub async fn execute(self) -> Result<serde_json::Value>
    where
        Query: GQLObject + Send + Sync,
        Mutation: GQLObject + Send + Sync,
    {
        let ctx = ContextBase {
            item: &self.selection_set,
            variables: &self.variables,
            variable_definitions: self.variable_definitions.as_deref(),
            registry: self.registry.clone(),
            data: self.data,
            fragments: &self.fragments,
        };

        match self.root {
            Root::Query(query) => return GQLOutputValue::resolve(query, &ctx).await,
            Root::Mutation(mutation) => return GQLOutputValue::resolve(mutation, &ctx).await,
        }
    }
}
