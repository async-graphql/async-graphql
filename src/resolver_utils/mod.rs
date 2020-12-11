//! Utilities for implementing
//! [`OutputType::resolve`](trait.OutputType.html#tymethod.resolve).

mod container;
mod r#enum;
mod list;
mod scalar;

pub use container::*;
pub use list::*;
pub use r#enum::*;
pub use scalar::*;
