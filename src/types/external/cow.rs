use std::borrow::Cow;

use async_graphql_parser::types::Field;

use crate::{registry, ContextSelectionSet, OutputType, Positioned, ServerResult, Value};

#[async_trait::async_trait]
impl<'a, T> OutputType for Cow<'a, T>
where
    T: OutputType + ToOwned + ?Sized,
    <T as ToOwned>::Owned: Send + Sync,
{
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        <T as OutputType>::create_type_info(registry)
    }

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
    use crate::*;
    use std::borrow::Cow;

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
