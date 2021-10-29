# Derived fields

When you are working on a GraphQL project, you usually have to explain and share how your scalars should
be interpreted by your consumers. Sometimes, you event want to have the same data and the same logic exposing
the data in another type.

Within `async-graphql` you can create derived fields for objects to generate derived fields.

Consider you want to create a `Date` scalar, to represent an event of time.
How will you represent and format this date? You could create a scalar `Date` where you specified it's the RFCXXX
implemented to format it.

With derived fields there is a simple way to support multiple representation of a `Date` easily:

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

## Wrapper types

A derived field won't be able to manage everythings easily: without the specialization from the Rust language, you won't be able to implement specialized trait like:
```
impl From<Vec<U>> for Vec<T> {
  ...
}
```

So you wouldn't be able to generate derived fields for existing wrapper type structures like `Vec` or `Option`. But when you implement a `From<U> for T` you should be able to derived a `From<Vec<U>> for Vec<T>` and a `From<Option<U>> for Option<T>`.
We included a `with` parameter to help you define a function to call instead of using the `Into` trait implementation between wrapper structures.


### Example

```rust
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
