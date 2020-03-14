use actix_multipart::Multipart;
use actix_web::http::{header, HeaderMap};
use actix_web::web::Payload;
use actix_web::{web, FromRequest, HttpRequest, HttpResponse, Responder};
use async_graphql::http::{GQLRequest, GQLResponse};
use async_graphql::{GQLObject, Schema};
use futures::StreamExt;
use mime::Mime;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub struct HandlerBuilder<Query, Mutation> {
    schema: Schema<Query, Mutation>,
    max_file_size: Option<usize>,
}

impl<Query, Mutation> HandlerBuilder<Query, Mutation>
where
    Query: GQLObject + Send + Sync + 'static,
    Mutation: GQLObject + Send + Sync + 'static,
{
    pub fn new(schema: Schema<Query, Mutation>) -> Self {
        Self {
            schema,
            max_file_size: Some(1024 * 1024 * 2),
        }
    }

    pub fn max_file_size(self, size: usize) -> Self {
        Self {
            max_file_size: Some(size),
            ..self
        }
    }

    pub fn build(
        self,
    ) -> impl Fn(
        HttpRequest,
        Payload,
    ) -> Pin<Box<dyn Future<Output = actix_web::Result<HttpResponse>>>>
           + 'static
           + Clone {
        let schema = Arc::new(self.schema);
        let max_file_size = self.max_file_size;

        move |req: HttpRequest, mut payload: Payload| {
            let schema = schema.clone();
            Box::pin(async move {
                if req.method() != "POST" {
                    return Ok(HttpResponse::MethodNotAllowed().finish());
                }

                if let Ok(ct) = get_content_type(req.headers()) {
                    if ct.essence_str() == mime::MULTIPART_FORM_DATA {
                        let mut multipart = Multipart::from_request(&req, &mut payload.0).await?;

                        // read operators
                        let mut gql_request = {
                            let data = read_multipart(&mut multipart, "operations").await?;
                            serde_json::from_slice::<GQLRequest>(&data)
                                .map_err(|err| actix_web::error::ErrorBadRequest(err))?
                        };

                        // read map
                        let mut map = {
                            let data = read_multipart(&mut multipart, "map").await?;
                            serde_json::from_slice::<HashMap<String, Vec<String>>>(&data)
                                .map_err(|err| actix_web::error::ErrorBadRequest(err))?
                        };

                        let mut query = match gql_request.prepare(&schema) {
                            Ok(query) => query,
                            Err(err) => {
                                return Ok(web::Json(GQLResponse(Err(err)))
                                    .respond_to(&req)
                                    .await?)
                            }
                        };

                        if !query.is_upload() {
                            return Err(actix_web::error::ErrorBadRequest(
                                "It's not an upload operation",
                            ));
                        }

                        // read files
                        while let Some(field) = multipart.next().await {
                            let mut field = field?;
                            if let Some(content_disposition) = field.content_disposition() {
                                if let (Some(name), Some(filename)) = (
                                    content_disposition.get_name(),
                                    content_disposition.get_filename(),
                                ) {
                                    if let Some(var_paths) = map.remove(name) {
                                        let content_type = field.content_type().to_string();
                                        let mut data = Vec::<u8>::new();
                                        while let Some(part) = field.next().await {
                                            let part = part.map_err(|err| {
                                                actix_web::error::ErrorBadRequest(err)
                                            })?;
                                            data.extend(&part);

                                            if let Some(max_file_size) = max_file_size {
                                                if data.len() > max_file_size {
                                                    return Err(
                                                        actix_web::error::ErrorPayloadTooLarge(
                                                            "payload to large",
                                                        ),
                                                    );
                                                }
                                            }
                                        }

                                        for var_path in var_paths {
                                            query.set_upload(
                                                &var_path,
                                                filename,
                                                Some(&content_type),
                                                data.clone(),
                                            );
                                        }
                                    } else {
                                        return Err(actix_web::error::ErrorBadRequest(
                                            "bad request",
                                        ));
                                    }
                                } else {
                                    return Err(actix_web::error::ErrorBadRequest("bad request"));
                                }
                            } else {
                                return Err(actix_web::error::ErrorBadRequest("bad request"));
                            }
                        }

                        if !map.is_empty() {
                            return Err(actix_web::error::ErrorBadRequest("missing files"));
                        }

                        Ok(web::Json(GQLResponse(query.execute().await))
                            .respond_to(&req)
                            .await?)
                    } else if ct.essence_str() == mime::APPLICATION_JSON {
                        let gql_req =
                            web::Json::<GQLRequest>::from_request(&req, &mut payload.0).await?;
                        Ok(web::Json(gql_req.into_inner().execute(&schema).await)
                            .respond_to(&req)
                            .await?)
                    } else {
                        Ok(HttpResponse::UnsupportedMediaType().finish())
                    }
                } else {
                    Ok(HttpResponse::UnsupportedMediaType().finish())
                }
            })
        }
    }
}

fn get_content_type(headers: &HeaderMap) -> actix_web::Result<Mime> {
    if let Some(content_type) = headers.get(&header::CONTENT_TYPE) {
        if let Ok(content_type) = content_type.to_str() {
            if let Ok(ct) = content_type.parse::<Mime>() {
                return Ok(ct);
            }
        }
    }
    Err(actix_web::error::ErrorUnsupportedMediaType(
        "unsupported media type",
    ))
}

async fn read_multipart(multipart: &mut Multipart, name: &str) -> actix_web::Result<Vec<u8>> {
    let data = match multipart.next().await {
        Some(Ok(mut field)) => {
            if let Some(content_disposition) = field.content_disposition() {
                if let Some(current_name) = content_disposition.get_name() {
                    if current_name != name {
                        return Err(actix_web::error::ErrorBadRequest(format!(
                            "expect \"{}\"",
                            name
                        )));
                    }

                    let mut data = Vec::<u8>::new();
                    while let Some(part) = field.next().await {
                        let part = part.map_err(|err| actix_web::error::ErrorBadRequest(err))?;
                        data.extend(&part);
                    }
                    data
                } else {
                    return Err(actix_web::error::ErrorBadRequest("missing \"operations\""));
                }
            } else {
                return Err(actix_web::error::ErrorBadRequest("bad request"));
            }
        }
        Some(Err(err)) => return Err(err.into()),
        None => return Err(actix_web::error::ErrorBadRequest("bad request")),
    };
    Ok(data)
}
