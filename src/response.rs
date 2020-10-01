use crate::{CacheControl, Result, ServerError};
use serde::Serialize;

/// Query response
#[derive(Debug, Default, Serialize)]
pub struct Response {
    /// Data of query result
    pub data: serde_json::Value,

    /// Extensions result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,

    /// Cache control value
    #[serde(skip)]
    pub cache_control: CacheControl,

    /// Errors
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ServerError>,
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

    /// Create a response from some errors.
    #[must_use]
    pub fn from_errors(errors: Vec<ServerError>) -> Self {
        Self {
            errors,
            ..Default::default()
        }
    }

    /// Set the extensions result of the response.
    #[must_use]
    pub fn extensions(self, extensions: Option<serde_json::Value>) -> Self {
        Self { extensions, ..self }
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
        self.errors.is_empty()
    }

    /// Returns `true` if the response is error.
    #[inline]
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Extract the error from the response. Only if the `error` field is empty will this return
    /// `Ok`.
    #[inline]
    pub fn into_result(self) -> Result<Self, Vec<ServerError>> {
        if self.is_err() {
            Err(self.errors)
        } else {
            Ok(self)
        }
    }
}

/// Response for batchable queries
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum BatchResponse {
    /// Response for single queries
    Single(Response),

    /// Response for batch queries
    Batch(Vec<Response>),
}

impl BatchResponse {
    /// Get cache control value
    pub fn cache_control(&self) -> CacheControl {
        match self {
            BatchResponse::Single(resp) => resp.cache_control,
            BatchResponse::Batch(resp) => resp.iter().fold(CacheControl::default(), |acc, item| {
                acc.merge(&item.cache_control)
            }),
        }
    }

    /// Returns `true` if all responses are ok.
    pub fn is_ok(&self) -> bool {
        match self {
            BatchResponse::Single(resp) => resp.is_ok(),
            BatchResponse::Batch(resp) => resp.iter().all(Response::is_ok),
        }
    }
}

impl From<Response> for BatchResponse {
    fn from(response: Response) -> Self {
        Self::Single(response)
    }
}

impl From<Vec<Response>> for BatchResponse {
    fn from(responses: Vec<Response>) -> Self {
        Self::Batch(responses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_response_single() {
        let resp = BatchResponse::Single(Response::new(serde_json::Value::Bool(true)));
        assert_eq!(serde_json::to_string(&resp).unwrap(), r#"{"data":true}"#);
    }

    #[test]
    fn test_batch_response_batch() {
        let resp = BatchResponse::Batch(vec![
            Response::new(serde_json::Value::Bool(true)),
            Response::new(serde_json::Value::String("1".to_string())),
        ]);
        assert_eq!(
            serde_json::to_string(&resp).unwrap(),
            r#"[{"data":true},{"data":"1"}]"#
        );
    }
}
