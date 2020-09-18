# Enum

It's easy to define an `Enum`, here we have an example:

**Async-graphql will automatically change the name of each item to GraphQL's CONSTANT_CASE convention. You can use `name` to rename.**

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
