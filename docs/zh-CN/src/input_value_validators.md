# 输入值校验器

字段的参数，[输入对象(InputObject)](define_input_object.md)的字段在GraphQL中称之为`Input Value`，如果给出的参数值类型不匹配，查询会报类型不匹配错误。但有时候我们希望给特定类型的值更多的限制，例如我们希望一个参数总是一个格式合法的Email地址，我们可以通过[自定义标量](custom_scalars.md)来解决，但这种类型如果很多，而且我们希望能够进行自由组合，比如一个`String`类型参数，我们希望它既可以是Email地址，也可以是一个手机号码，自定义标量就特别麻烦。`Async-graphql`提供了输入值校验器来解决这个问题。

输入值校验器可以通过`and`和`or`操作符自由组合。

下面时一个参数校验器，它校验`String`类型的参数`a`，必须是一个合法Email或者MAC地址：

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

下面的例子校验`i32`类型的参数`a`必须大于10，并且小于100，或者等于0：

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

## 自定义校验器

```rust
struct MustBeZero {}

impl InputValueValidator for MustBeZero {
    fn is_valid(&self, value: &Value) -> Result<(), String> {
        if let Value::Int(n) = value {
            if n.as_i64().unwrap() != 0 {
                // 校验失败
                Err(format!(
                    "the value is {}, but must be zero",
                    n.as_i64().unwrap(),
                ))
            } else {
                // 校验通过
                Ok(())
            }
        } else {
            // 类型不匹配，直接返回None，内置校验器会发现这个错误
            Ok(())
        }
    }
}
```
