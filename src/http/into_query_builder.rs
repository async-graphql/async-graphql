use crate::http::GQLRequest;
use crate::query::{IntoQueryBuilder, IntoQueryBuilderOpts};
use crate::{ParseRequestError, QueryBuilder};
use bytes::Bytes;
use futures::{stream, AsyncRead, AsyncReadExt, Stream};
use multer::{Constraints, Multipart, SizeLimit};
use std::collections::HashMap;
use std::io::{self, Seek, SeekFrom, Write};
use std::pin::Pin;
use std::task::Poll;

impl From<multer::Error> for ParseRequestError {
    fn from(err: multer::Error) -> Self {
        match err {
            multer::Error::FieldSizeExceeded { .. } | multer::Error::StreamSizeExceeded { .. } => {
                ParseRequestError::PayloadTooLarge
            }
            _ => ParseRequestError::InvalidMultipart(err),
        }
    }
}

#[async_trait::async_trait]
impl<CT, Body> IntoQueryBuilder for (Option<CT>, Body)
where
    CT: AsRef<str> + Send,
    Body: AsyncRead + Send + Unpin + 'static,
{
    async fn into_query_builder_opts(
        mut self,
        opts: &IntoQueryBuilderOpts,
    ) -> std::result::Result<QueryBuilder, ParseRequestError> {
        if let Some(boundary) = self.0.and_then(|ct| multer::parse_boundary(ct).ok()) {
            // multipart
            let mut multipart = Multipart::new_with_constraints(
                reader_stream(self.1),
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

            let mut builder = None;
            let mut map = None;
            let mut files = Vec::new();

            while let Some(mut field) = multipart.next_field().await? {
                match field.name() {
                    Some("operations") => {
                        let request_str = field.text().await?;
                        let request: GQLRequest = serde_json::from_str(&request_str)
                            .map_err(ParseRequestError::InvalidRequest)?;
                        builder = Some(request.into_query_builder().await?);
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

            let mut builder = builder.ok_or(ParseRequestError::MissingOperatorsPart)?;
            let map = map.as_mut().ok_or(ParseRequestError::MissingMapPart)?;

            for (name, filename, content_type, file) in files {
                if let Some(var_paths) = map.remove(&name) {
                    for var_path in var_paths {
                        builder.set_upload(
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

            Ok(builder)
        } else {
            let mut data = Vec::new();
            self.1
                .read_to_end(&mut data)
                .await
                .map_err(ParseRequestError::Io)?;
            let gql_request: GQLRequest =
                serde_json::from_slice(&data).map_err(ParseRequestError::InvalidRequest)?;
            gql_request.into_query_builder().await
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
