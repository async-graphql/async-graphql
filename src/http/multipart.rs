use futures::stream::{self, Stream};
use futures::io::AsyncRead;
use crate::{GQLQuery, ParseRequestError};
use std::collections::HashMap;
use multer::{Multipart, Constraints, SizeLimit};
use std::io::{self, SeekFrom};
use std::task::Poll;
use bytes::Bytes;
use std::pin::Pin;

/// Options for `receive_multipart`.
#[derive(Default, Clone)]
#[non_exhaustive]
pub struct MultipartOptions {
    /// Maximum file size.
    pub max_file_size: Option<usize>,

    /// Maximum number of files.
    pub max_num_files: Option<usize>,
}

/// Receive a multipart request.
pub async fn receive_multipart(
    body: impl AsyncRead,
    boundary: impl Into<String>,
    opts: MultipartOptions,
) -> Result<GQLQuery, ParseRequestError> {
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
                query = Some(GQLQuery::new_with_http_request(
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
