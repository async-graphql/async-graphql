use crate::base::Type;
use crate::extensions::Extensions;
use crate::parser::types::{
    Directive, ExecutableDocumentData, Field, Name, SelectionSet, Value as InputValue,
};
use crate::schema::SchemaEnv;
use crate::{FieldResult, InputValueType, Lookahead, Pos, Positioned, QueryError, Result, Value};
use fnv::FnvHashMap;
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};
use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::ops::Deref;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

/// Variables of a query.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Variables(pub BTreeMap<Name, Value>);

impl Display for Variables {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("{")?;
        for (i, (name, value)) in self.0.iter().enumerate() {
            write!(f, "{}{}: {}", if i == 0 { "" } else { ", " }, name, value)?;
        }
        f.write_str("}")
    }
}

impl Variables {
    /// Parse variables from JSON object.
    ///
    /// If the value is not a map, or the keys of map are not valid GraphQL names, then an empty
    /// `Variables` instance will be returned.
    pub fn parse_from_json(value: serde_json::Value) -> Self {
        if let Ok(Value::Object(obj)) = Value::from_json(value) {
            Self(obj)
        } else {
            Default::default()
        }
    }

    pub(crate) fn variable_path(&mut self, path: &str) -> Option<&mut Value> {
        let mut parts = path.strip_prefix("variables.")?.split('.');

        let initial = self.0.get_mut(parts.next().unwrap())?;

        parts.try_fold(initial, |current, part| match current {
            Value::List(list) => part
                .parse::<u32>()
                .ok()
                .and_then(|idx| usize::try_from(idx).ok())
                .and_then(move |idx| list.get_mut(idx)),
            Value::Object(obj) => obj.get_mut(part),
            _ => None,
        })
    }
}

/// Schema/Context data.
#[derive(Default)]
pub struct Data(FnvHashMap<TypeId, Box<dyn Any + Sync + Send>>);

impl Data {
    #[allow(missing_docs)]
    pub fn insert<D: Any + Send + Sync>(&mut self, data: D) {
        self.0.insert(TypeId::of::<D>(), Box::new(data));
    }
}

/// Context for `SelectionSet`
pub type ContextSelectionSet<'a> = ContextBase<'a, &'a Positioned<SelectionSet>>;

/// Context object for resolve field
pub type Context<'a> = ContextBase<'a, &'a Positioned<Field>>;

/// The query path segment
#[derive(Clone)]
pub enum QueryPathSegment<'a> {
    /// Index
    Index(usize),

    /// Field name
    Name(&'a str),
}

/// The query path node
#[derive(Clone)]
pub struct QueryPathNode<'a> {
    /// Parent node
    pub parent: Option<&'a QueryPathNode<'a>>,

    /// Current path segment
    pub segment: QueryPathSegment<'a>,
}

impl<'a> serde::Serialize for QueryPathNode<'a> {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;
        self.for_each(|segment| match segment {
            QueryPathSegment::Index(idx) => {
                seq.serialize_element(&idx).ok();
            }
            QueryPathSegment::Name(name) => {
                seq.serialize_element(name).ok();
            }
        });
        seq.end()
    }
}

impl<'a> Display for QueryPathNode<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut first = true;
        self.for_each(|segment| {
            if !first {
                write!(f, ".").ok();
            }
            match segment {
                QueryPathSegment::Index(idx) => {
                    write!(f, "{}", *idx).ok();
                }
                QueryPathSegment::Name(name) => {
                    write!(f, "{}", name).ok();
                }
            }
            first = false;
        });
        Ok(())
    }
}

impl<'a> QueryPathNode<'a> {
    /// Get the current field name.
    pub fn field_name(&self) -> &str {
        let mut p = self;
        loop {
            if let QueryPathSegment::Name(name) = &p.segment {
                return name;
            }
            p = p.parent.unwrap();
        }
    }

    pub(crate) fn for_each<F: FnMut(&QueryPathSegment<'a>)>(&self, mut f: F) {
        self.for_each_ref(&mut f);
    }

    fn for_each_ref<F: FnMut(&QueryPathSegment<'a>)>(&self, f: &mut F) {
        if let Some(parent) = &self.parent {
            parent.for_each_ref(f);
        }
        f(&self.segment);
    }
}

/// Represents the unique id of the resolve
#[derive(Copy, Clone, Debug)]
pub struct ResolveId {
    /// Parent id
    pub parent: Option<usize>,

    /// Current id
    pub current: usize,
}

impl ResolveId {
    pub(crate) fn root() -> ResolveId {
        ResolveId {
            parent: None,
            current: 0,
        }
    }
}

impl Display for ResolveId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(parent) = self.parent {
            write!(f, "{}:{}", parent, self.current)
        } else {
            write!(f, "{}", self.current)
        }
    }
}

/// Query context
#[derive(Clone)]
pub struct ContextBase<'a, T> {
    #[allow(missing_docs)]
    pub path_node: Option<QueryPathNode<'a>>,
    pub(crate) resolve_id: ResolveId,
    pub(crate) inc_resolve_id: &'a AtomicUsize,
    #[doc(hidden)]
    pub item: T,
    pub(crate) schema_env: &'a SchemaEnv,
    pub(crate) query_env: &'a QueryEnv,
}

impl<'a, T> Deref for ContextBase<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

#[doc(hidden)]
pub struct QueryEnvInner {
    pub extensions: spin::Mutex<Extensions>,
    pub variables: Variables,
    pub document: ExecutableDocumentData,
    pub ctx_data: Arc<Data>,
}

#[doc(hidden)]
#[derive(Clone)]
pub struct QueryEnv(Arc<QueryEnvInner>);

impl Deref for QueryEnv {
    type Target = QueryEnvInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl QueryEnv {
    #[doc(hidden)]
    pub fn new(
        extensions: spin::Mutex<Extensions>,
        variables: Variables,
        document: ExecutableDocumentData,
        ctx_data: Arc<Data>,
    ) -> QueryEnv {
        QueryEnv(Arc::new(QueryEnvInner {
            extensions,
            variables,
            document,
            ctx_data,
        }))
    }

    #[doc(hidden)]
    pub fn create_context<'a, T>(
        &'a self,
        schema_env: &'a SchemaEnv,
        path_node: Option<QueryPathNode<'a>>,
        item: T,
        inc_resolve_id: &'a AtomicUsize,
    ) -> ContextBase<'a, T> {
        ContextBase {
            path_node,
            resolve_id: ResolveId::root(),
            inc_resolve_id,
            item,
            schema_env,
            query_env: self,
        }
    }
}

impl<'a, T> ContextBase<'a, T> {
    fn get_child_resolve_id(&self) -> ResolveId {
        let id = self
            .inc_resolve_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        ResolveId {
            parent: Some(self.resolve_id.current),
            current: id,
        }
    }

    #[doc(hidden)]
    pub fn with_field(
        &'a self,
        field: &'a Positioned<Field>,
    ) -> ContextBase<'a, &'a Positioned<Field>> {
        ContextBase {
            path_node: Some(QueryPathNode {
                parent: self.path_node.as_ref(),
                segment: QueryPathSegment::Name(&field.node.response_key().node),
            }),
            item: field,
            resolve_id: self.get_child_resolve_id(),
            inc_resolve_id: self.inc_resolve_id,
            schema_env: self.schema_env,
            query_env: self.query_env,
        }
    }

    #[doc(hidden)]
    pub fn with_selection_set(
        &self,
        selection_set: &'a Positioned<SelectionSet>,
    ) -> ContextBase<'a, &'a Positioned<SelectionSet>> {
        ContextBase {
            path_node: self.path_node.clone(),
            item: selection_set,
            resolve_id: self.resolve_id,
            inc_resolve_id: &self.inc_resolve_id,
            schema_env: self.schema_env,
            query_env: self.query_env,
        }
    }

    /// Gets the global data defined in the `Context` or `Schema`.
    ///
    /// If both `Schema` and `Query` have the same data type, the data in the `Query` is obtained.
    ///
    /// # Errors
    ///
    /// Returns a `FieldError` if the specified type data does not exist.
    pub fn data<D: Any + Send + Sync>(&self) -> FieldResult<&D> {
        self.data_opt::<D>()
            .ok_or_else(|| format!("Data `{}` does not exist.", std::any::type_name::<D>()).into())
    }

    /// Gets the global data defined in the `Context` or `Schema`.
    ///
    /// # Panics
    ///
    /// It will panic if the specified data type does not exist.
    pub fn data_unchecked<D: Any + Send + Sync>(&self) -> &D {
        self.data_opt::<D>()
            .unwrap_or_else(|| panic!("Data `{}` does not exist.", std::any::type_name::<D>()))
    }

    /// Gets the global data defined in the `Context` or `Schema` or `None` if the specified type data does not exist.
    pub fn data_opt<D: Any + Send + Sync>(&self) -> Option<&D> {
        self.query_env
            .ctx_data
            .0
            .get(&TypeId::of::<D>())
            .or_else(|| self.schema_env.data.0.get(&TypeId::of::<D>()))
            .and_then(|d| d.downcast_ref::<D>())
    }

    fn var_value(&self, name: &str, pos: Pos) -> Result<Value> {
        self.query_env
            .document
            .operation
            .node
            .variable_definitions
            .iter()
            .find(|def| def.node.name.node == name)
            .and_then(|def| {
                self.query_env
                    .variables
                    .0
                    .get(&def.node.name.node)
                    .or_else(|| def.node.default_value())
            })
            .cloned()
            .ok_or_else(|| {
                QueryError::VarNotDefined {
                    var_name: name.to_owned(),
                }
                .into_error(pos)
            })
    }

    fn resolve_input_value(&self, value: Positioned<InputValue>) -> Result<Value> {
        let pos = value.pos;
        value
            .node
            .into_const_with(|name| self.var_value(&name, pos))
    }

    #[doc(hidden)]
    pub fn is_ifdef(&self, directives: &[Positioned<Directive>]) -> bool {
        directives
            .iter()
            .any(|directive| directive.node.name.node == "ifdef")
    }

    #[doc(hidden)]
    pub fn is_skip(&self, directives: &[Positioned<Directive>]) -> Result<bool> {
        for directive in directives {
            let include = match &*directive.node.name.node {
                "skip" => false,
                "include" => true,
                _ => continue,
            };

            let condition_input = directive
                .node
                .get_argument("if")
                .ok_or_else(|| {
                    QueryError::RequiredDirectiveArgs {
                        directive: if include { "@skip" } else { "@include" },
                        arg_name: "if",
                        arg_type: "Boolean!",
                    }
                    .into_error(directive.pos)
                })?
                .clone();

            let pos = condition_input.pos;
            let condition_input = self.resolve_input_value(condition_input)?;

            if include
                != <bool as InputValueType>::parse(Some(condition_input))
                    .map_err(|e| e.into_error(pos, bool::qualified_type_name()))?
            {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl<'a> ContextBase<'a, &'a Positioned<SelectionSet>> {
    #[doc(hidden)]
    pub fn with_index(&'a self, idx: usize) -> ContextBase<'a, &'a Positioned<SelectionSet>> {
        ContextBase {
            path_node: Some(QueryPathNode {
                parent: self.path_node.as_ref(),
                segment: QueryPathSegment::Index(idx),
            }),
            item: self.item,
            resolve_id: self.get_child_resolve_id(),
            inc_resolve_id: self.inc_resolve_id,
            schema_env: self.schema_env,
            query_env: self.query_env,
        }
    }
}

impl<'a> ContextBase<'a, &'a Positioned<Field>> {
    #[doc(hidden)]
    pub fn param_value<T: InputValueType>(
        &self,
        name: &str,
        default: Option<fn() -> T>,
    ) -> Result<T> {
        let value = self.item.node.get_argument(name).cloned();
        if value.is_none() {
            if let Some(default) = default {
                return Ok(default());
            }
        }
        let (pos, value) = match value {
            Some(value) => (value.pos, Some(self.resolve_input_value(value)?)),
            None => (Pos::default(), None),
        };
        InputValueType::parse(value).map_err(|e| e.into_error(pos, T::qualified_type_name()))
    }

    /// Get the position of the current field in the query code.
    pub fn position(&self) -> Pos {
        self.pos
    }

    /// Creates a uniform interface to inspect the forthcoming selections.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use async_graphql::*;
    ///
    /// #[SimpleObject]
    /// struct Detail {
    ///     c: i32,
    ///     d: i32,
    /// }
    ///
    /// #[SimpleObject]
    /// struct MyObj {
    ///     a: i32,
    ///     b: i32,
    ///     detail: Detail,
    /// }
    ///
    /// struct Query;
    ///
    /// #[Object]
    /// impl Query {
    ///     async fn obj(&self, ctx: &Context<'_>) -> MyObj {
    ///         if ctx.look_ahead().field("a").exists() {
    ///             // This is a query like `obj { a }`
    ///         } else if ctx.look_ahead().field("detail").field("c").exists() {
    ///             // This is a query like `obj { detail { c } }`
    ///         } else {
    ///             // This query doesn't have `a`
    ///         }
    ///         unimplemented!()
    ///     }
    /// }
    /// ```
    pub fn look_ahead(&self) -> Lookahead {
        Lookahead {
            document: &self.query_env.document,
            field: Some(&self.item.node),
        }
    }
}
