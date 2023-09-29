use std::{any::Any, collections::HashMap, fmt::Debug, sync::Arc};

use async_graphql_parser::types::OperationType;
use futures_util::{stream::BoxStream, Stream, StreamExt, TryFutureExt};
use indexmap::IndexMap;

use crate::{
    dynamic::{
        field::BoxResolverFn, r#type::Type, resolve::resolve_container, DynamicRequest,
        FieldFuture, FieldValue, Object, ResolverContext, Scalar, SchemaError, Subscription,
        TypeRef, Union,
    },
    extensions::{ExtensionFactory, Extensions},
    registry::{MetaType, Registry},
    schema::{prepare_request, SchemaEnvInner},
    Data, Executor, IntrospectionMode, QueryEnv, Request, Response, SDLExportOptions, SchemaEnv,
    ServerError, ServerResult, ValidationMode,
};

/// Dynamic schema builder
pub struct SchemaBuilder {
    query_type: String,
    mutation_type: Option<String>,
    subscription_type: Option<String>,
    types: IndexMap<String, Type>,
    data: Data,
    extensions: Vec<Box<dyn ExtensionFactory>>,
    validation_mode: ValidationMode,
    recursive_depth: usize,
    complexity: Option<usize>,
    depth: Option<usize>,
    enable_suggestions: bool,
    introspection_mode: IntrospectionMode,
    enable_federation: bool,
    entity_resolver: Option<BoxResolverFn>,
}

impl SchemaBuilder {
    /// Register a GraphQL type
    #[must_use]
    pub fn register(mut self, ty: impl Into<Type>) -> Self {
        let ty = ty.into();
        self.types.insert(ty.name().to_string(), ty);
        self
    }

    /// Enable uploading files (register Upload type).
    pub fn enable_uploading(mut self) -> Self {
        self.types.insert(TypeRef::UPLOAD.to_string(), Type::Upload);
        self
    }

    /// Add a global data that can be accessed in the `Schema`. You access it
    /// with `Context::data`.
    #[must_use]
    pub fn data<D: Any + Send + Sync>(mut self, data: D) -> Self {
        self.data.insert(data);
        self
    }

    /// Add an extension to the schema.
    #[must_use]
    pub fn extension(mut self, extension: impl ExtensionFactory) -> Self {
        self.extensions.push(Box::new(extension));
        self
    }

    /// Set the maximum complexity a query can have. By default, there is no
    /// limit.
    #[must_use]
    pub fn limit_complexity(mut self, complexity: usize) -> Self {
        self.complexity = Some(complexity);
        self
    }

    /// Set the maximum depth a query can have. By default, there is no limit.
    #[must_use]
    pub fn limit_depth(mut self, depth: usize) -> Self {
        self.depth = Some(depth);
        self
    }

    /// Set the maximum recursive depth a query can have. (default: 32)
    ///
    /// If the value is too large, stack overflow may occur, usually `32` is
    /// enough.
    #[must_use]
    pub fn limit_recursive_depth(mut self, depth: usize) -> Self {
        self.recursive_depth = depth;
        self
    }

    /// Set the validation mode, default is `ValidationMode::Strict`.
    #[must_use]
    pub fn validation_mode(mut self, validation_mode: ValidationMode) -> Self {
        self.validation_mode = validation_mode;
        self
    }

    /// Disable field suggestions.
    #[must_use]
    pub fn disable_suggestions(mut self) -> Self {
        self.enable_suggestions = false;
        self
    }

    /// Disable introspection queries.
    #[must_use]
    pub fn disable_introspection(mut self) -> Self {
        self.introspection_mode = IntrospectionMode::Disabled;
        self
    }

    /// Only process introspection queries, everything else is processed as an
    /// error.
    #[must_use]
    pub fn introspection_only(mut self) -> Self {
        self.introspection_mode = IntrospectionMode::IntrospectionOnly;
        self
    }

    /// Enable federation, which is automatically enabled if the Query has least
    /// one entity definition.
    #[must_use]
    pub fn enable_federation(mut self) -> Self {
        self.enable_federation = true;
        self
    }

    /// Set the entity resolver for federation
    pub fn entity_resolver<F>(self, resolver_fn: F) -> Self
    where
        F: for<'a> Fn(ResolverContext<'a>) -> FieldFuture<'a> + Send + Sync + 'static,
    {
        Self {
            entity_resolver: Some(Box::new(resolver_fn)),
            ..self
        }
    }

    /// Consumes this builder and returns a schema.
    pub fn finish(mut self) -> Result<Schema, SchemaError> {
        let mut registry = Registry {
            types: Default::default(),
            directives: Default::default(),
            implements: Default::default(),
            query_type: self.query_type,
            mutation_type: self.mutation_type,
            subscription_type: self.subscription_type,
            introspection_mode: self.introspection_mode,
            enable_federation: false,
            federation_subscription: false,
            ignore_name_conflicts: Default::default(),
            enable_suggestions: self.enable_suggestions,
        };
        registry.add_system_types();

        for ty in self.types.values() {
            ty.register(&mut registry)?;
        }
        update_interface_possible_types(&mut self.types, &mut registry);

        // create system scalars
        for ty in ["Int", "Float", "Boolean", "String", "ID"] {
            self.types
                .insert(ty.to_string(), Type::Scalar(Scalar::new(ty)));
        }

        // create introspection types
        if matches!(
            self.introspection_mode,
            IntrospectionMode::Enabled | IntrospectionMode::IntrospectionOnly
        ) {
            registry.create_introspection_types();
        }

        // create entity types
        if self.enable_federation || registry.has_entities() {
            registry.enable_federation = true;
            registry.create_federation_types();

            // create _Entity type
            let entity = self
                .types
                .values()
                .filter(|ty| match ty {
                    Type::Object(obj) => obj.is_entity(),
                    Type::Interface(interface) => interface.is_entity(),
                    _ => false,
                })
                .fold(Union::new("_Entity"), |entity, ty| {
                    entity.possible_type(ty.name())
                });
            self.types
                .insert("_Entity".to_string(), Type::Union(entity));
        }

        let inner = SchemaInner {
            env: SchemaEnv(Arc::new(SchemaEnvInner {
                registry,
                data: self.data,
                custom_directives: Default::default(),
            })),
            extensions: self.extensions,
            types: self.types,
            recursive_depth: self.recursive_depth,
            complexity: self.complexity,
            depth: self.depth,
            validation_mode: self.validation_mode,
            entity_resolver: self.entity_resolver,
        };
        inner.check()?;
        Ok(Schema(Arc::new(inner)))
    }
}

/// Dynamic GraphQL schema.
///
/// Cloning a schema is cheap, so it can be easily shared.
#[derive(Clone)]
pub struct Schema(pub(crate) Arc<SchemaInner>);

impl Debug for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Schema").finish()
    }
}

pub struct SchemaInner {
    pub(crate) env: SchemaEnv,
    pub(crate) types: IndexMap<String, Type>,
    extensions: Vec<Box<dyn ExtensionFactory>>,
    recursive_depth: usize,
    complexity: Option<usize>,
    depth: Option<usize>,
    validation_mode: ValidationMode,
    pub(crate) entity_resolver: Option<BoxResolverFn>,
}

impl Schema {
    /// Create a schema builder
    pub fn build(query: &str, mutation: Option<&str>, subscription: Option<&str>) -> SchemaBuilder {
        SchemaBuilder {
            query_type: query.to_string(),
            mutation_type: mutation.map(ToString::to_string),
            subscription_type: subscription.map(ToString::to_string),
            types: Default::default(),
            data: Default::default(),
            extensions: Default::default(),
            validation_mode: ValidationMode::Strict,
            recursive_depth: 32,
            complexity: None,
            depth: None,
            enable_suggestions: true,
            introspection_mode: IntrospectionMode::Enabled,
            entity_resolver: None,
            enable_federation: false,
        }
    }

    fn create_extensions(&self, session_data: Arc<Data>) -> Extensions {
        Extensions::new(
            self.0.extensions.iter().map(|f| f.create()),
            self.0.env.clone(),
            session_data,
        )
    }

    fn query_root(&self) -> ServerResult<&Object> {
        self.0
            .types
            .get(&self.0.env.registry.query_type)
            .and_then(Type::as_object)
            .ok_or_else(|| ServerError::new("Query root not found", None))
    }

    fn mutation_root(&self) -> ServerResult<&Object> {
        self.0
            .env
            .registry
            .mutation_type
            .as_ref()
            .and_then(|mutation_name| self.0.types.get(mutation_name))
            .and_then(Type::as_object)
            .ok_or_else(|| ServerError::new("Mutation root not found", None))
    }

    fn subscription_root(&self) -> ServerResult<&Subscription> {
        self.0
            .env
            .registry
            .subscription_type
            .as_ref()
            .and_then(|subscription_name| self.0.types.get(subscription_name))
            .and_then(Type::as_subscription)
            .ok_or_else(|| ServerError::new("Subscription root not found", None))
    }

    /// Returns SDL(Schema Definition Language) of this schema.
    pub fn sdl(&self) -> String {
        self.0.env.registry.export_sdl(Default::default())
    }

    /// Returns SDL(Schema Definition Language) of this schema with options.
    pub fn sdl_with_options(&self, options: SDLExportOptions) -> String {
        self.0.env.registry.export_sdl(options)
    }

    async fn execute_once(&self, env: QueryEnv, root_value: &FieldValue<'static>) -> Response {
        // execute
        let ctx = env.create_context(&self.0.env, None, &env.operation.node.selection_set);
        let res = match &env.operation.node.ty {
            OperationType::Query => {
                async move { self.query_root() }
                    .and_then(|query_root| {
                        resolve_container(self, query_root, &ctx, root_value, false)
                    })
                    .await
            }
            OperationType::Mutation => {
                async move { self.mutation_root() }
                    .and_then(|query_root| {
                        resolve_container(self, query_root, &ctx, root_value, true)
                    })
                    .await
            }
            OperationType::Subscription => Err(ServerError::new(
                "Subscriptions are not supported on this transport.",
                None,
            )),
        };

        let mut resp = match res {
            Ok(value) => Response::new(value.unwrap_or_default()),
            Err(err) => Response::from_errors(vec![err]),
        }
        .http_headers(std::mem::take(&mut *env.http_headers.lock().unwrap()));

        resp.errors
            .extend(std::mem::take(&mut *env.errors.lock().unwrap()));
        resp
    }

    /// Execute a GraphQL query.
    pub async fn execute(&self, request: impl Into<DynamicRequest>) -> Response {
        let request = request.into();
        let extensions = self.create_extensions(Default::default());
        let request_fut = {
            let extensions = extensions.clone();
            async move {
                match prepare_request(
                    extensions,
                    request.inner,
                    Default::default(),
                    &self.0.env.registry,
                    self.0.validation_mode,
                    self.0.recursive_depth,
                    self.0.complexity,
                    self.0.depth,
                )
                .await
                {
                    Ok((env, cache_control)) => {
                        let fut = async {
                            self.execute_once(env.clone(), &request.root_value)
                                .await
                                .cache_control(cache_control)
                        };
                        futures_util::pin_mut!(fut);
                        env.extensions
                            .execute(env.operation_name.as_deref(), &mut fut)
                            .await
                    }
                    Err(errors) => Response::from_errors(errors),
                }
            }
        };
        futures_util::pin_mut!(request_fut);
        extensions.request(&mut request_fut).await
    }

    /// Execute a GraphQL subscription with session data.
    pub fn execute_stream_with_session_data(
        &self,
        request: impl Into<DynamicRequest>,
        session_data: Arc<Data>,
    ) -> impl Stream<Item = Response> + Send + Unpin {
        let schema = self.clone();
        let request = request.into();
        let extensions = self.create_extensions(session_data.clone());

        let stream = {
            let extensions = extensions.clone();

            async_stream::stream! {
                let subscription = match schema.subscription_root() {
                    Ok(subscription) => subscription,
                    Err(err) => {
                        yield Response::from_errors(vec![err]);
                        return;
                    }
                };

                let (env, _) = match prepare_request(
                    extensions,
                    request.inner,
                    session_data,
                    &schema.0.env.registry,
                    schema.0.validation_mode,
                    schema.0.recursive_depth,
                    schema.0.complexity,
                    schema.0.depth,
                )
                .await {
                    Ok(res) => res,
                    Err(errors) => {
                        yield Response::from_errors(errors);
                        return;
                    }
                };

                if env.operation.node.ty != OperationType::Subscription {
                    yield schema.execute_once(env, &request.root_value).await;
                    return;
                }

                let ctx = env.create_context(
                    &schema.0.env,
                    None,
                    &env.operation.node.selection_set,
                );
                let mut streams = Vec::new();
                subscription.collect_streams(&schema, &ctx, &mut streams, &request.root_value);

                let mut stream = futures_util::stream::select_all(streams);
                while let Some(resp) = stream.next().await {
                    yield resp;
                }
            }
        };
        extensions.subscribe(stream.boxed())
    }

    /// Execute a GraphQL subscription.
    pub fn execute_stream(
        &self,
        request: impl Into<DynamicRequest>,
    ) -> impl Stream<Item = Response> + Send + Unpin {
        self.execute_stream_with_session_data(request, Default::default())
    }
}

#[async_trait::async_trait]
impl Executor for Schema {
    async fn execute(&self, request: Request) -> Response {
        Schema::execute(self, request).await
    }

    fn execute_stream(
        &self,
        request: Request,
        session_data: Option<Arc<Data>>,
    ) -> BoxStream<'static, Response> {
        Schema::execute_stream_with_session_data(self, request, session_data.unwrap_or_default())
            .boxed()
    }
}

fn update_interface_possible_types(types: &mut IndexMap<String, Type>, registry: &mut Registry) {
    let mut interfaces = registry
        .types
        .values_mut()
        .filter_map(|ty| match ty {
            MetaType::Interface {
                ref name,
                possible_types,
                ..
            } => Some((name, possible_types)),
            _ => None,
        })
        .collect::<HashMap<_, _>>();

    let objs = types.values().filter_map(|ty| match ty {
        Type::Object(obj) => Some((&obj.name, &obj.implements)),
        _ => None,
    });

    for (obj_name, implements) in objs {
        for interface in implements {
            if let Some(possible_types) = interfaces.get_mut(interface) {
                possible_types.insert(obj_name.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_graphql_parser::{types::ExecutableDocument, Pos};
    use async_graphql_value::Variables;
    use futures_util::{stream::BoxStream, StreamExt};
    use tokio::sync::Mutex;

    use crate::{
        dynamic::{DynamicRequestExt, *},
        extensions::*,
        value, PathSegment, Request, Response, ServerError, ServerResult, ValidationResult, Value,
    };

    #[tokio::test]
    async fn basic_query() {
        let myobj = Object::new("MyObj")
            .field(Field::new("a", TypeRef::named(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(123))) })
            }))
            .field(Field::new("b", TypeRef::named(TypeRef::STRING), |_| {
                FieldFuture::new(async { Ok(Some(Value::from("abc"))) })
            }));

        let query = Object::new("Query")
            .field(Field::new("value", TypeRef::named(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(100))) })
            }))
            .field(Field::new(
                "valueObj",
                TypeRef::named_nn(myobj.type_name()),
                |_| FieldFuture::new(async { Ok(Some(FieldValue::NULL)) }),
            ));
        let schema = Schema::build("Query", None, None)
            .register(query)
            .register(myobj)
            .finish()
            .unwrap();

        assert_eq!(
            schema
                .execute("{ value valueObj { a b } }")
                .await
                .into_result()
                .unwrap()
                .data,
            value!({
                "value": 100,
                "valueObj": {
                    "a": 123,
                    "b": "abc",
                }
            })
        );
    }

    #[tokio::test]
    async fn root_value() {
        let query =
            Object::new("Query").field(Field::new("value", TypeRef::named(TypeRef::INT), |ctx| {
                FieldFuture::new(async {
                    Ok(Some(Value::Number(
                        (*ctx.parent_value.try_downcast_ref::<i32>()?).into(),
                    )))
                })
            }));

        let schema = Schema::build("Query", None, None)
            .register(query)
            .finish()
            .unwrap();
        assert_eq!(
            schema
                .execute("{ value }".root_value(FieldValue::owned_any(100)))
                .await
                .into_result()
                .unwrap()
                .data,
            value!({ "value": 100, })
        );
    }

    #[tokio::test]
    async fn field_alias() {
        let query =
            Object::new("Query").field(Field::new("value", TypeRef::named(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(100))) })
            }));
        let schema = Schema::build("Query", None, None)
            .register(query)
            .finish()
            .unwrap();

        assert_eq!(
            schema
                .execute("{ a: value }")
                .await
                .into_result()
                .unwrap()
                .data,
            value!({
                "a": 100,
            })
        );
    }

    #[tokio::test]
    async fn fragment_spread() {
        let myobj = Object::new("MyObj")
            .field(Field::new("a", TypeRef::named(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(123))) })
            }))
            .field(Field::new("b", TypeRef::named(TypeRef::STRING), |_| {
                FieldFuture::new(async { Ok(Some(Value::from("abc"))) })
            }));

        let query = Object::new("Query").field(Field::new(
            "valueObj",
            TypeRef::named_nn(myobj.type_name()),
            |_| FieldFuture::new(async { Ok(Some(Value::Null)) }),
        ));
        let schema = Schema::build("Query", None, None)
            .register(query)
            .register(myobj)
            .finish()
            .unwrap();

        let query = r#"
            fragment A on MyObj {
                a b
            }

            { valueObj { ... A } }
            "#;

        assert_eq!(
            schema.execute(query).await.into_result().unwrap().data,
            value!({
                "valueObj": {
                    "a": 123,
                    "b": "abc",
                }
            })
        );
    }

    #[tokio::test]
    async fn inline_fragment() {
        let myobj = Object::new("MyObj")
            .field(Field::new("a", TypeRef::named(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(123))) })
            }))
            .field(Field::new("b", TypeRef::named(TypeRef::STRING), |_| {
                FieldFuture::new(async { Ok(Some(Value::from("abc"))) })
            }));

        let query = Object::new("Query").field(Field::new(
            "valueObj",
            TypeRef::named_nn(myobj.type_name()),
            |_| FieldFuture::new(async { Ok(Some(FieldValue::NULL)) }),
        ));
        let schema = Schema::build("Query", None, None)
            .register(query)
            .register(myobj)
            .finish()
            .unwrap();

        let query = r#"
            {
                valueObj {
                     ... on MyObj { a }
                     ... { b }
                }
            }
            "#;

        assert_eq!(
            schema.execute(query).await.into_result().unwrap().data,
            value!({
                "valueObj": {
                    "a": 123,
                    "b": "abc",
                }
            })
        );
    }

    #[tokio::test]
    async fn non_null() {
        let query = Object::new("Query")
            .field(Field::new(
                "valueA",
                TypeRef::named_nn(TypeRef::INT),
                |_| FieldFuture::new(async { Ok(FieldValue::none()) }),
            ))
            .field(Field::new(
                "valueB",
                TypeRef::named_nn(TypeRef::INT),
                |_| FieldFuture::new(async { Ok(Some(Value::from(100))) }),
            ))
            .field(Field::new("valueC", TypeRef::named(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(FieldValue::none()) })
            }))
            .field(Field::new("valueD", TypeRef::named(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(200))) })
            }));
        let schema = Schema::build("Query", None, None)
            .register(query)
            .finish()
            .unwrap();

        assert_eq!(
            schema
                .execute("{ valueA }")
                .await
                .into_result()
                .unwrap_err(),
            vec![ServerError {
                message: "internal: non-null types require a return value".to_owned(),
                source: None,
                locations: vec![Pos { column: 3, line: 1 }],
                path: vec![PathSegment::Field("valueA".to_owned())],
                extensions: None,
            }]
        );

        assert_eq!(
            schema
                .execute("{ valueB }")
                .await
                .into_result()
                .unwrap()
                .data,
            value!({
                "valueB": 100
            })
        );

        assert_eq!(
            schema
                .execute("{ valueC valueD }")
                .await
                .into_result()
                .unwrap()
                .data,
            value!({
                "valueC": null,
                "valueD": 200,
            })
        );
    }

    #[tokio::test]
    async fn list() {
        let query = Object::new("Query")
            .field(Field::new(
                "values",
                TypeRef::named_nn_list_nn(TypeRef::INT),
                |_| {
                    FieldFuture::new(async {
                        Ok(Some(vec![Value::from(3), Value::from(6), Value::from(9)]))
                    })
                },
            ))
            .field(Field::new(
                "values2",
                TypeRef::named_nn_list_nn(TypeRef::INT),
                |_| {
                    FieldFuture::new(async {
                        Ok(Some(Value::List(vec![
                            Value::from(3),
                            Value::from(6),
                            Value::from(9),
                        ])))
                    })
                },
            ))
            .field(Field::new(
                "values3",
                TypeRef::named_nn_list(TypeRef::INT),
                |_| FieldFuture::new(async { Ok(None::<Vec<Value>>) }),
            ));
        let schema = Schema::build("Query", None, None)
            .register(query)
            .finish()
            .unwrap();

        assert_eq!(
            schema
                .execute("{ values values2 values3 }")
                .await
                .into_result()
                .unwrap()
                .data,
            value!({
                "values": [3, 6, 9],
                "values2": [3, 6, 9],
                "values3": null,
            })
        );
    }

    #[tokio::test]
    async fn extensions() {
        struct MyExtensionImpl {
            calls: Arc<Mutex<Vec<&'static str>>>,
        }

        #[async_trait::async_trait]
        #[allow(unused_variables)]
        impl Extension for MyExtensionImpl {
            async fn request(&self, ctx: &ExtensionContext<'_>, next: NextRequest<'_>) -> Response {
                self.calls.lock().await.push("request_start");
                let res = next.run(ctx).await;
                self.calls.lock().await.push("request_end");
                res
            }

            fn subscribe<'s>(
                &self,
                ctx: &ExtensionContext<'_>,
                mut stream: BoxStream<'s, Response>,
                next: NextSubscribe<'_>,
            ) -> BoxStream<'s, Response> {
                let calls = self.calls.clone();
                next.run(
                    ctx,
                    Box::pin(async_stream::stream! {
                        calls.lock().await.push("subscribe_start");
                        while let Some(item) = stream.next().await {
                            yield item;
                        }
                        calls.lock().await.push("subscribe_end");
                    }),
                )
            }

            async fn prepare_request(
                &self,
                ctx: &ExtensionContext<'_>,
                request: Request,
                next: NextPrepareRequest<'_>,
            ) -> ServerResult<Request> {
                self.calls.lock().await.push("prepare_request_start");
                let res = next.run(ctx, request).await;
                self.calls.lock().await.push("prepare_request_end");
                res
            }

            async fn parse_query(
                &self,
                ctx: &ExtensionContext<'_>,
                query: &str,
                variables: &Variables,
                next: NextParseQuery<'_>,
            ) -> ServerResult<ExecutableDocument> {
                self.calls.lock().await.push("parse_query_start");
                let res = next.run(ctx, query, variables).await;
                self.calls.lock().await.push("parse_query_end");
                res
            }

            async fn validation(
                &self,
                ctx: &ExtensionContext<'_>,
                next: NextValidation<'_>,
            ) -> Result<ValidationResult, Vec<ServerError>> {
                self.calls.lock().await.push("validation_start");
                let res = next.run(ctx).await;
                self.calls.lock().await.push("validation_end");
                res
            }

            async fn execute(
                &self,
                ctx: &ExtensionContext<'_>,
                operation_name: Option<&str>,
                next: NextExecute<'_>,
            ) -> Response {
                assert_eq!(operation_name, Some("Abc"));
                self.calls.lock().await.push("execute_start");
                let res = next.run(ctx, operation_name).await;
                self.calls.lock().await.push("execute_end");
                res
            }

            async fn resolve(
                &self,
                ctx: &ExtensionContext<'_>,
                info: ResolveInfo<'_>,
                next: NextResolve<'_>,
            ) -> ServerResult<Option<Value>> {
                self.calls.lock().await.push("resolve_start");
                let res = next.run(ctx, info).await;
                self.calls.lock().await.push("resolve_end");
                res
            }
        }

        struct MyExtension {
            calls: Arc<Mutex<Vec<&'static str>>>,
        }

        impl ExtensionFactory for MyExtension {
            fn create(&self) -> Arc<dyn Extension> {
                Arc::new(MyExtensionImpl {
                    calls: self.calls.clone(),
                })
            }
        }

        {
            let query = Object::new("Query")
                .field(Field::new(
                    "value1",
                    TypeRef::named_nn(TypeRef::INT),
                    |_| FieldFuture::new(async { Ok(Some(Value::from(10))) }),
                ))
                .field(Field::new(
                    "value2",
                    TypeRef::named_nn(TypeRef::INT),
                    |_| FieldFuture::new(async { Ok(Some(Value::from(10))) }),
                ));

            let calls: Arc<Mutex<Vec<&'static str>>> = Default::default();
            let schema = Schema::build(query.type_name(), None, None)
                .register(query)
                .extension(MyExtension {
                    calls: calls.clone(),
                })
                .finish()
                .unwrap();

            let _ = schema
                .execute("query Abc { value1 value2 }")
                .await
                .into_result()
                .unwrap();
            let calls = calls.lock().await;
            assert_eq!(
                &*calls,
                &vec![
                    "request_start",
                    "prepare_request_start",
                    "prepare_request_end",
                    "parse_query_start",
                    "parse_query_end",
                    "validation_start",
                    "validation_end",
                    "execute_start",
                    "resolve_start",
                    "resolve_end",
                    "resolve_start",
                    "resolve_end",
                    "execute_end",
                    "request_end",
                ]
            );
        }

        {
            let query = Object::new("Query").field(Field::new(
                "value1",
                TypeRef::named_nn(TypeRef::INT),
                |_| FieldFuture::new(async { Ok(Some(Value::from(10))) }),
            ));

            let subscription = Subscription::new("Subscription").field(SubscriptionField::new(
                "value",
                TypeRef::named_nn(TypeRef::INT),
                |_| {
                    SubscriptionFieldFuture::new(async {
                        Ok(futures_util::stream::iter([1, 2, 3])
                            .map(|value| Ok(Value::from(value))))
                    })
                },
            ));

            let calls: Arc<Mutex<Vec<&'static str>>> = Default::default();
            let schema = Schema::build(query.type_name(), None, Some(subscription.type_name()))
                .register(query)
                .register(subscription)
                .extension(MyExtension {
                    calls: calls.clone(),
                })
                .finish()
                .unwrap();

            let mut stream = schema.execute_stream("subscription Abc { value }");
            while stream.next().await.is_some() {}
            let calls = calls.lock().await;
            assert_eq!(
                &*calls,
                &vec![
                    "subscribe_start",
                    "prepare_request_start",
                    "prepare_request_end",
                    "parse_query_start",
                    "parse_query_end",
                    "validation_start",
                    "validation_end",
                    // push 1
                    "execute_start",
                    "resolve_start",
                    "resolve_end",
                    "execute_end",
                    // push 2
                    "execute_start",
                    "resolve_start",
                    "resolve_end",
                    "execute_end",
                    // push 3
                    "execute_start",
                    "resolve_start",
                    "resolve_end",
                    "execute_end",
                    // end
                    "subscribe_end",
                ]
            );
        }
    }

    #[tokio::test]
    async fn federation() {
        let user = Object::new("User")
            .field(Field::new(
                "name",
                TypeRef::named_nn(TypeRef::STRING),
                |_| FieldFuture::new(async { Ok(Some(FieldValue::value("test"))) }),
            ))
            .key("name");

        let query =
            Object::new("Query").field(Field::new("value", TypeRef::named(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(100))) })
            }));

        let schema = Schema::build("Query", None, None)
            .register(query)
            .register(user)
            .entity_resolver(|ctx| {
                FieldFuture::new(async move {
                    let representations = ctx.args.try_get("representations")?.list()?;
                    let mut values = Vec::new();

                    for item in representations.iter() {
                        let item = item.object()?;
                        let typename = item
                            .try_get("__typename")
                            .and_then(|value| value.string())?;

                        if typename == "User" {
                            values.push(FieldValue::borrowed_any(&()).with_type("User"));
                        }
                    }

                    Ok(Some(FieldValue::list(values)))
                })
            })
            .finish()
            .unwrap();

        assert_eq!(
            schema
                .execute(
                    r#"
                {
                    _entities(representations: [{__typename: "User", name: "test"}]) {
                        __typename
                        ... on User {
                            name
                        }
                    }
                }
                "#
                )
                .await
                .into_result()
                .unwrap()
                .data,
            value!({
                "_entities": [{
                    "__typename": "User",
                    "name": "test",
                }],
            })
        );
    }
}
