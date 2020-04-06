use crate::context::Data;
use crate::extensions::{BoxExtension, Extension};
use crate::model::__DirectiveLocation;
use crate::query::QueryBuilder;
use crate::registry::{Directive, InputValue, Registry};
use crate::subscription::{create_connection, create_subscription_stream, SubscriptionTransport};
use crate::types::QueryRoot;
use crate::validation::{check_rules, CheckResult};
use crate::{
    ContextSelectionSet, Error, ObjectType, Pos, QueryError, QueryResponse, Result,
    SubscriptionStream, SubscriptionType, Type, Variables,
};
use bytes::Bytes;
use futures::channel::mpsc;
use futures::{Stream, TryFutureExt};
use graphql_parser::parse_query;
use graphql_parser::query::{Definition, OperationDefinition};
use itertools::Itertools;
use std::any::Any;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

pub(crate) struct SchemaInner<Query, Mutation, Subscription> {
    pub(crate) query: QueryRoot<Query>,
    pub(crate) mutation: Mutation,
    pub(crate) subscription: Subscription,
    pub(crate) registry: Registry,
    pub(crate) data: Data,
    pub(crate) complexity: Option<usize>,
    pub(crate) depth: Option<usize>,
    pub(crate) extensions: Vec<Box<dyn Fn() -> BoxExtension + Send + Sync>>,
}

/// Schema builder
pub struct SchemaBuilder<Query, Mutation, Subscription>(SchemaInner<Query, Mutation, Subscription>);

impl<Query: ObjectType, Mutation: ObjectType, Subscription: SubscriptionType>
    SchemaBuilder<Query, Mutation, Subscription>
{
    /// Disable introspection query
    pub fn disable_introspection(mut self) -> Self {
        self.0.query.disable_introspection = true;
        self
    }

    /// Set limit complexity, Default no limit.
    pub fn limit_complexity(mut self, complexity: usize) -> Self {
        self.0.complexity = Some(complexity);
        self
    }

    /// Set limit complexity, Default no limit.
    pub fn limit_depth(mut self, depth: usize) -> Self {
        self.0.depth = Some(depth);
        self
    }

    /// Add an extension
    pub fn extension<F: Fn() -> E + Send + Sync + 'static, E: Extension>(
        mut self,
        extension_factory: F,
    ) -> Self {
        self.0
            .extensions
            .push(Box::new(move || Box::new(extension_factory())));
        self
    }

    /// Add a global data that can be accessed in the `Schema`, you access it with `Context::data`.
    pub fn data<D: Any + Send + Sync>(mut self, data: D) -> Self {
        self.0.data.insert(data);
        self
    }

    /// Build schema.
    pub fn finish(self) -> Schema<Query, Mutation, Subscription> {
        Schema(Arc::new(self.0))
    }
}

/// GraphQL schema
pub struct Schema<Query, Mutation, Subscription>(
    pub(crate) Arc<SchemaInner<Query, Mutation, Subscription>>,
);

impl<Query, Mutation, Subscription> Clone for Schema<Query, Mutation, Subscription> {
    fn clone(&self) -> Self {
        Schema(self.0.clone())
    }
}

impl<Query, Mutation, Subscription> Schema<Query, Mutation, Subscription>
where
    Query: ObjectType + Send + Sync + 'static,
    Mutation: ObjectType + Send + Sync + 'static,
    Subscription: SubscriptionType + Send + Sync + 'static,
{
    /// Create a schema builder
    ///
    /// The root object for the query and Mutation needs to be specified.
    /// If there is no mutation, you can use `EmptyMutation`.
    /// If there is no subscription, you can use `EmptySubscription`.
    pub fn build(
        query: Query,
        mutation: Mutation,
        subscription: Subscription,
    ) -> SchemaBuilder<Query, Mutation, Subscription> {
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
            subscription_type: if Subscription::is_empty() {
                None
            } else {
                Some(Subscription::type_name().to_string())
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
                    default_value: None,
                    validator: None,
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
                    default_value: None,
                    validator: None,
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
        if !Subscription::is_empty() {
            Subscription::create_type_info(&mut registry);
        }

        SchemaBuilder(SchemaInner {
            query: QueryRoot {
                inner: query,
                disable_introspection: false,
            },
            mutation,
            subscription,
            registry,
            data: Default::default(),
            complexity: None,
            depth: None,
            extensions: Default::default(),
        })
    }

    /// Create a schema
    pub fn new(
        query: Query,
        mutation: Mutation,
        subscription: Subscription,
    ) -> Schema<Query, Mutation, Subscription> {
        Self::build(query, mutation, subscription).finish()
    }

    #[doc(hidden)]
    pub fn data(&self) -> &Data {
        &self.0.data
    }

    #[doc(hidden)]
    pub fn registry(&self) -> &Registry {
        &self.0.registry
    }

    /// Start a query and return `QueryBuilder`.
    pub fn query(&self, source: &str) -> Result<QueryBuilder<Query, Mutation, Subscription>> {
        let extensions = self
            .0
            .extensions
            .iter()
            .map(|factory| factory())
            .collect_vec();
        extensions.iter().for_each(|e| e.parse_start(source));
        let document = parse_query(source).map_err(Into::<Error>::into)?;
        extensions.iter().for_each(|e| e.parse_end());

        extensions.iter().for_each(|e| e.validation_start());
        let CheckResult {
            cache_control,
            complexity,
            depth,
        } = check_rules(&self.0.registry, &document)?;
        extensions.iter().for_each(|e| e.validation_end());

        if let Some(limit_complexity) = self.0.complexity {
            if complexity > limit_complexity {
                return Err(QueryError::TooComplex.into_error(Pos::default()));
            }
        }

        if let Some(limit_depth) = self.0.depth {
            if depth > limit_depth {
                return Err(QueryError::TooDeep.into_error(Pos::default()));
            }
        }

        Ok(QueryBuilder {
            extensions,
            schema: self.clone(),
            document,
            operation_name: None,
            variables: Default::default(),
            ctx_data: None,
            cache_control,
        })
    }

    /// Execute query without create the `QueryBuilder`.
    pub async fn execute(&self, source: &str) -> Result<QueryResponse> {
        futures::future::ready(self.query(source))
            .and_then(|builder| builder.execute())
            .await
    }

    /// Create subscription stream, typically called inside the `SubscriptionTransport::handle_request` method
    pub fn create_subscription_stream(
        &self,
        source: &str,
        operation_name: Option<&str>,
        variables: Variables,
    ) -> Result<Pin<Box<dyn Stream<Item = serde_json::Value>>>> {
        let document = parse_query(source).map_err(Into::<Error>::into)?;
        check_rules(&self.0.registry, &document)?;

        let mut fragments = HashMap::new();
        let mut subscription = None;

        for definition in document.definitions {
            match definition {
                Definition::Operation(OperationDefinition::Subscription(s)) => {
                    if s.name.as_deref() == operation_name {
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

        let subscription = subscription.ok_or(if let Some(name) = operation_name {
            QueryError::UnknownOperationNamed {
                name: name.to_string(),
            }
            .into_error(Pos::default())
        } else {
            QueryError::MissingOperation.into_error(Pos::default())
        })?;

        let resolve_id = AtomicUsize::default();
        let ctx = ContextSelectionSet {
            path_node: None,
            extensions: &[],
            item: &subscription.selection_set,
            resolve_id: &resolve_id,
            variables: &variables,
            variable_definitions: &subscription.variable_definitions,
            registry: &self.0.registry,
            data: &Default::default(),
            ctx_data: None,
            fragments: &fragments,
        };

        let mut streams = Vec::new();
        create_subscription_stream(self, Arc::new(ctx.create_environment()), &ctx, &mut streams)?;
        Ok(Box::pin(futures::stream::select_all(streams)))
    }

    /// Create subscription connection, returns `Sink` and `Stream`.
    pub fn subscription_connection<T: SubscriptionTransport>(
        &self,
        transport: T,
    ) -> (
        mpsc::Sender<Bytes>,
        SubscriptionStream<Query, Mutation, Subscription, T>,
    ) {
        create_connection(self.clone(), transport)
    }
}
