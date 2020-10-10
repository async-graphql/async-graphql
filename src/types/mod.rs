//! Useful GraphQL types.

pub mod connection;

mod any;
mod empty_mutation;
mod empty_subscription;
mod id;
mod json;
mod maybe_undefined;
mod merged_object;
mod query_root;
#[cfg(feature = "string_number")]
mod string_number;
mod upload;

mod external;

pub use any::Any;
pub use empty_mutation::EmptyMutation;
pub use empty_subscription::EmptySubscription;
pub use id::ID;
pub use json::{Json, OutputJson};
pub use maybe_undefined::MaybeUndefined;
pub use merged_object::{MergedObject, MergedObjectTail};
#[cfg(feature = "string_number")]
pub use string_number::StringNumber;
pub use upload::{Upload, UploadValue};

pub(crate) use query_root::QueryRoot;
