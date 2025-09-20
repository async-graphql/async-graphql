use std::borrow::Cow;

use async_graphql_parser::types::Field;

use crate::{
    ContextSelectionSet, OutputType, OutputTypeMarker, Positioned, ServerResult, Value, registry,
};

impl<T> OutputTypeMarker for Cow<'_, T>
where
    T: OutputTypeMarker + ToOwned + ?Sized,
    <T as ToOwned>::Owned: Send + Sync,
{
    fn type_name() -> Cow<'static, str> {
        <T as OutputTypeMarker>::type_name()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        <T as OutputTypeMarker>::create_type_info(registry)
    }
}

#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl<T> OutputType for Cow<'_, T>
where
    T: OutputType + ToOwned + ?Sized + OutputTypeMarker,
    <T as ToOwned>::Owned: Send + Sync,
{

    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        self.as_ref().resolve(ctx, field).await
    }
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use crate::*;

    #[tokio::test]
    async fn test_cow_type() {
        struct Query {
            obj: MyObj,
        }

        #[derive(SimpleObject, Clone)]
        #[graphql(internal)]
        struct MyObj {
            a: i32,
            b: i32,
        }

        #[Object(internal)]
        impl Query {
            async fn value1(&self) -> Cow<'_, str> {
                Cow::Borrowed("abc")
            }

            async fn value2(&self) -> Cow<'_, str> {
                Cow::Owned("def".to_string())
            }

            async fn obj1(&self) -> Cow<'_, MyObj> {
                Cow::Borrowed(&self.obj)
            }

            async fn obj2(&self) -> Cow<'_, MyObj> {
                Cow::Owned(MyObj { a: 300, b: 400 })
            }
        }

        let query = r#"{
            value1
            value2
            obj1 {
                a b
            }
            obj2 {
                a b
            }
        }"#;
        let schema = Schema::new(
            Query {
                obj: MyObj { a: 100, b: 200 },
            },
            EmptyMutation,
            EmptySubscription,
        );

        assert_eq!(
            schema.execute(query).await.into_result().unwrap().data,
            value!({
             "value1": "abc",
             "value2": "def",
             "obj1": {"a": 100, "b": 200},
             "obj2": {"a": 300, "b": 400},
            })
        );
    }
}
