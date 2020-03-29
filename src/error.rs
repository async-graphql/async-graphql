use crate::Error;
use graphql_parser::query::Value;
use graphql_parser::Pos;
use std::fmt::{Debug, Display, Formatter};

/// Error for query parser
#[derive(Debug, Error)]
#[error("{0}")]
pub struct QueryParseError(pub(crate) String);

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

    #[error("Argument \"{field_name}\" must be a non-negative integer")]
    ArgumentMustBeNonNegative {
        /// Field name
        field_name: String,
    },

    #[error("Invalid global id")]
    InvalidGlobalID,

    #[error("Invalid global id, expected type \"{expect}\", found {actual}.")]
    InvalidGlobalIDType {
        /// Expect type
        expect: String,

        /// Actual type
        actual: String,
    },

    #[error("Too complex.")]
    TooComplex,

    #[error("Too deep.")]
    TooDeep,
}

/// Creates a wrapper with an error location
#[allow(missing_docs)]
pub trait ErrorWithPosition {
    type Result;

    fn with_position(self, position: Pos) -> PositionError;
}

impl<T: Into<Error>> ErrorWithPosition for T {
    type Result = PositionError;

    fn with_position(self, position: Pos) -> PositionError {
        PositionError {
            position,
            inner: self.into(),
        }
    }
}

/// A wrapper with the wrong location
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub struct PositionError {
    pub position: Pos,
    pub inner: Error,
}

impl PositionError {
    #[allow(missing_docs)]
    pub fn new(position: Pos, inner: Error) -> Self {
        Self { position, inner }
    }

    #[allow(missing_docs)]
    pub fn into_inner(self) -> Error {
        self.inner
    }
}

impl Display for PositionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Debug)]
pub struct RuleError {
    pub locations: Vec<Pos>,
    pub message: String,
}

#[derive(Debug, Error)]
pub struct RuleErrors {
    pub errors: Vec<RuleError>,
}

impl Display for RuleErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for error in &self.errors {
            writeln!(f, "{}", error.message)?;
        }
        Ok(())
    }
}

/// A wrapped Error with extensions.
#[derive(Debug, Error)]
pub struct ExtendedError(pub String, pub serde_json::Value);

impl Display for ExtendedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[allow(missing_docs)]
pub trait ResultExt<T, E>
where
    Self: Sized,
    E: std::fmt::Display + Sized,
{
    fn extend_err<CB>(self, cb: CB) -> crate::Result<T>
    where
        CB: FnOnce(&E) -> serde_json::Value;
}

impl<T, E> ResultExt<T, E> for std::result::Result<T, E>
where
    E: std::fmt::Display + Sized,
{
    fn extend_err<C>(self, cb: C) -> crate::Result<T>
    where
        C: FnOnce(&E) -> serde_json::Value,
    {
        match self {
            Err(e) => Err(anyhow::anyhow!(ExtendedError(e.to_string(), cb(&e)))),
            Ok(value) => Ok(value),
        }
    }
}
