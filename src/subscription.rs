use crate::registry::Registry;
use crate::validation::check_rules;
use crate::{ContextBase, GQLType, QueryError, QueryParseError, Result, Schema, Variables};
use graphql_parser::parse_query;
use graphql_parser::query::{
    Definition, Field, FragmentDefinition, OperationDefinition, SelectionSet, VariableDefinition,
};
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct Subscribe {
    types: HashMap<TypeId, Field>,
    variables: Variables,
    variable_definitions: Vec<VariableDefinition>,
    fragments: HashMap<String, FragmentDefinition>,
}

impl Subscribe {
    pub async fn resolve<Query, Mutation, Subscription>(
        &self,
        schema: &Schema<Query, Mutation, Subscription>,
        msg: &(dyn Any + Send + Sync),
    ) -> Result<Option<serde_json::Value>>
    where
        Subscription: GQLSubscription + Sync + Send + 'static,
    {
        let ctx = ContextBase::<()> {
            item: (),
            variables: &self.variables,
            variable_definitions: Some(&self.variable_definitions),
            registry: &schema.registry,
            data: &schema.data,
            fragments: &self.fragments,
        };
        schema.subscription.resolve(&ctx, &self.types, msg).await
    }
}

/// Represents a GraphQL subscription object
#[async_trait::async_trait]
pub trait GQLSubscription: GQLType {
    /// This function returns true of type `GQLEmptySubscription` only
    #[doc(hidden)]
    fn is_empty() -> bool {
        return false;
    }

    fn create_types(selection_set: SelectionSet) -> Result<HashMap<TypeId, Field>>;

    fn create_subscribe(
        &self,
        selection_set: SelectionSet,
        variables: Variables,
        variable_definitions: Vec<VariableDefinition>,
        fragments: HashMap<String, FragmentDefinition>,
    ) -> Result<Subscribe> {
        Ok(Subscribe {
            types: Self::create_types(selection_set)?,
            variables,
            variable_definitions,
            fragments,
        })
    }

    /// Resolve a subscription message, If no message of this type is subscribed, None is returned.
    async fn resolve(
        &self,
        ctx: &ContextBase<'_, ()>,
        types: &HashMap<TypeId, Field>,
        msg: &(dyn Any + Send + Sync),
    ) -> Result<Option<serde_json::Value>>;
}

pub struct SubscribeBuilder<'a, Subscription> {
    pub(crate) subscription: &'a Subscription,
    pub(crate) registry: &'a Registry,
    pub(crate) source: &'a str,
    pub(crate) operation_name: Option<&'a str>,
    pub(crate) variables: Option<Variables>,
}

impl<'a, Subscription> SubscribeBuilder<'a, Subscription>
where
    Subscription: GQLSubscription,
{
    /// Specify the operation name.
    pub fn operator_name(self, name: &'a str) -> Self {
        SubscribeBuilder {
            operation_name: Some(name),
            ..self
        }
    }

    /// Specify the variables.
    pub fn variables(self, vars: Variables) -> Self {
        SubscribeBuilder {
            variables: Some(vars),
            ..self
        }
    }

    pub fn execute(self) -> Result<Subscribe> {
        let document = parse_query(self.source).map_err(|err| QueryParseError(err.to_string()))?;
        check_rules(self.registry, &document)?;

        let mut fragments = HashMap::new();
        let mut subscription = None;

        for definition in document.definitions {
            match definition {
                Definition::Operation(OperationDefinition::Subscription(s)) => {
                    if s.name.as_deref() == self.operation_name {
                        subscription = Some(s);
                        break;
                    }
                }
                Definition::Fragment(fragment) => {
                    fragments.insert(fragment.name.clone(), fragment);
                }
                _ => {}
            }
        }

        let subscription = subscription.ok_or(if let Some(name) = self.operation_name {
            QueryError::UnknownOperationNamed {
                name: name.to_string(),
            }
        } else {
            QueryError::MissingOperation
        })?;

        self.subscription.create_subscribe(
            subscription.selection_set,
            self.variables.unwrap_or_default(),
            subscription.variable_definitions,
            fragments,
        )
    }
}
