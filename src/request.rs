use crate::extensions::{BoxExtension, Extension};
use crate::parser::types::UploadValue;
use crate::{Data, ParseRequestError, Value, Variables};
use serde::{Deserialize, Deserializer};
use std::any::Any;
use std::fmt::{self, Debug, Formatter};
use std::fs::File;

/// GraphQL request.
///
/// This can be deserialized from a structure of the query string, the operation name and the
/// variables. The names are all in `camelCase` (e.g. `operationName`).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    /// The query source of the request.
    pub query: String,

    /// The operation name of the request.
    #[serde(default, rename = "operationName")]
    pub operation_name: Option<String>,

    /// The variables of the request.
    #[serde(default)]
    pub variables: Variables,

    /// The data of the request that can be accessed through `Context::data`.
    ///
    /// **This data is only valid for this request**
    #[serde(skip)]
    pub data: Data,

    /// Extensions for this request.
    #[serde(skip)]
    pub extensions: Vec<Box<dyn Fn() -> BoxExtension + Send + Sync>>,
}

impl Request {
    /// Create a request object with query source.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            operation_name: None,
            variables: Variables::default(),
            data: Data::default(),
            extensions: Vec::default(),
        }
    }

    /// Specify the operation name of the request.
    pub fn operation_name<T: Into<String>>(self, name: T) -> Self {
        Self {
            operation_name: Some(name.into()),
            ..self
        }
    }

    /// Specify the variables.
    pub fn variables(self, variables: Variables) -> Self {
        Self { variables, ..self }
    }

    /// Insert some data for this request.
    pub fn data<D: Any + Send + Sync>(mut self, data: D) -> Self {
        self.data.insert(data);
        self
    }

    /// Set a variable to an upload value.
    ///
    /// `var_path` is a dot-separated path to the item that begins with `variables`, for example
    /// `variables.files.2.content` is equivalent to the Rust code
    /// `request.variables["files"][2]["content"]`. If no variable exists at the path this function
    /// won't do anything.
    pub fn set_upload(
        &mut self,
        var_path: &str,
        filename: String,
        content_type: Option<String>,
        content: File,
    ) {
        let variable = match self.variables.variable_path(var_path) {
            Some(variable) => variable,
            None => return,
        };
        *variable = Value::Upload(UploadValue {
            filename,
            content_type,
            content,
        });
    }

    /// Add an extension
    pub fn extension<F: Fn() -> E + Send + Sync + 'static, E: Extension>(
        mut self,
        extension_factory: F,
    ) -> Self {
        self.extensions
            .push(Box::new(move || Box::new(extension_factory())));
        self
    }
}

impl<T: Into<String>> From<T> for Request {
    fn from(query: T) -> Self {
        Self::new(query)
    }
}

impl Debug for Request {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Request")
            .field("query", &self.query)
            .field("operation_name", &self.operation_name)
            .field("variables", &self.variables)
            .finish()
    }
}

/// Batch support for GraphQL requests, which is either a single query, or an array of queries
///
/// **Reference:** <https://www.apollographql.com/blog/batching-client-graphql-queries-a685f5bcd41b/>
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BatchRequest {
    /// Single query
    Single(Request),

    /// Non-empty array of queries
    #[serde(deserialize_with = "deserialize_non_empty_vec")]
    Batch(Vec<Request>),
}

impl BatchRequest {
    /// Attempt to convert the batch request into a single request.
    ///
    /// # Errors
    ///
    /// Fails if the batch request is a list of requests with a message saying that batch requests
    /// aren't supported.
    pub fn into_single(self) -> Result<Request, ParseRequestError> {
        match self {
            Self::Single(req) => Ok(req),
            Self::Batch(_) => Err(ParseRequestError::UnsupportedBatch),
        }
    }
}

fn deserialize_non_empty_vec<'de, D, T>(deserializer: D) -> std::result::Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    use serde::de::Error as _;

    let v = Vec::<T>::deserialize(deserializer)?;
    if v.is_empty() {
        Err(D::Error::invalid_length(0, &"a positive integer"))
    } else {
        Ok(v)
    }
}

impl From<Request> for BatchRequest {
    fn from(r: Request) -> Self {
        BatchRequest::Single(r)
    }
}

impl From<Vec<Request>> for BatchRequest {
    fn from(r: Vec<Request>) -> Self {
        BatchRequest::Batch(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request() {
        let request: Request = serde_json::from_value(json! ({
            "query": "{ a b c }"
        }))
        .unwrap();
        assert!(request.variables.0.is_empty());
        assert!(request.operation_name.is_none());
        assert_eq!(request.query, "{ a b c }");
    }

    #[test]
    fn test_request_with_operation_name() {
        let request: Request = serde_json::from_value(json! ({
            "query": "{ a b c }",
            "operationName": "a"
        }))
        .unwrap();
        assert!(request.variables.0.is_empty());
        assert_eq!(request.operation_name.as_deref(), Some("a"));
        assert_eq!(request.query, "{ a b c }");
    }

    #[test]
    fn test_request_with_variables() {
        let request: Request = serde_json::from_value(json! ({
            "query": "{ a b c }",
            "variables": {
                "v1": 100,
                "v2": [1, 2, 3],
                "v3": "str",
            }
        }))
        .unwrap();
        assert_eq!(
            request.variables.into_value().into_json().unwrap(),
            json!({
                "v1": 100,
                "v2": [1, 2, 3],
                "v3": "str",
            })
        );
        assert!(request.operation_name.is_none());
        assert_eq!(request.query, "{ a b c }");
    }

    #[test]
    fn test_batch_request_single() {
        let request: BatchRequest = serde_json::from_value(json! ({
            "query": "{ a b c }"
        }))
        .unwrap();

        if let BatchRequest::Single(request) = request {
            assert!(request.variables.0.is_empty());
            assert!(request.operation_name.is_none());
            assert_eq!(request.query, "{ a b c }");
        } else {
            unreachable!()
        }
    }

    #[test]
    fn test_batch_request_batch() {
        let request: BatchRequest = serde_json::from_value(json!([
            {
                "query": "{ a b c }"
            },
            {
                "query": "{ d e }"
            }
        ]))
        .unwrap();

        if let BatchRequest::Batch(requests) = request {
            assert!(requests[0].variables.0.is_empty());
            assert!(requests[0].operation_name.is_none());
            assert_eq!(requests[0].query, "{ a b c }");

            assert!(requests[1].variables.0.is_empty());
            assert!(requests[1].operation_name.is_none());
            assert_eq!(requests[1].query, "{ d e }");
        } else {
            unreachable!()
        }
    }
}
