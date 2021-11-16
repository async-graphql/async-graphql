# Input value validators

`Async-graphql` has some common validators built-in, you can use them on the parameters of object fields or on the fields of `InputObject`.

- **maximum=N** the number cannot be greater than `N`.
- **minimum=N** the number cannot be less than `N`.
- **multiple_of=N** the number must be a multiple of `N`.
- **max_items=N** the length of the list cannot be greater than `N`.
- **min_items=N** the length of the list cannot be less than `N`.
- **max_length=N** the length of the string cannot be greater than `N`.
- **min_length=N** the length of the string cannot be less than `N`.

```rust
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    /// The length of the name must be greater than or equal to 5 and less than or equal to 10.
    async fn input(#[graphql(validator(min_length = 5, max_length = 10))] name: String) {
    }
}
```

## Check every member of the list

You can enable the `list` attribute, and the validator will check all members in list:

```rust
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    async fn input(#[graphql(validator(list, max_length = 10))] names: Vec<String>) {
    }
}
```

## Custom validator

```rust
struct MyValidator {
    expect: i32,
}

impl MyValidator {
    pub fn new(n: i32) -> Self {
        MyValidator { expect: n }
    }
}

#[async_trait::async_trait]
impl CustomValidator<i32> for MyValidator {
    async fn check(&self, _ctx: &Context<'_>, value: &i32) -> Result<(), String> {
        if *value == self.expect {
            Ok(())
        } else {
            Err(format!("expect 100, actual {}", value))
        }
    }
}

struct Query;

#[Object]
impl Query {
    /// n must be equal to 100
    async fn value(
        &self,
        #[graphql(validator(custom = "MyValidator::new(100)"))] n: i32,
    ) -> i32 {
        n
    }
}
```
