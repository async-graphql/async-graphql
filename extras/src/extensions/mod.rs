//! Extra extensions for async-graphql services

#[cfg(feature = "opentelemetry")]
mod opentelemetry;

#[cfg(feature = "opentelemetry")]
pub use self::opentelemetry::OpenTelemetry;
