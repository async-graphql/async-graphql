use actix::clock::Duration;
use actix_web::{web, App, HttpServer};
use async_graphql::{Context, FieldResult, Schema, SimpleBroker, ID};
use futures::lock::Mutex;
use futures::{Stream, StreamExt};
use slab::Slab;
use std::sync::Arc;

#[derive(Clone)]
struct Book {
    id: ID,
    name: String,
    author: String,
}

#[async_graphql::Object]
impl Book {
    #[field]
    async fn id(&self) -> &str {
        &self.id
    }

    #[field]
    async fn name(&self) -> &str {
        &self.name
    }

    #[field]
    async fn author(&self) -> &str {
        &self.author
    }
}

type Storage = Arc<Mutex<Slab<Book>>>;

struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    #[field]
    async fn books(&self, ctx: &Context<'_>) -> Vec<Book> {
        let books = ctx.data::<Storage>().lock().await;
        books.iter().map(|(_, book)| book).cloned().collect()
    }
}

struct MutationRoot;

#[async_graphql::Object]
impl MutationRoot {
    #[field]
    async fn create_book(&self, ctx: &Context<'_>, name: String, author: String) -> ID {
        let mut books = ctx.data::<Storage>().lock().await;
        let entry = books.vacant_entry();
        let id: ID = entry.key().into();
        let book = Book {
            id: id.clone(),
            name,
            author,
        };
        entry.insert(book);
        SimpleBroker::publish(BookChanged {
            mutation_type: MutationType::Created,
            id: id.clone(),
        });
        id
    }

    #[field]
    async fn delete_book(&self, ctx: &Context<'_>, id: ID) -> FieldResult<bool> {
        let mut books = ctx.data::<Storage>().lock().await;
        let id = id.parse::<usize>()?;
        if books.contains(id) {
            books.remove(id);
            SimpleBroker::publish(BookChanged {
                mutation_type: MutationType::Deleted,
                id: id.into(),
            });
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[async_graphql::Enum]
enum MutationType {
    Created,
    Deleted,
}

#[async_graphql::SimpleObject]
#[derive(Clone)]
struct BookChanged {
    #[field]
    mutation_type: MutationType,

    #[field]
    id: ID,
}

struct SubscriptionRoot;

#[async_graphql::Subscription]
impl SubscriptionRoot {
    #[field]
    async fn interval(&self, n: i32) -> impl Stream<Item = i32> {
        let mut value = 0;
        actix_rt::time::interval(Duration::from_secs(1)).map(move |_| {
            value += n;
            value
        })
    }

    #[field]
    async fn books(&self, mutation_type: Option<MutationType>) -> impl Stream<Item = BookChanged> {
        SimpleBroker::<BookChanged>::subscribe().filter(move |event| {
            let res = if let Some(mutation_type) = mutation_type {
                event.mutation_type == mutation_type
            } else {
                true
            };
            futures::future::ready(res)
        })
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        let schema = Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
            .data(Storage::default())
            .finish();
        let handler = async_graphql_actix_web::HandlerBuilder::new(schema)
            .enable_ui("http://localhost:8000", Some("ws://localhost:8000"))
            .enable_subscription()
            .build();
        App::new().service(web::resource("/").to(handler))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
