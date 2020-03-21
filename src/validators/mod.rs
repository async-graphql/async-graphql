mod int_validators;
mod string_validators;

use graphql_parser::schema::Value;

pub use int_validators::{IntGreaterThan, IntLessThan, IntRange};
pub use string_validators::{Email, MAC};

/// Input value validator
///
/// You can create your own input value validator by implementing this trait.
///
/// ```no_run
/// use async_graphql::*;
/// use async_graphql::validators::{Email, MAC, IntRange};
///
/// struct QueryRoot;
///
/// #[Object]
/// impl QueryRoot {
///     // Input is email address
///     #[field]
///     async fn value1(&self, #[arg(validators(Email))] email: String) -> i32 {
///         unimplemented!()
///     }
///
///     // Input is email or MAC address
///     #[field]
///     async fn value2(&self, #[arg(validators(or(Email, MAC(colon: false))))] email_or_mac: String) -> i32 {
///         unimplemented!()
///     }
///
///     // Input is integer between 100 and 200
///     #[field]
///     async fn value3(&self, #[arg(validators(IntRange(min = 100, max = 200)))] value: i32) -> i32 {
///         unimplemented!()
///     }
/// }
/// ```
pub trait InputValueValidator
where
    Self: Sync + Send,
{
    /// Check value is valid, returns the reason for the error if it fails, otherwise None.
    fn is_valid(&self, value: &Value) -> Option<String>;
}

/// Merge the two validators and return None only if both validators are successful.
#[doc(hidden)]
pub fn and<A, B>(a: A, b: B) -> And<A, B>
where
    A: InputValueValidator,
    B: InputValueValidator,
{
    And(a, b)
}

/// Merge two validators, and return None when either validator verifies successfully.
#[doc(hidden)]
pub fn or<A, B>(a: A, b: B) -> Or<A, B>
where
    A: InputValueValidator,
    B: InputValueValidator,
{
    Or(a, b)
}

#[doc(hidden)]
pub struct And<A, B>(A, B);

impl<A, B> InputValueValidator for And<A, B>
where
    A: InputValueValidator,
    B: InputValueValidator,
{
    fn is_valid(&self, value: &Value) -> Option<String> {
        self.0.is_valid(value).and(self.1.is_valid(value))
    }
}

#[doc(hidden)]
pub struct Or<A, B>(A, B);

impl<A, B> InputValueValidator for Or<A, B>
where
    A: InputValueValidator,
    B: InputValueValidator,
{
    fn is_valid(&self, value: &Value) -> Option<String> {
        self.0.is_valid(value).or_else(|| self.1.is_valid(value))
    }
}
