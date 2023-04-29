# 优化查询（解决 N+1 问题）

您是否注意到某些 GraphQL 查询需要执行数百个数据库查询，这些查询通常包含重复的数据，让我们来看看为什么以及如何修复它。

## 查询解析

想象一下，如果您有一个简单的查询，例如：

```graphql
query { todos { users { name } } }
```

实现`User`的 resolver 代码如下：

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

执行查询将调用`Todos`的 resolver，该 resolver 执行`SELECT * FROM todo`并返回 N 个`Todo`对象。然后对每个`Todo`对象同时调用`User`的
resolver 执行`SELECT name FROM user where id = $1`。

例如：

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

执行了多次`SELECT name FROM user WHERE id = $1`，并且，大多数`Todo`对象都属于同一个用户，我们需要优化这些代码！

## Dataloader

我们需要对查询分组，并且排除重复的查询。`Dataloader`就能完成这个工作，[facebook](https://github.com/facebook/dataloader) 给出了一个请求范围的批处理和缓存解决方案。

下面是使用`DataLoader`来优化查询请求的例子：

```rust,ignore
use async_graphql::*;
use async_graphql::dataloader::*;
use itertools::Itertools;
use std::sync::Arc;

struct UserNameLoader {
    pool: sqlx::Pool<Postgres>,
}

#[async_trait::async_trait]
impl Loader<u64> for UserNameLoader {
    type Value = String;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[u64]) -> Result<HashMap<u64, Self::Value>, Self::Error> {
        let query = format!("SELECT name FROM user WHERE id IN ({})", keys.iter().join(","));
        Ok(sqlx::query_as(query)
            .fetch(&self.pool)
            .map_ok(|name: String| name)
            .map_err(Arc::new)
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

要在 `ctx` 中获取 `UserNameLoader`，您必须将其和任务生成器（例如 `async_std::task::spawn`）注册到 `Schema` 中：

```rust,ignore
let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
    .data(DataLoader::new(
        UserNameLoader,
        async_std::task::spawn, // 或者 `tokio::spawn`
    ))
    .finish();
```

最终只需要两个查询语句，就查询出了我们想要的结果！

```sql
SELECT id, todo, user_id FROM todo
SELECT name FROM user WHERE id IN (1, 2, 3, 4)
```

## 同一个 Loader 支持多种数据类型

你可以为同一个`Loader`实现多种数据类型，就像下面这样：

```rust,ignore
struct PostgresLoader {
    pool: sqlx::Pool<Postgres>,
}

#[async_trait::async_trait]
impl Loader<UserId> for PostgresLoader {
    type Value = User;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[UserId]) -> Result<HashMap<UserId, Self::Value>, Self::Error> {
        // 从数据库中加载 User
    }
}

#[async_trait::async_trait]
impl Loader<TodoId> for PostgresLoader {
    type Value = Todo;
    type Error = sqlx::Error;

    async fn load(&self, keys: &[TodoId]) -> Result<HashMap<TodoId, Self::Value>, Self::Error> {
        // 从数据库中加载 Todo
    }
}
```
