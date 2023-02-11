# 输入值校验器

`Async-graphql`内置了一些常用的校验器，你可以在对象字段的参数或者`InputObject`的字段上使用它们。

- **maximum=N** 指定数字不能大于`N`
- **minimum=N** 指定数字不能小于`N`
- **multiple_of=N** 指定数字必须是`N`的倍数
- **max_items=N** 指定列表的长度不能大于`N`
- **min_items=N** 指定列表的长度不能小于`N`
- **max_length=N** 字符串的长度不能大于`N`
- **min_length=N** 字符串的长度不能小于`N`
- **chars_max_length=N** 字符串中 unicode 字符的的数量不能小于`N`
- **chars_min_length=N** 字符串中 unicode 字符的的数量不能大于`N`
- **email** 有效的 email
- **url** 有效的 url
- **ip** 有效的 ip 地址
- **regex=RE** 匹配正则表达式

```rust
# extern crate async_graphql;
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    /// name 参数的长度必须大于等于 5，小于等于 10
    async fn input(&self, #[graphql(validator(min_length = 5, max_length = 10))] name: String) -> Result<i32> {
#        todo!()    
    }
}
```

## 校验列表成员

你可以打开`list`属性表示校验器作用于一个列表内的所有成员：

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

## 自定义校验器

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
    /// n 的值必须等于 100
    async fn value(
        &self,
        #[graphql(validator(custom = "MyValidator::new(100)"))] n: i32,
    ) -> i32 {
        n
    }
}
```
