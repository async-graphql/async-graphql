use crate::connection::edge::Edge;
use crate::connection::page_info::PageInfo;
use crate::types::connection::{CursorType, EmptyFields};
use crate::{
    do_resolve, registry, Context, ContextSelectionSet, FieldResult, ObjectType, OutputValueType,
    Positioned, QueryError, Result, Type,
};
use async_graphql_parser::query::Field;
use futures::{Stream, StreamExt, TryStreamExt};
use indexmap::map::IndexMap;
use std::borrow::Cow;

/// Connection type
///
/// Connection is the result of a query for `DataSource`.
pub struct Connection<
    C: CursorType,
    T,
    EC: ObjectType + Send = EmptyFields,
    EE: ObjectType + Send = EmptyFields,
> {
    /// All edges of the current page.
    edges: Vec<Edge<C, T, EE>>,
    additional_fields: EC,
    has_previous_page: bool,
    has_next_page: bool,
}

impl<C, T, EE> Connection<C, T, EmptyFields, EE>
where
    C: CursorType + Send,
    T: OutputValueType + Send,
    EE: ObjectType + Send,
{
    /// Create a new connection.
    pub fn new(has_previous_page: bool, has_next_page: bool) -> Self {
        Connection {
            additional_fields: EmptyFields,
            has_previous_page,
            has_next_page,
            edges: Vec::new(),
        }
    }
}

impl<C, T, EC, EE> Connection<C, T, EC, EE>
where
    C: CursorType + Send,
    T: OutputValueType + Send,
    EC: ObjectType + Send,
    EE: ObjectType + Send,
{
    /// Create a new connection, it can have some additional fields.
    pub fn with_additional_fields(
        has_previous_page: bool,
        has_next_page: bool,
        additional_fields: EC,
    ) -> Self {
        Connection {
            additional_fields,
            has_previous_page,
            has_next_page,
            edges: Vec::new(),
        }
    }
}

impl<C, T, EC, EE> Connection<C, T, EC, EE>
where
    C: CursorType,
    T: OutputValueType + Send + Sync,
    EC: ObjectType + Sync + Send,
    EE: ObjectType + Sync + Send,
{
    /// Append edges with `IntoIterator<Item = Edge<C, T, EE>>`
    pub fn append<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Edge<C, T, EE>>,
    {
        self.edges.extend(iter);
    }

    /// Append edges with `IntoIterator<Item = Edge<C, T, EE>>`
    pub fn try_append<I>(&mut self, iter: I) -> FieldResult<()>
    where
        I: IntoIterator<Item = FieldResult<Edge<C, T, EE>>>,
    {
        for edge in iter {
            self.edges.push(edge?);
        }
        Ok(())
    }

    /// Append edges with `Stream<Item = FieldResult<Edge<C, T, EE>>>`
    pub async fn append_stream<S>(&mut self, stream: S)
    where
        S: Stream<Item = Edge<C, T, EE>> + Unpin,
    {
        self.edges.extend(stream.collect::<Vec<_>>().await);
    }

    /// Append edges with `Stream<Item = FieldResult<Edge<C, T, EE>>>`
    pub async fn try_append_stream<S>(&mut self, stream: S) -> FieldResult<()>
    where
        S: Stream<Item = FieldResult<Edge<C, T, EE>>> + Unpin,
    {
        self.edges.extend(stream.try_collect::<Vec<_>>().await?);
        Ok(())
    }
}

impl<C, T, EC, EE> Type for Connection<C, T, EC, EE>
where
    C: CursorType,
    T: OutputValueType + Send + Sync,
    EC: ObjectType + Sync + Send,
    EE: ObjectType + Sync + Send,
{
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("{}Connection", T::type_name()))
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<Self, _>(|registry| {
            EC::create_type_info(registry);
            let additional_fields = if let Some(registry::MetaType::Object { fields, .. }) =
                registry.types.remove(EC::type_name().as_ref())
            {
                fields
            } else {
                unreachable!()
            };

            registry::MetaType::Object {
                name: Self::type_name().to_string(),
                description: None,
                fields: {
                    let mut fields = IndexMap::new();

                    fields.insert(
                        "pageInfo".to_string(),
                        registry::MetaField {
                            name: "pageInfo".to_string(),
                            description: Some("Information to aid in pagination."),
                            args: Default::default(),
                            ty: PageInfo::create_type_info(registry),
                            deprecation: None,
                            cache_control: Default::default(),
                            external: false,
                            requires: None,
                            provides: None,
                        },
                    );

                    fields.insert(
                        "edges".to_string(),
                        registry::MetaField {
                            name: "edges".to_string(),
                            description: Some("A list of edges."),
                            args: Default::default(),
                            ty: <Option<Vec<Option<Edge<C, T, EE>>>> as Type>::create_type_info(
                                registry,
                            ),
                            deprecation: None,
                            cache_control: Default::default(),
                            external: false,
                            requires: None,
                            provides: None,
                        },
                    );

                    fields.extend(additional_fields);
                    fields
                },
                cache_control: Default::default(),
                extends: false,
                keys: None,
            }
        })
    }
}

#[async_trait::async_trait]
impl<C, T, EC, EE> ObjectType for Connection<C, T, EC, EE>
where
    C: CursorType + Send + Sync,
    T: OutputValueType + Send + Sync,
    EC: ObjectType + Sync + Send,
    EE: ObjectType + Sync + Send,
{
    async fn resolve_field(&self, ctx: &Context<'_>) -> Result<serde_json::Value> {
        if ctx.name.node == "pageInfo" {
            let page_info = PageInfo {
                has_previous_page: self.has_previous_page,
                has_next_page: self.has_next_page,
                start_cursor: match self.edges.first() {
                    Some(edge) => Some(edge.cursor.encode_cursor().map_err(|err| {
                        QueryError::FieldError {
                            err: err.to_string(),
                            extended_error: None,
                        }
                        .into_error(ctx.position())
                    })?),
                    None => None,
                },
                end_cursor: match self.edges.last() {
                    Some(edge) => Some(edge.cursor.encode_cursor().map_err(|err| {
                        QueryError::FieldError {
                            err: err.to_string(),
                            extended_error: None,
                        }
                        .into_error(ctx.position())
                    })?),
                    None => None,
                },
            };
            let ctx_obj = ctx.with_selection_set(&ctx.selection_set);
            return OutputValueType::resolve(&page_info, &ctx_obj, ctx.item).await;
        } else if ctx.name.node == "edges" {
            let ctx_obj = ctx.with_selection_set(&ctx.selection_set);
            return OutputValueType::resolve(&self.edges, &ctx_obj, ctx.item).await;
        }

        self.additional_fields.resolve_field(ctx).await
    }
}

#[async_trait::async_trait]
impl<C, T, EC, EE> OutputValueType for Connection<C, T, EC, EE>
where
    C: CursorType + Send + Sync,
    T: OutputValueType + Send + Sync,
    EC: ObjectType + Sync + Send,
    EE: ObjectType + Sync + Send,
{
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> Result<serde_json::Value> {
        do_resolve(ctx, self).await
    }
}
