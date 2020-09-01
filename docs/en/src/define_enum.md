# Enum

It's easy to define an `Enum`, here we have an example:

**Async-graphql will automatically change the name of each item to GraphQL's CONSTANT_CASE convention. You can use `name` to rename.**

```rust
use async_graphql::*;

#[Enum(desc = "One of the films in the Star Wars Trilogy")]
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
