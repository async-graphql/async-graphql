//! Validator newtypes for `async-graphql`

#[cfg(feature = "email-address")]
mod email_address;

#[cfg(feature = "email-address")]
pub use self::email_address::EmailAddress;
