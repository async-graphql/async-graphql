use crate::http::multipart::{Multipart, PartData};
use crate::http::GQLRequest;
use crate::query::IntoQueryBuilder;
use crate::{ParseRequestError, QueryBuilder};
use futures::{AsyncRead, AsyncReadExt};
use mime::Mime;
use std::collections::HashMap;

#[async_trait::async_trait]
impl<CT, Body> IntoQueryBuilder for (Option<CT>, Body)
where
    CT: AsRef<str> + Send,
    Body: AsyncRead + Send + Unpin,
{
    async fn into_query_builder(mut self) -> std::result::Result<QueryBuilder, ParseRequestError> {
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
            let mut multipart = Multipart::parse(self.1, boundary.as_str())
                .await
                .map_err(ParseRequestError::InvalidMultipart)?;
            let gql_request: GQLRequest = {
                let part = multipart
                    .remove("operations")
                    .ok_or_else(|| ParseRequestError::MissingOperatorsPart)?;
                let reader = part.create_reader().map_err(ParseRequestError::PartData)?;
                serde_json::from_reader(reader).map_err(ParseRequestError::InvalidRequest)?
            };
            let mut map: HashMap<String, Vec<String>> = {
                let part = multipart
                    .remove("map")
                    .ok_or_else(|| ParseRequestError::MissingMapPart)?;
                let reader = part.create_reader().map_err(ParseRequestError::PartData)?;
                serde_json::from_reader(reader).map_err(ParseRequestError::InvalidFilesMap)?
            };

            let mut builder = gql_request.into_query_builder().await?;

            // read files
            for part in &multipart.parts {
                if let Some(name) = &part.name {
                    if let Some(var_paths) = map.remove(name) {
                        for var_path in var_paths {
                            if let (Some(filename), PartData::File(path)) =
                                (&part.filename, &part.data)
                            {
                                builder.set_upload(
                                    &var_path,
                                    &filename,
                                    part.content_type.as_deref(),
                                    path,
                                );
                            }
                        }
                    }
                }
            }

            if !map.is_empty() {
                return Err(ParseRequestError::MissingFiles);
            }

            if let Some(temp_dir) = multipart.temp_dir {
                builder.set_files_holder(temp_dir);
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
