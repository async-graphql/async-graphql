mod chars_max_length;
mod chars_min_length;
#[cfg(feature = "email-validator")]
mod email;
mod ip;
mod max_items;
mod max_length;
mod maximum;
mod min_items;
mod min_length;
#[cfg(feature = "password-strength-validator")]
mod min_password_strength;
mod minimum;
mod multiple_of;
mod regex;
mod url;
#[cfg(feature = "uuid-validator")]
mod uuid;

pub use chars_max_length::chars_max_length;
pub use chars_min_length::chars_min_length;
#[cfg(feature = "email-validator")]
pub use email::email;
pub use ip::ip;
pub use max_items::max_items;
pub use max_length::max_length;
pub use maximum::maximum;
pub use min_items::min_items;
pub use min_length::min_length;
#[cfg(feature = "password-strength-validator")]
pub use min_password_strength::min_password_strength;
pub use minimum::minimum;
pub use multiple_of::multiple_of;
#[cfg(feature = "uuid-validator")]
pub use uuid::uuid;

pub use self::{regex::regex, url::url};
use crate::{InputType, InputValueError};

/// Represents a custom input value validator.
pub trait CustomValidator<T: InputType> {
    /// Check the value is valid.
    fn check(&self, value: &T) -> Result<(), InputValueError<T>>;
}

impl<T, F, E> CustomValidator<T> for F
where
    T: InputType,
    E: Into<InputValueError<T>>,
    F: Fn(&T) -> Result<(), E>,
{
    #[inline]
    fn check(&self, value: &T) -> Result<(), InputValueError<T>> {
        (self)(value).map_err(Into::into)
    }
}
