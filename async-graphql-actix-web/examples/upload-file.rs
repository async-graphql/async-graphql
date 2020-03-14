use actix_web::{web, App, HttpServer};
use async_graphql::{Schema, Upload};

struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {}

struct MutationRoot;

#[async_graphql::Object]
impl MutationRoot {
    #[field]
    async fn single_upload(&self, file: Upload) -> bool {
        println!(
            "upload: filename={} size={}",
            file.filename,
            file.content.len()
        );
        true
    }

    #[field]
    async fn multiple_upload(&self, files: Vec<Upload>) -> bool {
        for upload in files {
            println!(
                "upload: filename={} size={}",
                upload.filename,
                upload.content.len()
            );
        }
        true
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        let schema = Schema::new(QueryRoot, MutationRoot);
        let handler = async_graphql_actix_web::HandlerBuilder::new(schema).build();
        App::new().service(web::resource("/").to(handler))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
