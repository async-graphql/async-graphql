//! Re-exports most common types with an extra GQL* prefix to prevent name clashes.

// types
pub use crate::{
    Context as GQLContext, Data as GQLData, Error as GQLError, FieldResult as GQLFieldResult,
    InputValueResult as GQLInputValueResult, QueryBuilder as GQLQueryBuilder, Result as GQLResult,
    Schema as GQLSchema, Value as GQLValue, Variables as GQLVariables, ID as GQL_ID,
};

// traits
pub use crate::{ErrorExtensions, IntoQueryBuilder, ResultExt, Type};

// procedural macros
pub use crate::{
    Enum as GQLEnum, InputObject as GQLInputObject, Interface as GQLInterface, Object as GQLObject,
    Scalar as GQLScalar, SimpleObject as GQLSimpleObject, Subscription as GQLSubscription,
    Union as GQLUnion,
};
