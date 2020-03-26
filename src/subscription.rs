use crate::extensions::BoxExtension;
use crate::registry::Registry;
use crate::validation::check_rules;
use crate::{
    ContextBase, ContextSelectionSet, QueryError, QueryParseError, Result, Schema, Type, Variables,
};
use graphql_parser::parse_query;
use graphql_parser::query::{
    Definition, Field, FragmentDefinition, OperationDefinition, Selection, SelectionSet,
    VariableDefinition,
};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;

/// Subscribe stub
///
/// When a new push message is generated, a JSON object that needs to be pushed can be obtained by
/// `Subscribe::resolve`, and if None is returned, the Subscribe is not subscribed to a message of this type.
pub struct Subscribe {
    types: HashMap<TypeId, Field>,
    variables: Variables,
    variable_definitions: Vec<VariableDefinition>,
    fragments: HashMap<String, FragmentDefinition>,
}

impl Subscribe {
    #[allow(missing_docs)]
    pub async fn resolve<Query, Mutation, Subscription>(
        &self,
        schema: &Schema<Query, Mutation, Subscription>,
        msg: &(dyn Any + Send + Sync),
    ) -> Result<Option<serde_json::Value>>
    where
        Subscription: SubscriptionType + Sync + Send + 'static,
    {
        let resolve_id = AtomicUsize::default();
        let ctx = ContextBase::<()> {
            extensions: &[],
            item: (),
            resolve_id: &resolve_id,
            variables: &self.variables,
            variable_definitions: Some(&self.variable_definitions),
            registry: &schema.registry,
            data: &schema.data,
            fragments: &self.fragments,
            current_path: Default::default(),
        };
        schema.subscription.resolve(&ctx, &self.types, msg).await
    }
}

/// Represents a GraphQL subscription object
#[allow(missing_docs)]
#[async_trait::async_trait]
pub trait SubscriptionType: Type {
    /// This function returns true of type `EmptySubscription` only
    #[doc(hidden)]
    fn is_empty() -> bool {
        false
    }

    fn create_type(field: &Field, types: &mut HashMap<TypeId, Field>) -> Result<()>;

    fn create_subscribe(
        &self,
        extensions: &[BoxExtension],
        registry: &Registry,
        selection_set: SelectionSet,
        variables: Variables,
        variable_definitions: Vec<VariableDefinition>,
        fragments: HashMap<String, FragmentDefinition>,
    ) -> Result<Subscribe>
    where
        Self: Sized,
    {
        let mut types = HashMap::new();
        let resolve_id = AtomicUsize::default();
        let ctx = ContextSelectionSet {
            extensions,
            item: &selection_set,
            resolve_id: &resolve_id,
            variables: &variables,
            variable_definitions: Some(&variable_definitions),
            registry,
            data: &Default::default(),
            fragments: &fragments,
            current_path: Default::default(),
        };
        create_types::<Self>(&ctx, &fragments, &mut types)?;
        Ok(Subscribe {
            types,
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

fn create_types<T: SubscriptionType>(
    ctx: &ContextSelectionSet<'_>,
    fragments: &HashMap<String, FragmentDefinition>,
    types: &mut HashMap<TypeId, Field>,
) -> Result<()> {
    for selection in &ctx.items {
        match selection {
            Selection::Field(field) => {
                if ctx.is_skip(&field.directives)? {
                    continue;
                }
                T::create_type(field, types)?;
            }
            Selection::FragmentSpread(fragment_spread) => {
                if ctx.is_skip(&fragment_spread.directives)? {
                    continue;
                }

                if let Some(fragment) = fragments.get(&fragment_spread.fragment_name) {
                    create_types::<T>(
                        &ctx.with_selection_set(&fragment.selection_set),
                        fragments,
                        types,
                    )?;
                } else {
                    return Err(QueryError::UnknownFragment {
                        name: fragment_spread.fragment_name.clone(),
                    }
                    .into());
                }
            }
            Selection::InlineFragment(inline_fragment) => {
                if ctx.is_skip(&inline_fragment.directives)? {
                    continue;
                }
                create_types::<T>(
                    &ctx.with_selection_set(&inline_fragment.selection_set),
                    fragments,
                    types,
                )?;
            }
        }
    }
    Ok(())
}

/// Subscribe builder
pub struct SubscribeBuilder<'a, Subscription> {
    pub(crate) subscription: &'a Subscription,
    pub(crate) extensions: &'a [BoxExtension],
    pub(crate) registry: &'a Registry,
    pub(crate) source: &'a str,
    pub(crate) operation_name: Option<&'a str>,
    pub(crate) variables: Option<Variables>,
}

impl<'a, Subscription> SubscribeBuilder<'a, Subscription>
where
    Subscription: SubscriptionType,
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

    /// Perform a subscription operation and return `Subscribe`.
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
            self.extensions,
            self.registry,
            subscription.selection_set,
            self.variables.unwrap_or_default(),
            subscription.variable_definitions,
            fragments,
        )
    }
}
