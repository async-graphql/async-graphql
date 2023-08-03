# Derived fields

Sometimes two fields have the same query logic, but the output type is different. In `async-graphql`, you can create a derived field for it.

In the following example, you already have a `date_rfc2822` field outputting the time format in `RFC2822` format, and then reuse it to derive a new `date_rfc3339` field.

```rust
# extern crate chrono;
# use chrono::Utc;
# extern crate async_graphql;
# use async_graphql::*;
struct DateRFC3339(chrono::DateTime<Utc>);
struct DateRFC2822(chrono::DateTime<Utc>);

#[Scalar]
impl ScalarType for DateRFC3339 {
  fn parse(value: Value) -> InputValueResult<Self> { todo!() } 

  fn to_value(&self) -> Value {
    Value::String(self.0.to_rfc3339())
  }
}

#[Scalar]
impl ScalarType for DateRFC2822 {
  fn parse(value: Value) -> InputValueResult<Self> { todo!() } 

  fn to_value(&self) -> Value {
    Value::String(self.0.to_rfc2822())
  }
}

impl From<DateRFC2822> for DateRFC3339 {
    fn from(value: DateRFC2822) -> Self {
      DateRFC3339(value.0)
    }
}

struct Query;

#[Object]
impl Query {
    #[graphql(derived(name = "date_rfc3339", into = "DateRFC3339"))]
    async fn date_rfc2822(&self, arg: String) -> DateRFC2822 {
        todo!()
    }
}
```

It will render a GraphQL like:

```graphql
type Query {
	date_rfc2822(arg: String): DateRFC2822!
	date_rfc3339(arg: String): DateRFC3339!
}
```

## Wrapper types

A derived field won't be able to manage everything easily: Rust's [orphan rule](https://doc.rust-lang.org/book/traits.html#rules-for-implementing-traits) requires that either the
trait or the type for which you are implementing the trait must be defined in the same crate as the impl, so the following code cannot be compiled:

```rust,ignore
impl From<Vec<U>> for Vec<T> {
  ...
}
```

So you wouldn't be able to generate derived fields for existing wrapper type structures like `Vec` or `Option`. But when you implement a `From<U> for T` you should be able to derived a `From<Vec<U>> for Vec<T>` and a `From<Option<U>> for Option<T>`.
We included a `with` parameter to help you define a function to call instead of using the `Into` trait implementation between wrapper structures.


### Example

```rust
# extern crate serde;
# use serde::{Serialize, Deserialize};
# extern crate async_graphql;
# use async_graphql::*;
#[derive(Serialize, Deserialize, Clone)]
struct ValueDerived(String);

#[derive(Serialize, Deserialize, Clone)]
struct ValueDerived2(String);

scalar!(ValueDerived);
scalar!(ValueDerived2);

impl From<ValueDerived> for ValueDerived2 {
    fn from(value: ValueDerived) -> Self {
        ValueDerived2(value.0)
    }
}

fn option_to_option<T, U: From<T>>(value: Option<T>) -> Option<U> {
    value.map(|x| x.into())
}

#[derive(SimpleObject)]
struct TestObj {
    #[graphql(derived(owned, name = "value2", into = "Option<ValueDerived2>", with = "option_to_option"))]
    pub value1: Option<ValueDerived>,
}
```
