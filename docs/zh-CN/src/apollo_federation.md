# Apollo Federation集成
 
`Apollo Federation`是一个`GraphQL`网关，它可以组合多个GraphQL服务，允许每服务仅实现它负责的那一部分数据，参考[官方文档](https://www.apollographql.com/docs/apollo-server/federation/introduction)。

`Async-graphql`可以完全支持`Apollo Federation`的所有功能，但需要对`Schema`定义做一些小小的改造。

- `async_graphql::Object`和`async_graphql::Interface`的`extends`属性声明这个类别是一个已有类型的扩充。

- 字段的`external`属性声明这个字段定义来自其它服务。

- 字段的`provides`属性用于要求网关提供的字段集。

- 字段的`provides`属性表示解析该字段值需要依赖该类型的字段集。

类型的key属性定义稍有不同，必须在查询根类型上定义一个实体查找函数。

类似下面这样

```rust
struct Query;

#[Object]
impl Query {
    #[entity]
    async fn find_user_by_id(&self, id: ID) -> User {
        User { id }
    }
}
```

这相当于

```graphql
type User @key(id: ID!) {
    id: ID!,
}
```

你必须在这个实体查找函数中根据key来创建对应的对象。

完整的例子请参考https://github.com/async-graphql/examples/tree/master/federation
