use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use crate::extensions::{ErrorLogger, ExtensionContext, ResolveInfo};
use crate::parser::types::Selection;
use crate::registry::MetaType;
use crate::{
    Context, ContextSelectionSet, Name, OutputType, PathSegment, ServerError, ServerResult, Value,
};

/// Represents a GraphQL container object.
///
/// This helper trait allows the type to call `resolve_container` on itself in its
/// `OutputType::resolve` implementation.
#[async_trait::async_trait]
pub trait ContainerType: OutputType {
    /// This function returns true of type `EmptyMutation` only.
    #[doc(hidden)]
    fn is_empty() -> bool {
        false
    }

    /// Resolves a field value and outputs it as a json value `async_graphql::Value`.
    ///
    /// If the field was not found returns None.
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>>;

    /// Collect all the fields of the container that are queried in the selection set.
    ///
    /// Objects do not have to override this, but interfaces and unions must call it on their
    /// internal type.
    fn collect_all_fields<'a>(
        &'a self,
        ctx: &ContextSelectionSet<'a>,
        fields: &mut Fields<'a>,
    ) -> ServerResult<()>
    where
        Self: Send + Sync,
    {
        fields.add_set(ctx, self)
    }

    /// Find the GraphQL entity with the given name from the parameter.
    ///
    /// Objects should override this in case they are the query root.
    async fn find_entity(&self, _: &Context<'_>, _params: &Value) -> ServerResult<Option<Value>> {
        Ok(None)
    }
}

#[async_trait::async_trait]
impl<T: ContainerType> ContainerType for &T {
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
        T::resolve_field(*self, ctx).await
    }

    async fn find_entity(&self, ctx: &Context<'_>, params: &Value) -> ServerResult<Option<Value>> {
        T::find_entity(*self, ctx, params).await
    }
}

/// Resolve an container by executing each of the fields concurrently.
pub async fn resolve_container<'a, T: ContainerType + ?Sized>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a T,
) -> ServerResult<Value> {
    resolve_container_inner(ctx, root, true).await
}

/// Resolve an container by executing each of the fields serially.
pub async fn resolve_container_serial<'a, T: ContainerType + ?Sized>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a T,
) -> ServerResult<Value> {
    resolve_container_inner(ctx, root, false).await
}

fn insert_value(target: &mut BTreeMap<Name, Value>, name: Name, value: Value) {
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

async fn resolve_container_inner<'a, T: ContainerType + ?Sized>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a T,
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

    let mut map = BTreeMap::new();
    for (name, value) in res {
        insert_value(&mut map, name, value);
    }
    Ok(Value::Object(map))
}

type BoxFieldFuture<'a> = Pin<Box<dyn Future<Output = ServerResult<(Name, Value)>> + 'a + Send>>;

/// A set of fields on an container that are being selected.
pub struct Fields<'a>(Vec<BoxFieldFuture<'a>>);

impl<'a> Fields<'a> {
    /// Add another set of fields to this set of fields using the given container.
    pub fn add_set<T: ContainerType + ?Sized>(
        &mut self,
        ctx: &ContextSelectionSet<'a>,
        root: &'a T,
    ) -> ServerResult<()> {
        for selection in &ctx.item.node.items {
            if ctx.is_skip(&selection.node.directives())? {
                continue;
            }

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

                    if ctx.is_ifdef(&field.node.directives) {
                        if let Some(MetaType::Object { fields, .. }) =
                            ctx.schema_env.registry.types.get(T::type_name().as_ref())
                        {
                            if !fields.contains_key(field.node.name.node.as_str()) {
                                continue;
                            }
                        }
                    }

                    self.0.push(Box::pin({
                        // TODO: investigate removing this
                        let ctx = ctx.clone();
                        async move {
                            let ctx_field = ctx.with_field(field);
                            let field_name = ctx_field.item.node.response_key().node.clone();

                            let res = if ctx_field.query_env.extensions.is_empty() {
                                match root.resolve_field(&ctx_field).await {
                                    Ok(value) => Ok((field_name, value.unwrap_or_default())),
                                    Err(e) => {
                                        Err(e.path(PathSegment::Field(field_name.to_string())))
                                    }
                                }?
                            } else {
                                let ctx_extension = ExtensionContext {
                                    schema_data: &ctx.schema_env.data,
                                    query_data: &ctx.query_env.ctx_data,
                                };

                                let type_name = T::type_name();
                                let resolve_info = ResolveInfo {
                                    resolve_id: ctx_field.resolve_id,
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
                                            return Err(ServerError::new(format!(
                                                r#"Cannot query field "{}" on type "{}"."#,
                                                field_name, type_name
                                            ))
                                            .at(ctx_field.item.pos)
                                            .path(PathSegment::Field(field_name.to_string())));
                                        }
                                    },
                                };

                                ctx_field
                                    .query_env
                                    .extensions
                                    .resolve_start(&ctx_extension, &resolve_info);

                                let res = match root.resolve_field(&ctx_field).await {
                                    Ok(value) => Ok((field_name, value.unwrap_or_default())),
                                    Err(e) => {
                                        Err(e.path(PathSegment::Field(field_name.to_string())))
                                    }
                                }
                                .log_error(&ctx_extension, &ctx_field.query_env.extensions)?;

                                ctx_field
                                    .query_env
                                    .extensions
                                    .resolve_end(&ctx_extension, &resolve_info);

                                res
                            };

                            Ok(res)
                        }
                    }));
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
                                    return Err(ServerError::new(format!(
                                        r#"Unknown fragment "{}"."#,
                                        spread.node.fragment_name.node
                                    ))
                                    .at(spread.pos));
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

                    let applies_concrete_object = type_condition.map_or(false, |condition| {
                        introspection_type_name == condition
                            || ctx
                                .schema_env
                                .registry
                                .implements
                                .get(&*introspection_type_name)
                                .map_or(false, |interfaces| interfaces.contains(condition))
                    });
                    if applies_concrete_object {
                        // The fragment applies to the concrete object type.

                        // TODO: This solution isn't ideal. If there are two interfaces InterfaceA
                        // and InterfaceB and one type MyObj that implements both, then if you have
                        // a type condition for `InterfaceA` on an `InterfaceB` and when resolving,
                        // the `InterfaceB` is actually a `MyObj` then the contents of the fragment
                        // will be treated as a `MyObj` rather than an `InterfaceB`. Example:
                        //
                        // myObjAsInterfaceB {
                        //     ... on InterfaceA {
                        //         # here you can query MyObj fields even when you should only be
                        //         # able to query InterfaceA fields.
                        //     }
                        // }
                        root.collect_all_fields(&ctx.with_selection_set(selection_set), self)?;
                    } else if type_condition.map_or(true, |condition| T::type_name() == condition) {
                        // The fragment applies to an interface type.
                        self.add_set(&ctx.with_selection_set(selection_set), root)?;
                    }
                }
            }
        }
        Ok(())
    }
}
