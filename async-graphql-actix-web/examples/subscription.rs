use actix_web::{web, App, HttpServer};
use async_graphql::{publish, Context, Result, Schema, ID};
use futures::lock::Mutex;
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

#[async_graphql::Object(cache_control(max_age = 5))]
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
        publish(BookChanged {
            mutation_type: MutationType::Created,
            id: id.clone(),
        })
        .await;
        id
    }

    #[field]
    async fn delete_book(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let mut books = ctx.data::<Storage>().lock().await;
        let id = id.parse::<usize>()?;
        if books.contains(id) {
            books.remove(id);
            publish(BookChanged {
                mutation_type: MutationType::Deleted,
                id: id.into(),
            })
            .await;
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

struct BookChanged {
    mutation_type: MutationType,
    id: ID,
}

#[async_graphql::Object]
impl BookChanged {
    #[field]
    async fn mutation_type(&self) -> &MutationType {
        &self.mutation_type
    }

    #[field]
    async fn id(&self) -> &ID {
        &self.id
    }
}

struct SubscriptionRoot;

#[async_graphql::Subscription]
impl SubscriptionRoot {
    #[field]
    fn books(&self, changed: &BookChanged, mutation_type: Option<MutationType>) -> bool {
        if let Some(mutation_type) = mutation_type {
            return changed.mutation_type == mutation_type;
        }
        true
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
