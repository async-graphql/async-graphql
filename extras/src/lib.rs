//! Extra extensions for async-graphql

#[cfg(feature = "opentelemetry")]
pub mod opentelemetry;

#[cfg(feature = "opentelemetry")]
pub use self::opentelemetry::OpenTelemetry;
