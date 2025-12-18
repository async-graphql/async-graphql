use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    ContextSelectionSet, InputType, InputValueResult, OutputType, OutputTypeMarker, Positioned,
    ServerResult, Value, from_value,
    parser::types::Field,
    registry::{MetaType, MetaTypeId, Registry},
    to_value,
};

/// A scalar that can represent any JSON value.
///
/// If the inner type cannot be serialized as JSON (e.g. it has non-string keys)
/// it will be `null`.
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

impl<T> From<T> for Json<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: DeserializeOwned + Serialize + Send + Sync> InputType for Json<T> {
    type RawValueType = T;

    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("JSON")
    }

    fn create_type_info(registry: &mut Registry) -> String {
        registry.create_input_type::<Json<T>, _>(MetaTypeId::Scalar, |_| MetaType::Scalar {
            name: <Self as InputType>::type_name().to_string(),
            description: Some("A scalar that can represent any JSON value.".to_string()),
            is_valid: None,
            visible: None,
            inaccessible: false,
            tags: Default::default(),
            specified_by_url: None,
            directive_invocations: Default::default(),
            requires_scopes: Default::default(),
        })
    }

    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        Ok(from_value(value.unwrap_or_default())?)
    }

    fn to_value(&self) -> Value {
        Value::String(serde_json::to_string(&self.0).unwrap_or_default())
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(&self.0)
    }
}

#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl<T: Serialize + Send + Sync> OutputTypeMarker for Json<T> {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("JSON")
    }

    fn create_type_info(registry: &mut Registry) -> String {
        registry.create_output_type::<Json<T>, _>(MetaTypeId::Scalar, |_| MetaType::Scalar {
            name: <Self as OutputTypeMarker>::type_name().to_string(),
            description: Some("A scalar that can represent any JSON value.".to_string()),
            is_valid: None,
            visible: None,
            inaccessible: false,
            tags: Default::default(),
            specified_by_url: None,
            directive_invocations: Default::default(),
            requires_scopes: Default::default(),
        })
    }
}

#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl<T: Serialize + Send + Sync> OutputType for Json<T> {
    async fn resolve(
        &self,
        _ctx: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        Ok(to_value(&self.0).ok().unwrap_or_default())
    }
}

impl InputType for serde_json::Value {
    type RawValueType = serde_json::Value;

    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("JSON")
    }

    fn create_type_info(registry: &mut Registry) -> String {
        registry.create_input_type::<serde_json::Value, _>(MetaTypeId::Scalar, |_| {
            MetaType::Scalar {
                name: <Self as InputType>::type_name().to_string(),
                description: Some("A scalar that can represent any JSON value.".to_string()),
                is_valid: None,
                visible: None,
                inaccessible: false,
                tags: Default::default(),
                specified_by_url: None,
                directive_invocations: Default::default(),
                requires_scopes: Default::default(),
            }
        })
    }

    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        Ok(from_value(value.unwrap_or_default())?)
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(&self)
    }
}

impl OutputTypeMarker for serde_json::Value {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("JSON")
    }

    fn create_type_info(registry: &mut Registry) -> String {
        registry.create_output_type::<serde_json::Value, _>(MetaTypeId::Scalar, |_| {
            MetaType::Scalar {
                name: <Self as OutputTypeMarker>::type_name().to_string(),
                description: Some("A scalar that can represent any JSON value.".to_string()),
                is_valid: None,
                visible: None,
                inaccessible: false,
                tags: Default::default(),
                specified_by_url: None,
                directive_invocations: Default::default(),
                requires_scopes: Default::default(),
            }
        })
    }
}
#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl OutputType for serde_json::Value {

    async fn resolve(
        &self,
        _ctx: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        Ok(to_value(self).ok().unwrap_or_default())
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};

    use crate::*;

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
    async fn test_json_type_for_serialize_only() {
        #[derive(Serialize)]
        struct MyStruct {
            a: i32,
            b: i32,
            c: HashMap<String, i32>,
        }

        struct Query;

        #[Object(internal)]
        impl Query {
            async fn obj(&self) -> Json<MyStruct> {
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
