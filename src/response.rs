use crate::{CacheControl, Error, Result};

/// Query response
#[derive(Debug, Default)]
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
    /// Create a new successful response with the data.
    #[must_use]
    pub fn new(data: impl Into<serde_json::Value>) -> Self {
        Self {
            data: data.into(),
            ..Default::default()
        }
    }

    /// Create a response from the error.
    #[must_use]
    pub fn from_error(error: impl Into<Error>) -> Self {
        Self {
            error: Some(error.into()),
            ..Default::default()
        }
    }

    /// Create a response from the result of the data and an error.
    #[must_use]
    pub fn from_result(result: Result<serde_json::Value>) -> Self {
        match result {
            Ok(data) => Self::new(data),
            Err(e) => Self::from_error(e),
        }
    }

    /// Set the extensions result of the response.
    #[must_use]
    pub fn extensions(self, extensions: Option<serde_json::Value>) -> Self {
        Self {
            extensions,
            ..self
        }
    }

    /// Set the cache control of the response.
    #[must_use]
    pub fn cache_control(self, cache_control: CacheControl) -> Self {
        Self {
            cache_control,
            ..self
        }
    }

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

    /// Extract the error from the response. Only if the `error` field is `None` will this return
    /// `Ok`.
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
        Self::from_error(err)
    }
}
