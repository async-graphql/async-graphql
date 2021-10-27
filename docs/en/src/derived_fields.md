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

So you wouldn't be able to generate derived fields for existing wrapper type structures like `Vec` or `Option`. But when you implement a `From<U> for T` you should be able to derived a `From<Vec<U>> for Vec<T>` and a `From<Option<U>> for Option<T>`, so a coercion mecanism has been included so you'll be able to use the derived macro argument with `Vec` and `Option`.

This coercion mecanism impose these derived to be `owned`.

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

#[derive(SimpleObject)]
struct TestObj {
    #[graphql(derived(owned, name = "value2", into = "Option<ValueDerived2>"))]
    pub value1: Option<ValueDerived>,
    #[graphql(derived(owned, name = "value_vec_2", into = "Vec<ValueDerived2>"))]
    pub value_vec_1: Vec<ValueDerived>,
    #[graphql(derived(owned, name = "value_opt_vec_2", into = "Option<Vec<ValueDerived2>>"))]
    pub value_opt_vec_1: Option<Vec<ValueDerived>>,
    #[graphql(derived(owned, name = "value_vec_opt_2", into = "Vec<Option<ValueDerived2>>"))]
    pub value_vec_opt_1: Vec<Option<ValueDerived>>,
}
```
