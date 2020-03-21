mod int_validators;

use graphql_parser::schema::Value;

/// Input value validator
///
/// You can create your own input value validator by implementing this trait.
pub trait InputValueValidator
where
    Self: Sync + Send,
{
    /// Check value is valid, returns the reason for the error if it fails, otherwise None.
    fn is_valid(&self, value: &Value) -> Option<String>;
}

pub use int_validators::IntRange;
