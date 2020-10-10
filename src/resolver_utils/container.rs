use crate::extensions::{ErrorLogger, Extension, ExtensionContext, ResolveInfo};
use crate::parser::types::{Name, Selection};
use crate::registry::MetaType;
use crate::{
    Context, ContextSelectionSet, OutputValueType, PathSegment, ServerError, ServerResult, Value,
};
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

/// A GraphQL container.
///
/// This helper trait allows the type to call `resolve_container` on itself in its
/// `OutputValueType::resolve` implementation.
#[async_trait::async_trait]
pub trait ContainerType: OutputValueType {
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
        Self: Sized + Send + Sync,
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
impl<T: ContainerType + Send + Sync> ContainerType for &T {
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
        T::resolve_field(*self, ctx).await
    }
}

// TODO: reduce code duplication between the two below functions?

/// Resolve an container by executing each of the fields concurrently.
pub async fn resolve_container<'a, T: ContainerType + Send + Sync>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a T,
) -> ServerResult<Value> {
    let mut fields = Fields(Vec::new());
    fields.add_set(ctx, root)?;
    let futures = fields.0;

    let res = futures::future::try_join_all(futures).await?;
    let mut map = BTreeMap::new();
    for (name, value) in res {
        if let Value::Object(b) = value {
            if let Some(Value::Object(a)) = map.get_mut(&name) {
                a.extend(b);
            } else {
                map.insert(name, Value::Object(b));
            }
        } else {
            map.insert(name, value);
        }
    }
    Ok(Value::Object(map))
}

/// Resolve an container by executing each of the fields serially.
pub async fn resolve_container_serial<'a, T: ContainerType + Send + Sync>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a T,
) -> ServerResult<Value> {
    let mut fields = Fields(Vec::new());
    fields.add_set(ctx, root)?;
    let futures = fields.0;

    let mut map = BTreeMap::new();
    for field in futures {
        let (name, value) = field.await?;

        if let Value::Object(b) = value {
            if let Some(Value::Object(a)) = map.get_mut(&name) {
                a.extend(b);
            } else {
                map.insert(name, Value::Object(b));
            }
        } else {
            map.insert(name, value);
        }
    }
    Ok(Value::Object(map))
}

type BoxFieldFuture<'a> = Pin<Box<dyn Future<Output = ServerResult<(Name, Value)>> + 'a + Send>>;

/// A set of fields on an container that are being selected.
pub struct Fields<'a>(Vec<BoxFieldFuture<'a>>);

impl<'a> Fields<'a> {
    /// Add another set of fields to this set of fields using the given container.
    pub fn add_set<T: ContainerType + Send + Sync>(
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
                            let ctx_extension = ExtensionContext {
                                schema_data: &ctx.schema_env.data,
                                query_data: &ctx.query_env.ctx_data,
                            };

                            let resolve_info = ResolveInfo {
                                resolve_id: ctx_field.resolve_id,
                                path_node: ctx_field.path_node.as_ref().unwrap(),
                                parent_type: &T::type_name(),
                                return_type: match ctx_field
                                    .schema_env
                                    .registry
                                    .types
                                    .get(T::type_name().as_ref())
                                    .and_then(|ty| ty.field_by_name(field.node.name.node.as_str()))
                                    .map(|field| &field.ty)
                                {
                                    Some(ty) => &ty,
                                    None => {
                                        return Err(ServerError::new(format!(
                                            r#"Cannot query field "{}" on type "{}"."#,
                                            field_name,
                                            T::type_name()
                                        ))
                                        .at(ctx_field.item.pos)
                                        .path(PathSegment::Field(field_name.to_string())));
                                    }
                                },
                            };

                            ctx_field
                                .query_env
                                .extensions
                                .lock()
                                .resolve_start(&ctx_extension, &resolve_info);

                            let res = match root.resolve_field(&ctx_field).await {
                                Ok(value) => Ok((field_name, value.unwrap())),
                                Err(e) => Err(e.path(PathSegment::Field(field_name.to_string()))),
                            }
                            .log_error(&ctx_extension, &ctx_field.query_env.extensions)?;

                            ctx_field
                                .query_env
                                .extensions
                                .lock()
                                .resolve_end(&ctx_extension, &resolve_info);
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
