use crate::scalars::any::gql_value_to_json_value;
use crate::{Result, ScalarType, Value};
use async_graphql_derive::Scalar;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::ops::{Deref, DerefMut};

/// A scalar that can represent any JSON value.
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

#[Scalar(internal)]
impl<T: DeserializeOwned + Serialize + Send + Sync> ScalarType for Json<T> {
    fn type_name() -> &'static str {
        "JSON"
    }

    fn parse(value: &Value) -> Option<Self> {
        serde_json::from_value(gql_value_to_json_value(value.clone()))
            .map(|inner| Json(inner))
            .ok()
    }

    fn to_json(&self) -> Result<serde_json::Value> {
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
            schema.execute(&query).await.unwrap().data,
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
