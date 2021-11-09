use std::borrow::Cow;
use std::ops::{Deref, DerefMut};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::parser::types::Field;
use crate::registry::{MetaType, Registry};
use crate::{
    from_value, to_value, ContextSelectionSet, InputValueError, InputValueResult, OutputType,
    Positioned, Scalar, ScalarType, ServerResult, Value,
};

/// A scalar that can represent any JSON value.
///
/// If the inner type cannot be serialized as JSON (e.g. it has non-string keys) it will be `null`.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Default)]
#[serde(transparent)]
pub struct Json<T>(pub T);

impl<T> Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Json<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: DeserializeOwned + Serialize> From<T> for Json<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

/// A scalar that can represent any JSON value.
#[Scalar(internal, name = "JSON")]
impl<T: DeserializeOwned + Serialize + Send + Sync> ScalarType for Json<T> {
    fn parse(value: Value) -> InputValueResult<Self> {
        Ok(from_value(value)?)
    }

    fn to_value(&self) -> Value {
        to_value(&self.0).unwrap_or_default()
    }
}

/// A `Json` type that only implements `OutputType`.
#[derive(Serialize, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct OutputJson<T>(pub T);

impl<T> Deref for OutputJson<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for OutputJson<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Serialize> From<T> for OutputJson<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

#[async_trait::async_trait]
impl<T: Serialize + Send + Sync> OutputType for OutputJson<T> {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("Json")
    }

    fn create_type_info(registry: &mut Registry) -> String {
        registry.create_output_type::<OutputJson<T>, _>(|_| MetaType::Scalar {
            name: Self::type_name().to_string(),
            description: None,
            is_valid: |_| true,
            visible: None,
            specified_by_url: None,
        })
    }

    async fn resolve(
        &self,
        _ctx: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        Ok(to_value(&self.0).ok().unwrap_or_default())
    }
}

/// A scalar that can represent any JSON value.
#[Scalar(internal, name = "JSON")]
impl ScalarType for serde_json::Value {
    fn parse(value: Value) -> InputValueResult<Self> {
        value
            .into_json()
            .map_err(|_| InputValueError::custom("Invalid JSON"))
    }

    fn to_value(&self) -> Value {
        Value::from_json(self.clone()).unwrap_or_default()
    }
}

#[cfg(test)]
mod test {
    use crate::*;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_json_type() {
        #[derive(Serialize, Deserialize)]
        struct MyStruct {
            a: i32,
            b: i32,
            c: HashMap<String, i32>,
        }

        struct Query;

        #[Object(internal)]
        impl Query {
            async fn obj(&self, input: Json<MyStruct>) -> Json<MyStruct> {
                input
            }
        }

        let query = r#"{ obj(input: { a: 1, b: 2, c: { a: 11, b: 22 } } ) }"#;
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        assert_eq!(
            schema.execute(query).await.into_result().unwrap().data,
            value!({
             "obj": {
                 "a": 1,
                 "b": 2,
                 "c": { "a": 11, "b": 22 }
             }
            })
        );
    }

    #[tokio::test]
    async fn test_output_json_type() {
        #[derive(Serialize)]
        struct MyStruct {
            a: i32,
            b: i32,
            c: HashMap<String, i32>,
        }

        struct Query;

        #[Object(internal)]
        impl Query {
            async fn obj(&self) -> OutputJson<MyStruct> {
                MyStruct {
                    a: 1,
                    b: 2,
                    c: {
                        let mut values = HashMap::new();
                        values.insert("a".to_string(), 11);
                        values.insert("b".to_string(), 22);
                        values
                    },
                }
                .into()
            }
        }

        let query = r#"{ obj }"#;
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        assert_eq!(
            schema.execute(query).await.into_result().unwrap().data,
            value!({
             "obj": {
                 "a": 1,
                 "b": 2,
                 "c": { "a": 11, "b": 22 }
             }
            })
        );
    }
}
