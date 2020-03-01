use crate::Error;
use graphql_parser::query::Value;
use graphql_parser::Pos;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Error)]
#[error("{0}")]
pub struct GQLQueryParseError(pub(crate) String);

#[derive(Debug, Error)]
pub enum GQLQueryError {
    #[error("Not supported.")]
    NotSupported,

    #[error("Expected type \"{expect}\", found {actual}.")]
    ExpectedType { expect: String, actual: Value },

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

    #[error("Invalid value for enum \"{enum_type}\"")]
    InvalidEnumValue { enum_type: String, value: String },
}

pub trait GQLErrorWithPosition {
    type Result;
    fn with_position(self, position: Pos) -> GQLPositionError;
}

impl<T: Into<Error>> GQLErrorWithPosition for T {
    type Result = GQLPositionError;

    fn with_position(self, position: Pos) -> GQLPositionError {
        GQLPositionError {
            position,
            inner: self.into(),
        }
    }
}

#[derive(Debug, Error)]
pub struct GQLPositionError {
    pub position: Pos,
    pub inner: Error,
}

impl GQLPositionError {
    pub fn new(position: Pos, inner: Error) -> Self {
        Self { position, inner }
    }

    pub fn into_inner(self) -> Error {
        self.inner
    }
}

impl Display for GQLPositionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}: {}",
            self.position.line, self.position.column, self.inner
        )
    }
}
