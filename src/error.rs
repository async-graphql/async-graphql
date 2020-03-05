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
    FieldNotFound {
        field_name: String,
        object: &'static str,
    },

    #[error("Unknown operation named \"{name}\"")]
    UnknownOperationNamed { name: String },

    #[error("Type \"{object}\" must have a selection of subfields.")]
    MustHaveSubFields { object: &'static str },

    #[error("Schema is not configured for mutations.")]
    NotConfiguredMutations,

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
