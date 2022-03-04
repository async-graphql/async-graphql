# Custom scalars

In `Async-graphql` most common scalar types are built in, but you can also create your own scalar types.

Using `async-graphql::Scalar`, you can add support for a scalar when you implement it. You only need to implement parsing and output functions.

The following example defines a 64-bit integer scalar where its input and output are strings.

```rust
use async_graphql::*;

struct StringNumber(i64);

#[Scalar]
impl ScalarType for StringNumber {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            // Parse the integer value
            Ok(value.parse().map(StringNumber)?)
        } else {
            // If the type does not match
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}
```

## Use `scalar!` macro to define scalar

If your type implemented `serde::Serialize` and `serde::Deserialize`, then you can use this macro to define a scalar more simply.

```rust
#[derive(Serialize, Deserialize)]
struct MyValue {
    a: i32,
    b: HashMap<String, i32>,     
}

scalar!(MyValue);

// Rename to `MV`.
// scalar!(MyValue, "MV");

// Rename to `MV` and add description.
// scalar!(MyValue, "MV", "This is my value");
```
