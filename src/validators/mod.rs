mod max_items;
mod max_length;
mod maximum;
mod min_items;
mod min_length;
mod minimum;
mod multiple_of;

pub use max_items::max_items;
pub use max_length::max_length;
pub use maximum::maximum;
pub use min_items::min_items;
pub use min_length::min_length;
pub use minimum::minimum;
pub use multiple_of::multiple_of;

use crate::{Context, InputType};

/// Represents a custom input value validator.
#[async_trait::async_trait]
pub trait CustomValidator<T: InputType> {
    /// Check the value is valid.
    async fn check(&self, ctx: &Context<'_>, value: &T) -> Result<(), String>;
}
