# Derived fields

When you are working on a GraphQL project, you usually have to explain and share how your scalars should
be interpreted by your consumers. Sometimes, you event want to have the same data and the same logic exposing
the data in another type.

Within `async-graphql` you can create derivated fields for objects to generate derivated fields.

Consider you want to create a `Date` scalar, to represent an event of time.
How will you represent and format this date? You could create a scalar `Date` where you specified it's the RFCXXX
implemented to format it.

With derivated fields there is a simple way to support multiple representation of a `Date` easily:

```rust
struct DateRFC3339(chrono::DateTime);
struct DateRFC2822(chrono::DateTime);

#[Scalar]
impl ScalarType for DateRFC3339 {
  fn parse(value: Value) -> InputValueResult { ... } 

  fn to_value(&self) -> Value {
    Value::String(self.0.to_rfc3339())
  }
}

#[Scalar]
impl ScalarType for DateRFC2822 {
  fn parse(value: Value) -> InputValueResult { ... } 

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
    async fn duration_rfc2822(&self, arg: String) -> DateRFC2822 {
        todo!()
    }
}
```

It will render a GraphQL like:

```graphql
type Query {
	duration_rfc2822(arg: String): DateRFC2822!
	duration_rfc3339(arg: String): DateRFC3339!
}
```
