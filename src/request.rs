use crate::parser::types::UploadValue;
use crate::{http, Data, ParseRequestError, Value, Variables};
use bytes::Bytes;
use futures::stream;
use futures::task::Poll;
use futures::{AsyncRead, AsyncReadExt, Stream};
use multer::{Constraints, Multipart, SizeLimit};
use std::any::Any;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Seek, SeekFrom, Write};
use std::pin::Pin;

pub struct Request {
    pub(crate) query: String,
    pub(crate) operation_name: Option<String>,
    pub(crate) variables: Variables,
    pub(crate) ctx_data: Data,
}

impl Request {
    /// Create a request object with query source.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            operation_name: None,
            variables: Variables::default(),
            ctx_data: Data::default(),
        }
    }

    pub fn new_with_http_request(request: http::GQLRequest) -> Self {
        Self {
            query: request.query,
            operation_name: request.operation_name,
            variables: request
                .variables
                .map(|value| Variables::parse_from_json(value))
                .unwrap_or_default(),
            ctx_data: Data::default(),
        }
    }

    /// Specify the operation name.
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

    /// Add a context data that can be accessed in the `Context`, you access it with `Context::data`.
    ///
    /// **This data is only valid for this query**
    pub fn data<D: Any + Send + Sync>(mut self, data: D) -> Self {
        self.ctx_data.insert(data);
        self
    }

    /// Set uploaded file path
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

impl From<http::GQLRequest> for Request {
    fn from(request: http::GQLRequest) -> Self {
        Self::new_with_http_request(request)
    }
}
