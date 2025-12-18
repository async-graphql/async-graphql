use std::{future::Future, pin::Pin, sync::Arc};

use futures_util::FutureExt;
use indexmap::IndexMap;

use crate::{
    Context, ContextBase, ContextSelectionSet, Error, Name, OutputType, OutputTypeMarker,
    ServerError, ServerResult, Value, extensions::ResolveInfo, parser::types::Selection,
};

/// Represents a GraphQL container object.
///
/// This helper trait allows the type to call `resolve_container` on itself in
/// its `OutputType::resolve` implementation.
#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
pub trait ContainerType: OutputType {
    /// This function returns true of type `EmptyMutation` only.
    #[doc(hidden)]
    fn is_empty(&self) -> bool {
        false
    }

    /// Resolves a field value and outputs it as a json value
    /// `async_graphql::Value`.
    ///
    /// If the field was not found returns None.
    #[cfg(feature = "boxed-trait")]
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>>;

    /// Resolves a field value and outputs it as a json value
    /// `async_graphql::Value`.
    ///
    /// If the field was not found returns None.
    #[cfg(not(feature = "boxed-trait"))]
    fn resolve_field(
        &self,
        ctx: &Context<'_>,
    ) -> impl Future<Output = ServerResult<Option<Value>>> + Send;

    /// Collect all the fields of the container that are queried in the
    /// selection set.
    ///
    /// Objects do not have to override this, but interfaces and unions must
    /// call it on their internal type.
    #[cfg(feature = "boxed-trait")]
    fn collect_all_fields<'a>(
        &'a self,
        ctx: &ContextSelectionSet<'a>,
        fields: &mut Fields<'a>,
    ) -> ServerResult<()>
    where
        Self: Send + Sync + Sized + OutputTypeMarker,
    {
        // Use the existing reference to self with lifetime 'a; avoid creating a Box
        // that would be dropped.
        fields.add_set(ctx, self, self)
    }

    /// Collect all the fields of the container that are queried in the
    /// selection set.
    ///
    /// Objects do not have to override this, but interfaces and unions must
    /// call it on their internal type.
    #[cfg(not(feature = "boxed-trait"))]
    fn collect_all_fields<'a>(
        &'a self,
        ctx: &ContextSelectionSet<'a>,
        fields: &mut Fields<'a>,
    ) -> ServerResult<()>
    where
        Self: Send + Sync + OutputTypeMarker,
    {
        fields.add_set(ctx, self)
    }

    /// Find the GraphQL entity with the given name from the parameter.
    ///
    /// Objects should override this in case they are the query root.
    #[cfg(feature = "boxed-trait")]
    async fn find_entity(&self, _: &Context<'_>, _: &Value) -> ServerResult<Option<Value>> {
        Ok(None)
    }

    /// Find the GraphQL entity with the given name from the parameter.
    ///
    /// Objects should override this in case they are the query root.
    #[cfg(not(feature = "boxed-trait"))]
    fn find_entity(
        &self,
        _: &Context<'_>,
        _params: &Value,
    ) -> impl Future<Output = ServerResult<Option<Value>>> + Send {
        async { Ok(None) }
    }
}

#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl<T: ContainerType + ?Sized> ContainerType for &T {
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
        T::resolve_field(*self, ctx).await
    }

    async fn find_entity(&self, ctx: &Context<'_>, params: &Value) -> ServerResult<Option<Value>> {
        T::find_entity(*self, ctx, params).await
    }
}

#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl<T: ContainerType + ?Sized> ContainerType for Arc<T> {
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
        T::resolve_field(self, ctx).await
    }

    async fn find_entity(&self, ctx: &Context<'_>, params: &Value) -> ServerResult<Option<Value>> {
        T::find_entity(self, ctx, params).await
    }
}

#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl<T: ContainerType + ?Sized> ContainerType for Box<T> {
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
        T::resolve_field(self, ctx).await
    }

    async fn find_entity(&self, ctx: &Context<'_>, params: &Value) -> ServerResult<Option<Value>> {
        T::find_entity(self, ctx, params).await
    }
}

#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl<T: ContainerType + OutputTypeMarker, E: Into<Error> + Send + Sync + Clone> ContainerType
    for Result<T, E>
{
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
        match self {
            Ok(value) => T::resolve_field(value, ctx).await,
            Err(err) => Err(ctx.set_error_path(err.clone().into().into_server_error(ctx.item.pos))),
        }
    }

    async fn find_entity(&self, ctx: &Context<'_>, params: &Value) -> ServerResult<Option<Value>> {
        match self {
            Ok(value) => T::find_entity(value, ctx, params).await,
            Err(err) => Err(ctx.set_error_path(err.clone().into().into_server_error(ctx.item.pos))),
        }
    }
}

#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl<T: ContainerType + OutputTypeMarker> ContainerType for Option<T> {
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
        match self {
            Some(value) => T::resolve_field(value, ctx).await,
            None => ServerResult::Ok(None),
        }
    }

    async fn find_entity(&self, ctx: &Context<'_>, params: &Value) -> ServerResult<Option<Value>> {
        match self {
            Some(value) => T::find_entity(value, ctx, params).await,
            None => ServerResult::Ok(None),
        }
    }
}

/// Resolve an container by executing each of the fields concurrently.
#[cfg(feature = "boxed-trait")]
pub async fn resolve_container<'a, T: OutputTypeMarker>(
    ctx: &ContextSelectionSet<'a>,
    root: &dyn ContainerType,
    root_marker: &T,
) -> ServerResult<Value> {
    resolve_container_inner(ctx, root, root_marker, true).await
}
/// Resolve an container by executing each of the fields concurrently.
#[cfg(not(feature = "boxed-trait"))]
pub async fn resolve_container<'a, T: ContainerType + OutputTypeMarker>(
    ctx: &ContextSelectionSet<'a>,
    root: &T,
) -> ServerResult<Value> {
    resolve_container_inner(ctx, root, true).await
}

/// Resolve an container by executing each of the fields serially.
#[cfg(feature = "boxed-trait")]
pub async fn resolve_container_serial<'a, T: OutputTypeMarker>(
    ctx: &ContextSelectionSet<'a>,
    root: &dyn ContainerType,
    root_marker: &T,
) -> ServerResult<Value> {
    resolve_container_inner(ctx, root, root_marker, false).await
}

/// Resolve an container by executing each of the fields serially.
#[cfg(not(feature = "boxed-trait"))]
pub async fn resolve_container_serial<'a, T: OutputTypeMarker + ContainerType>(
    ctx: &ContextSelectionSet<'a>,
    root: &T,
) -> ServerResult<Value> {
    resolve_container_inner(ctx, root, false).await
}

pub(crate) fn create_value_object(values: Vec<(Name, Value)>) -> Value {
    let mut map = IndexMap::new();
    for (name, value) in values {
        insert_value(&mut map, name, value);
    }
    Value::Object(map)
}

fn insert_value(target: &mut IndexMap<Name, Value>, name: Name, value: Value) {
    if let Some(prev_value) = target.get_mut(&name) {
        if let Value::Object(target_map) = prev_value {
            if let Value::Object(obj) = value {
                for (key, value) in obj.into_iter() {
                    insert_value(target_map, key, value);
                }
            }
        } else if let Value::List(target_list) = prev_value {
            if let Value::List(list) = value {
                for (idx, value) in list.into_iter().enumerate() {
                    if let Some(Value::Object(target_map)) = target_list.get_mut(idx) {
                        if let Value::Object(obj) = value {
                            for (key, value) in obj.into_iter() {
                                insert_value(target_map, key, value);
                            }
                        }
                    }
                }
            }
        }
    } else {
        target.insert(name, value);
    }
}

#[cfg(feature = "boxed-trait")]
async fn resolve_container_inner<'a, T: OutputTypeMarker>(
    ctx: &ContextSelectionSet<'a>,
    root: &dyn ContainerType,
    root_marker: &T,
    parallel: bool,
) -> ServerResult<Value> {
    let mut fields = Fields(Vec::new());
    fields.add_set(ctx, root, root_marker)?;

    let res = if parallel {
        futures_util::future::try_join_all(fields.0).await?
    } else {
        let mut results = Vec::with_capacity(fields.0.len());
        for field in fields.0 {
            results.push(field.await?);
        }
        results
    };

    Ok(create_value_object(res))
}

#[cfg(not(feature = "boxed-trait"))]
async fn resolve_container_inner<'a, T: OutputTypeMarker + ContainerType>(
    ctx: &ContextSelectionSet<'a>,
    root: &T,
    parallel: bool,
) -> ServerResult<Value> {
    let mut fields = Fields(Vec::new());
    fields.add_set(ctx, root)?;

    let res = if parallel {
        futures_util::future::try_join_all(fields.0).await?
    } else {
        let mut results = Vec::with_capacity(fields.0.len());
        for field in fields.0 {
            results.push(field.await?);
        }
        results
    };

    Ok(create_value_object(res))
}

type BoxFieldFuture<'a> = Pin<Box<dyn Future<Output = ServerResult<(Name, Value)>> + Send + 'a>>;

/// A set of fields on an container that are being selected.
pub struct Fields<'a>(Vec<BoxFieldFuture<'a>>);

impl<'a> Fields<'a> {
    /// Add another set of fields to this set of fields using the given
    /// container.
    #[cfg(feature = "boxed-trait")]
    pub fn add_set<T: OutputTypeMarker + ?Sized>(
        &mut self,
        ctx: &ContextSelectionSet<'a>,
        root: &'a dyn ContainerType,
        root_marker: &'a T,
    ) -> ServerResult<()> {
        let introspection_type_name = root_marker.introspection_type_name();
        let type_name = T::type_name();

        fn add_set_inner<'a>(
            fields: &mut Fields<'a>,
            ctx: &ContextSelectionSet<'a>,
            root: &'a dyn ContainerType,
            introspection_type_name: std::borrow::Cow<'a, str>,
            type_name: std::borrow::Cow<'a, str>,
        ) -> Result<(), ServerError> {
            let mut stack = vec![ctx.clone()];

            while let Some(ctx) = stack.pop() {
                for selection in &ctx.item.node.items {
                    match &selection.node {
                        Selection::Field(field) => {
                            if field.node.name.node == "__typename" {
                                // Get the typename
                                let ctx_field = ctx.with_field(field);
                                let field_name = ctx_field.item.node.response_key().node.clone();
                                let typename = introspection_type_name.clone().into_owned();

                                fields.0.push(Box::pin(async move {
                                    Ok((field_name, Value::String(typename)))
                                }));
                                continue;
                            }

                            let resolve_fut = Box::pin({
                                let ctx = ctx.clone();
                                let type_name = type_name.clone();
                                async move {
                                    let ctx_field = ctx.with_field(field);
                                    let field_name =
                                        ctx_field.item.node.response_key().node.clone();
                                    let extensions = &ctx.query_env.extensions;

                                    if extensions.is_empty() && field.node.directives.is_empty() {
                                        Ok((
                                            field_name,
                                            root.resolve_field(&ctx_field)
                                                .await?
                                                .unwrap_or_default(),
                                        ))
                                    } else {
                                        let type_name = type_name;
                                        let resolve_info = ResolveInfo {
                                            path_node: ctx_field.path_node.as_ref().unwrap(),
                                            parent_type: &type_name,
                                            return_type: match ctx_field
                                                .schema_env
                                                .registry
                                                .types
                                                .get(type_name.as_ref())
                                                .and_then(|ty| {
                                                    ty.field_by_name(field.node.name.node.as_str())
                                                })
                                                .map(|field| &field.ty)
                                            {
                                                Some(ty) => &ty,
                                                None => {
                                                    return Err(ServerError::new(
                                                        format!(
                                                            r#"Cannot query field "{}" on type "{}"."#,
                                                            field_name, type_name
                                                        ),
                                                        Some(ctx_field.item.pos),
                                                    ));
                                                }
                                            },
                                            name: field.node.name.node.as_str(),
                                            alias: field
                                                .node
                                                .alias
                                                .as_ref()
                                                .map(|alias| alias.node.as_str()),
                                            is_for_introspection: ctx_field.is_for_introspection,
                                            field: &field.node,
                                        };

                                        let resolve_fut = root.resolve_field(&ctx_field);

                                        if field.node.directives.is_empty() {
                                            futures_util::pin_mut!(resolve_fut);
                                            Ok((
                                                field_name,
                                                extensions
                                                    .resolve(resolve_info, &mut resolve_fut)
                                                    .await?
                                                    .unwrap_or_default(),
                                            ))
                                        } else {
                                            let mut resolve_fut = resolve_fut.boxed();

                                            for directive in &field.node.directives {
                                                if let Some(directive_factory) = ctx
                                                    .schema_env
                                                    .custom_directives
                                                    .get(directive.node.name.node.as_str())
                                                {
                                                    let ctx_directive = ContextBase {
                                                        path_node: ctx_field.path_node,
                                                        is_for_introspection: false,
                                                        item: directive,
                                                        schema_env: ctx_field.schema_env,
                                                        query_env: ctx_field.query_env,
                                                        execute_data: ctx_field.execute_data,
                                                    };
                                                    let directive_instance = directive_factory
                                                        .create(&ctx_directive, &directive.node)?;
                                                    resolve_fut = Box::pin({
                                                        let ctx_field = ctx_field.clone();
                                                        async move {
                                                            directive_instance
                                                                .resolve_field(
                                                                    &ctx_field,
                                                                    &mut resolve_fut,
                                                                )
                                                                .await
                                                        }
                                                    });
                                                }
                                            }

                                            Ok((
                                                field_name,
                                                extensions
                                                    .resolve(resolve_info, &mut resolve_fut)
                                                    .await?
                                                    .unwrap_or_default(),
                                            ))
                                        }
                                    }
                                }
                            });

                            fields.0.push(resolve_fut);
                        }
                        selection => {
                            let (type_condition, selection_set) = match selection {
                                Selection::Field(_) => unreachable!(),
                                Selection::FragmentSpread(spread) => {
                                    let fragment = ctx
                                        .query_env
                                        .fragments
                                        .get(&spread.node.fragment_name.node);
                                    let fragment = match fragment {
                                        Some(fragment) => fragment,
                                        None => {
                                            return Err(ServerError::new(
                                                format!(
                                                    r#"Unknown fragment "{}"."#,
                                                    spread.node.fragment_name.node
                                                ),
                                                Some(spread.pos),
                                            ));
                                        }
                                    };
                                    (
                                        Some(&fragment.node.type_condition),
                                        &fragment.node.selection_set,
                                    )
                                }
                                Selection::InlineFragment(fragment) => (
                                    fragment.node.type_condition.as_ref(),
                                    &fragment.node.selection_set,
                                ),
                            };
                            let type_condition =
                                type_condition.map(|condition| condition.node.on.node.as_str());

                            let introspection_type_name = introspection_type_name.clone();

                            let applies_concrete_object = type_condition.is_some_and(|condition| {
                                introspection_type_name == condition
                                    || ctx
                                        .schema_env
                                        .registry
                                        .implements
                                        .get(&*introspection_type_name)
                                        .is_some_and(|interfaces| interfaces.contains(condition))
                            });
                            if applies_concrete_object {
                                stack.push(ctx.with_selection_set(selection_set));
                            } else if type_condition.is_none_or(|condition| type_name == condition)
                            {
                                // The fragment applies to an interface type.
                                stack.push(ctx.with_selection_set(selection_set));
                            }
                        }
                    }
                }
            }

            Ok(())
        }

        add_set_inner(self, ctx, root, introspection_type_name, type_name)
    }

    /// Add another set of fields to this set of fields using the given
    /// container.
    #[cfg(not(feature = "boxed-trait"))]
    pub fn add_set<T: ContainerType + OutputTypeMarker + ?Sized>(
        &mut self,
        ctx: &ContextSelectionSet<'a>,
        root: &'a T,
    ) -> ServerResult<()> {
        for selection in &ctx.item.node.items {
            match &selection.node {
                Selection::Field(field) => {
                    if field.node.name.node == "__typename" {
                        // Get the typename
                        let ctx_field = ctx.with_field(field);
                        let field_name = ctx_field.item.node.response_key().node.clone();
                        let typename = root.introspection_type_name().into_owned();

                        self.0.push(Box::pin(async move {
                            Ok((field_name, Value::String(typename)))
                        }));
                        continue;
                    }

                    let resolve_fut = Box::pin({
                        let ctx = ctx.clone();
                        async move {
                            let ctx_field = ctx.with_field(field);
                            let field_name = ctx_field.item.node.response_key().node.clone();
                            let extensions = &ctx.query_env.extensions;

                            if extensions.is_empty() && field.node.directives.is_empty() {
                                Ok((
                                    field_name,
                                    root.resolve_field(&ctx_field).await?.unwrap_or_default(),
                                ))
                            } else {
                                let type_name = T::type_name();
                                let resolve_info = ResolveInfo {
                                    path_node: ctx_field.path_node.as_ref().unwrap(),
                                    parent_type: &type_name,
                                    return_type: match ctx_field
                                        .schema_env
                                        .registry
                                        .types
                                        .get(type_name.as_ref())
                                        .and_then(|ty| {
                                            ty.field_by_name(field.node.name.node.as_str())
                                        })
                                        .map(|field| &field.ty)
                                    {
                                        Some(ty) => &ty,
                                        None => {
                                            return Err(ServerError::new(
                                                format!(
                                                    r#"Cannot query field "{}" on type "{}"."#,
                                                    field_name, type_name
                                                ),
                                                Some(ctx_field.item.pos),
                                            ));
                                        }
                                    },
                                    name: field.node.name.node.as_str(),
                                    alias: field
                                        .node
                                        .alias
                                        .as_ref()
                                        .map(|alias| alias.node.as_str()),
                                    is_for_introspection: ctx_field.is_for_introspection,
                                    field: &field.node,
                                };

                                let resolve_fut = root.resolve_field(&ctx_field);

                                if field.node.directives.is_empty() {
                                    futures_util::pin_mut!(resolve_fut);
                                    Ok((
                                        field_name,
                                        extensions
                                            .resolve(resolve_info, &mut resolve_fut)
                                            .await?
                                            .unwrap_or_default(),
                                    ))
                                } else {
                                    let mut resolve_fut = resolve_fut.boxed();

                                    for directive in &field.node.directives {
                                        if let Some(directive_factory) = ctx
                                            .schema_env
                                            .custom_directives
                                            .get(directive.node.name.node.as_str())
                                        {
                                            let ctx_directive = ContextBase {
                                                path_node: ctx_field.path_node,
                                                is_for_introspection: false,
                                                item: directive,
                                                schema_env: ctx_field.schema_env,
                                                query_env: ctx_field.query_env,
                                                execute_data: ctx_field.execute_data,
                                            };
                                            let directive_instance = directive_factory
                                                .create(&ctx_directive, &directive.node)?;
                                            resolve_fut = Box::pin({
                                                let ctx_field = ctx_field.clone();
                                                async move {
                                                    directive_instance
                                                        .resolve_field(&ctx_field, &mut resolve_fut)
                                                        .await
                                                }
                                            });
                                        }
                                    }

                                    Ok((
                                        field_name,
                                        extensions
                                            .resolve(resolve_info, &mut resolve_fut)
                                            .await?
                                            .unwrap_or_default(),
                                    ))
                                }
                            }
                        }
                    });

                    self.0.push(resolve_fut);
                }
                selection => {
                    let (type_condition, selection_set) = match selection {
                        Selection::Field(_) => unreachable!(),
                        Selection::FragmentSpread(spread) => {
                            let fragment =
                                ctx.query_env.fragments.get(&spread.node.fragment_name.node);
                            let fragment = match fragment {
                                Some(fragment) => fragment,
                                None => {
                                    return Err(ServerError::new(
                                        format!(
                                            r#"Unknown fragment "{}"."#,
                                            spread.node.fragment_name.node
                                        ),
                                        Some(spread.pos),
                                    ));
                                }
                            };
                            (
                                Some(&fragment.node.type_condition),
                                &fragment.node.selection_set,
                            )
                        }
                        Selection::InlineFragment(fragment) => (
                            fragment.node.type_condition.as_ref(),
                            &fragment.node.selection_set,
                        ),
                    };
                    let type_condition =
                        type_condition.map(|condition| condition.node.on.node.as_str());

                    let introspection_type_name = root.introspection_type_name();

                    let applies_concrete_object = type_condition.is_some_and(|condition| {
                        introspection_type_name == condition
                            || ctx
                                .schema_env
                                .registry
                                .implements
                                .get(&*introspection_type_name)
                                .is_some_and(|interfaces| interfaces.contains(condition))
                    });
                    if applies_concrete_object {
                        root.collect_all_fields(&ctx.with_selection_set(selection_set), self)?;
                    } else if type_condition.is_none_or(|condition| T::type_name() == condition) {
                        // The fragment applies to an interface type.
                        self.add_set(&ctx.with_selection_set(selection_set), root)?;
                    }
                }
            }
        }
        Ok(())
    }
}
