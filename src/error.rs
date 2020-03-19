use crate::Error;
use graphql_parser::query::Value;
use graphql_parser::Pos;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Error)]
#[error("{0}")]
pub struct QueryParseError(pub(crate) String);

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Not supported.")]
    NotSupported,

    #[error("Expected type \"{expect}\", found {actual}.")]
    ExpectedType { expect: String, actual: Value },

    #[error("Expected type \"{expect}\", found {actual}.")]
    ExpectedJsonType {
        expect: String,
        actual: serde_json::Value,
    },

    #[error("Cannot query field \"{field_name}\" on type \"{object}\".")]
    FieldNotFound { field_name: String, object: String },

    #[error("Missing operation")]
    MissingOperation,

    #[error("Unknown operation named \"{name}\"")]
    UnknownOperationNamed { name: String },

    #[error("Type \"{object}\" must have a selection of subfields.")]
    MustHaveSubFields { object: String },

    #[error("Schema is not configured for mutations.")]
    NotConfiguredMutations,

    #[error("Schema is not configured for subscriptions.")]
    NotConfiguredSubscriptions,

    #[error("Invalid value for enum \"{ty}\".")]
    InvalidEnumValue { ty: String, value: String },

    #[error("Required field \"{field_name}\" for InputObject \"{object}\" does not exist.")]
    RequiredField {
        field_name: String,
        object: &'static str,
    },

    #[error("Variable \"${var_name}\" is not defined")]
    VarNotDefined { var_name: String },

    #[error(
        "Directive \"{directive}\" argument \"{arg_name}\" of type \"{arg_type}\" is required, but it was not provided."
    )]
    RequiredDirectiveArgs {
        directive: &'static str,
        arg_name: &'static str,
        arg_type: &'static str,
    },

    #[error("Unknown directive \"{name}\".")]
    UnknownDirective { name: String },

    #[error("Unknown fragment \"{name}\".")]
    UnknownFragment { name: String },

    #[error("Object \"{object}\" does not implement interface \"{interface}\"")]
    NotImplementedInterface { object: String, interface: String },

    #[error("Unrecognized inline fragment \"{name}\" on type \"{object}\"")]
    UnrecognizedInlineFragment { object: String, name: String },

    #[error("Argument \"{field_name}\" must be a non-negative integer")]
    ArgumentMustBeNonNegative { field_name: String },
}

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

#[derive(Debug, Error)]
pub struct PositionError {
    pub position: Pos,
    pub inner: Error,
}

impl PositionError {
    pub fn new(position: Pos, inner: Error) -> Self {
        Self { position, inner }
    }

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
