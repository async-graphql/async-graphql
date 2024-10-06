use std::{
    any::Any,
    collections::HashMap,
    fmt::{self, Debug, Formatter},
};

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    parser::{parse_query, types::ExecutableDocument},
    schema::IntrospectionMode,
    Data, Extensions, ParseRequestError, ServerError, UploadValue, Value, Variables,
};

/// GraphQL request.
///
/// This can be deserialized from a structure of the query string, the operation
/// name and the variables. The names are all in `camelCase` (e.g.
/// `operationName`).
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    /// The query source of the request.
    #[serde(default)]
    pub query: String,

    /// The operation name of the request.
    #[serde(default, rename = "operationName")]
    pub operation_name: Option<String>,

    /// The variables of the request.
    #[serde(default)]
    pub variables: Variables,

    /// Uploads sent with the request.
    #[serde(skip)]
    pub uploads: Vec<UploadValue>,

    /// The data of the request that can be accessed through `Context::data`.
    ///
    /// **This data is only valid for this request**
    #[serde(skip)]
    pub data: Data,

    /// The extensions config of the request.
    #[serde(default)]
    pub extensions: Extensions,

    /// Extra fields that are not part of the GraphQL spec.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,

    #[serde(skip)]
    pub(crate) parsed_query: Option<ExecutableDocument>,

    /// Sets the introspection mode for this request (defaults to
    /// [IntrospectionMode::Enabled]).
    #[serde(skip)]
    pub introspection_mode: IntrospectionMode,
}

impl Request {
    /// Create a request object with query source.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            operation_name: None,
            variables: Variables::default(),
            uploads: Vec::default(),
            data: Data::default(),
            extensions: Default::default(),
            extra: Default::default(),
            parsed_query: None,
            introspection_mode: IntrospectionMode::Enabled,
        }
    }

    /// Specify the operation name of the request.
    #[must_use]
    pub fn operation_name<T: Into<String>>(self, name: T) -> Self {
        Self {
            operation_name: Some(name.into()),
            ..self
        }
    }

    /// Specify the variables.
    #[must_use]
    pub fn variables(self, variables: Variables) -> Self {
        Self { variables, ..self }
    }

    /// Insert some data for this request.
    #[must_use]
    pub fn data<D: Any + Send + Sync>(mut self, data: D) -> Self {
        self.data.insert(data);
        self
    }

    /// Disable introspection queries for this request.
    #[must_use]
    pub fn disable_introspection(mut self) -> Self {
        self.introspection_mode = IntrospectionMode::Disabled;
        self
    }

    /// Only allow introspection queries for this request.
    #[must_use]
    pub fn only_introspection(mut self) -> Self {
        self.introspection_mode = IntrospectionMode::IntrospectionOnly;
        self
    }

    #[inline]
    /// Performs parsing of query ahead of execution.
    ///
    /// This effectively allows to inspect query information, before passing
    /// request to schema for execution as long as query is valid.
    pub fn parsed_query(&mut self) -> Result<&ExecutableDocument, ServerError> {
        if self.parsed_query.is_none() {
            match parse_query(&self.query) {
                Ok(parsed) => self.parsed_query = Some(parsed),
                Err(error) => return Err(error.into()),
            }
        }

        // forbid_unsafe effectively bans optimize away else branch here so use unwrap
        // but this unwrap never panics
        Ok(self.parsed_query.as_ref().unwrap())
    }

    /// Sets the parsed query into the request.
    ///
    /// This is useful special with dynamic schema when the query has been
    /// parsed ahead of time. It can reduce performance overhead of parsing
    /// the query again.
    pub fn set_parsed_query(&mut self, doc: ExecutableDocument) {
        self.parsed_query = Some(doc);
    }

    /// Set a variable to an upload value.
    ///
    /// `var_path` is a dot-separated path to the item that begins with
    /// `variables`, for example `variables.files.2.content` is equivalent
    /// to the Rust code `request.variables["files"][2]["content"]`. If no
    /// variable exists at the path this function won't do anything.
    pub fn set_upload(&mut self, var_path: &str, upload: UploadValue) {
        fn variable_path<'a>(variables: &'a mut Variables, path: &str) -> Option<&'a mut Value> {
            let mut parts = path.strip_prefix("variables.")?.split('.');

            let initial = variables.get_mut(parts.next().unwrap())?;

            parts.try_fold(initial, |current, part| match current {
                Value::List(list) => part
                    .parse::<u32>()
                    .ok()
                    .and_then(|idx| usize::try_from(idx).ok())
                    .and_then(move |idx| list.get_mut(idx)),
                Value::Object(obj) => obj.get_mut(part),
                _ => None,
            })
        }

        let variable = match variable_path(&mut self.variables, var_path) {
            Some(variable) => variable,
            None => return,
        };
        self.uploads.push(upload);
        *variable = Value::String(format!("#__graphql_file__:{}", self.uploads.len() - 1));
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
            .field("extensions", &self.extensions)
            .finish()
    }
}

/// Batch support for GraphQL requests, which is either a single query, or an
/// array of queries
///
/// **Reference:** <https://www.apollographql.com/blog/batching-client-graphql-queries-a685f5bcd41b/>
#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)] // Request is at fault
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
    /// Fails if the batch request is a list of requests with a message saying
    /// that batch requests aren't supported.
    pub fn into_single(self) -> Result<Request, ParseRequestError> {
        match self {
            Self::Single(req) => Ok(req),
            Self::Batch(_) => Err(ParseRequestError::UnsupportedBatch),
        }
    }

    /// Returns an iterator over the requests.
    pub fn iter(&self) -> impl Iterator<Item = &Request> {
        match self {
            BatchRequest::Single(request) => {
                Box::new(std::iter::once(request)) as Box<dyn Iterator<Item = &Request>>
            }
            BatchRequest::Batch(requests) => Box::new(requests.iter()),
        }
    }

    /// Returns an iterator that allows modifying each request.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Request> {
        match self {
            BatchRequest::Single(request) => {
                Box::new(std::iter::once(request)) as Box<dyn Iterator<Item = &mut Request>>
            }
            BatchRequest::Batch(requests) => Box::new(requests.iter_mut()),
        }
    }

    /// Specify the variables for each requests.
    #[must_use]
    pub fn variables(mut self, variables: Variables) -> Self {
        for request in self.iter_mut() {
            request.variables = variables.clone();
        }
        self
    }

    /// Insert some data for  for each requests.
    #[must_use]
    pub fn data<D: Any + Clone + Send + Sync>(mut self, data: D) -> Self {
        for request in self.iter_mut() {
            request.data.insert(data.clone());
        }
        self
    }

    /// Disable introspection queries for each request.
    #[must_use]
    pub fn disable_introspection(mut self) -> Self {
        for request in self.iter_mut() {
            request.introspection_mode = IntrospectionMode::Disabled;
        }
        self
    }

    /// Only allow introspection queries for each request.
    #[must_use]
    pub fn introspection_only(mut self) -> Self {
        for request in self.iter_mut() {
            request.introspection_mode = IntrospectionMode::IntrospectionOnly;
        }
        self
    }
}

fn deserialize_non_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    use serde::de::Error as _;

    let v = <Vec<T>>::deserialize(deserializer)?;
    if v.is_empty() {
        Err(D::Error::invalid_length(0, &"a non-empty sequence"))
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
    use crate::*;

    #[test]
    fn test_request() {
        let request: Request = from_value(value! ({
            "query": "{ a b c }"
        }))
        .unwrap();
        assert!(request.variables.is_empty());
        assert!(request.operation_name.is_none());
        assert_eq!(request.query, "{ a b c }");
    }

    #[test]
    fn test_request_with_operation_name() {
        let request: Request = from_value(value! ({
            "query": "{ a b c }",
            "operationName": "a"
        }))
        .unwrap();
        assert!(request.variables.is_empty());
        assert_eq!(request.operation_name.as_deref(), Some("a"));
        assert_eq!(request.query, "{ a b c }");
    }

    #[test]
    fn test_request_with_variables() {
        let request: Request = from_value(value! ({
            "query": "{ a b c }",
            "variables": {
                "v1": 100,
                "v2": [1, 2, 3],
                "v3": "str",
            }
        }))
        .unwrap();
        assert_eq!(
            request.variables.into_value(),
            value!({
                "v1": 100,
                "v2": [1, 2, 3],
                "v3": "str",
            })
        );
        assert!(request.operation_name.is_none());
        assert_eq!(request.query, "{ a b c }");
    }

    #[test]
    fn test_deserialize_request_with_empty_object_variables() {
        let request: Request = from_value(value! ({
            "query": "{ a b c }",
            "variables": {}
        }))
        .unwrap();
        assert!(request.operation_name.is_none());
        assert!(request.variables.is_empty());
    }

    #[test]
    fn test_deserialize_request_with_empty_array_variables() {
        let error: DeserializerError = from_value::<Request>(value! ({
            "query": "{ a b c }",
            "variables": []
        }))
        .unwrap_err();
        assert_eq!(error.to_string(), "invalid type: sequence, expected a map");
    }

    #[test]
    fn test_deserialize_request_with_null_variables() {
        let request: Request = from_value(value! ({
            "query": "{ a b c }",
            "variables": null
        }))
        .unwrap();
        assert!(request.operation_name.is_none());
        assert!(request.variables.is_empty());
    }

    #[test]
    fn test_batch_request_single() {
        let request: BatchRequest = from_value(value! ({
            "query": "{ a b c }"
        }))
        .unwrap();

        if let BatchRequest::Single(request) = request {
            assert!(request.variables.is_empty());
            assert!(request.operation_name.is_none());
            assert_eq!(request.query, "{ a b c }");
        } else {
            unreachable!()
        }
    }

    #[test]
    fn test_batch_request_batch() {
        let request: BatchRequest = from_value(value!([
            {
                "query": "{ a b c }"
            },
            {
                "query": "{ d e }"
            }
        ]))
        .unwrap();

        if let BatchRequest::Batch(requests) = request {
            assert!(requests[0].variables.is_empty());
            assert!(requests[0].operation_name.is_none());
            assert_eq!(requests[0].query, "{ a b c }");

            assert!(requests[1].variables.is_empty());
            assert!(requests[1].operation_name.is_none());
            assert_eq!(requests[1].query, "{ d e }");
        } else {
            unreachable!()
        }
    }
}
