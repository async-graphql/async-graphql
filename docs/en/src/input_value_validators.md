# Input value validators

`Async-graphql` has some common validators built-in, you can use them on the parameters of object fields or on the fields of `InputObject`.

- **maximum=N** the number cannot be greater than `N`.
- **minimum=N** the number cannot be less than `N`.
- **multiple_of=N** the number must be a multiple of `N`.
- **max_items=N** the length of the list cannot be greater than `N`.
- **min_items=N** the length of the list cannot be less than `N`.
- **max_length=N** the length of the string cannot be greater than `N`.
- **min_length=N** the length of the string cannot be less than `N`.
- **chars_max_length=N** the count of the unicode chars cannot be greater than `N`.
- **chars_min_length=N** the count of the unicode chars cannot be less than `N`.
- **email** is valid email.
- **url** is valid url.
- **ip** is valid ip address.
- **regex=RE** is match for the regex.
- **uuid=V** the string or ID is a valid UUID with version `V`. You may omit `V` to accept any UUID version. 

```rust
# extern crate async_graphql;
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    /// The length of the name must be greater than or equal to 5 and less than or equal to 10.
    async fn input(&self, #[graphql(validator(min_length = 5, max_length = 10))] name: String) -> Result<i32> {
#         todo!()
    }
}
```

## Check every member of the list

You can enable the `list` attribute, and the validator will check all members in list:

```rust
# extern crate async_graphql;
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    async fn input(&self, #[graphql(validator(list, max_length = 10))] names: Vec<String>) -> Result<i32> {
#        todo!()
    }
}
```

## Custom validator

```rust
# extern crate async_graphql;
# use async_graphql::*;
struct MyValidator {
    expect: i32,
}

impl MyValidator {
    pub fn new(n: i32) -> Self {
        MyValidator { expect: n }
    }
}

impl CustomValidator<i32> for MyValidator {
    fn check(&self, value: &i32) -> Result<(), InputValueError<i32>> {
        if *value == self.expect {
            Ok(())
        } else {
            Err(InputValueError::custom(format!("expect 100, actual {}", value)))
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
