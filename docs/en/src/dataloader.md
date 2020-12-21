# Optimizing N+1 queries

Have you noticed some GraphQL queries end can make hundreds of database queries, often with mostly repeated data? Lets take a look why and how to fix it.

## Query Resolution

Imagine if you have a simple query like this:

```graphql
query { todos { users { name } } }
```

and `User` resolver is like this:

```rust
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

The following is an example of using `DataLoader` to optimize queries::

```rust
use async_graphql::*;
use async_graphql::dataloader::*;
use itertools::Itertools;

struct UserNameLoader {
    pool: sqlx::Pool<Postgres>,
}

#[async_trait::async_trait]
impl Loader for UserNameLoader {
    type Key = u64;
    type Value = String;
    type Error = sqlx::Error;
    
    async fn load(&self, keys: HashSet<Self::Key>) -> Result<HashMap<Self::Key, Self::Value>, Self::Error> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let query = format!("SELECT name FROM user WHERE id IN ({})", keys.iter().join(","));
        Ok(sqlx::query_as(query)
            .fetch(&self.pool)
            .map_ok(|name: String| name)
            .try_collect().await?)
    }
}

struct User {
    id: u64,
}

#[Object]
impl User {
    async fn name(&self, ctx: &Context<'_>) -> Result<String> {
        let loader = ctx.data_unchecked::<DataLoader<UserNameLoader>>();
        let name: Option<String> = loader.load_one(self.id).await?;
        name.ok_or_else(|| "Not found".into())
    }
}
```

In the end, only two SQLs are needed to query the results we want!

```sql
SELECT id, todo, user_id FROM todo
SELECT name FROM user WHERE id IN (1, 2, 3, 4)
```
