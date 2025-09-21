use std::{
    any::Any,
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::Arc,
};

use async_graphql_parser::types::ExecutableDocument;
use futures_util::stream::{self, BoxStream, FuturesOrdered, Stream, StreamExt};

use crate::{
    BatchRequest, BatchResponse, CacheControl, ContainerType, ContextBase, EmptyMutation,
    EmptySubscription, Executor, InputType, ObjectType, OutputTypeMarker, QueryEnv,
    Request, Response, ServerError, ServerResult, SubscriptionType, Variables,
    context::{Data, QueryEnvInner},
    custom_directive::CustomDirectiveFactory,
    extensions::{ExtensionFactory, Extensions},
    parser::{
        Positioned, parse_query,
        types::{Directive, DocumentOperations, OperationType, Selection, SelectionSet},
    },
    registry::{Registry, SDLExportOptions},
    resolver_utils::{resolve_container, resolve_container_serial},
    subscription::collect_subscription_streams,
    types::QueryRoot,
    validation::{ValidationMode, check_rules},
};

/// Introspection mode
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum IntrospectionMode {
    /// Introspection only
    IntrospectionOnly,
    /// Enables introspection
    #[default]
    Enabled,
    /// Disables introspection
    Disabled,
}

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
    recursive_depth: usize,
    max_directives: Option<usize>,
    extensions: Vec<Box<dyn ExtensionFactory>>,
    custom_directives: HashMap<String, Box<dyn CustomDirectiveFactory>>,
}

impl<Query, Mutation, Subscription> SchemaBuilder<Query, Mutation, Subscription> {
    /// Manually register a input type in the schema.
    ///
    /// You can use this function to register schema types that are not directly
    /// referenced.
    #[must_use]
    pub fn register_input_type<T: InputType>(mut self) -> Self {
        T::create_type_info(&mut self.registry);
        self
    }

    /// Manually register a output type in the schema.
    ///
    /// You can use this function to register schema types that are not directly
    /// referenced.
    #[must_use]
    pub fn register_output_type<T: OutputTypeMarker>(mut self) -> Self {
        T::create_type_info(&mut self.registry);
        self
    }

    /// Disable introspection queries.
    #[must_use]
    pub fn disable_introspection(mut self) -> Self {
        self.registry.introspection_mode = IntrospectionMode::Disabled;
        self
    }

    /// Only process introspection queries, everything else is processed as an
    /// error.
    #[must_use]
    pub fn introspection_only(mut self) -> Self {
        self.registry.introspection_mode = IntrospectionMode::IntrospectionOnly;
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

    /// Set the maximum number of directives on a single field. (default: no
    /// limit)
    pub fn limit_directives(mut self, max_directives: usize) -> Self {
        self.max_directives = Some(max_directives);
        self
    }

    /// Add an extension to the schema.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use async_graphql::*;
    ///
    /// struct Query;
    ///
    /// #[Object]
    /// impl Query {
    ///     async fn value(&self) -> i32 {
    ///         100
    ///     }
    /// }
    ///
    /// let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
    ///     .extension(extensions::Logger)
    ///     .finish();
    /// ```
    #[must_use]
    pub fn extension(mut self, extension: impl ExtensionFactory) -> Self {
        self.extensions.push(Box::new(extension));
        self
    }

    /// Add a global data that can be accessed in the `Schema`. You access it
    /// with `Context::data`.
    #[must_use]
    pub fn data<D: Any + Send + Sync>(mut self, data: D) -> Self {
        self.data.insert(data);
        self
    }

    /// Set the validation mode, default is `ValidationMode::Strict`.
    #[must_use]
    pub fn validation_mode(mut self, validation_mode: ValidationMode) -> Self {
        self.validation_mode = validation_mode;
        self
    }

    /// Enable federation, which is automatically enabled if the Query has least
    /// one entity definition.
    #[must_use]
    pub fn enable_federation(mut self) -> Self {
        self.registry.enable_federation = true;
        self
    }

    /// Make the Federation SDL include subscriptions.
    ///
    /// Note: Not included by default, in order to be compatible with Apollo
    /// Server.
    #[must_use]
    pub fn enable_subscription_in_federation(mut self) -> Self {
        self.registry.federation_subscription = true;
        self
    }

    /// Override the name of the specified input type.
    #[must_use]
    pub fn override_input_type_description<T: InputType>(mut self, desc: &'static str) -> Self {
        self.registry.set_description(&*T::type_name(), desc);
        self
    }

    /// Override the name of the specified output type.
    #[must_use]
    pub fn override_output_type_description<T: OutputTypeMarker>(
        mut self,
        desc: &'static str,
    ) -> Self {
        self.registry.set_description(&*T::type_name(), desc);
        self
    }

    /// Register a custom directive.
    ///
    /// # Panics
    ///
    /// Panics if the directive with the same name is already registered.
    #[must_use]
    pub fn directive<T: CustomDirectiveFactory>(mut self, directive: T) -> Self {
        let name = directive.name();
        let instance = Box::new(directive);

        instance.register(&mut self.registry);

        if name == "skip"
            || name == "include"
            || self
                .custom_directives
                .insert(name.clone().into(), instance)
                .is_some()
        {
            panic!("Directive `{}` already exists", name);
        }

        self
    }

    /// Disable field suggestions.
    #[must_use]
    pub fn disable_suggestions(mut self) -> Self {
        self.registry.enable_suggestions = false;
        self
    }

    /// Make all fields sorted on introspection queries.
    pub fn with_sorted_fields(mut self) -> Self {
        use crate::registry::MetaType;
        for ty in self.registry.types.values_mut() {
            match ty {
                MetaType::Object { fields, .. } | MetaType::Interface { fields, .. } => {
                    fields.sort_keys();
                }
                MetaType::InputObject { input_fields, .. } => {
                    input_fields.sort_keys();
                }
                MetaType::Scalar { .. } | MetaType::Enum { .. } | MetaType::Union { .. } => {
                    // have no fields
                }
            }
        }
        self
    }

    /// Make all enum variants sorted on introspection queries.
    pub fn with_sorted_enums(mut self) -> Self {
        use crate::registry::MetaType;
        for ty in &mut self.registry.types.values_mut() {
            if let MetaType::Enum { enum_values, .. } = ty {
                enum_values.sort_keys();
            }
        }
        self
    }

    /// Consumes this builder and returns a schema.
    pub fn finish(mut self) -> Schema<Query, Mutation, Subscription> {
        // federation
        if self.registry.enable_federation || self.registry.has_entities() {
            self.registry.create_federation_types();
        }

        Schema(Arc::new(SchemaInner {
            validation_mode: self.validation_mode,
            query: self.query,
            mutation: self.mutation,
            subscription: self.subscription,
            complexity: self.complexity,
            depth: self.depth,
            recursive_depth: self.recursive_depth,
            max_directives: self.max_directives,
            extensions: self.extensions,
            env: SchemaEnv(Arc::new(SchemaEnvInner {
                registry: self.registry,
                data: self.data,
                custom_directives: self.custom_directives,
            })),
        }))
    }
}

#[doc(hidden)]
pub struct SchemaEnvInner {
    pub registry: Registry,
    pub data: Data,
    pub custom_directives: HashMap<String, Box<dyn CustomDirectiveFactory>>,
}

#[doc(hidden)]
#[derive(Clone)]
pub struct SchemaEnv(pub(crate) Arc<SchemaEnvInner>);

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
    pub(crate) recursive_depth: usize,
    pub(crate) max_directives: Option<usize>,
    pub(crate) extensions: Vec<Box<dyn ExtensionFactory>>,
    pub(crate) env: SchemaEnv,
}

/// GraphQL schema.
///
/// Cloning a schema is cheap, so it can be easily shared.
pub struct Schema<Query, Mutation, Subscription>(
    pub(crate) Arc<SchemaInner<Query, Mutation, Subscription>>,
);

impl<Query, Mutation, Subscription> Clone for Schema<Query, Mutation, Subscription> {
    fn clone(&self) -> Self {
        Schema(self.0.clone())
    }
}

impl<Query, Mutation, Subscription> Default for Schema<Query, Mutation, Subscription>
where
    Query: Default + ObjectType + 'static,
    Mutation: Default + ObjectType + 'static,
    Subscription: Default + SubscriptionType + 'static,
{
    fn default() -> Self {
        Schema::new(
            Query::default(),
            Mutation::default(),
            Subscription::default(),
        )
    }
}

impl<Query, Mutation, Subscription> Schema<Query, Mutation, Subscription>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
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
        Self::build_with_ignore_name_conflicts(query, mutation, subscription, [] as [&str; 0])
    }

    /// Create a schema builder and specifies a list to ignore type conflict
    /// detection.
    ///
    /// NOTE: It is not recommended to use it unless you know what it does.
    #[must_use]
    pub fn build_with_ignore_name_conflicts<I, T>(
        query: Query,
        mutation: Mutation,
        subscription: Subscription,
        ignore_name_conflicts: I,
    ) -> SchemaBuilder<Query, Mutation, Subscription>
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        SchemaBuilder {
            validation_mode: ValidationMode::Strict,
            query: QueryRoot { inner: query },
            mutation,
            subscription,
            registry: Self::create_registry(
                ignore_name_conflicts.into_iter().map(Into::into).collect(),
            ),
            data: Default::default(),
            complexity: None,
            depth: None,
            recursive_depth: 32,
            max_directives: None,
            extensions: Default::default(),
            custom_directives: Default::default(),
        }
    }

    pub(crate) fn create_registry(ignore_name_conflicts: HashSet<String>) -> Registry {
        let mut registry = Registry {
            types: Default::default(),
            directives: Default::default(),
            implements: Default::default(),
            query_type: <Query as OutputTypeMarker>::type_name().to_string(),
            mutation_type: if Option::<Mutation>::is_empty(&None) {
                None
            } else {
                Some(<Mutation as OutputTypeMarker>::type_name().to_string())
            },
            subscription_type: if Subscription::is_empty() {
                None
            } else {
                Some(Subscription::type_name().to_string())
            },
            introspection_mode: IntrospectionMode::Enabled,
            enable_federation: false,
            federation_subscription: false,
            ignore_name_conflicts,
            enable_suggestions: true,
        };
        registry.add_system_types();
        <QueryRoot<Query> as OutputTypeMarker>::create_type_info(&mut registry);

        if !Option::<Mutation>::is_empty(&None) {
            <Mutation as OutputTypeMarker>::create_type_info(&mut registry);
        }
        if !Subscription::is_empty() {
            Subscription::create_type_info(&mut registry);
        }

        registry.remove_unused_types();
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

    #[inline]
    #[allow(unused)]
    pub(crate) fn registry(&self) -> &Registry {
        &self.0.env.registry
    }

    /// Returns SDL(Schema Definition Language) of this schema.
    pub fn sdl(&self) -> String {
        self.0.env.registry.export_sdl(Default::default())
    }

    /// Returns SDL(Schema Definition Language) of this schema with options.
    pub fn sdl_with_options(&self, options: SDLExportOptions) -> String {
        self.0.env.registry.export_sdl(options)
    }

    /// Get all names in this schema
    ///
    /// Maybe you want to serialize a custom binary protocol. In order to
    /// minimize message size, a dictionary is usually used to compress type
    /// names, field names, directive names, and parameter names. This function
    /// gets all the names, so you can create this dictionary.
    pub fn names(&self) -> Vec<String> {
        self.0.env.registry.names()
    }

    fn create_extensions(&self, session_data: Arc<Data>) -> Extensions {
        Extensions::new(
            self.0.extensions.iter().map(|f| f.create()),
            self.0.env.clone(),
            session_data,
        )
    }

    async fn execute_once(&self, env: QueryEnv, execute_data: Option<&Data>) -> Response {
        // execute
        let ctx = ContextBase {
            path_node: None,
            is_for_introspection: false,
            item: &env.operation.node.selection_set,
            schema_env: &self.0.env,
            query_env: &env,
            execute_data,
        };

        let res = match &env.operation.node.ty {
            #[cfg(feature = "boxed-trait")]
            OperationType::Query => resolve_container(&ctx, &self.0.query, &self.0.query).await,
            #[cfg(not(feature = "boxed-trait"))]
            OperationType::Query => resolve_container(&ctx, &self.0.query).await,
            OperationType::Mutation => {
                if self.0.env.registry.introspection_mode == IntrospectionMode::IntrospectionOnly
                    || env.introspection_mode == IntrospectionMode::IntrospectionOnly
                {
                    #[cfg(feature = "boxed-trait")]
                    {
                        resolve_container_serial(&ctx, &EmptyMutation, &EmptyMutation).await
                    }
                    #[cfg(not(feature = "boxed-trait"))]
                    {
                        resolve_container_serial(&ctx, &EmptyMutation).await
                    }
                } else {
                    #[cfg(feature = "boxed-trait")]
                    {
                        resolve_container_serial(&ctx, &self.0.mutation, &self.0.mutation).await
                    }
                    #[cfg(not(feature = "boxed-trait"))]
                    {
                        resolve_container_serial(&ctx, &self.0.mutation).await
                    }
                }
            }
            OperationType::Subscription => Err(ServerError::new(
                "Subscriptions are not supported on this transport.",
                None,
            )),
        };

        let mut resp = match res {
            Ok(value) => Response::new(value),
            Err(err) => Response::from_errors(vec![err]),
        }
        .http_headers(std::mem::take(&mut *env.http_headers.lock().unwrap()));

        resp.errors
            .extend(std::mem::take(&mut *env.errors.lock().unwrap()));
        resp
    }

    /// Execute a GraphQL query.
    pub async fn execute(&self, request: impl Into<Request>) -> Response {
        let request = request.into();
        let extensions = self.create_extensions(Default::default());
        let request_fut = {
            let extensions = extensions.clone();
            async move {
                match prepare_request(
                    extensions,
                    request,
                    Default::default(),
                    &self.0.env.registry,
                    self.0.validation_mode,
                    self.0.recursive_depth,
                    self.0.max_directives,
                    self.0.complexity,
                    self.0.depth,
                )
                .await
                {
                    Ok((env, cache_control)) => {
                        let f = |execute_data: Option<Data>| {
                            let env = env.clone();
                            async move {
                                self.execute_once(env, execute_data.as_ref())
                                    .await
                                    .cache_control(cache_control)
                            }
                        };
                        env.extensions
                            .execute(env.operation_name.as_deref(), f)
                            .await
                    }
                    Err(errors) => Response::from_errors(errors),
                }
            }
        };
        futures_util::pin_mut!(request_fut);
        extensions.request(&mut request_fut).await
    }

    /// Execute a GraphQL batch query.
    pub async fn execute_batch(&self, batch_request: BatchRequest) -> BatchResponse {
        match batch_request {
            BatchRequest::Single(request) => BatchResponse::Single(self.execute(request).await),
            BatchRequest::Batch(requests) => BatchResponse::Batch(
                FuturesOrdered::from_iter(
                    requests.into_iter().map(|request| self.execute(request)),
                )
                .collect()
                .await,
            ),
        }
    }

    /// Execute a GraphQL subscription with session data.
    pub fn execute_stream_with_session_data(
        &self,
        request: impl Into<Request>,
        session_data: Arc<Data>,
    ) -> impl Stream<Item = Response> + Send + Unpin + 'static {
        let schema = self.clone();
        let request = request.into();
        let extensions = self.create_extensions(session_data.clone());

        let stream = futures_util::stream::StreamExt::boxed({
            let extensions = extensions.clone();
            let env = self.0.env.clone();
            async_stream::stream! {
                let (env, cache_control) = match prepare_request(
                        extensions, request, session_data, &env.registry,
                        schema.0.validation_mode, schema.0.recursive_depth,
                        schema.0.max_directives, schema.0.complexity, schema.0.depth
                ).await {
                    Ok(res) => res,
                    Err(errors) => {
                        yield Response::from_errors(errors);
                        return;
                    }
                };

                if env.operation.node.ty != OperationType::Subscription {
                    let f = |execute_data: Option<Data>| {
                        let env = env.clone();
                        let schema = schema.clone();
                        async move {
                            schema.execute_once(env, execute_data.as_ref())
                                .await
                                .cache_control(cache_control)
                        }
                    };
                    yield env.extensions
                        .execute(env.operation_name.as_deref(), f)
                        .await
                        .cache_control(cache_control);
                    return;
                }

                let ctx = env.create_context(
                    &schema.0.env,
                    None,
                    &env.operation.node.selection_set,
                    None,
                );

                let mut streams = Vec::new();
                let collect_result = if schema.0.env.registry.introspection_mode
                    == IntrospectionMode::IntrospectionOnly
                    || env.introspection_mode == IntrospectionMode::IntrospectionOnly
                {
                    collect_subscription_streams(&ctx, &EmptySubscription, &mut streams)
                } else {
                    collect_subscription_streams(&ctx, &schema.0.subscription, &mut streams)
                };
                if let Err(err) = collect_result {
                    yield Response::from_errors(vec![err]);
                }

                let mut stream = stream::select_all(streams);
                while let Some(resp) = stream.next().await {
                    yield resp;
                }
            }
        });
        extensions.subscribe(stream)
    }

    /// Execute a GraphQL subscription.
    pub fn execute_stream(
        &self,
        request: impl Into<Request>,
    ) -> impl Stream<Item = Response> + Send + Unpin {
        self.execute_stream_with_session_data(request, Default::default())
    }
}

#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl<Query, Mutation, Subscription> Executor for Schema<Query, Mutation, Subscription>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
{
    async fn execute(&self, request: Request) -> Response {
        Schema::execute(self, request).await
    }

    fn execute_stream(
        &self,
        request: Request,
        session_data: Option<Arc<Data>>,
    ) -> BoxStream<'static, Response> {
        Schema::execute_stream_with_session_data(&self, request, session_data.unwrap_or_default())
            .boxed()
    }
}

fn check_max_directives(doc: &ExecutableDocument, max_directives: usize) -> ServerResult<()> {
    fn check_selection_set(
        doc: &ExecutableDocument,
        selection_set: &Positioned<SelectionSet>,
        limit_directives: usize,
    ) -> ServerResult<()> {
        for selection in &selection_set.node.items {
            match &selection.node {
                Selection::Field(field) => {
                    if field.node.directives.len() > limit_directives {
                        return Err(ServerError::new(
                            format!(
                                "The number of directives on the field `{}` cannot be greater than `{}`",
                                field.node.name.node, limit_directives
                            ),
                            Some(field.pos),
                        ));
                    }
                    check_selection_set(doc, &field.node.selection_set, limit_directives)?;
                }
                Selection::FragmentSpread(fragment_spread) => {
                    if let Some(fragment) =
                        doc.fragments.get(&fragment_spread.node.fragment_name.node)
                    {
                        check_selection_set(doc, &fragment.node.selection_set, limit_directives)?;
                    }
                }
                Selection::InlineFragment(inline_fragment) => {
                    check_selection_set(
                        doc,
                        &inline_fragment.node.selection_set,
                        limit_directives,
                    )?;
                }
            }
        }

        Ok(())
    }

    for (_, operation) in doc.operations.iter() {
        check_selection_set(doc, &operation.node.selection_set, max_directives)?;
    }

    Ok(())
}

fn check_recursive_depth(doc: &ExecutableDocument, max_depth: usize) -> ServerResult<()> {
    fn check_selection_set(
        doc: &ExecutableDocument,
        selection_set: &Positioned<SelectionSet>,
        current_depth: usize,
        max_depth: usize,
    ) -> ServerResult<()> {
        if current_depth > max_depth {
            return Err(ServerError::new(
                format!(
                    "The recursion depth of the query cannot be greater than `{}`",
                    max_depth
                ),
                Some(selection_set.pos),
            ));
        }

        for selection in &selection_set.node.items {
            match &selection.node {
                Selection::Field(field) => {
                    if !field.node.selection_set.node.items.is_empty() {
                        check_selection_set(
                            doc,
                            &field.node.selection_set,
                            current_depth + 1,
                            max_depth,
                        )?;
                    }
                }
                Selection::FragmentSpread(fragment_spread) => {
                    if let Some(fragment) =
                        doc.fragments.get(&fragment_spread.node.fragment_name.node)
                    {
                        check_selection_set(
                            doc,
                            &fragment.node.selection_set,
                            current_depth + 1,
                            max_depth,
                        )?;
                    }
                }
                Selection::InlineFragment(inline_fragment) => {
                    check_selection_set(
                        doc,
                        &inline_fragment.node.selection_set,
                        current_depth + 1,
                        max_depth,
                    )?;
                }
            }
        }

        Ok(())
    }

    for (_, operation) in doc.operations.iter() {
        check_selection_set(doc, &operation.node.selection_set, 0, max_depth)?;
    }

    Ok(())
}

fn remove_skipped_selection(selection_set: &mut SelectionSet, variables: &Variables) {
    fn is_skipped(directives: &[Positioned<Directive>], variables: &Variables) -> bool {
        for directive in directives {
            let include = match &*directive.node.name.node {
                "skip" => false,
                "include" => true,
                _ => continue,
            };

            if let Some(condition_input) = directive.node.get_argument("if") {
                let value = condition_input
                    .node
                    .clone()
                    .into_const_with(|name| variables.get(&name).cloned().ok_or(()))
                    .unwrap_or_default();
                let value: bool = InputType::parse(Some(value)).unwrap_or_default();
                if include != value {
                    return true;
                }
            }
        }

        false
    }

    selection_set
        .items
        .retain(|selection| !is_skipped(selection.node.directives(), variables));

    for selection in &mut selection_set.items {
        selection.node.directives_mut().retain(|directive| {
            directive.node.name.node != "skip" && directive.node.name.node != "include"
        });
    }

    for selection in &mut selection_set.items {
        match &mut selection.node {
            Selection::Field(field) => {
                remove_skipped_selection(&mut field.node.selection_set.node, variables);
            }
            Selection::FragmentSpread(_) => {}
            Selection::InlineFragment(inline_fragment) => {
                remove_skipped_selection(&mut inline_fragment.node.selection_set.node, variables);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn prepare_request(
    mut extensions: Extensions,
    request: Request,
    session_data: Arc<Data>,
    registry: &Registry,
    validation_mode: ValidationMode,
    recursive_depth: usize,
    max_directives: Option<usize>,
    complexity: Option<usize>,
    depth: Option<usize>,
) -> Result<(QueryEnv, CacheControl), Vec<ServerError>> {
    let mut request = extensions.prepare_request(request).await?;
    let query_data = Arc::new(std::mem::take(&mut request.data));
    extensions.attach_query_data(query_data.clone());

    let mut document = {
        let query = &request.query;
        let parsed_doc = request.parsed_query.take();
        let fut_parse = async move {
            let doc = match parsed_doc {
                Some(parsed_doc) => parsed_doc,
                None => parse_query(query)?,
            };
            check_recursive_depth(&doc, recursive_depth)?;
            if let Some(max_directives) = max_directives {
                check_max_directives(&doc, max_directives)?;
            }
            Ok(doc)
        };
        futures_util::pin_mut!(fut_parse);
        extensions
            .parse_query(query, &request.variables, &mut fut_parse)
            .await?
    };

    // check rules
    let validation_result = {
        let validation_fut = async {
            check_rules(
                registry,
                &document,
                Some(&request.variables),
                validation_mode,
                complexity,
                depth,
            )
        };
        futures_util::pin_mut!(validation_fut);
        extensions.validation(&mut validation_fut).await?
    };

    let operation = if let Some(operation_name) = &request.operation_name {
        match document.operations {
            DocumentOperations::Single(_) => None,
            DocumentOperations::Multiple(mut operations) => operations
                .remove(operation_name.as_str())
                .map(|operation| (Some(operation_name.clone()), operation)),
        }
        .ok_or_else(|| {
            ServerError::new(
                format!(r#"Unknown operation named "{}""#, operation_name),
                None,
            )
        })
    } else {
        match document.operations {
            DocumentOperations::Single(operation) => Ok((None, operation)),
            DocumentOperations::Multiple(map) if map.len() == 1 => {
                let (operation_name, operation) = map.into_iter().next().unwrap();
                Ok((Some(operation_name.to_string()), operation))
            }
            DocumentOperations::Multiple(_) => Err(ServerError::new(
                "Operation name required in request.",
                None,
            )),
        }
    };

    let (operation_name, mut operation) = operation.map_err(|err| vec![err])?;

    // remove skipped fields
    for fragment in document.fragments.values_mut() {
        remove_skipped_selection(&mut fragment.node.selection_set.node, &request.variables);
    }
    remove_skipped_selection(&mut operation.node.selection_set.node, &request.variables);

    let env = QueryEnvInner {
        extensions,
        variables: request.variables,
        operation_name,
        operation,
        fragments: document.fragments,
        uploads: request.uploads,
        session_data,
        query_data,
        http_headers: Default::default(),
        introspection_mode: request.introspection_mode,
        errors: Default::default(),
    };
    Ok((QueryEnv::new(env), validation_result.cache_control))
}
