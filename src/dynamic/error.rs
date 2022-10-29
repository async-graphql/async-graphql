/// An error can occur when building dynamic schema
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
#[error("{0}")]
pub struct SchemaError(pub String);

impl<T: Into<String>> From<T> for SchemaError {
    fn from(err: T) -> Self {
        SchemaError(err.into())
    }
}
