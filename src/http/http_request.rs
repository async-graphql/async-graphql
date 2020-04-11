use crate::error::RequestError;
use crate::http::multipart::{Multipart, PartData};
use crate::http::{GQLRequest, IntoQueryBuilder};
use crate::{Error, ObjectType, QueryBuilder, Result, Schema, SubscriptionType};
use futures::{AsyncRead, AsyncReadExt};
use mime::Mime;
use std::collections::HashMap;

/// Http request trait for GraphQL
#[allow(missing_docs)]
pub trait GQLHttpRequest {
    type Body: AsyncRead + Send + Unpin;

    fn content_type(&self) -> Option<&str>;

    fn into_body(self) -> Self::Body;
}

#[async_trait::async_trait]
impl<R: GQLHttpRequest + Send + Sync> IntoQueryBuilder for R {
    async fn into_query_builder<Query, Mutation, Subscription>(
        mut self,
        schema: &Schema<Query, Mutation, Subscription>,
    ) -> Result<QueryBuilder<Query, Mutation, Subscription>>
    where
        Query: ObjectType + Send + Sync + 'static,
        Mutation: ObjectType + Send + Sync + 'static,
        Subscription: SubscriptionType + Send + Sync + 'static,
    {
        if let Some(boundary) = self
            .content_type()
            .and_then(|value| value.parse::<Mime>().ok())
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
            let mut multipart = Multipart::parse(self.into_body(), boundary.as_str())
                .await
                .map_err(RequestError::InvalidMultipart)?;
            let gql_request: GQLRequest = {
                let part = multipart
                    .remove("operations")
                    .ok_or_else(|| Error::Request(RequestError::MissingOperatorsPart))?;
                let reader = part
                    .create_reader()
                    .map_err(|err| Error::Request(RequestError::PartData(err)))?;
                serde_json::from_reader(reader).map_err(RequestError::InvalidRequest)?
            };
            let mut map: HashMap<String, Vec<String>> = {
                let part = multipart
                    .remove("map")
                    .ok_or_else(|| Error::Request(RequestError::MissingMapPart))?;
                let reader = part
                    .create_reader()
                    .map_err(|err| Error::Request(RequestError::PartData(err)))?;
                serde_json::from_reader(reader).map_err(RequestError::InvalidFilesMap)?
            };

            let mut builder = gql_request.into_query_builder(schema).await?;
            if !builder.is_upload() {
                return Err(RequestError::NotUpload.into());
            }

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
                return Err(RequestError::MissingFiles.into());
            }

            if let Some(temp_dir) = multipart.temp_dir {
                builder.set_files_holder(temp_dir);
            }

            Ok(builder)
        } else {
            let mut data = Vec::new();
            self.into_body()
                .read_to_end(&mut data)
                .await
                .map_err(RequestError::Io)?;
            let gql_request: GQLRequest =
                serde_json::from_slice(&data).map_err(RequestError::InvalidRequest)?;
            gql_request.into_query_builder(schema).await
        }
    }
}
