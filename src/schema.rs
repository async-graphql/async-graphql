use crate::context::{Data, ResolveId};
use crate::extensions::{BoxExtension, ErrorLogger, Extension, Extensions};
use crate::model::__DirectiveLocation;
use crate::parser::parse_query;
use crate::parser::types::OperationType;
use crate::registry::{MetaDirective, MetaInputValue, Registry};
use crate::resolver_utils::{resolve_object, resolve_object_serial, ObjectType};
use crate::subscription::collect_subscription_streams;
use crate::types::QueryRoot;
use crate::validation::{check_rules, CheckResult, ValidationMode};
use crate::{
    BatchRequest, BatchResponse, CacheControl, ContextBase, Error, Pos, QueryEnv, QueryError,
    Request, Response, Result, SubscriptionType, Type, Variables, ID,
};
use async_graphql_parser::types::ExecutableDocumentData;
use futures::stream::{self, Stream, StreamExt};
use indexmap::map::IndexMap;
use itertools::Itertools;
use std::any::Any;
use std::ops::Deref;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

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
    extensions: Vec<Box<dyn Fn() -> BoxExtension + Send + Sync>>,
    enable_federation: bool,
}

impl<Query: ObjectType, Mutation: ObjectType, Subscription: SubscriptionType>
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
    pub fn extension<F: Fn() -> E + Send + Sync + 'static, E: Extension>(
        mut self,
        extension_factory: F,
    ) -> Self {
        self.extensions
            .push(Box::new(move || Box::new(extension_factory())));
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
    pub(crate) extensions: Vec<Box<dyn Fn() -> BoxExtension + Send + Sync>>,
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

        SchemaBuilder {
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
            enable_federation: false,
        }
    }

    /// Create a schema
    pub fn new(
        query: Query,
        mutation: Mutation,
        subscription: Subscription,
    ) -> Schema<Query, Mutation, Subscription> {
        Self::build(query, mutation, subscription).finish()
    }

    fn prepare_request(
        &self,
        request: &Request,
    ) -> Result<(
        ExecutableDocumentData,
        CacheControl,
        spin::Mutex<Extensions>,
    )> {
        // create extension instances
        let extensions = spin::Mutex::new(Extensions(
            self.0
                .extensions
                .iter()
                .map(|factory| factory())
                .collect_vec(),
        ));

        extensions
            .lock()
            .parse_start(&request.query, &request.variables);
        let document = parse_query(&request.query)
            .map_err(Into::<Error>::into)
            .log_error(&extensions)?;
        extensions.lock().parse_end(&document);

        // check rules
        extensions.lock().validation_start();
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
        .log_error(&extensions)?;
        extensions.lock().validation_end();

        // check limit
        if let Some(limit_complexity) = self.complexity {
            if complexity > limit_complexity {
                return Err(QueryError::TooComplex.into_error(Pos::default()))
                    .log_error(&extensions);
            }
        }

        if let Some(limit_depth) = self.depth {
            if depth > limit_depth {
                return Err(QueryError::TooDeep.into_error(Pos::default())).log_error(&extensions);
            }
        }

        let document = match document.into_data(request.operation_name.as_deref()) {
            Some(document) => document,
            None => {
                let err = if let Some(operation_name) = &request.operation_name {
                    Error::Query {
                        pos: Pos::default(),
                        path: None,
                        err: QueryError::UnknownOperationNamed {
                            name: operation_name.to_string(),
                        },
                    }
                } else {
                    Error::Query {
                        pos: Pos::default(),
                        path: None,
                        err: QueryError::MissingOperation,
                    }
                };
                extensions.lock().error(&err);
                return Err(err);
            }
        };

        Ok((document, cache_control, extensions))
    }

    async fn execute_once(
        &self,
        document: ExecutableDocumentData,
        extensions: spin::Mutex<Extensions>,
        variables: Variables,
        ctx_data: Data,
    ) -> Response {
        // execute
        let inc_resolve_id = AtomicUsize::default();
        let env = QueryEnv::new(extensions, variables, document, Arc::new(ctx_data));
        let ctx = ContextBase {
            path_node: None,
            resolve_id: ResolveId::root(),
            inc_resolve_id: &inc_resolve_id,
            item: &env.document.operation.node.selection_set,
            schema_env: &self.env,
            query_env: &env,
        };

        env.extensions.lock().execution_start();

        let data = match &env.document.operation.node.ty {
            OperationType::Query => resolve_object(&ctx, &self.query).await,
            OperationType::Mutation => resolve_object_serial(&ctx, &self.mutation).await,
            OperationType::Subscription => {
                return Error::Query {
                    pos: Pos::default(),
                    path: None,
                    err: QueryError::NotSupported,
                }
                .into()
            }
        };

        env.extensions.lock().execution_end();
        let extensions = env.extensions.lock().result();

        Response::from_result(data).extensions(extensions)
    }

    /// Execute an GraphQL query.
    pub async fn execute(&self, request: impl Into<Request>) -> Response {
        let request = request.into();
        match self.prepare_request(&request) {
            Ok((document, cache_control, extensions)) => self
                .execute_once(document, extensions, request.variables, request.data)
                .await
                .cache_control(cache_control),
            Err(e) => Response::from_error(e),
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
        request: impl Into<Request>,
        ctx_data: Arc<Data>,
    ) -> impl Stream<Item = Response> {
        let schema = self.clone();

        async_stream::stream! {
            let request = request.into();
            let (document, cache_control, extensions) = match schema.prepare_request(&request) {
                Ok(res) => res,
                Err(err) => {
                    yield Response::from(err);
                    return;
                }
            };

            if document.operation.node.ty != OperationType::Subscription {
                yield schema
                    .execute_once(document, extensions, request.variables, request.data)
                    .await
                    .cache_control(cache_control);
                return;
            }

            let resolve_id = AtomicUsize::default();
            let env = QueryEnv::new(
                extensions,
                request.variables,
                document,
                ctx_data,
            );

            let ctx = env.create_context(
                &schema.env,
                None,
                &env.document.operation.node.selection_set,
                &resolve_id,
            );

            // TODO: Invoke extensions

            let mut streams = Vec::new();
            if let Err(e) = collect_subscription_streams(&ctx, &schema.subscription, &mut streams) {
                yield Response::from(e);
                return;
            }

            let mut stream = stream::select_all(streams);
            while let Some(data) = stream.next().await {
                let is_err = data.is_err();
                let extensions = env.extensions.lock().result();
                yield Response::from_result(data).extensions(extensions);
                if is_err {
                    break;
                }
            }
        }
    }

    /// Execute an GraphQL subscription.
    pub fn execute_stream(&self, request: impl Into<Request>) -> impl Stream<Item = Response> {
        let mut request = request.into();
        let ctx_data = std::mem::take(&mut request.data);
        self.execute_stream_with_ctx_data(request, Arc::new(ctx_data))
    }
}
