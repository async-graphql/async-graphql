//! Suppport for dynamic schema
//!
//! # Create a simple GraphQL schema
//!
//! ```
//! use async_graphql::{dynamic::*, value, Value};
//!
//! let query = Object::new("Query").field(Field::new("value", TypeRef::named_nn(TypeRef::INT), |ctx| {
//!     FieldFuture::new(async move { Ok(Some(Value::from(100))) })
//! }));
//!
//! # tokio::runtime::Runtime::new().unwrap().block_on(async move {
//!
//! let schema = Schema::build(query.type_name(), None, None)
//!     .register(query)
//!     .finish()?;
//!
//! assert_eq!(
//!    schema
//!        .execute("{ value }")
//!        .await
//!        .into_result()
//!        .unwrap()
//!        .data,
//!    value!({ "value": 100 })
//! );
//!
//! # Ok::<_, SchemaError>(())
//! # }).unwrap();
//! ```

#[macro_use]
mod macros;

mod base;
mod check;
mod r#enum;
mod error;
mod field;
mod input_object;
mod input_value;
mod interface;
mod object;
mod request;
mod resolve;
mod scalar;
mod schema;
mod subscription;
mod r#type;
mod type_ref;
mod union;
mod value_accessor;

pub use error::SchemaError;
pub use field::{Field, FieldFuture, FieldValue, ResolverContext};
pub use indexmap;
pub use input_object::InputObject;
pub use input_value::InputValue;
pub use interface::{Interface, InterfaceField};
pub use object::Object;
pub use r#enum::{Enum, EnumItem};
pub use r#type::Type;
pub use request::{DynamicRequest, DynamicRequestExt};
pub use scalar::Scalar;
pub use schema::{Schema, SchemaBuilder};
pub use subscription::{Subscription, SubscriptionField, SubscriptionFieldFuture};
pub use type_ref::TypeRef;
pub use union::Union;
pub use value_accessor::{ListAccessor, ObjectAccessor, ValueAccessor};
