# 派生字段

有时两个字段有一样的查询逻辑，仅仅是输出的类型不同，在 `async-graphql` 中，你可以为它创建派生字段。

在以下例子中，你已经有一个`duration_rfc2822`字段输出`RFC2822`格式的时间格式，然后复用它派生一个新的`date_rfc3339`字段。

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
    async fn duration_rfc2822(&self, arg: String) -> DateRFC2822 {
        todo!()
    }
}
```

它将呈现为如下 GraphQL：

```graphql
type Query {
	duration_rfc2822(arg: String): DateRFC2822!
	duration_rfc3339(arg: String): DateRFC3339!
}
```

## 包装类型

因为 [孤儿规则](https://doc.rust-lang.org/book/traits.html#rules-for-implementing-traits)，以下代码无法通过编译：

```rust,ignore
impl From<Vec<U>> for Vec<T> {
  ...
}
```

因此，你将无法为现有的包装类型结构（如`Vec`或`Option`）生成派生字段。 
但是当你为 `T` 实现了 `From<U>` 后，你可以为 `Vec<T>` 实现 `From<Vec<U>>`，为 `Option<T>` 实现 `From<Option<U>>`.
使用 `with` 参数来定义一个转换函数，而不是用 `Into::into`。

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
