//! Utilities for implementing
//! [`OutputValueType::resolve`](trait.OutputValueType.html#tymethod.resolve).

mod r#enum;
mod list;
mod object;
mod scalar;

pub use list::*;
pub use object::*;
pub use r#enum::*;
pub use scalar::*;
