# 查询的深度和复杂度

⚠️GraphQL 提供了非常灵活的查询方法，但在客户端上滥用复杂的查询可能造成风险，限制查询语句的深度和复杂度可以减轻这种风险。

## 昂贵的查询

考虑一种允许列出博客文章的架构。每个博客帖子也与其他帖子相关。

```graphql
type Query {
	posts(count: Int = 10): [Post!]!
}

type Post {
	title: String!
	text: String!
	related(count: Int = 10): [Post!]!
}
```

创建一个会引起很大响应的查询不是很困难：

```graphql
{
    posts(count: 100) {
        related(count: 100) {
            related(count: 100) {
                related(count: 100) {
                    title
                }
            }
        }
    }
}
```

响应的大小随`related`字段的每个其他级别呈指数增长。幸运的是，`Async-graphql`提供了一种防止此类查询的方法。

## 限制查询的深度

查询的深度是字段嵌套的层数，下面是一个深度为`3`的查询。

```graphql
{
    a {
        b {
            c
        }
    }
}
```

在创建`Schema`的时候可以限制深度，如果查询语句超过这个限制，则会出错并且返回`Query is nested too deep.`消息。

```rust
# extern crate async_graphql;
# use async_graphql::*;
# struct Query;
# #[Object]
# impl Query { async fn version(&self) -> &str { "1.0" } }
let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
    .limit_depth(5) // 限制最大深度为 5
    .finish();
```

## 限制查询的复杂度

复杂度是查询语句中字段的数量，每个字段的复杂度默认为`1`，下面是一个复杂度为`6`的查询。

```graphql
{
    a b c {
        d {
            e f
        }
    }
}
```

在创建`Schema`的时候可以限制复杂度，如果查询语句超过这个限制，则会出错并且返回`Query is too complex.`。

```rust
# extern crate async_graphql;
# use async_graphql::*;
# struct Query;
# #[Object]
# impl Query { async fn version(&self) -> &str { "1.0" } }
let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
    .limit_complexity(5) // 限制最大深度为 5
    .finish();
```

## 自定义字段的复杂度

针对非列表类型和列表类型的字段，有两种自定义复杂度的方法。
下面的代码中，`value`字段的复杂度为`5`。而`values`字段的复杂度为`count * child_complexity`，`child_complexity`是一个特殊的变量，表示子查询的复杂度，
`count`是字段的参数，这个表达式用于计算`values`字段的复杂度，并且返回值的类型必须是`usize`。

```rust
# extern crate async_graphql;
# use async_graphql::*;
struct Query;

#[Object]
impl Query {
    #[graphql(complexity = 5)]
    async fn value(&self) -> i32 {
        todo!()
    }

    #[graphql(complexity = "count * child_complexity")]
    async fn values(&self, count: usize) -> i32 {
        todo!()
    }
}
```

**注意：计算复杂度是在验证阶段完成而不是在执行阶段，所以你不用担心超限的查询语句会导致查询只执行一部分。**
