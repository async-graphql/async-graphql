#[cfg(feature = "multipart")]
use crate::http::{multipart::ReaderStream, MultipartOptions};
use crate::{BatchRequest, ParseRequestError};
use futures::{AsyncRead, AsyncReadExt};
#[cfg(feature = "multipart")]
use multer::{Constraints, Multipart, SizeLimit};
#[cfg(feature = "multipart")]
use std::{
    collections::HashMap,
    io::{Seek, SeekFrom, Write},
};

#[cfg(feature = "multipart")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "multipart")))]
/// Receive a GraphQL batch request from a content type and body.
pub async fn receive_batch_body(
    content_type: Option<impl AsRef<str>>,
    body: impl AsyncRead + Send + 'static,
    opts: MultipartOptions,
) -> Result<BatchRequest, ParseRequestError> {
    if let Some(Ok(boundary)) = content_type.map(multer::parse_boundary) {
        receive_batch_multipart(body, boundary, opts).await
    } else {
        receive_batch_json(body).await
    }
}

/// Receive a GraphQL batch request from a body as JSON.
pub async fn receive_batch_json(
    body: impl AsyncRead + Send + 'static,
) -> Result<BatchRequest, ParseRequestError> {
    let mut data = Vec::new();
    futures::pin_mut!(body);
    body.read_to_end(&mut data)
        .await
        .map_err(ParseRequestError::Io)?;
    Ok(serde_json::from_slice::<BatchRequest>(&data).map_err(ParseRequestError::InvalidRequest)?)
}

#[cfg(feature = "multipart")]
async fn receive_batch_multipart(
    body: impl AsyncRead + Send + 'static,
    boundary: impl Into<String>,
    opts: MultipartOptions,
) -> Result<BatchRequest, ParseRequestError> {
    let mut multipart = Multipart::new_with_constraints(
        ReaderStream::new(body),
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

    let mut batch_request = None;
    let mut map = None;
    let mut files = Vec::new();

    while let Some(mut field) = multipart.next_field().await? {
        match field.name() {
            Some("operations") => {
                let request_str = field.text().await?;
                batch_request = Some(
                    serde_json::from_str::<BatchRequest>(&request_str)
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

    let mut batch_request: BatchRequest =
        batch_request.ok_or(ParseRequestError::MissingOperatorsPart)?;
    let map = map.as_mut().ok_or(ParseRequestError::MissingMapPart)?;

    for (name, filename, content_type, file) in files {
        if let Some(var_paths) = map.remove(&name) {
            for var_path in var_paths {
                match &mut batch_request {
                    BatchRequest::Single(request) => {
                        request.set_upload(
                            &var_path,
                            filename.clone(),
                            content_type.clone(),
                            file.try_clone().unwrap(),
                        );
                    }
                    BatchRequest::Batch(requests) => {
                        let mut s = var_path.splitn(2, '.');
                        let idx = s.next().and_then(|idx| idx.parse::<usize>().ok());
                        let path = s.next();

                        if let (Some(idx), Some(path)) = (idx, path) {
                            if let Some(request) = requests.get_mut(idx) {
                                request.set_upload(
                                    path,
                                    filename.clone(),
                                    content_type.clone(),
                                    file.try_clone().unwrap(),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    if !map.is_empty() {
        return Err(ParseRequestError::MissingFiles);
    }

    Ok(batch_request)
}
