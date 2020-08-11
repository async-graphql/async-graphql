# 合并对象(MergedObject)

## 为同一类型实现多次Object

通常我们在Rust中可以为同一类型创建多个实现，但由于过程宏的限制，无法为同一个类型创建多个Object实现。例如，下面的代码将无法通过编译。

```rust
#[Object]
impl MyObject {
    async fn field1(&self) -> i32 {
        todo!()
    }
}

#[Object]
impl MyObject {
    async fn field2(&self) -> i32 {
        todo!()    
    }
}
```

用 `#[derive(GQLMergedObject)]` 宏允许你合并多个独立的GQLObject为一个.

**提示:** 每个`#[Object]`需要一个唯一的名称，即使在一个`GQLMergedObject`内，所以确保每个对象有单独的名称。

```rust
#[Object]
impl UserQuery {
    async fn users(&self) -> Vec<User> {
        todo!()
    }
}

#[Object]
impl MovieQuery {
    async fn movies(&self) -> Vec<Movie> {
        todo!()
    }
}

#[derive(GQLMergedObject, Default)]
struct Query(UserQuery, MovieQuery);

let schema = Schema::new(
    Query::default(),
    EmptyMutation,
    EmptySubscription
);
```
