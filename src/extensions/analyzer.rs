use std::sync::Arc;

use futures_util::lock::Mutex;

use crate::{
    extensions::{Extension, ExtensionContext, ExtensionFactory, NextRequest, NextValidation},
    value, Response, ServerError, ValidationResult,
};

/// Analyzer extension
///
/// This extension will output the `analyzer` field containing `complexity` and
/// `depth` in the response extension of each query.
pub struct Analyzer;

impl ExtensionFactory for Analyzer {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(AnalyzerExtension::default())
    }
}

#[derive(Default)]
struct AnalyzerExtension {
    validation_result: Mutex<Option<ValidationResult>>,
}

#[async_trait::async_trait]
impl Extension for AnalyzerExtension {
    async fn request(&self, ctx: &ExtensionContext<'_>, next: NextRequest<'_>) -> Response {
        let mut resp = next.run(ctx).await;
        let validation_result = self.validation_result.lock().await.take();
        if let Some(validation_result) = validation_result {
            resp = resp.extension(
                "analyzer",
                value! ({
                    "complexity": validation_result.complexity,
                    "depth": validation_result.depth,
                }),
            );
        }
        resp
    }

    async fn validation(
        &self,
        ctx: &ExtensionContext<'_>,
        next: NextValidation<'_>,
    ) -> Result<ValidationResult, Vec<ServerError>> {
        let res = next.run(ctx).await?;
        *self.validation_result.lock().await = Some(res);
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    struct Query;

    #[derive(Copy, Clone)]
    struct MyObj;

    #[Object(internal)]
    impl MyObj {
        async fn value(&self) -> i32 {
            1
        }

        async fn obj(&self) -> MyObj {
            MyObj
        }
    }

    #[Object(internal)]
    impl Query {
        async fn value(&self) -> i32 {
            1
        }

        async fn obj(&self) -> MyObj {
            MyObj
        }

        #[graphql(complexity = "count * child_complexity")]
        async fn objs(&self, count: usize) -> Vec<MyObj> {
            vec![MyObj; count as usize]
        }
    }

    #[tokio::test]
    async fn analyzer() {
        let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
            .extension(extensions::Analyzer)
            .finish();

        let res = schema
            .execute(
                r#"{
            value obj {
                value obj {
                    value
                }
            }
            objs(count: 10) { value }
        }"#,
            )
            .await
            .into_result()
            .unwrap()
            .extensions
            .remove("analyzer");
        assert_eq!(
            res,
            Some(value!({
                "complexity": 5 + 10,
                "depth": 3,
            }))
        );
    }
}
