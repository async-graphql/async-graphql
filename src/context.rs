use crate::extensions::BoxExtension;
use crate::parser::query::{Directive, Field, SelectionSet};
use crate::schema::SchemaEnv;
use crate::{
    InputValueType, Lookahead, Pos, Positioned, QueryError, QueryResponse, Result, Type, Value,
};
use async_graphql_parser::query::Document;
use async_graphql_parser::UploadValue;
use fnv::FnvHashMap;
use futures::Future;
use parking_lot::Mutex;
use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

/// Variables of query
#[derive(Debug, Clone)]
pub struct Variables(Value);

impl Default for Variables {
    fn default() -> Self {
        Self(Value::Object(Default::default()))
    }
}

impl Deref for Variables {
    type Target = BTreeMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        if let Value::Object(obj) = &self.0 {
            obj
        } else {
            unreachable!()
        }
    }
}

impl DerefMut for Variables {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Value::Object(obj) = &mut self.0 {
            obj
        } else {
            unreachable!()
        }
    }
}

impl Variables {
    /// Parse variables from JSON object.
    pub fn parse_from_json(value: serde_json::Value) -> Result<Self> {
        if let Value::Object(obj) = value.into() {
            Ok(Variables(Value::Object(obj)))
        } else {
            Ok(Default::default())
        }
    }

    pub(crate) fn set_upload(
        &mut self,
        var_path: &str,
        filename: String,
        content_type: Option<String>,
        content: File,
    ) {
        let mut it = var_path.split('.').peekable();

        if let Some(first) = it.next() {
            if first != "variables" {
                return;
            }
        }

        let mut current = &mut self.0;
        while let Some(s) = it.next() {
            let has_next = it.peek().is_some();

            if let Ok(idx) = s.parse::<i32>() {
                if let Value::List(ls) = current {
                    if let Some(value) = ls.get_mut(idx as usize) {
                        if !has_next {
                            *value = Value::Upload(UploadValue {
                                filename,
                                content_type,
                                content,
                            });
                            return;
                        } else {
                            current = value;
                        }
                    } else {
                        return;
                    }
                }
            } else if let Value::Object(obj) = current {
                if let Some(value) = obj.get_mut(s) {
                    if !has_next {
                        *value = Value::Upload(UploadValue {
                            filename,
                            content_type,
                            content,
                        });
                        return;
                    } else {
                        current = value;
                    }
                } else {
                    return;
                }
            }
        }
    }
}

#[derive(Default)]
/// Schema/Context data
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

impl<'a> std::fmt::Display for QueryPathNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    pub(crate) fn field_name(&self) -> &str {
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

    #[doc(hidden)]
    pub fn to_json(&self) -> Vec<serde_json::Value> {
        let mut path: Vec<serde_json::Value> = Vec::new();
        self.for_each(|segment| {
            path.push(match segment {
                QueryPathSegment::Index(idx) => (*idx).into(),
                QueryPathSegment::Name(name) => (*name).to_string().into(),
            })
        });
        path
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

impl std::fmt::Display for ResolveId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parent) = self.parent {
            write!(f, "{}:{}", parent, self.current)
        } else {
            write!(f, "{}", self.current)
        }
    }
}

#[doc(hidden)]
pub type BoxDeferFuture =
    Pin<Box<dyn Future<Output = Result<(QueryResponse, DeferList)>> + Send + 'static>>;

#[doc(hidden)]
pub struct DeferList {
    pub path_prefix: Vec<serde_json::Value>,
    pub futures: Mutex<Vec<BoxDeferFuture>>,
}

impl DeferList {
    pub(crate) fn append<F>(&self, fut: F)
    where
        F: Future<Output = Result<(QueryResponse, DeferList)>> + Send + 'static,
    {
        self.futures.lock().push(Box::pin(fut));
    }
}

/// Query context
#[derive(Clone)]
pub struct ContextBase<'a, T> {
    #[allow(missing_docs)]
    pub path_node: Option<QueryPathNode<'a>>,
    pub(crate) resolve_id: ResolveId,
    pub(crate) inc_resolve_id: &'a AtomicUsize,
    pub(crate) extensions: &'a [BoxExtension],
    #[doc(hidden)]
    pub item: T,
    pub(crate) schema_env: &'a SchemaEnv,
    pub(crate) query_env: &'a QueryEnv,
    pub(crate) defer_list: Option<&'a DeferList>,
}

impl<'a, T> Deref for ContextBase<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

#[doc(hidden)]
pub struct QueryEnvInner {
    pub variables: Variables,
    pub document: Document,
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
    pub fn new(variables: Variables, document: Document, ctx_data: Arc<Data>) -> QueryEnv {
        QueryEnv(Arc::new(QueryEnvInner {
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
        defer_list: Option<&'a DeferList>,
    ) -> ContextBase<'a, T> {
        ContextBase {
            path_node,
            resolve_id: ResolveId::root(),
            inc_resolve_id,
            extensions: &[],
            item,
            schema_env,
            query_env: self,
            defer_list,
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
                segment: QueryPathSegment::Name(
                    field
                        .alias
                        .as_ref()
                        .map(|alias| alias.as_str())
                        .unwrap_or_else(|| field.name.as_str()),
                ),
            }),
            extensions: self.extensions,
            item: field,
            resolve_id: self.get_child_resolve_id(),
            inc_resolve_id: self.inc_resolve_id,
            schema_env: self.schema_env,
            query_env: self.query_env,
            defer_list: self.defer_list,
        }
    }

    #[doc(hidden)]
    pub fn with_selection_set(
        &self,
        selection_set: &'a Positioned<SelectionSet>,
    ) -> ContextBase<'a, &'a Positioned<SelectionSet>> {
        ContextBase {
            path_node: self.path_node.clone(),
            extensions: self.extensions,
            item: selection_set,
            resolve_id: self.resolve_id,
            inc_resolve_id: &self.inc_resolve_id,
            schema_env: self.schema_env,
            query_env: self.query_env,
            defer_list: self.defer_list,
        }
    }

    /// Gets the global data defined in the `Context` or `Schema`.
    pub fn data<D: Any + Send + Sync>(&self) -> &D {
        self.data_opt::<D>()
            .expect("The specified data type does not exist.")
    }

    /// Gets the global data defined in the `Context` or `Schema`, returns `None` if the specified type data does not exist.
    pub fn data_opt<D: Any + Send + Sync>(&self) -> Option<&D> {
        self.query_env
            .ctx_data
            .0
            .get(&TypeId::of::<D>())
            .or_else(|| self.schema_env.data.0.get(&TypeId::of::<D>()))
            .and_then(|d| d.downcast_ref::<D>())
    }

    fn var_value(&self, name: &str, pos: Pos) -> Result<Value> {
        let def = self
            .query_env
            .document
            .current_operation()
            .variable_definitions
            .iter()
            .find(|def| def.name.node == name);
        if let Some(def) = def {
            if let Some(var_value) = self.query_env.variables.get(def.name.as_str()) {
                return Ok(var_value.clone());
            } else if let Some(default) = &def.default_value {
                return Ok(default.clone_inner());
            }
        }
        Err(QueryError::VarNotDefined {
            var_name: name.to_string(),
        }
        .into_error(pos))
    }

    fn resolve_input_value(&self, mut value: Value, pos: Pos) -> Result<Value> {
        match value {
            Value::Variable(var_name) => self.var_value(&var_name, pos),
            Value::List(ref mut ls) => {
                for value in ls {
                    if let Value::Variable(var_name) = value {
                        *value = self.var_value(&var_name, pos)?;
                    }
                }
                Ok(value)
            }
            Value::Object(ref mut obj) => {
                for value in obj.values_mut() {
                    if let Value::Variable(var_name) = value {
                        *value = self.var_value(&var_name, pos)?;
                    }
                }
                Ok(value)
            }
            _ => Ok(value),
        }
    }

    #[doc(hidden)]
    pub fn is_skip(&self, directives: &[Positioned<Directive>]) -> Result<bool> {
        for directive in directives {
            if directive.name.node == "skip" {
                if let Some(value) = directive.get_argument("if") {
                    match InputValueType::parse(
                        self.resolve_input_value(value.clone_inner(), value.position())?,
                    ) {
                        Ok(true) => return Ok(true),
                        Ok(false) => {}
                        Err(err) => {
                            return Err(err.into_error(value.pos, bool::qualified_type_name()))
                        }
                    }
                } else {
                    return Err(QueryError::RequiredDirectiveArgs {
                        directive: "@skip",
                        arg_name: "if",
                        arg_type: "Boolean!",
                    }
                    .into_error(directive.position()));
                }
            } else if directive.name.node == "include" {
                if let Some(value) = directive.get_argument("if") {
                    match InputValueType::parse(
                        self.resolve_input_value(value.clone_inner(), value.position())?,
                    ) {
                        Ok(false) => return Ok(true),
                        Ok(true) => {}
                        Err(err) => {
                            return Err(err.into_error(value.pos, bool::qualified_type_name()))
                        }
                    }
                } else {
                    return Err(QueryError::RequiredDirectiveArgs {
                        directive: "@include",
                        arg_name: "if",
                        arg_type: "Boolean!",
                    }
                    .into_error(directive.position()));
                }
            }
        }

        Ok(false)
    }

    #[doc(hidden)]
    pub fn is_defer(&self, directives: &[Positioned<Directive>]) -> bool {
        directives.iter().any(|d| d.name.node == "defer")
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
            extensions: self.extensions,
            item: self.item,
            resolve_id: self.get_child_resolve_id(),
            inc_resolve_id: self.inc_resolve_id,
            schema_env: self.schema_env,
            query_env: self.query_env,
            defer_list: self.defer_list,
        }
    }
}

impl<'a> ContextBase<'a, &'a Positioned<Field>> {
    #[doc(hidden)]
    pub fn param_value<T: InputValueType, F: FnOnce() -> Value>(
        &self,
        name: &str,
        default: F,
    ) -> Result<T> {
        match self.get_argument(name).cloned() {
            Some(value) => {
                let pos = value.position();
                let value = self.resolve_input_value(value.into_inner(), pos)?;
                match InputValueType::parse(value) {
                    Ok(res) => Ok(res),
                    Err(err) => Err(err.into_error(pos, T::qualified_type_name())),
                }
            }
            None => {
                let value = default();
                match InputValueType::parse(value) {
                    Ok(res) => Ok(res),
                    Err(err) => {
                        // The default value has no valid location.
                        Err(err.into_error(Pos::default(), T::qualified_type_name()))
                    }
                }
            }
        }
    }

    #[doc(hidden)]
    pub fn result_name(&self) -> &str {
        self.item
            .alias
            .as_ref()
            .map(|alias| alias.as_str())
            .unwrap_or_else(|| self.item.name.as_str())
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
    ///     #[field(ref)]
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
