use crate::{CacheControl, Error};

/// Query response
#[derive(Debug)]
pub struct Response {
    /// Data of query result
    pub data: serde_json::Value,

    /// Extensions result
    pub extensions: Option<serde_json::Value>,

    /// Cache control value
    pub cache_control: CacheControl,

    /// Error
    pub error: Option<Error>,
}

impl Response {
    #[inline]
    pub fn is_err(&self) -> bool {
        self.error.is_some()
    }

    #[inline]
    pub fn unwrap_err(self) -> Error {
        self.error.unwrap()
    }
}

impl From<Error> for Response {
    fn from(err: Error) -> Self {
        Self {
            data: serde_json::Value::Null,
            extensions: None,
            cache_control: CacheControl::default(),
            error: Some(err),
        }
    }
}
