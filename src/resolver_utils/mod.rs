//! Utilities for implementing
//! [`OutputValueType::resolve`](trait.OutputValueType.html#tymethod.resolve).

mod r#enum;
mod object;
mod scalar;

pub use object::*;
pub use r#enum::*;
pub use scalar::*;
