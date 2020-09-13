# 枚举(Enum)

定义枚举相当简单，直接给出一个例子。

**Async-graphql会自动把枚举项的名称转换为GraphQL标准的大写加下划线形式，你也可以用`name`属性自已定义名称。**

```rust
use async_graphql::*;

/// One of the films in the Star Wars Trilogy
#[derive(GQLEnum, Copy, Clone, Eq, PartialEq)]
pub enum Episode {
    /// Released in 1977.
    NewHope,

    /// Released in 1980.
    Empire,

    /// Released in 1983.
    #[item(name="AAA")]
    Jedi,
}
```