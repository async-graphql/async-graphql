# Enum

It's easy to define an `Enum`, here we have an example:

**Async-graphql can automatically change the name of each item to GraphQL's CONSTANT_CASE convension, you can also use `name` to rename.**

```rust
use async_graphql::*;

#[GqlEnum(desc = "One of the films in the Star Wars Trilogy")]
pub enum Episode {
    #[item(desc = "Released in 1977.")]
    NewHope,

    #[item(desc = "Released in 1980.")]
    Empire,

    // rename to `AAA`
    #[item(name="AAA", desc = "Released in 1983.")]
    Jedi,
}
```