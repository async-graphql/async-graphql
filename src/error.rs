use crate::{Pos, QueryPathNode, Value};
use std::fmt::{Debug, Display};
use thiserror::Error;

/// An error in the format of an input value.
#[derive(Debug)]
pub enum InputValueError {
    /// Custom input value parsing error.
    Custom(String),

    /// The type of input value does not match the expectation. Contains the value that was found.
    ExpectedType(Value),
}

impl<T: Display> From<T> for InputValueError {
    fn from(err: T) -> Self {
        InputValueError::Custom(err.to_string())
    }
}

impl InputValueError {
    /// Convert this error to a regular `Error` type.
    pub fn into_error(self, pos: Pos, expected_type: String) -> Error {
        match self {
            InputValueError::Custom(reason) => Error::Query {
                pos,
                path: None,
                err: QueryError::ParseInputValue { reason },
            },
            InputValueError::ExpectedType(value) => Error::Query {
                pos,
                path: None,
                err: QueryError::ExpectedInputType {
                    expect: expected_type,
                    actual: value,
                },
            },
        }
    }
}

/// An alias for `Result<T, InputValueError>`.
pub type InputValueResult<T> = std::result::Result<T, InputValueError>;

/// An error in a field resolver.
#[derive(Clone, Debug)]
pub struct FieldError(pub String, pub Option<serde_json::Value>);

impl FieldError {
    #[doc(hidden)]
    pub fn into_error(self, pos: Pos) -> Error {
        Error::Query {
            pos,
            path: None,
            err: QueryError::FieldError {
                err: self.0,
                extended_error: self.1,
            },
        }
    }

    #[doc(hidden)]
    pub fn into_error_with_path(self, pos: Pos, path: Option<&QueryPathNode<'_>>) -> Error {
        Error::Query {
            pos,
            path: path.and_then(|path| serde_json::to_value(path).ok()),
            err: QueryError::FieldError {
                err: self.0,
                extended_error: self.1,
            },
        }
    }
}

/// An alias for `Result<T, InputValueError>`.
pub type FieldResult<T> = std::result::Result<T, FieldError>;

impl<E: Display> From<E> for FieldError {
    fn from(err: E) -> Self {
        FieldError(format!("{}", err), None)
    }
}

/// An error which can be extended into a `FieldError`.
pub trait ErrorExtensions: Sized {
    /// Convert the error to a `FieldError`.
    fn extend(&self) -> FieldError;

    /// Add extensions to the error, using a callback to make the extensions.
    fn extend_with<C>(self, cb: C) -> FieldError
    where
        C: FnOnce(&Self) -> serde_json::Value,
    {
        let name = self.extend().0;

        if let Some(mut base) = self.extend().1 {
            let mut cb_res = cb(&self);
            if let Some(base_map) = base.as_object_mut() {
                if let Some(cb_res_map) = cb_res.as_object_mut() {
                    base_map.append(cb_res_map);
                }
                return FieldError(name, Some(serde_json::json!(base_map)));
            } else {
                return FieldError(name, Some(cb_res));
            }
        }

        FieldError(name, Some(cb(&self)))
    }
}

impl ErrorExtensions for FieldError {
    fn extend(&self) -> FieldError {
        self.clone()
    }
}

// implementing for &E instead of E gives the user the possibility to implement for E which does
// not conflict with this implementation acting as a fallback.
impl<E: std::fmt::Display> ErrorExtensions for &E {
    fn extend(&self) -> FieldError {
        FieldError(format!("{}", self), None)
    }
}

/// Extend a `Result`'s error value with [`ErrorExtensions`](trait.ErrorExtensions.html).
pub trait ResultExt<T, E>: Sized {
    /// Extend the error value of the result with the callback.
    fn extend_err<C>(self, cb: C) -> FieldResult<T>
    where
        C: FnOnce(&E) -> serde_json::Value;

    /// Extend the result to a `FieldResult`.
    fn extend(self) -> FieldResult<T>;
}

// This is implemented on E and not &E which means it cannot be used on foreign types.
// (see example).
impl<T, E> ResultExt<T, E> for std::result::Result<T, E>
where
    E: ErrorExtensions + Send + Sync + 'static,
{
    fn extend_err<C>(self, cb: C) -> FieldResult<T>
    where
        C: FnOnce(&E) -> serde_json::Value,
    {
        match self {
            Err(err) => Err(err.extend_with(|e| cb(e))),
            Ok(value) => Ok(value),
        }
    }

    fn extend(self) -> FieldResult<T> {
        match self {
            Err(err) => Err(err.extend()),
            Ok(value) => Ok(value),
        }
    }
}

/// An error processing a GraphQL query.
#[derive(Debug, Error, PartialEq)]
pub enum QueryError {
    /// The feature is not supported.
    #[error("Not supported.")]
    NotSupported,

    /// The actual input type did not match the expected input type.
    #[error("Expected input type \"{expect}\", found {actual}.")]
    ExpectedInputType {
        /// The expected input type.
        expect: String,

        /// The actual input type.
        actual: Value,
    },

    /// Parsing of an input value failed.
    #[error("Failed to parse input value: {reason}")]
    ParseInputValue {
        /// The reason for the failure to resolve.
        reason: String,
    },

    /// A field was not found on an object type.
    #[error("Cannot query field \"{field_name}\" on type \"{object}\".")]
    FieldNotFound {
        /// Field name
        field_name: String,

        /// Object name
        object: String,
    },

    /// An operation was missing from the query.
    #[error("Missing operation")]
    MissingOperation,

    /// The operation name was unknown.
    #[error("Unknown operation named \"{name}\"")]
    UnknownOperationNamed {
        /// Operation name for query.
        name: String,
    },

    /// The user attempted to query an object without selecting any subfields.
    #[error("Type \"{object}\" must have a selection of subfields.")]
    MustHaveSubFields {
        /// Object name
        object: String,
    },

    /// The schema does not have mutations.
    #[error("Schema is not configured for mutations.")]
    NotConfiguredMutations,

    /// The schema does not have subscriptions.
    #[error("Schema is not configured for subscriptions.")]
    NotConfiguredSubscriptions,

    /// The value does not exist in the enum.
    #[error("Invalid value for enum \"{ty}\".")]
    InvalidEnumValue {
        /// Enum type name
        ty: String,

        /// Enum value
        value: String,
    },

    /// A required field in an input object was not present.
    #[error("Required field \"{field_name}\" for InputObject \"{object}\" does not exist.")]
    RequiredField {
        /// Field name
        field_name: String,

        /// Object name
        object: &'static str,
    },

    /// A variable is used but not defined.
    #[error("Variable \"${var_name}\" is not defined")]
    VarNotDefined {
        /// Variable name
        var_name: String,
    },

    /// A directive was required but not provided.
    #[error(
        "Directive \"{directive}\" argument \"{arg_name}\" of type \"{arg_type}\" is required, but it was not provided."
    )]
    RequiredDirectiveArgs {
        /// Directive name
        directive: &'static str,

        /// Argument name
        arg_name: &'static str,

        /// Argument type
        arg_type: &'static str,
    },

    /// An unknown directive name was encountered.
    #[error("Unknown directive \"{name}\".")]
    UnknownDirective {
        /// Directive name
        name: String,
    },

    /// An unknown fragment was encountered.
    #[error("Unknown fragment \"{name}\".")]
    UnknownFragment {
        /// Fragment name
        name: String,
    },

    /// The query was too complex.
    // TODO: Expand on this
    #[error("Too complex")]
    TooComplex,

    /// The query was nested too deep.
    #[error("Too deep")]
    TooDeep,

    /// A field handler errored.
    #[error("Failed to resolve field: {err}")]
    FieldError {
        /// The error description.
        err: String,
        /// Extensions to the error provided through the [`ErrorExtensions`](trait.ErrorExtensions)
        /// or [`ResultExt`](trait.ResultExt) traits.
        extended_error: Option<serde_json::Value>,
    },

    /// Entity not found.
    #[error("Entity not found")]
    EntityNotFound,

    /// "__typename" must be an existing string.
    #[error("\"__typename\" must be an existing string")]
    TypeNameNotExists,
}

impl QueryError {
    /// Convert this error to a regular `Error` type.
    pub fn into_error(self, pos: Pos) -> Error {
        Error::Query {
            pos,
            path: None,
            err: self,
        }
    }
}

/// An error parsing the request.
#[derive(Debug, Error)]
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
}

/// Verification error.
#[derive(Debug, PartialEq)]
pub struct RuleError {
    /// Location of this error in query string.
    pub locations: Vec<Pos>,

    /// A description of this error.
    pub message: String,
}

/// An error serving a GraphQL query.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// Parsing the query failed.
    #[error("Parse error: {0}")]
    Parse(#[from] crate::parser::Error),

    /// Processing the query failed.
    #[error("Query error: {err}")]
    Query {
        /// The position at which the processing failed.
        pos: Pos,

        /// Node path.
        path: Option<serde_json::Value>,

        /// The query error.
        err: QueryError,
    },

    /// The query statement verification failed.
    #[error("Rule error")]
    Rule {
        /// List of errors.
        errors: Vec<RuleError>,
    },
}
