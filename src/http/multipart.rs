use std::{
    collections::HashMap,
    io,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{io::AsyncRead, stream::Stream};
use multer::{Constraints, Multipart, SizeLimit};
use pin_project_lite::pin_project;

use crate::{BatchRequest, ParseRequestError, UploadValue};

/// Options for `receive_multipart`.
#[derive(Default, Clone, Copy)]
#[non_exhaustive]
pub struct MultipartOptions {
    /// The maximum file size.
    pub max_file_size: Option<usize>,
    /// The maximum number of files.
    pub max_num_files: Option<usize>,
}

impl MultipartOptions {
    /// Set maximum file size.
    #[must_use]
    pub fn max_file_size(self, size: usize) -> Self {
        MultipartOptions {
            max_file_size: Some(size),
            ..self
        }
    }

    /// Set maximum number of files.
    #[must_use]
    pub fn max_num_files(self, n: usize) -> Self {
        MultipartOptions {
            max_num_files: Some(n),
            ..self
        }
    }
}

pub(super) async fn receive_batch_multipart(
    body: impl AsyncRead + Send,
    boundary: impl Into<String>,
    opts: MultipartOptions,
) -> Result<BatchRequest, ParseRequestError> {
    let mut multipart = Multipart::with_constraints(
        ReaderStream::new(body),
        boundary,
        Constraints::new().size_limit({
            let mut limit = SizeLimit::new();
            if let (Some(max_file_size), Some(max_num_files)) =
                (opts.max_file_size, opts.max_num_files)
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

    while let Some(field) = multipart.next_field().await? {
        // in multipart, each field / file can actually have a own Content-Type.
        // We use this to determine the encoding of the graphql query
        let content_type = field
            .content_type()
            // default to json
            .unwrap_or(&mime::APPLICATION_JSON)
            .clone();
        match field.name() {
            Some("operations") => {
                let body = field.bytes().await?;
                request = Some(
                    super::receive_batch_body_no_multipart(&content_type, body.as_ref()).await?,
                )
            }
            Some("map") => {
                let map_bytes = field.bytes().await?;

                match (content_type.type_(), content_type.subtype()) {
                    // cbor is in application/octet-stream.
                    // TODO: wait for mime to add application/cbor and match against that too
                    // Note: we actually differ here from the inoffical spec for this:
                    // (https://github.com/jaydenseric/graphql-multipart-request-spec#multipart-form-field-structure)
                    // It says: "map: A JSON encoded map of where files occurred in the operations.
                    // For each file, the key is the file multipart form field name and the value is
                    // an array of operations paths." However, I think, that
                    // since we accept CBOR as operation, which is valid, we should also accept it
                    // as the mapping for the files.
                    #[cfg(feature = "cbor")]
                    (mime::OCTET_STREAM, _) | (mime::APPLICATION, mime::OCTET_STREAM) => {
                        map = Some(
                            serde_cbor::from_slice::<HashMap<String, Vec<String>>>(&map_bytes)
                                .map_err(|e| ParseRequestError::InvalidFilesMap(Box::new(e)))?,
                        );
                    }
                    // default to json
                    _ => {
                        map = Some(
                            serde_json::from_slice::<HashMap<String, Vec<String>>>(&map_bytes)
                                .map_err(|e| ParseRequestError::InvalidFilesMap(Box::new(e)))?,
                        );
                    }
                }
            }
            _ => {
                if let Some(name) = field.name().map(ToString::to_string) {
                    if let Some(filename) = field.file_name().map(ToString::to_string) {
                        let content_type = field.content_type().map(ToString::to_string);

                        #[cfg(feature = "tempfile")]
                        let content = {
                            use std::io::{Seek, Write};

                            let mut field = field;
                            let mut file = tempfile::tempfile().map_err(ParseRequestError::Io)?;
                            while let Some(chunk) = field.chunk().await? {
                                file.write(&chunk).map_err(ParseRequestError::Io)?;
                            }
                            file.rewind()?;
                            file
                        };

                        #[cfg(not(feature = "tempfile"))]
                        let content = field.bytes().await?;

                        files.push((name, filename, content_type, content));
                    }
                }
            }
        }
    }

    let mut request: BatchRequest = request.ok_or(ParseRequestError::MissingOperatorsPart)?;
    let map = map.as_mut().ok_or(ParseRequestError::MissingMapPart)?;

    for (name, filename, content_type, file) in files {
        if let Some(var_paths) = map.remove(&name) {
            let upload = UploadValue {
                filename,
                content_type,
                content: file,
            };

            for var_path in var_paths {
                match &mut request {
                    BatchRequest::Single(request) => {
                        request.set_upload(&var_path, upload.try_clone()?);
                    }
                    BatchRequest::Batch(requests) => {
                        let mut s = var_path.splitn(2, '.');
                        let idx = s.next().and_then(|idx| idx.parse::<usize>().ok());
                        let path = s.next();

                        if let (Some(idx), Some(path)) = (idx, path) {
                            if let Some(request) = requests.get_mut(idx) {
                                request.set_upload(path, upload.try_clone()?);
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

    Ok(request)
}

pin_project! {
    pub(crate) struct ReaderStream<T> {
        buf: [u8; 2048],
        #[pin]
        reader: T,
    }
}

impl<T> ReaderStream<T> {
    pub(crate) fn new(reader: T) -> Self {
        Self {
            buf: [0; 2048],
            reader,
        }
    }
}

impl<T: AsyncRead> Stream for ReaderStream<T> {
    type Item = io::Result<Vec<u8>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();

        Poll::Ready(
            match futures_util::ready!(this.reader.poll_read(cx, this.buf)?) {
                0 => None,
                size => Some(Ok(this.buf[..size].to_vec())),
            },
        )
    }
}
