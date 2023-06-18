# Apollo Federation 集成
 
`Apollo Federation`是一个`GraphQL`网关，它可以组合多个 GraphQL 服务，允许每服务仅实现它负责的那一部分数据，参考[官方文档](https://www.apollographql.com/docs/apollo-server/federation/introduction)。

`Async-graphql`可以完全支持`Apollo Federation`的所有功能，但需要对`Schema`定义做一些小小的改造。

- `async_graphql::Object`和`async_graphql::Interface`的`extends`属性声明这个类别是一个已有类型的扩充。

- 字段的`external`属性声明这个字段定义来自其它服务。

- 字段的`provides`属性用于要求网关提供的字段集。

- 字段的`requires`属性表示解析该字段值需要依赖该类型的字段集。

## 实体查找函数

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[derive(SimpleObject)]
# struct User { id: ID }
struct Query;

#[Object]
impl Query {
    #[graphql(entity)]
    async fn find_user_by_id(&self, id: ID) -> User {
        User { id }
    }

    #[graphql(entity)]
    async fn find_user_by_id_with_username(&self, #[graphql(key)] id: ID, username: String) -> User {
        User { id }
    }

    #[graphql(entity)]
    async fn find_user_by_id_and_username(&self, id: ID, username: String) -> User {
        User { id }
    }
}
```

**注意这三个查找函数的不同，他们都是查找 User 对象。**

- find_user_by_id

    使用`id`查找`User`对象，`User`对象的 key 是`id`。

- find_user_by_id_with_username

    使用`id`查找`User`对象，`User`对象的 key 是`id`，并且请求`User`对象的`username`字段值。

- find_user_by_id_and_username

    使用`id`和`username`查找`User`对象，`User`对象的 key 是`id`和`username`。

完整的例子请参考 https://github.com/async-graphql/examples/tree/master/federation

## 定义复合主键

一个主键可以包含多个字段，什么包含嵌套字段，你可以用`InputObject`来实现一个嵌套字段的 Key 类型。

下面的例子中`User`对象的主键是`key { a b }`。

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[derive(SimpleObject)]
# struct User { id: i32 }
#[derive(InputObject)]
struct NestedKey {
  a: i32,
  b: i32,
}

struct Query;

#[Object]
impl Query {
  #[graphql(entity)]
  async fn find_user_by_key(&self, key: NestedKey) -> User {
    User { id: key.a }
  }
}
```
