use crate::{parser, InputValueType, Pos, Value};
use serde::Serialize;
use std::fmt::{self, Debug, Display, Formatter};
use std::marker::PhantomData;
use thiserror::Error;

/// An error in a GraphQL server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerError {
    /// An explanatory message of the error.
    pub message: String,
    /// Where the error occurred.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<Pos>,
    /// If the error occurred in a resolver, the path to the error.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub path: Vec<PathSegment>,
    /// Extensions to the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Map<String, serde_json::Value>>,
}

impl ServerError {
    /// Create a new server error with the message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            locations: Vec::new(),
            path: Vec::new(),
            extensions: None,
        }
    }

    /// Add a position to the error.
    pub fn at(mut self, at: Pos) -> Self {
        self.locations.push(at);
        self
    }

    /// Prepend a path to the error.
    pub fn path(mut self, path: PathSegment) -> Self {
        self.path.insert(0, path);
        self
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<ServerError> for Vec<ServerError> {
    fn from(single: ServerError) -> Self {
        vec![single]
    }
}

impl From<Error> for ServerError {
    fn from(e: Error) -> Self {
        e.into_server_error()
    }
}

impl From<parser::Error> for ServerError {
    fn from(e: parser::Error) -> Self {
        Self {
            message: e.to_string(),
            locations: e.positions().collect(),
            path: Vec::new(),
            extensions: None,
        }
    }
}

/// A segment of path to a resolver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum PathSegment {
    /// A field in an object.
    Field(String),
    /// An index in a list.
    Index(usize),
}

/// Alias for `Result<T, ServerError>`.
pub type ServerResult<T> = std::result::Result<T, ServerError>;

/// An error parsing an input value.
///
/// This type is generic over T as it uses T's type name when converting to a regular error.
#[derive(Debug)]
pub struct InputValueError<T> {
    message: String,
    phantom: PhantomData<T>,
}

impl<T: InputValueType> InputValueError<T> {
    fn new(message: String) -> Self {
        Self {
            message,
            phantom: PhantomData,
        }
    }

    /// The expected input type did not match the actual input type.
    #[must_use]
    pub fn expected_type(actual: Value) -> Self {
        Self::new(format!(
            r#"Expected input type "{}", found {}."#,
            T::type_name(),
            actual
        ))
    }

    /// A custom error message.
    ///
    /// Any type that implements `Display` is automatically converted to this if you use the `?`
    /// operator.
    #[must_use]
    pub fn custom(msg: impl Display) -> Self {
        Self::new(format!(r#"Failed to parse "{}": {}"#, T::type_name(), msg))
    }

    /// Propogate the error message to a different type.
    pub fn propogate<U: InputValueType>(self) -> InputValueError<U> {
        InputValueError::new(format!(
            r#"{} (occurred while parsing "{}")"#,
            self.message,
            U::type_name()
        ))
    }

    /// Convert the error into a server error.
    pub fn into_server_error(self) -> ServerError {
        ServerError::new(self.message)
    }
}

impl<T: InputValueType, E: Display> From<E> for InputValueError<T> {
    fn from(error: E) -> Self {
        Self::custom(error)
    }
}

/// An error parsing a value of type `T`.
pub type InputValueResult<T> = Result<T, InputValueError<T>>;

/// An error with a message and optional extensions.
#[derive(Debug, Clone, Serialize)]
pub struct Error {
    /// The error message.
    pub message: String,
    /// Extensions to the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Map<String, serde_json::Value>>,
}

impl Error {
    /// Create an error from the given error message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            extensions: None,
        }
    }

    /// Convert the error to a server error.
    #[must_use]
    pub fn into_server_error(self) -> ServerError {
        ServerError {
            message: self.message,
            locations: Vec::new(),
            path: Vec::new(),
            extensions: self.extensions,
        }
    }
}

impl<T: Display> From<T> for Error {
    fn from(e: T) -> Self {
        Self {
            message: e.to_string(),
            extensions: None,
        }
    }
}

/// An alias for `Result<T, Error>`.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/*
/// Extend errors with additional information.
///
/// This trait is implemented for `Error` and `Result<T>`.
pub trait ExtendError {
    /// Extend the error with the extensions.
    ///
    /// The value must be a map otherwise this function will panic. It takes a value for the
    /// ergonomics of being able to use serde_json's `json!` macro.
    ///
    /// If the error already contains extensions they are appended on.
    fn extend(self, extensions: serde_json::Value) -> Self;

    /// Extend the error with a callback to make the extensions.
    fn extend_with(self, f: impl FnOnce(&Error) -> serde_json::Value) -> Self;
}

impl ExtendError for Error {
    fn extend(self, extensions: serde_json::Value) -> Self {
        let mut extensions = match extensions {
            serde_json::Value::Object(map) => map,
            _ => panic!("Extend must be called with a map"),
        };
        Self {
            extensions: Some(match self.extensions {
                Some(mut existing) => {
                    existing.append(&mut extensions);
                    existing
                }
                None => extensions,
            }),
            ..self
        }
    }
    fn extend_with(self, f: impl FnOnce(&Error) -> serde_json::Value) -> Self {
        let ext = f(&self);
        self.extend(ext)
    }
}

impl<T> ExtendError for Result<T> {
    fn extend(self, extensions: serde_json::Value) -> Self {
        self.map_err(|e| e.extend(extensions))
    }
    fn extend_with(self, f: impl FnOnce(&Error) -> serde_json::Value) -> Self {
        self.map_err(|e| e.extend_with(f))
    }
}*/

/// An error parsing the request.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ParseRequestError {
    /// An IO error occurred.
    #[error("{0}")]
    Io(#[from] std::io::Error),

    /// The request's syntax was invalid.
    #[error("Invalid request: {0}")]
    InvalidRequest(serde_json::Error),

    /// The request's files map was invalid.
    #[error("Invalid files map: {0}")]
    InvalidFilesMap(serde_json::Error),

    /// The request's multipart data was invalid.
    #[error("Invalid multipart data")]
    #[cfg(feature = "multipart")]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "multipart")))]
    InvalidMultipart(multer::Error),

    /// Missing "operators" part for multipart request.
    #[error("Missing \"operators\" part")]
    MissingOperatorsPart,

    /// Missing "map" part for multipart request.
    #[error("Missing \"map\" part")]
    MissingMapPart,

    /// It's not an upload operation
    #[error("It's not an upload operation")]
    NotUpload,

    /// Files were missing the request.
    #[error("Missing files")]
    MissingFiles,

    /// The request's payload is too large, and this server rejected it.
    #[error("Payload too large")]
    PayloadTooLarge,

    /// The request is a batch request, but the server does not support batch requests.
    #[error("Batch requests are not supported")]
    UnsupportedBatch,
}

#[cfg(feature = "multipart")]
impl From<multer::Error> for ParseRequestError {
    fn from(err: multer::Error) -> Self {
        match err {
            multer::Error::FieldSizeExceeded { .. } | multer::Error::StreamSizeExceeded { .. } => {
                ParseRequestError::PayloadTooLarge
            }
            _ => ParseRequestError::InvalidMultipart(err),
        }
    }
}

/// An error which can be extended into a `FieldError`.
pub trait ErrorExtensions: Sized {
    /// Convert the error to a `Error`.
    fn extend(&self) -> Error;

    /// Add extensions to the error, using a callback to make the extensions.
    fn extend_with<C>(self, cb: C) -> Error
    where
        C: FnOnce(&Self) -> serde_json::Value,
    {
        let message = self.extend().message;
        match cb(&self) {
            serde_json::Value::Object(cb_res) => {
                let extensions = match self.extend().extensions {
                    Some(mut extensions) => {
                        extensions.extend(cb_res);
                        extensions
                    }
                    None => cb_res.into_iter().collect(),
                };
                Error {
                    message,
                    extensions: Some(extensions),
                }
            }
            _ => panic!("Extend must be called with a map"),
        }
    }
}

impl ErrorExtensions for Error {
    fn extend(&self) -> Error {
        self.clone()
    }
}

// implementing for &E instead of E gives the user the possibility to implement for E which does
// not conflict with this implementation acting as a fallback.
impl<E: std::fmt::Display> ErrorExtensions for &E {
    fn extend(&self) -> Error {
        Error {
            message: format!("{}", self),
            extensions: None,
        }
    }
}

/// Extend a `Result`'s error value with [`ErrorExtensions`](trait.ErrorExtensions.html).
pub trait ResultExt<T, E>: Sized {
    /// Extend the error value of the result with the callback.
    fn extend_err<C>(self, cb: C) -> Result<T>
    where
        C: FnOnce(&E) -> serde_json::Value;

    /// Extend the result to a `Result`.
    fn extend(self) -> Result<T>;
}

// This is implemented on E and not &E which means it cannot be used on foreign types.
// (see example).
impl<T, E> ResultExt<T, E> for std::result::Result<T, E>
where
    E: ErrorExtensions + Send + Sync + 'static,
{
    fn extend_err<C>(self, cb: C) -> Result<T>
    where
        C: FnOnce(&E) -> serde_json::Value,
    {
        match self {
            Err(err) => Err(err.extend_with(|e| cb(e))),
            Ok(value) => Ok(value),
        }
    }

    fn extend(self) -> Result<T> {
        match self {
            Err(err) => Err(err.extend()),
            Ok(value) => Ok(value),
        }
    }
}
