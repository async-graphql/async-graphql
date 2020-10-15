use std::any::Any;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use futures::stream::{self, Stream, StreamExt};
use indexmap::map::IndexMap;
use itertools::Itertools;

use crate::context::{Data, QueryEnvInner, ResolveId};
use crate::extensions::{ErrorLogger, ExtensionContext, ExtensionFactory, Extensions};
use crate::model::__DirectiveLocation;
use crate::parser::parse_query;
use crate::parser::types::{DocumentOperations, OperationType};
use crate::registry::{MetaDirective, MetaInputValue, Registry};
use crate::resolver_utils::{resolve_container, resolve_container_serial, ContainerType};
use crate::subscription::collect_subscription_streams;
use crate::types::QueryRoot;
use crate::validation::{check_rules, CheckResult, ValidationMode};
use crate::{
    BatchRequest, BatchResponse, CacheControl, ContextBase, ObjectType, QueryEnv, Request,
    Response, ServerError, SubscriptionType, Type, Value, ID,
};

/// Schema builder
pub struct SchemaBuilder<Query, Mutation, Subscription> {
    validation_mode: ValidationMode,
    query: QueryRoot<Query>,
    mutation: Mutation,
    subscription: Subscription,
    registry: Registry,
    data: Data,
    complexity: Option<usize>,
    depth: Option<usize>,
    extensions: Vec<Box<dyn ExtensionFactory>>,
    enable_federation: bool,
}

impl<Query: ContainerType, Mutation: ContainerType, Subscription: SubscriptionType>
    SchemaBuilder<Query, Mutation, Subscription>
{
    /// Manually register a type in the schema.
    ///
    /// You can use this function to register schema types that are not directly referenced.
    pub fn register_type<T: Type>(mut self) -> Self {
        T::create_type_info(&mut self.registry);
        self
    }

    /// Disable introspection queries.
    pub fn disable_introspection(mut self) -> Self {
        self.query.disable_introspection = true;
        self
    }

    /// Set the maximum complexity a query can have. By default there is no limit.
    pub fn limit_complexity(mut self, complexity: usize) -> Self {
        self.complexity = Some(complexity);
        self
    }

    /// Set the maximum depth a query can have. By default there is no limit.
    pub fn limit_depth(mut self, depth: usize) -> Self {
        self.depth = Some(depth);
        self
    }

    /// Add an extension to the schema.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use async_graphql::*;
    ///
    /// #[derive(SimpleObject)]
    /// struct Query;
    ///
    /// let schema = Schema::build(Query, EmptyMutation,EmptySubscription)
    ///     .extension(extensions::Logger)
    ///     .finish();
    /// ```
    pub fn extension(mut self, extension: impl ExtensionFactory) -> Self {
        self.extensions.push(Box::new(extension));
        self
    }

    /// Add a global data that can be accessed in the `Schema`. You access it with `Context::data`.
    pub fn data<D: Any + Send + Sync>(mut self, data: D) -> Self {
        self.data.insert(data);
        self
    }

    /// Set the validation mode, default is `ValidationMode::Strict`.
    pub fn validation_mode(mut self, validation_mode: ValidationMode) -> Self {
        self.validation_mode = validation_mode;
        self
    }

    /// Enable federation, which is automatically enabled if the Query has least one entity definition.
    pub fn enable_federation(mut self) -> Self {
        self.enable_federation = true;
        self
    }

    /// Build schema.
    pub fn finish(mut self) -> Schema<Query, Mutation, Subscription> {
        // federation
        if self.enable_federation || self.registry.has_entities() {
            self.registry.create_federation_types();
        }

        Schema(Arc::new(SchemaInner {
            validation_mode: self.validation_mode,
            query: self.query,
            mutation: self.mutation,
            subscription: self.subscription,
            complexity: self.complexity,
            depth: self.depth,
            extensions: self.extensions,
            env: SchemaEnv(Arc::new(SchemaEnvInner {
                registry: self.registry,
                data: self.data,
            })),
        }))
    }
}

#[doc(hidden)]
pub struct SchemaEnvInner {
    pub registry: Registry,
    pub data: Data,
}

#[doc(hidden)]
#[derive(Clone)]
pub struct SchemaEnv(Arc<SchemaEnvInner>);

impl Deref for SchemaEnv {
    type Target = SchemaEnvInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[doc(hidden)]
pub struct SchemaInner<Query, Mutation, Subscription> {
    pub(crate) validation_mode: ValidationMode,
    pub(crate) query: QueryRoot<Query>,
    pub(crate) mutation: Mutation,
    pub(crate) subscription: Subscription,
    pub(crate) complexity: Option<usize>,
    pub(crate) depth: Option<usize>,
    pub(crate) extensions: Vec<Box<dyn ExtensionFactory>>,
    pub(crate) env: SchemaEnv,
}

/// GraphQL schema.
///
/// Cloning a schema is cheap, so it can be easily shared.
pub struct Schema<Query, Mutation, Subscription>(Arc<SchemaInner<Query, Mutation, Subscription>>);

impl<Query, Mutation, Subscription> Clone for Schema<Query, Mutation, Subscription> {
    fn clone(&self) -> Self {
        Schema(self.0.clone())
    }
}

impl<Query, Mutation, Subscription> Default for Schema<Query, Mutation, Subscription>
where
    Query: Default + ObjectType + Send + Sync + 'static,
    Mutation: Default + ObjectType + Send + Sync + 'static,
    Subscription: Default + SubscriptionType + Send + Sync + 'static,
{
    fn default() -> Self {
        Schema::new(
            Query::default(),
            Mutation::default(),
            Subscription::default(),
        )
    }
}

impl<Query, Mutation, Subscription> Deref for Schema<Query, Mutation, Subscription> {
    type Target = SchemaInner<Query, Mutation, Subscription>;

    fn deref(&self) -> &Self::Target {
        &self.0
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
        SchemaBuilder {
            validation_mode: ValidationMode::Strict,
            query: QueryRoot {
                inner: query,
                disable_introspection: false,
            },
            mutation,
            subscription,
            registry: Self::create_registry(),
            data: Default::default(),
            complexity: None,
            depth: None,
            extensions: Default::default(),
            enable_federation: false,
        }
    }

    fn create_registry() -> Registry {
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

        registry.add_directive(MetaDirective {
            name: "include",
            description: Some("Directs the executor to include this field or fragment only when the `if` argument is true."),
            locations: vec![
                __DirectiveLocation::FIELD,
                __DirectiveLocation::FRAGMENT_SPREAD,
                __DirectiveLocation::INLINE_FRAGMENT
            ],
            args: {
                let mut args = IndexMap::new();
                args.insert("if", MetaInputValue {
                    name: "if",
                    description: Some("Included when true."),
                    ty: "Boolean!".to_string(),
                    default_value: None,
                    validator: None,
                });
                args
            }
        });

        registry.add_directive(MetaDirective {
            name: "skip",
            description: Some("Directs the executor to skip this field or fragment when the `if` argument is true."),
            locations: vec![
                __DirectiveLocation::FIELD,
                __DirectiveLocation::FRAGMENT_SPREAD,
                __DirectiveLocation::INLINE_FRAGMENT
            ],
            args: {
                let mut args = IndexMap::new();
                args.insert("if", MetaInputValue {
                    name: "if",
                    description: Some("Skipped when true."),
                    ty: "Boolean!".to_string(),
                    default_value: None,
                    validator: None,
                });
                args
            }
        });

        registry.add_directive(MetaDirective {
            name: "ifdef",
            description: Some("Directs the executor to query only when the field exists."),
            locations: vec![__DirectiveLocation::FIELD],
            args: Default::default(),
        });

        // register scalars
        bool::create_type_info(&mut registry);
        i32::create_type_info(&mut registry);
        f32::create_type_info(&mut registry);
        String::create_type_info(&mut registry);
        ID::create_type_info(&mut registry);

        QueryRoot::<Query>::create_type_info(&mut registry);
        if !Mutation::is_empty() {
            Mutation::create_type_info(&mut registry);
        }
        if !Subscription::is_empty() {
            Subscription::create_type_info(&mut registry);
        }

        registry
    }

    /// Create a schema
    pub fn new(
        query: Query,
        mutation: Mutation,
        subscription: Subscription,
    ) -> Schema<Query, Mutation, Subscription> {
        Self::build(query, mutation, subscription).finish()
    }

    /// Returns SDL(Schema Definition Language) of this schema.
    pub fn sdl() -> String {
        Self::create_registry().export_sdl(false)
    }

    async fn prepare_request(
        &self,
        request: Request,
    ) -> Result<(QueryEnvInner, CacheControl), Vec<ServerError>> {
        // create extension instances
        let extensions: Extensions = self
            .0
            .extensions
            .iter()
            .map(|factory| factory.create())
            .collect_vec()
            .into();

        let request = extensions
            .prepare_request(
                &ExtensionContext {
                    schema_data: &self.env.data,
                    query_data: &Default::default(),
                },
                request,
            )
            .await?;

        let ctx_extension = ExtensionContext {
            schema_data: &self.env.data,
            query_data: &request.data,
        };

        extensions.parse_start(&ctx_extension, &request.query, &request.variables);
        let document = parse_query(&request.query)
            .map_err(Into::<ServerError>::into)
            .log_error(&ctx_extension, &extensions)?;
        extensions.parse_end(&ctx_extension, &document);

        // check rules
        extensions.validation_start(&ctx_extension);
        let CheckResult {
            cache_control,
            complexity,
            depth,
        } = check_rules(
            &self.env.registry,
            &document,
            Some(&request.variables),
            self.validation_mode,
        )
        .log_error(&ctx_extension, &extensions)?;
        extensions.validation_end(&ctx_extension);

        // check limit
        if let Some(limit_complexity) = self.complexity {
            if complexity > limit_complexity {
                return Err(vec![ServerError::new("Query is too complex.")])
                    .log_error(&ctx_extension, &extensions);
            }
        }

        if let Some(limit_depth) = self.depth {
            if depth > limit_depth {
                return Err(vec![ServerError::new("Query is nested too deep.")])
                    .log_error(&ctx_extension, &extensions);
            }
        }

        let operation = if let Some(operation_name) = &request.operation_name {
            match document.operations {
                DocumentOperations::Single(_) => None,
                DocumentOperations::Multiple(mut operations) => {
                    operations.remove(operation_name.as_str())
                }
            }
            .ok_or_else(|| {
                ServerError::new(format!(r#"Unknown operation named "{}""#, operation_name))
            })
        } else {
            match document.operations {
                DocumentOperations::Single(operation) => Ok(operation),
                DocumentOperations::Multiple(map) if map.len() == 1 => {
                    Ok(map.into_iter().next().unwrap().1)
                }
                DocumentOperations::Multiple(_) => {
                    Err(ServerError::new("Operation name required in request."))
                }
            }
        };
        let operation = match operation {
            Ok(operation) => operation,
            Err(e) => {
                extensions.error(&ctx_extension, &e);
                return Err(vec![e]);
            }
        };

        let env = QueryEnvInner {
            extensions,
            variables: request.variables,
            operation,
            fragments: document.fragments,
            uploads: request.uploads,
            ctx_data: Arc::new(request.data),
        };
        Ok((env, cache_control))
    }

    async fn execute_once(&self, env: QueryEnv) -> Response {
        // execute
        let inc_resolve_id = AtomicUsize::default();
        let ctx = ContextBase {
            path_node: None,
            resolve_id: ResolveId::root(),
            inc_resolve_id: &inc_resolve_id,
            item: &env.operation.node.selection_set,
            schema_env: &self.env,
            query_env: &env,
        };
        let ctx_extension = ExtensionContext {
            schema_data: &self.env.data,
            query_data: &env.ctx_data,
        };

        env.extensions.execution_start(&ctx_extension);

        let data = match &env.operation.node.ty {
            OperationType::Query => resolve_container(&ctx, &self.query).await,
            OperationType::Mutation => resolve_container_serial(&ctx, &self.mutation).await,
            OperationType::Subscription => {
                return Response::from_errors(vec![ServerError::new(
                    "Subscriptions are not supported on this transport.",
                )])
            }
        };

        env.extensions.execution_end(&ctx_extension);
        let extensions = env.extensions.result(&ctx_extension);

        match data {
            Ok(data) => Response::new(data),
            Err(e) => Response::from_errors(vec![e]),
        }
        .extensions(extensions)
    }

    /// Execute an GraphQL query.
    pub async fn execute(&self, request: impl Into<Request>) -> Response {
        let request = request.into();
        match self.prepare_request(request).await {
            Ok((env, cache_control)) => self
                .execute_once(QueryEnv::new(env))
                .await
                .cache_control(cache_control),
            Err(errors) => Response::from_errors(errors),
        }
    }

    /// Execute an GraphQL batch query.
    pub async fn execute_batch(&self, batch_request: BatchRequest) -> BatchResponse {
        match batch_request {
            BatchRequest::Single(request) => BatchResponse::Single(self.execute(request).await),
            BatchRequest::Batch(requests) => BatchResponse::Batch(
                futures::stream::iter(requests.into_iter())
                    .then(|request| self.execute(request))
                    .collect()
                    .await,
            ),
        }
    }

    pub(crate) fn execute_stream_with_ctx_data(
        &self,
        request: impl Into<Request> + Send,
        ctx_data: Arc<Data>,
    ) -> impl Stream<Item = Response> + Send {
        let schema = self.clone();

        async_stream::stream! {
            let request = request.into();
            let (mut env, cache_control) = match schema.prepare_request(request).await {
                Ok(res) => res,
                Err(errors) => {
                    yield Response::from_errors(errors);
                    return;
                }
            };
            env.ctx_data = ctx_data;
            let env = QueryEnv::new(env);

            if env.operation.node.ty != OperationType::Subscription {
                yield schema
                    .execute_once(env)
                    .await
                    .cache_control(cache_control);
                return;
            }

            let resolve_id = AtomicUsize::default();
            let ctx = env.create_context(
                &schema.env,
                None,
                &env.operation.node.selection_set,
                ResolveId::root(),
                &resolve_id,
            );
            let ctx_extension = ExtensionContext {
                schema_data: &schema.env.data,
                query_data: &env.ctx_data,
            };

            env.extensions.execution_start(&ctx_extension);

            let mut streams = Vec::new();
            if let Err(e) = collect_subscription_streams(&ctx, &schema.subscription, &mut streams) {
                env.extensions.execution_end(&ctx_extension);
                yield Response::from_errors(vec![e]);
                return;
            }

            env.extensions.execution_end(&ctx_extension);

            let mut stream = stream::select_all(streams);
            while let Some(data) = stream.next().await {
                let is_err = data.is_err();
                let extensions = env.extensions.result(&ctx_extension);
                yield match data {
                    Ok((name, value)) => {
                        let mut map = BTreeMap::new();
                        map.insert(name, value);
                        Response::new(Value::Object(map))
                    },
                    Err(e) => Response::from_errors(vec![e]),
                }.extensions(extensions);
                if is_err {
                    break;
                }
            }
        }
    }

    /// Execute an GraphQL subscription.
    pub fn execute_stream(
        &self,
        request: impl Into<Request>,
    ) -> impl Stream<Item = Response> + Send {
        let mut request = request.into();
        let ctx_data = std::mem::take(&mut request.data);
        self.execute_stream_with_ctx_data(request, Arc::new(ctx_data))
    }
}
