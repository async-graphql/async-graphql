use crate::extensions::{Extension, ExtensionContext, ExtensionFactory};
use crate::{value, ValidationResult, Value};

/// Analyzer extension
///
/// This extension will output the `analyzer` field containing `complexity` and `depth` in the response extension of each query.
pub struct Analyzer;

impl ExtensionFactory for Analyzer {
    fn create(&self) -> Box<dyn Extension> {
        Box::new(AnalyzerExtension::default())
    }
}

#[derive(Default)]
struct AnalyzerExtension {
    complexity: usize,
    depth: usize,
}

impl Extension for AnalyzerExtension {
    fn name(&self) -> Option<&'static str> {
        Some("analyzer")
    }

    fn validation_end(&mut self, _ctx: &ExtensionContext<'_>, result: &ValidationResult) {
        self.complexity = result.complexity;
        self.depth = result.depth;
    }

    fn result(&mut self, _ctx: &ExtensionContext<'_>) -> Option<Value> {
        Some(value! ({
            "complexity": self.complexity,
            "depth": self.depth,
        }))
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

        let extensions = schema
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
            .unwrap();
        assert_eq!(
            extensions,
            value!({
                "analyzer": {
                    "complexity": 5 + 10,
                    "depth": 3,
                }
            })
        );
    }
}
