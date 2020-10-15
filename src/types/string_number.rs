use std::fmt::Display;

use serde::{Deserialize, Serialize};
use num_traits::Num;

use crate::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

/// A numeric value represented by a string.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "string_number")))]
pub struct StringNumber<T: Num + Display>(pub T);

#[Scalar(internal)]
impl<T: Num + Display + Send + Sync> ScalarType for StringNumber<T>
where
    <T as Num>::FromStrRadixErr: Display,
{
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => {
                let n = T::from_str_radix(&s, 10)
                    .map_err(|err| InputValueError::custom(err.to_string()))?;
                Ok(StringNumber(n))
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::String(_))
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[async_std::test]
    async fn test_string_number() {
        struct Query;

        #[Object(internal)]
        impl Query {
            async fn value(&self, n: StringNumber<i32>) -> StringNumber<i32> {
                n
            }
        }

        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        assert_eq!(
            schema
                .execute(
                    r#"{
                    value1: value(n: "100")
                    value2: value(n: "-100")
                    value3: value(n: "0")
                    value4: value(n: "1")
                }"#
                )
                .await
                .into_result()
                .unwrap()
                .data,
            value!({
                "value1": "100",
                "value2": "-100",
                "value3": "0",
                "value4": "1",
            })
        );
    }
}
