use crate::parser::types::UploadValue;
use crate::{Data, Value, Variables};
use serde::Deserialize;
use std::any::Any;
use std::fs::File;

/// GraphQL request.
///
/// This can be deserialized from a structure of the query string, the operation name and the
/// variables. The names are all in `camelCase` (e.g. `operationName`).
#[derive(Debug, Deserialize)]
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
}

impl Request {
    /// Create a request object with query source.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            operation_name: None,
            variables: Variables::default(),
            data: Data::default(),
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
}

impl<T: Into<String>> From<T> for Request {
    fn from(query: T) -> Self {
        Self::new(query)
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
}
