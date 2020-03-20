use crate::types::connection::edge::Edge;
use crate::types::connection::page_info::PageInfo;
use crate::{
    do_resolve, registry, Context, ContextSelectionSet, ErrorWithPosition, ObjectType,
    OutputValueType, QueryError, Result, Type,
};
use graphql_parser::query::Field;
use inflector::Inflector;
use std::borrow::Cow;
use std::collections::HashMap;

/// Connection type
///
/// Connection is the result of a query for `DataSource`,
/// If the `T` type is `OutputValueType`, you can return the value as a field function directly,
/// otherwise you can use the `Connection::map` function to convert to a type that implements `OutputValueType`.
/// `E` is an extension object type that extends the edge fields.
pub struct Connection<T, E: ObjectType + Sync + Send> {
    total_count: Option<usize>,
    page_info: PageInfo,
    nodes: Vec<(String, E, T)>,
}

impl<T, E: ObjectType + Sync + Send> Connection<T, E> {
    /// Create a connection object.
    pub fn new(
        total_count: Option<usize>,
        has_previous_page: bool,
        has_next_page: bool,
        nodes: Vec<(String, E, T)>,
    ) -> Self {
        Connection {
            total_count,
            page_info: PageInfo {
                has_previous_page,
                has_next_page,
                start_cursor: nodes.first().map(|(cursor, _, _)| cursor.clone()),
                end_cursor: nodes.last().map(|(cursor, _, _)| cursor.clone()),
            },
            nodes,
        }
    }

    /// Convert node type.
    pub fn map<O, F>(self, mut f: F) -> Connection<O, E>
    where
        F: FnMut(T) -> O,
    {
        Connection {
            total_count: self.total_count,
            page_info: self.page_info,
            nodes: self
                .nodes
                .into_iter()
                .map(|(cursor, edge_type, node)| (cursor, edge_type, f(node)))
                .collect(),
        }
    }
}

impl<T: OutputValueType + Send + Sync, E: ObjectType + Sync + Send> Type for Connection<T, E> {
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("{}Connection", T::type_name()))
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<Self, _>(|registry| registry::Type::Object {
            name: Self::type_name().to_string(),
            description: None,
            fields: {
                let mut fields = HashMap::new();

                fields.insert(
                    "pageInfo".to_string(),
                    registry::Field {
                        name: "pageInfo".to_string(),
                        description: Some("Information to aid in pagination."),
                        args: Default::default(),
                        ty: PageInfo::create_type_info(registry),
                        deprecation: None,
                    },
                );

                fields.insert(
                    "edges".to_string(),
                    registry::Field {
                        name: "edges".to_string(),
                        description: Some("A list of edges."),
                        args: Default::default(),
                        ty: <Option::<Vec<Option<Edge<T,E>>>> as Type>::create_type_info(registry),
                        deprecation: None,
                    },
                );

                fields.insert(
                    "totalCount".to_string(),
                    registry::Field {
                        name: "totalCount".to_string(),
                        description: Some(r#"A count of the total number of objects in this connection, ignoring pagination. This allows a client to fetch the first five objects by passing "5" as the argument to "first", then fetch the total count so it could display "5 of 83", for example."#),
                        args: Default::default(),
                        ty: Option::<i32>::create_type_info(registry),
                        deprecation: None,
                    },
                );

                let elements_name = T::type_name().to_plural().to_camel_case();
                fields.insert(elements_name.clone(),registry::Field{
                    name: elements_name,
                    description: Some(r#"A list of all of the objects returned in the connection. This is a convenience field provided for quickly exploring the API; rather than querying for "{ edges { node } }" when no edge data is needed, this field can be be used instead. Note that when clients like Relay need to fetch the "cursor" field on the edge to enable efficient pagination, this shortcut cannot be used, and the full "{ edges { node } }" version should be used instead."#),
                    args: Default::default(),
                    ty: Vec::<T>::type_name().to_string(),
                    deprecation: None
                });

                fields
            },
        })
    }
}

#[async_trait::async_trait]
impl<T: OutputValueType + Send + Sync, E: ObjectType + Sync + Send> ObjectType
    for Connection<T, E>
{
    async fn resolve_field(&self, ctx: &Context<'_>, field: &Field) -> Result<serde_json::Value> {
        if field.name.as_str() == "pageInfo" {
            let ctx_obj = ctx.with_item(&field.selection_set);
            let page_info = &self.page_info;
            return OutputValueType::resolve(page_info, &ctx_obj)
                .await
                .map_err(|err| err.with_position(field.position).into());
        } else if field.name.as_str() == "edges" {
            let ctx_obj = ctx.with_item(&field.selection_set);
            let edges = self
                .nodes
                .iter()
                .map(|(cursor, extra_type, node)| Edge {
                    cursor,
                    extra_type,
                    node,
                })
                .collect::<Vec<_>>();
            return OutputValueType::resolve(&edges, &ctx_obj)
                .await
                .map_err(|err| err.with_position(field.position).into());
        } else if field.name.as_str() == "totalCount" {
            return Ok(self
                .total_count
                .map(|n| (n as i32).into())
                .unwrap_or_else(|| serde_json::Value::Null));
        } else if field.name.as_str() == T::type_name().to_plural().to_camel_case() {
            let ctx_obj = ctx.with_item(&field.selection_set);
            let items = self
                .nodes
                .iter()
                .map(|(_, _, item)| item)
                .collect::<Vec<_>>();
            return OutputValueType::resolve(&items, &ctx_obj)
                .await
                .map_err(|err| err.with_position(field.position).into());
        }

        anyhow::bail!(QueryError::FieldNotFound {
            field_name: field.name.clone(),
            object: Connection::<T, E>::type_name().to_string(),
        }
        .with_position(field.position))
    }

    async fn resolve_inline_fragment(
        &self,
        name: &str,
        _ctx: &ContextSelectionSet<'_>,
        _result: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<()> {
        anyhow::bail!(QueryError::UnrecognizedInlineFragment {
            object: Connection::<T, E>::type_name().to_string(),
            name: name.to_string(),
        });
    }
}

#[async_trait::async_trait]
impl<T: OutputValueType + Send + Sync, E: ObjectType + Sync + Send> OutputValueType
    for Connection<T, E>
{
    async fn resolve(value: &Self, ctx: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        do_resolve(ctx, value).await
    }
}
