use crate::{ParseRequestError, Request};
use bytes::Bytes;
use futures::io::AsyncRead;
use futures::stream::{self, Stream};
use multer::{Constraints, Multipart, SizeLimit};
use std::collections::HashMap;
use std::io::{self, Seek, SeekFrom, Write};
use std::pin::Pin;
use std::task::Poll;

/// Options for `receive_multipart`.
#[derive(Default, Clone)]
pub struct MultipartOptions {
    max_file_size: Option<usize>,
    max_num_files: Option<usize>,
}

impl MultipartOptions {
    /// Set maximum file size.
    pub fn max_file_size(self, size: usize) -> Self {
        MultipartOptions {
            max_file_size: Some(size),
            ..self
        }
    }

    /// Set maximum number of files.
    pub fn max_num_files(self, n: usize) -> Self {
        MultipartOptions {
            max_num_files: Some(n),
            ..self
        }
    }
}

/// Receive a multipart request.
pub async fn receive_multipart(
    body: impl AsyncRead + Send + 'static,
    boundary: impl Into<String>,
    opts: MultipartOptions,
) -> Result<Request, ParseRequestError> {
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
                request = Some(
                    serde_json::from_str::<Request>(&request_str)
                        .map_err(ParseRequestError::InvalidRequest)?,
                );
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
                        let content_type = field.content_type().map(|mime| mime.to_string());
                        let mut file = tempfile::tempfile().map_err(ParseRequestError::Io)?;
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

    let mut request: Request = request.ok_or(ParseRequestError::MissingOperatorsPart)?;
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
}

fn reader_stream(
    reader: impl AsyncRead + Send + 'static,
) -> impl Stream<Item = io::Result<Bytes>> + Send + 'static {
    let mut buf = [0u8; 2048];
    let mut reader = Box::pin(reader);
    stream::poll_fn(move |cx| {
        Poll::Ready(
            match futures::ready!(Pin::new(&mut reader).poll_read(cx, &mut buf)?) {
                0 => None,
                size => Some(Ok(Bytes::copy_from_slice(&buf[..size]))),
            },
        )
    })
}
