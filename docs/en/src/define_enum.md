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
    #[graphql(name="AAA")]
    Jedi,
}
```

## Wrapping a remote enum

Rust's [orphan rule](https://doc.rust-lang.org/book/traits.html#rules-for-implementing-traits) requires that either the 
trait or the type for which you are implementing the trait must be defined in the same crate as the impl, so you cannot 
expose remote enumeration types to GraphQL. In order to provide an `Enum` type, a common workaround is to create a new 
enum that has parity with the existing, remote enum type.

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

The process is tedious and requires multiple steps to keep the local and remote enums in sync. `Async_graphql` provides a handy feature to generate the `From<remote_crate::RemoteEnum> for LocalEnum` as well as an opposite direction of `From<LocalEnum> for remote_crate::RemoteEnum` via an additional attribute after deriving `Enum`:

```rust
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(remote = "remote_crate::RemoteEnum")]
enum LocalEnum {
    A,
    B,
    C,
}
```
