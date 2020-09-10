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

/// Options for `GQLQuery::receive_multipart`
#[derive(Default, Clone)]
pub struct ReceiveMultipartOptions {
    /// Maximum file size.
    pub max_file_size: Option<usize>,

    /// Maximum number of files.
    pub max_num_files: Option<usize>,
}

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

    pub async fn receive_request(
        content_type: Option<impl AsRef<str>>,
        mut body: impl AsyncRead + Send + Unpin + 'static,
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

            let mut request = None;
            let mut map = None;
            let mut files = Vec::new();

            while let Some(mut field) = multipart.next_field().await? {
                match field.name() {
                    Some("operations") => {
                        let request_str = field.text().await?;
                        request = Some(Self::new_with_http_request(
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

            let mut request = request.ok_or(ParseRequestError::MissingOperatorsPart)?;
            let map = map.as_mut().ok_or(ParseRequestError::MissingMapPart)?;

            for (name, filename, content_type, file) in files {
                if let Some(var_paths) = map.remove(&name) {
                    for var_path in var_paths {
                        request.set_upload(
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

            Ok(request)
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
