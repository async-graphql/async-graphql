# 查询和变更

## 查询根对象

查询根对象是一个 GraphQL 对象，定义类似其它对象。查询对象的所有字段 Resolver 函数是并发执行的。

```rust
# extern crate async_graphql;
use async_graphql::*;
# #[derive(SimpleObject)]
# struct User { a: i32 }

struct Query;

#[Object]
impl Query {
    async fn user(&self, username: String) -> Result<Option<User>> {
        // 在数据库中查找用户
#        todo!()
    }
}

```

## 变更根对象

变更根对象也是一个 GraphQL，但变更根对象的执行是顺序的，只有第一个变更执行完成之后才会执行下一个。

下面的变更根对象提供用户注册和登录操作：

```rust
# extern crate async_graphql;
use async_graphql::*;

struct Mutation;

#[Object]
impl Mutation {
    async fn signup(&self, username: String, password: String) -> Result<bool> {
        // 用户注册
#        todo!()
    }

    async fn login(&self, username: String, password: String) -> Result<String> {
        // 用户登录并生成 token
#        todo!()
    }
}
```
