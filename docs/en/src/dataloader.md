# Optimizing N+1 queries

Have you noticed some GraphQL queries end can make hundreds of database queries, often with mostly repeated data? Lets take a look why and how to fix it.

## Query Resolution

Imagine if you have a simple query like this:

```graphql
query {
  todos {
    users {
      name
    }
  }
}
```

and `User` resolver is like this:

```rust,ignore
struct User {
    id: u64,
}

#[Object]
impl User {
    async fn name(&self, ctx: &Context<'_>) -> Result<String> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let (name,): (String,) = sqlx::query_as("SELECT name FROM user WHERE id = $1")
            .bind(self.id)
            .fetch_one(pool)
            .await?;
        Ok(name)
    }
}
```

The query executor will call the `Todos` resolver which does a `select * from todo and return N todos`. Then for each
of the todos, concurrently, call the `User` resolver, `SELECT from USER where id = todo.user_id`.

eg：

```sql
SELECT id, todo, user_id FROM todo
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
```

After executing `SELECT name FROM user WHERE id = $1` many times, and most `Todo` objects belong to the same user, we
need to optimize these codes!

## Dataloader

We need to group queries and exclude duplicate queries. `Dataloader` can do this.
[facebook](https://github.com/facebook/dataloader) gives a request-scope batch and caching solution.

The following is a simplified example of using `DataLoader` to optimize queries, there is also a [full code example available in GitHub](https://github.com/async-graphql/examples/tree/master/tide/dataloader-postgres).

```rust,ignore
use async_graphql::*;
use async_graphql::dataloader::*;
use std::sync::Arc;

struct UserNameLoader {
    pool: sqlx::PgPool,
}

#[async_trait::async_trait]
impl Loader<u64> for UserNameLoader {
    type Value = String;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[u64]) -> Result<HashMap<u64, Self::Value>, Self::Error> {
        Ok(sqlx::query_as("SELECT name FROM user WHERE id = ANY($1)")
            .bind(keys)
            .fetch(&self.pool)
            .map_ok(|name: String| name)
            .map_err(Arc::new)
            .try_collect().await?)
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
struct User {
    id: u64,
}

#[ComplexObject]
impl User {
    async fn name(&self, ctx: &Context<'_>) -> Result<String> {
        let loader = ctx.data_unchecked::<DataLoader<UserNameLoader>>();
        let name: Option<String> = loader.load_one(self.id).await?;
        name.ok_or_else(|| "Not found".into())
    }
}
```

To expose `UserNameLoader` in the `ctx`, you have to register it with the schema, along with a task spawner, e.g. `async_std::task::spawn`:

```rust,ignore
let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
    .data(DataLoader::new(
        UserNameLoader,
        async_std::task::spawn, // or `tokio::spawn`
    ))
    .finish();
```

In the end, only two SQLs are needed to query the results we want!

```sql
SELECT id, todo, user_id FROM todo
SELECT name FROM user WHERE id IN (1, 2, 3, 4)
```

## Implement multiple data types

You can implement multiple data types for the same `Loader`, like this:

```rust,ignore
# extern crate async_graphql;
# use async_graphql::*;
struct PostgresLoader {
    pool: sqlx::PgPool,
}

#[async_trait::async_trait]
impl Loader<UserId> for PostgresLoader {
    type Value = User;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[UserId]) -> Result<HashMap<UserId, Self::Value>, Self::Error> {
        // Load users from database
    }
}

#[async_trait::async_trait]
impl Loader<TodoId> for PostgresLoader {
    type Value = Todo;
    type Error = sqlx::Error;

    async fn load(&self, keys: &[TodoId]) -> Result<HashMap<TodoId, Self::Value>, Self::Error> {
        // Load todos from database
    }
}
```
