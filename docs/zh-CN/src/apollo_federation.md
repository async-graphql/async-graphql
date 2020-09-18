# Apollo Federation集成
 
`Apollo Federation`是一个`GraphQL`网关，它可以组合多个GraphQL服务，允许每服务仅实现它负责的那一部分数据，参考[官方文档](https://www.apollographql.com/docs/apollo-server/federation/introduction)。

`Async-graphql`可以完全支持`Apollo Federation`的所有功能，但需要对`Schema`定义做一些小小的改造。

- `async_graphql::Object`和`async_graphql::Interface`的`extends`属性声明这个类别是一个已有类型的扩充。

- 字段的`external`属性声明这个字段定义来自其它服务。

- 字段的`provides`属性用于要求网关提供的字段集。

- 字段的`provides`属性表示解析该字段值需要依赖该类型的字段集。

## 实体查找函数

```rust
struct Query;

#[Object]
impl Query {
    #[entity]
    async fn find_user_by_id(&self, id: ID) -> User {
        User { ... }
    }

    #[entity]
    async fn find_user_by_id_with_username(&self, #[arg(key)] id: ID, username: String) -> User {
        User { ... }
    }

    #[entity]
    async fn find_user_by_id_and_username(&self, id: ID, username: String) -> User {
        User { ... }
    }
}
```

**注意这三个查找函数的不同，他们都是查找User对象。**

- find_user_by_id

    使用`id`查找`User`对象，`User`对象的key是`id`。

- find_user_by_id_with_username

    使用`id`查找`User`对象，`User`对象的key是`id`，并且请求`User`对象的`username`字段值。

- find_user_by_id_and_username

    使用`id`和`username`查找`User`对象，`User`对象的key是`id`和`username`。

完整的例子请参考https://github.com/async-graphql/examples/tree/master/federation
