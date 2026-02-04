//! Utilities for implementing
//! [`OutputType::resolve`](trait.OutputType.html#tymethod.resolve).

mod container;
mod r#enum;
mod list;
mod object;
mod scalar;

pub use container::*;
pub use r#enum::*;
pub use list::*;
pub use object::*;
pub use scalar::*;
