use crate::context::Data;
use crate::extensions::{BoxExtension, Extension};
use crate::model::__DirectiveLocation;
use crate::query::QueryBuilder;
use crate::registry::{Directive, InputValue, Registry};
use crate::subscription::{create_connection, create_subscription_stream, SubscriptionTransport};
use crate::types::QueryRoot;
use crate::validation::{check_rules, ValidationMode};
use crate::{
    Environment, Error, ObjectType, Pos, QueryError, QueryResponse, Result, SubscriptionStream,
    SubscriptionType, Type, Variables,
};
use bytes::Bytes;
use futures::channel::mpsc;
use futures::Stream;
use graphql_parser::parse_query;
use graphql_parser::query::{Definition, OperationDefinition};
use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

pub(crate) struct SchemaInner<Query, Mutation, Subscription> {
    pub(crate) validation_mode: ValidationMode,
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
    /// You can use this function to register types that are not directly referenced.
    pub fn register_type<T: Type>(mut self) -> Self {
        T::create_type_info(&mut self.0.registry);
        self
    }

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

    /// Set the validation mode, default is `ValidationMode::Strict`.
    pub fn validation_mode(mut self, validation_mode: ValidationMode) -> Self {
        self.0.validation_mode = validation_mode;
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

        // federation
        registry.create_federation_types();

        SchemaBuilder(SchemaInner {
            validation_mode: ValidationMode::Strict,
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

    /// Execute query without create the `QueryBuilder`.
    pub async fn execute(&self, query_source: &str) -> Result<QueryResponse> {
        QueryBuilder::new(query_source).execute(self).await
    }

    /// Create subscription stream, typically called inside the `SubscriptionTransport::handle_request` method
    pub async fn create_subscription_stream(
        &self,
        source: &str,
        operation_name: Option<&str>,
        variables: Variables,
        ctx_data: Option<Arc<Data>>,
    ) -> Result<impl Stream<Item = serde_json::Value> + Send> {
        let document = parse_query(source).map_err(Into::<Error>::into)?;
        check_rules(&self.0.registry, &document, self.0.validation_mode)?;

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
        let environment = Arc::new(Environment {
            variables,
            variable_definitions: subscription.variable_definitions,
            fragments,
            ctx_data: ctx_data.unwrap_or_default(),
        });
        let ctx = environment.create_context(self, None, &subscription.selection_set, &resolve_id);
        let mut streams = Vec::new();
        create_subscription_stream(self, environment.clone(), &ctx, &mut streams).await?;
        Ok(futures::stream::select_all(streams))
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
