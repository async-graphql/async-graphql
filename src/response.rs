use crate::{CacheControl, Error, Result};

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
    /// Returns `true` if the response is ok.
    #[inline]
    pub fn is_ok(&self) -> bool {
        self.error.is_none()
    }

    /// Returns `true` if the response is error.
    #[inline]
    pub fn is_err(&self) -> bool {
        self.error.is_some()
    }

    /// Convert response to `Result<Response>`.
    #[inline]
    pub fn into_result(self) -> Result<Self> {
        if self.is_err() {
            Err(self.error.unwrap())
        } else {
            Ok(self)
        }
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
