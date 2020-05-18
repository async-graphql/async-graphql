use crate::http::GQLRequest;
use crate::query::{IntoQueryBuilder, IntoQueryBuilderOpts};
use crate::{ParseRequestError, QueryBuilder};
use bytes::Bytes;
use futures::{AsyncRead, AsyncReadExt, Stream};
use mime::Mime;
use multer::{Constraints, Multipart, SizeLimit};
use std::collections::HashMap;
use std::io::{Seek, SeekFrom, Write};

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
        if let Some(boundary) = self
            .0
            .and_then(|value| value.as_ref().parse::<Mime>().ok())
            .and_then(|ct| {
                if ct.essence_str() == mime::MULTIPART_FORM_DATA {
                    ct.get_param("boundary")
                        .map(|boundary| boundary.to_string())
                } else {
                    None
                }
            })
        {
            // multipart
            let stream = reader_stream(self.1);

            let mut multipart = Multipart::new_with_constraints(
                stream,
                boundary,
                Constraints::new().size_limit({
                    let mut limit = SizeLimit::new();
                    if let (Some(max_file_size), Some(max_num_files)) =
                        (opts.max_file_size, opts.max_file_size)
                    {
                        limit = limit.whole_stream(max_file_size * max_num_files);
                    }
                    if let Some(max_file_size) = opts.max_file_size {
                        limit = limit.per_field(max_file_size);
                    }
                    limit
                }),
            );

            let mut builder = None;
            let mut map = None;

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
                        let builder = match &mut builder {
                            Some(builder) => builder,
                            None => return Err(ParseRequestError::MissingOperatorsPart),
                        };
                        let map = match &mut map {
                            Some(map) => map,
                            None => return Err(ParseRequestError::MissingMapPart),
                        };
                        if let Some(name) = field.name() {
                            if let Some(filename) = field.file_name().map(ToString::to_string) {
                                if let Some(var_paths) = map.remove(name) {
                                    let content_type =
                                        field.content_type().map(|mime| mime.to_string());
                                    let mut file =
                                        tempfile::tempfile().map_err(ParseRequestError::Io)?;
                                    while let Some(chunk) = field.chunk().await.unwrap() {
                                        println!("{:?}", chunk);
                                        file.write(&chunk).map_err(ParseRequestError::Io)?;
                                    }
                                    file.seek(SeekFrom::Start(0))?;
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
                        }
                    }
                }
            }

            if let Some(map) = &map {
                if !map.is_empty() {
                    return Err(ParseRequestError::MissingFiles);
                }
            } else {
                return Err(ParseRequestError::MissingMapPart);
            }

            Ok(match builder {
                Some(builder) => builder,
                None => return Err(ParseRequestError::MissingOperatorsPart),
            })
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
    mut r: impl AsyncRead + Send + Unpin + 'static,
) -> impl Stream<Item = std::io::Result<Bytes>> + 'static {
    async_stream::try_stream! {
        let mut buf = [0u8; 2048];
        while let size = r.read(&mut buf[..]).await? {
            if size == 0 {
                return;
            }
            yield Bytes::from(buf[..size].to_vec());
        }
    }
}
