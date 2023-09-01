use std::{
    any::Any,
    borrow::Cow,
    fmt::{self, Debug},
    ops::Deref,
};

use futures_util::{future::BoxFuture, Future, FutureExt};
use indexmap::IndexMap;

use crate::{
    dynamic::{InputValue, ObjectAccessor, TypeRef},
    registry::Deprecation,
    Context, Error, Result, Value,
};

/// A value returned from the resolver function
pub struct FieldValue<'a>(pub(crate) FieldValueInner<'a>);

pub(crate) enum FieldValueInner<'a> {
    /// Const value
    Value(Value),
    /// Borrowed any value
    BorrowedAny(&'a (dyn Any + Send + Sync)),
    /// Owned any value
    OwnedAny(Box<dyn Any + Send + Sync>),
    /// A list
    List(Vec<FieldValue<'a>>),
    /// A typed Field value
    WithType {
        /// Field value
        value: Box<FieldValue<'a>>,
        /// Object name
        ty: Cow<'static, str>,
    },
}

impl<'a> From<()> for FieldValue<'a> {
    #[inline]
    fn from(_: ()) -> Self {
        Self(FieldValueInner::Value(Value::Null))
    }
}

impl<'a> From<Value> for FieldValue<'a> {
    #[inline]
    fn from(value: Value) -> Self {
        Self(FieldValueInner::Value(value))
    }
}

impl<'a, T: Into<FieldValue<'a>>> From<Vec<T>> for FieldValue<'a> {
    fn from(values: Vec<T>) -> Self {
        Self(FieldValueInner::List(
            values.into_iter().map(Into::into).collect(),
        ))
    }
}

impl<'a> FieldValue<'a> {
    /// A null value equivalent to `FieldValue::Value(Value::Null)`
    pub const NULL: FieldValue<'a> = Self(FieldValueInner::Value(Value::Null));

    /// A none value equivalent to `None::<FieldValue>`
    ///
    /// It is more convenient to use when your resolver needs to return `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_graphql::dynamic::*;
    ///
    /// let query =
    ///     Object::new("Query").field(Field::new("value", TypeRef::named(TypeRef::INT), |ctx| {
    ///         FieldFuture::new(async move { Ok(FieldValue::NONE) })
    ///     }));
    /// ```
    pub const NONE: Option<FieldValue<'a>> = None;

    /// Returns a `None::<FieldValue>` meaning the resolver no results.
    pub const fn none() -> Option<FieldValue<'a>> {
        None
    }

    /// Create a FieldValue from [`Value`]
    #[inline]
    pub fn value(value: impl Into<Value>) -> Self {
        Self(FieldValueInner::Value(value.into()))
    }

    /// Create a FieldValue from owned any value
    #[inline]
    pub fn owned_any(obj: impl Any + Send + Sync) -> Self {
        Self(FieldValueInner::OwnedAny(Box::new(obj)))
    }

    /// Create a FieldValue from unsized any value
    #[inline]
    pub fn boxed_any(obj: Box<dyn Any + Send + Sync>) -> Self {
        Self(FieldValueInner::OwnedAny(obj))
    }

    /// Create a FieldValue from owned any value
    #[inline]
    pub fn borrowed_any(obj: &'a (dyn Any + Send + Sync)) -> Self {
        Self(FieldValueInner::BorrowedAny(obj))
    }

    /// Create a FieldValue from list
    #[inline]
    pub fn list<I, T>(values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<FieldValue<'a>>,
    {
        Self(FieldValueInner::List(
            values.into_iter().map(Into::into).collect(),
        ))
    }

    /// Create a FieldValue and specify its type, which must be an object
    ///
    /// NOTE: Fields of type `Interface` or `Union` must return
    /// `FieldValue::WithType`.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_graphql::{dynamic::*, value, Value};
    ///
    /// struct MyObjData {
    ///     a: i32,
    /// }
    ///
    /// let my_obj = Object::new("MyObj").field(Field::new(
    ///     "a",
    ///     TypeRef::named_nn(TypeRef::INT),
    ///     |ctx| FieldFuture::new(async move {
    ///         let data = ctx.parent_value.try_downcast_ref::<MyObjData>()?;
    ///         Ok(Some(Value::from(data.a)))
    ///     }),
    /// ));
    ///
    /// let my_union = Union::new("MyUnion").possible_type(my_obj.type_name());
    ///
    /// let query = Object::new("Query").field(Field::new(
    ///     "obj",
    ///     TypeRef::named_nn(my_union.type_name()),
    ///     |_| FieldFuture::new(async move {
    ///         Ok(Some(FieldValue::owned_any(MyObjData { a: 10 }).with_type("MyObj")))
    ///     }),
    /// ));
    ///
    /// let schema = Schema::build("Query", None, None)
    ///     .register(my_obj)
    ///     .register(my_union)
    ///     .register(query)
    ///     .finish()
    ///     .unwrap();
    ///
    /// # tokio::runtime::Runtime::new().unwrap().block_on(async move {
    /// assert_eq!(
    ///    schema
    ///        .execute("{ obj { ... on MyObj { a } } }")
    ///        .await
    ///        .into_result()
    ///        .unwrap()
    ///        .data,
    ///    value!({ "obj": { "a": 10 } })
    /// );
    /// # });
    /// ```
    pub fn with_type(self, ty: impl Into<Cow<'static, str>>) -> Self {
        Self(FieldValueInner::WithType {
            value: Box::new(self),
            ty: ty.into(),
        })
    }

    /// If the FieldValue is a value, returns the associated
    /// Value. Returns `None` otherwise.
    #[inline]
    pub fn as_value(&self) -> Option<&Value> {
        match &self.0 {
            FieldValueInner::Value(value) => Some(value),
            _ => None,
        }
    }

    /// Like `as_value`, but returns `Result`.
    #[inline]
    pub fn try_to_value(&self) -> Result<&Value> {
        self.as_value()
            .ok_or_else(|| Error::new("internal: not a Value"))
    }

    /// If the FieldValue is a list, returns the associated
    /// vector. Returns `None` otherwise.
    #[inline]
    pub fn as_list(&self) -> Option<&[FieldValue]> {
        match &self.0 {
            FieldValueInner::List(values) => Some(values),
            _ => None,
        }
    }

    /// Like `as_list`, but returns `Result`.
    #[inline]
    pub fn try_to_list(&self) -> Result<&[FieldValue]> {
        self.as_list()
            .ok_or_else(|| Error::new("internal: not a list"))
    }

    /// If the FieldValue is a any, returns the associated
    /// vector. Returns `None` otherwise.
    #[inline]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        match &self.0 {
            FieldValueInner::BorrowedAny(value) => value.downcast_ref::<T>(),
            FieldValueInner::OwnedAny(value) => value.downcast_ref::<T>(),
            _ => None,
        }
    }

    /// Like `downcast_ref`, but returns `Result`.
    #[inline]
    pub fn try_downcast_ref<T: Any>(&self) -> Result<&T> {
        self.downcast_ref().ok_or_else(|| {
            Error::new(format!(
                "internal: not type \"{}\"",
                std::any::type_name::<T>()
            ))
        })
    }
}

type BoxResolveFut<'a> = BoxFuture<'a, Result<Option<FieldValue<'a>>>>;

/// A context for resolver function
pub struct ResolverContext<'a> {
    /// GraphQL context
    pub ctx: &'a Context<'a>,
    /// Field arguments
    pub args: ObjectAccessor<'a>,
    /// Parent value
    pub parent_value: &'a FieldValue<'a>,
}

impl<'a> Deref for ResolverContext<'a> {
    type Target = Context<'a>;

    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

/// A future that returned from field resolver
pub enum FieldFuture<'a> {
    /// A pure value without any async operation
    Value(Option<FieldValue<'a>>),

    /// A future that returned from field resolver
    Future(BoxResolveFut<'a>),
}

impl<'a> FieldFuture<'a> {
    /// Create a `FieldFuture` from a `Future`
    pub fn new<Fut, R>(future: Fut) -> Self
    where
        Fut: Future<Output = Result<Option<R>>> + Send + 'a,
        R: Into<FieldValue<'a>> + Send,
    {
        FieldFuture::Future(
            async move {
                let res = future.await?;
                Ok(res.map(Into::into))
            }
            .boxed(),
        )
    }

    /// Create a `FieldFuture` from a `Value`
    pub fn from_value(value: Option<Value>) -> Self {
        FieldFuture::Value(value.map(FieldValue::from))
    }
}

pub(crate) type BoxResolverFn =
    Box<(dyn for<'a> Fn(ResolverContext<'a>) -> FieldFuture<'a> + Send + Sync)>;

/// A GraphQL field
pub struct Field {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) arguments: IndexMap<String, InputValue>,
    pub(crate) ty: TypeRef,
    pub(crate) ty_str: String,
    pub(crate) resolver_fn: BoxResolverFn,
    pub(crate) deprecation: Deprecation,
    pub(crate) external: bool,
    pub(crate) requires: Option<String>,
    pub(crate) provides: Option<String>,
    pub(crate) shareable: bool,
    pub(crate) inaccessible: bool,
    pub(crate) tags: Vec<String>,
    pub(crate) override_from: Option<String>,
}

impl Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Field")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("arguments", &self.arguments)
            .field("ty", &self.ty)
            .field("deprecation", &self.deprecation)
            .finish()
    }
}

impl Field {
    /// Create a GraphQL field
    pub fn new<N, T, F>(name: N, ty: T, resolver_fn: F) -> Self
    where
        N: Into<String>,
        T: Into<TypeRef>,
        F: for<'a> Fn(ResolverContext<'a>) -> FieldFuture<'a> + Send + Sync + 'static,
    {
        let ty = ty.into();
        Self {
            name: name.into(),
            description: None,
            arguments: Default::default(),
            ty_str: ty.to_string(),
            ty,
            resolver_fn: Box::new(resolver_fn),
            deprecation: Deprecation::NoDeprecated,
            external: false,
            requires: None,
            provides: None,
            shareable: false,
            inaccessible: false,
            tags: Vec::new(),
            override_from: None,
        }
    }

    impl_set_description!();
    impl_set_deprecation!();
    impl_set_external!();
    impl_set_requires!();
    impl_set_provides!();
    impl_set_shareable!();
    impl_set_inaccessible!();
    impl_set_tags!();
    impl_set_override_from!();

    /// Add an argument to the field
    #[inline]
    pub fn argument(mut self, input_value: InputValue) -> Self {
        self.arguments.insert(input_value.name.clone(), input_value);
        self
    }
}
