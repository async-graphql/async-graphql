//! Input value validators

mod int_validators;
mod list_validators;
mod string_validators;

use crate::Value;

pub use int_validators::{IntEqual, IntGreaterThan, IntLessThan, IntNonZero, IntRange};
pub use list_validators::{ListMaxLength, ListMinLength};
pub use string_validators::{Email, StringMaxLength, StringMinLength, MAC};

/// Input value validator
///
/// You can create your own input value validator by implementing this trait.
///
/// # Examples
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
///     async fn value1(&self, #[arg(validator(Email))] email: String) -> i32 {
///         unimplemented!()
///     }
///
///     // Input is email or MAC address
///     async fn value2(&self, #[arg(validator(or(Email, MAC(colon = "false"))))] email_or_mac: String) -> i32 {
///         unimplemented!()
///     }
///
///     // Input is integer between 100 and 200
///     async fn value3(&self, #[arg(validator(IntRange(min = "100", max = "200")))] value: i32) -> i32 {
///         unimplemented!()
///     }
/// }
/// ```
pub trait InputValueValidator
where
    Self: Sync + Send,
{
    /// Check value is valid, returns the reason for the error if it fails, otherwise None.
    ///
    /// If the input type is different from the required type, return None directly, and other validators will find this error.
    fn is_valid(&self, value: &Value) -> Option<String>;
}

/// An extension trait for `InputValueValidator`
pub trait InputValueValidatorExt: InputValueValidator + Sized {
    /// Merge the two validators and return None only if both validators are successful.
    fn and<R: InputValueValidator>(self, other: R) -> And<Self, R> {
        And(self, other)
    }

    /// Merge two validators, and return None when either validator verifies successfully.
    fn or<R: InputValueValidator>(self, other: R) -> Or<Self, R> {
        Or(self, other)
    }

    /// Changes the error message
    fn map_err<F: Fn(String) -> String>(self, f: F) -> MapErr<Self, F> {
        MapErr(self, f)
    }
}

impl<I: InputValueValidator> InputValueValidatorExt for I {}

/// Invalidator for `InputValueValidatorExt::and`
pub struct And<A, B>(A, B);

impl<A, B> InputValueValidator for And<A, B>
where
    A: InputValueValidator,
    B: InputValueValidator,
{
    fn is_valid(&self, value: &Value) -> Option<String> {
        self.0.is_valid(value).or_else(|| self.1.is_valid(value))
    }
}

/// Invalidator for `InputValueValidator::or`
pub struct Or<A, B>(A, B);

impl<A, B> InputValueValidator for Or<A, B>
where
    A: InputValueValidator,
    B: InputValueValidator,
{
    fn is_valid(&self, value: &Value) -> Option<String> {
        if self.0.is_valid(value).is_some() {
            self.1.is_valid(value)
        } else {
            None
        }
    }
}

/// Invalidator for `InputValueValidator::map_err`
pub struct MapErr<I, F>(I, F);

impl<I, F> InputValueValidator for MapErr<I, F>
where
    I: InputValueValidator,
    F: Fn(String) -> String + Send + Sync,
{
    fn is_valid(&self, value: &Value) -> Option<String> {
        self.0.is_valid(value).map(&self.1)
    }
}
