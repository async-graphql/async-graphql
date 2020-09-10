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

/// Options for `GQLQuery::receive_multipart`
#[derive(Default, Clone)]
pub struct ReceiveMultipartOptions {
    /// Maximum file size.
    pub max_file_size: Option<usize>,

    /// Maximum number of files.
    pub max_num_files: Option<usize>,
}

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

    pub async fn receive_request(
        content_type: Option<impl AsRef<str>>,
        mut body: impl AsyncRead,
        opts: ReceiveMultipartOptions,
    ) -> std::result::Result<Self, ParseRequestError> {
        if let Some(boundary) = content_type.and_then(|ct| multer::parse_boundary(ct).ok()) {
            // multipart
            let mut multipart = Multipart::new_with_constraints(
                reader_stream(body),
                boundary,
                Constraints::new().size_limit({
                    let mut limit = SizeLimit::new();
                    if let (Some(max_file_size), Some(max_num_files)) =
                        (opts.max_file_size, opts.max_file_size)
                    {
                        limit = limit.whole_stream((max_file_size * max_num_files) as u64);
                    }
                    if let Some(max_file_size) = opts.max_file_size {
                        limit = limit.per_field(max_file_size as u64);
                    }
                    limit
                }),
            );

            let mut query = None;
            let mut map = None;
            let mut files = Vec::new();

            while let Some(mut field) = multipart.next_field().await? {
                match field.name() {
                    Some("operations") => {
                        let request_str = field.text().await?;
                        query = Some(Self::new_with_http_request(
                            serde_json::from_str(&request_str)
                                .map_err(ParseRequestError::InvalidRequest)?,
                        ));
                    }
                    Some("map") => {
                        let map_str = field.text().await?;
                        map = Some(
                            serde_json::from_str::<HashMap<String, Vec<String>>>(&map_str)
                                .map_err(ParseRequestError::InvalidFilesMap)?,
                        );
                    }
                    _ => {
                        if let Some(name) = field.name().map(ToString::to_string) {
                            if let Some(filename) = field.file_name().map(ToString::to_string) {
                                let content_type =
                                    field.content_type().map(|mime| mime.to_string());
                                let mut file =
                                    tempfile::tempfile().map_err(ParseRequestError::Io)?;
                                while let Some(chunk) = field.chunk().await.unwrap() {
                                    file.write(&chunk).map_err(ParseRequestError::Io)?;
                                }
                                file.seek(SeekFrom::Start(0))?;
                                files.push((name, filename, content_type, file));
                            }
                        }
                    }
                }
            }

            let mut query = query.ok_or(ParseRequestError::MissingOperatorsPart)?;
            let map = map.as_mut().ok_or(ParseRequestError::MissingMapPart)?;

            for (name, filename, content_type, file) in files {
                if let Some(var_paths) = map.remove(&name) {
                    for var_path in var_paths {
                        query.set_upload(
                            &var_path,
                            filename.clone(),
                            content_type.clone(),
                            file.try_clone().unwrap(),
                        );
                    }
                }
            }

            if !map.is_empty() {
                return Err(ParseRequestError::MissingFiles);
            }

            Ok(query)
        } else {
            let mut data = Vec::new();
            body.read_to_end(&mut data)
                .await
                .map_err(ParseRequestError::Io)?;
            Ok(Self::new_with_http_request(
                serde_json::from_slice(&data).map_err(ParseRequestError::InvalidRequest)?,
            ))
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

fn reader_stream(
    mut reader: impl AsyncRead + Unpin + Send + 'static,
) -> impl Stream<Item = io::Result<Bytes>> + Unpin + Send + 'static {
    let mut buf = [0u8; 2048];

    stream::poll_fn(move |cx| {
        Poll::Ready(
            match futures::ready!(Pin::new(&mut reader).poll_read(cx, &mut buf)?) {
                0 => None,
                size => Some(Ok(Bytes::copy_from_slice(&buf[..size]))),
            },
        )
    })
}
