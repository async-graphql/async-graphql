# 枚举(Enum)

定义枚举相当简单，直接给出一个例子。

**Async-graphql会自动把枚举项的名称转换为GraphQL标准的大写加下划线形式，你也可以用`name`属性自已定义名称。**

```rust
use async_graphql::*;

/// One of the films in the Star Wars Trilogy
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
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

## 封装外部枚举类型

Rust的 [孤儿规则](https://doc.rust-lang.org/book/traits.html#rules-for-implementing-traits) 要求特质或您要实现特质的类型必须在相同的板条箱中定义，因此你不能向GraphQL公开外部枚举类型。 为了创建`Enum`类型，一种常见的解决方法是创建一个新的与现有远程枚举类型同等的枚举。

```rust
use async_graphql::*;

/// Provides parity with a remote enum type
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum LocalEnum {
    A,
    B,
    C,
}

/// Conversion interface from remote type to our local GraphQL enum type
impl From<remote_crate::RemoteEnum> for LocalEnum {
    fn from(e: remote_crate::RemoteEnum) -> Self {
        match e {
            remote_crate::RemoteEnum::A => Self::A,
            remote_crate::RemoteEnum::B => Self::B,
            remote_crate::RemoteEnum::C => Self::C,
        }
    }
}
```

该过程很繁琐，需要多个步骤才能使本地枚举和远程枚举保持同步。`Async_graphql`提供了一个方便的功能，可在派生`Enum`之后通过附加属性生成LocalEnum的`From <remote_crate::RemoteEnum>`以及相反的`From<LocalEnum> for remote_crate::RemoteEnum`:

```rust
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(remote = "remote_crate::RemoteEnum")]
enum LocalEnum {
    A,
    B,
    C,
}
```
