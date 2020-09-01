# Input value validators


Arguments to a query ([InputObject](define_input_object.md)) are called `Input Objects` in GraphQL. If the provided input type does not match for a query, the query will return a type mismatch error. But sometimes we want to provide more restrictions on specific types of values. For example, we might want to require that an argument is a valid email address. `Async-graphql` provides an input validators to solve this problem.

An input validator can be combined via `and` and `or` operators.

The following is an input validator which checks that a `String` is a valid Email or MAC address:


```rust
use async_graphql::*;
use async_graphql::validators::{Email, MAC};

struct Query;

#[Object]
impl Query {
    async fn input(#[arg(validator(or(Email, MAC(colon = "false"))))] a: String) {
    }
}
```

The following example verifies that the `i32` parameter `a` is greater than 10 and less than 100, or else equal to 0:

```rust
use async_graphql:*;
use async_graphql::validators::{IntGreaterThan, IntLessThan, IntEqual};

struct Query;

#[Object]
impl Query {
    async fn input(#[validator(
        or(
            and(IntGreaterThan(value = "10"), IntLessThan(value = "100")),
            IntEqual(value = "0")
        ))] a: String) {
    } {
    }
}
```

## Custom validator

Here is an example of a custom validator:

```rust
struct MustBeZero {}

impl InputValueValidator for MustBeZero {
    fn is_valid(&self, value: &Value) -> Result<(), String> {
        if let Value::Int(n) = value {
            if n.as_i64().unwrap() != 0 {
                // Validation failed
                Err(format!(
                    "the value is {}, but must be zero",
                    n.as_i64().unwrap(),
                ))
            } else {
                // Validation succeeded
                Ok(())
            }
        } else {
            // If the type does not match we can return None and built-in validations
            // will pick up on the error
            Ok(())
        }
    }
}
```
