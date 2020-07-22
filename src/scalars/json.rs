use crate::registry::{MetaType, Registry};
use crate::{
    ContextSelectionSet, InputValueResult, OutputValueType, Positioned, Result, ScalarType, Type,
    Value,
};
use async_graphql_derive::Scalar;
use async_graphql_parser::query::Field;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::borrow::Cow;
use std::ops::{Deref, DerefMut};

/// A scalar that can represent any JSON value.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Default)]
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

impl From<serde_json::Value> for Json<serde_json::Value> {
    fn from(value: serde_json::Value) -> Self {
        Self(value)
    }
}

/// A scalar that can represent any JSON value.
#[Scalar(internal, name = "JSON")]
impl<T: DeserializeOwned + Serialize + Send + Sync> ScalarType for Json<T> {
    fn parse(value: Value) -> InputValueResult<Self> {
        Ok(serde_json::from_value(value.into()).map(Json)?)
    }

    fn to_value(&self) -> Value {
        serde_json::to_value(&self.0)
            .unwrap_or_else(|_| serde_json::Value::Null)
            .into()
    }
}

/// A `Json` type that only implements `OutputValueType`.
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

impl From<serde_json::Value> for OutputJson<serde_json::Value> {
    fn from(value: serde_json::Value) -> Self {
        Self(value)
    }
}

impl<T> Type for OutputJson<T> {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("Json")
    }

    fn create_type_info(registry: &mut Registry) -> String {
        registry.create_type::<OutputJson<T>, _>(|_| MetaType::Scalar {
            name: Self::type_name().to_string(),
            description: None,
            is_valid: |_| true,
        })
    }
}

#[async_trait::async_trait]
impl<T: Serialize + Send + Sync> OutputValueType for OutputJson<T> {
    async fn resolve(
        &self,
        _ctx: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(&self.0).unwrap_or_else(|_| serde_json::Value::Null))
    }
}

#[cfg(test)]
mod test {
    use crate::*;
    use std::collections::HashMap;

    #[async_std::test]
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
            schema.execute(&query).await.unwrap_single().unwrap().data,
            serde_json::json!({
             "obj": {
                 "a": 1,
                 "b": 2,
                 "c": { "a": 11, "b": 22 }
             }
            })
        );
    }
}
