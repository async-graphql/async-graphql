//! Input value validators

mod int_validators;
mod list_validators;
mod string_validators;

use crate::{Error, Value};

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
///     async fn value1(&self, #[graphql(validator(Email))] email: String) -> i32 {
///         unimplemented!()
///     }
///
///     // Input is email or MAC address
///     async fn value2(&self, #[graphql(validator(or(Email, MAC(colon = "false"))))] email_or_mac: String) -> i32 {
///         unimplemented!()
///     }
///
///     // Input is integer between 100 and 200
///     async fn value3(&self, #[graphql(validator(IntRange(min = "100", max = "200")))] value: i32) -> i32 {
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
    /// If the input type is different from the required type, return `Ok(())` directly, and other validators will find this error.
    fn is_valid(&self, _value: &Value) -> Result<(), String> {
        Ok(())
    }

    /// Check value is valid, returns the reason include extensions for the error if it fails, otherwise None.
    ///
    /// If the input type is different from the required type, return `Ok(())` directly, and other validators will find this error.
    ///
    /// # Examples:
    ///
    /// ```no_run
    /// use async_graphql::validators::InputValueValidator;
    /// use async_graphql::{Value, Error, ErrorExtensions};
    ///
    /// pub struct IntGreaterThanZero;
    ///
    /// impl InputValueValidator for IntGreaterThanZero {
    ///     fn is_valid_with_extensions(&self, value: &Value) -> Result<(), Error> {
    ///         if let Value::Number(n) = value {
    ///             if let Some(n) = n.as_i64() {
    ///                 if n <= 0 {
    ///                     return Err(
    ///                         Error::new("Value must be greater than 0").extend_with(|_, e| e.set("code", 400))
    ///                     );
    ///                 }
    ///             }
    ///         }
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn is_valid_with_extensions(&self, value: &Value) -> Result<(), Error> {
        // By default, use is_valid method to keep compatible with previous version
        self.is_valid(value)?;
        Ok(())
    }
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
    fn is_valid(&self, value: &Value) -> Result<(), String> {
        self.0.is_valid(value)?;
        self.1.is_valid(value)
    }
}

/// Invalidator for `InputValueValidator::or`
pub struct Or<A, B>(A, B);

impl<A, B> InputValueValidator for Or<A, B>
where
    A: InputValueValidator,
    B: InputValueValidator,
{
    fn is_valid(&self, value: &Value) -> Result<(), String> {
        if self.0.is_valid(value).is_err() {
            self.1.is_valid(value)
        } else {
            Ok(())
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
    fn is_valid(&self, value: &Value) -> Result<(), String> {
        self.0.is_valid(value).map_err(&self.1)
    }
}
