use graphql_parser::query::{ParseError, Value};
use graphql_parser::Pos;
use std::error::Error as StdError;
use std::fmt::Debug;

/// FieldError type
pub struct FieldError(anyhow::Error, Option<serde_json::Value>);

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
    pub fn into_error_with_path(self, pos: Pos, path: serde_json::Value) -> Error {
        Error::Query {
            pos,
            path: Some(path),
            err: QueryError::FieldError {
                err: self.0,
                extended_error: self.1,
            },
        }
    }
}

/// FieldResult type
pub type FieldResult<T> = std::result::Result<T, FieldError>;

impl<E> From<E> for FieldError
where
    E: StdError + Send + Sync + 'static,
{
    fn from(err: E) -> Self {
        FieldError(anyhow::Error::from(err), None)
    }
}

#[allow(missing_docs)]
pub trait ResultExt<T, E>
where
    Self: Sized,
    E: StdError + Send + Sync + 'static,
{
    fn extend_err<CB>(self, cb: CB) -> FieldResult<T>
    where
        CB: FnOnce(&E) -> serde_json::Value;
}

impl<T, E> ResultExt<T, E> for std::result::Result<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    fn extend_err<C>(self, cb: C) -> FieldResult<T>
    where
        C: FnOnce(&E) -> serde_json::Value,
    {
        match self {
            Err(err) => {
                let extended_err = cb(&err);
                Err(FieldError(err.into(), Some(extended_err)))
            }
            Ok(value) => Ok(value),
        }
    }
}

/// Error for query
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum QueryError {
    #[error("Not supported.")]
    NotSupported,

    #[error("Expected type \"{expect}\", found {actual}.")]
    ExpectedType {
        /// Expect input type
        expect: String,

        /// Actual input type
        actual: Value,
    },

    #[error("Expected type \"{expect}\", found {actual}.")]
    ExpectedJsonType {
        /// Expect input JSON type
        expect: String,

        /// Actual input JSON type
        actual: serde_json::Value,
    },

    #[error("Cannot query field \"{field_name}\" on type \"{object}\".")]
    FieldNotFound {
        /// Field name
        field_name: String,

        /// Object name
        object: String,
    },

    #[error("Missing operation")]
    MissingOperation,

    #[error("Unknown operation named \"{name}\"")]
    UnknownOperationNamed {
        /// Operation name for query
        name: String,
    },

    #[error("Type \"{object}\" must have a selection of subfields.")]
    MustHaveSubFields {
        /// Object name
        object: String,
    },

    #[error("Schema is not configured for mutations.")]
    NotConfiguredMutations,

    #[error("Schema is not configured for subscriptions.")]
    NotConfiguredSubscriptions,

    #[error("Invalid value for enum \"{ty}\".")]
    InvalidEnumValue {
        /// Enum type name
        ty: String,

        /// Enum value
        value: String,
    },

    #[error("Required field \"{field_name}\" for InputObject \"{object}\" does not exist.")]
    RequiredField {
        /// field name
        field_name: String,

        /// object name
        object: &'static str,
    },

    #[error("Variable \"${var_name}\" is not defined")]
    VarNotDefined {
        /// Variable name
        var_name: String,
    },

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

    #[error("Unknown directive \"{name}\".")]
    UnknownDirective {
        /// Directive name
        name: String,
    },

    #[error("Unknown fragment \"{name}\".")]
    UnknownFragment {
        // Fragment name
        name: String,
    },

    #[error("Object \"{object}\" does not implement interface \"{interface}\"")]
    NotImplementedInterface {
        /// Object name
        object: String,

        /// Interface name
        interface: String,
    },

    #[error("Unrecognized inline fragment \"{name}\" on type \"{object}\"")]
    UnrecognizedInlineFragment {
        /// Object name
        object: String,

        /// Inline fragment name
        name: String,
    },

    #[error("Too complex")]
    TooComplex,

    #[error("Too deep")]
    TooDeep,

    #[error("Failed to resolve field: {err}")]
    FieldError {
        err: anyhow::Error,
        extended_error: Option<serde_json::Value>,
    },
}

impl QueryError {
    #[doc(hidden)]
    pub fn into_error(self, pos: Pos) -> Error {
        Error::Query {
            pos,
            path: None,
            err: self,
        }
    }
}

#[derive(Debug)]
pub struct RuleError {
    pub locations: Vec<Pos>,
    pub message: String,
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        let msg = err.to_string();
        let mut s = msg.splitn(2, '\n');
        let first = s.next().unwrap();
        let ln = &first[first.rfind(' ').unwrap() + 1..];
        let (line, column) = {
            let mut s = ln.splitn(2, ':');
            (
                s.next().unwrap().parse().unwrap(),
                s.next().unwrap().parse().unwrap(),
            )
        };
        let tail = s.next().unwrap();
        Error::Parse {
            line,
            column,
            message: tail.to_string(),
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("Parse error: {message}")]
    Parse {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Query error: {err}")]
    Query {
        pos: Pos,
        path: Option<serde_json::Value>,
        err: QueryError,
    },

    #[error("Rule error")]
    Rule { errors: Vec<RuleError> },
}
