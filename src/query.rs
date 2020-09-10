use crate::context::{Data, ResolveId};
use crate::extensions::{BoxExtension, ErrorLogger, Extension};
use crate::parser::types::{OperationType, UploadValue};
use crate::{
    do_resolve, http, CacheControl, ContextBase, Error, ObjectType, ParseRequestError, Pos,
    QueryEnv, QueryError, Result, Schema, SubscriptionType, Value, Variables,
};
use bytes::Bytes;
use futures::stream;
use futures::task::Poll;
use futures::{AsyncRead, AsyncReadExt, Stream};
use multer::{Constraints, Multipart, SizeLimit};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};
use std::any::Any;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{Seek, SeekFrom, Write};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

pub struct GQLQuery {
    pub(crate) query: String,
    pub(crate) operation_name: Option<String>,
    pub(crate) variables: Variables,
    pub(crate) ctx_data: Data,
    pub(crate) extensions: Vec<Box<dyn Fn() -> BoxExtension + Send + Sync>>,
}

impl GQLQuery {
    /// Create a query with query source.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            operation_name: None,
            variables: Variables::default(),
            ctx_data: Data::default(),
            extensions: Vec::default(),
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
            extensions: Vec::default(),
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

    /// Add an extension
    pub fn extension<F: Fn() -> E + Send + Sync + 'static, E: Extension>(
        mut self,
        extension_factory: F,
    ) -> Self {
        self.extensions
            .push(Box::new(move || Box::new(extension_factory())));
        self
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

impl<T: Into<String>> From<T> for GQLQuery {
    fn from(query: T) -> Self {
        Self::new(query)
    }
}

impl From<http::GQLRequest> for GQLQuery {
    fn from(request: http::GQLRequest) -> Self {
        Self::new_with_http_request(request)
    }
}

/// Query response
#[derive(Debug)]
pub struct GQLQueryResponse {
    /// Data of query result
    pub data: serde_json::Value,

    /// Extensions result
    pub extensions: Option<serde_json::Value>,

    /// Cache control value
    pub cache_control: CacheControl,

    /// Error
    pub error: Option<Error>,
}

impl GQLQueryResponse {
    #[inline]
    pub fn is_err(&self) -> bool {
        self.error.is_some()
    }

    #[inline]
    pub fn unwrap_err(self) -> Error {
        self.error.unwrap()
    }
}

impl From<Error> for GQLQueryResponse {
    fn from(err: Error) -> Self {
        Self {
            data: serde_json::Value::Null,
            extensions: None,
            cache_control: CacheControl::default(),
            error: Some(err),
        }
    }
}
