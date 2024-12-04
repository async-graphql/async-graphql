use async_graphql_parser::types::Field;

use crate::{
    validation::visitor::{VisitMode, Visitor, VisitorContext},
    Positioned,
};

pub struct DepthCalculate<'a> {
    max_depth: &'a mut usize,
    current_depth: usize,
}

impl<'a> DepthCalculate<'a> {
    pub fn new(max_depth: &'a mut usize) -> Self {
        Self {
            max_depth,
            current_depth: 0,
        }
    }
}

impl<'ctx> Visitor<'ctx> for DepthCalculate<'_> {
    fn mode(&self) -> VisitMode {
        VisitMode::Inline
    }

    fn enter_field(&mut self, _ctx: &mut VisitorContext<'ctx>, _field: &'ctx Positioned<Field>) {
        self.current_depth += 1;
        *self.max_depth = (*self.max_depth).max(self.current_depth);
    }

    fn exit_field(&mut self, _ctx: &mut VisitorContext<'ctx>, _field: &'ctx Positioned<Field>) {
        self.current_depth -= 1;
    }
}

#[cfg(test)]
#[allow(clippy::diverging_sub_expression)]
mod tests {
    use super::*;
    use crate::{
        parser::parse_query, validation::visit, EmptyMutation, EmptySubscription, Object, Schema,
    };

    struct Query;

    struct MyObj;

    #[Object(internal)]
    #[allow(unreachable_code)]
    impl MyObj {
        async fn a(&self) -> i32 {
            todo!()
        }

        async fn b(&self) -> i32 {
            todo!()
        }

        async fn c(&self) -> MyObj {
            todo!()
        }
    }

    #[Object(internal)]
    #[allow(unreachable_code)]
    impl Query {
        async fn value(&self) -> i32 {
            todo!()
        }

        async fn obj(&self) -> MyObj {
            todo!()
        }
    }

    fn check_depth(query: &str, expect_depth: usize) {
        let registry =
            Schema::<Query, EmptyMutation, EmptySubscription>::create_registry(Default::default());
        let doc = parse_query(query).unwrap();
        let mut ctx = VisitorContext::new(&registry, &doc, None);
        let mut depth = 0;
        let mut depth_calculate = DepthCalculate::new(&mut depth);
        visit(&mut depth_calculate, &mut ctx, &doc);
        assert_eq!(depth, expect_depth);
    }

    #[test]
    fn depth() {
        check_depth(
            r#"{
            value #1
        }"#,
            1,
        );

        check_depth(
            r#"
        {
            obj { #1
                a b #2
            }
        }"#,
            2,
        );

        check_depth(
            r#"
        {
            obj { # 1
                a b c { # 2
                    a b c { # 3
                        a b # 4
                    }
                }
            }
        }"#,
            4,
        );

        check_depth(
            r#"
        fragment A on MyObj {
            a b ... A2 #2
        }

        fragment A2 on MyObj {
            obj {
                a #3
            }
        }

        query {
            obj { # 1
                ... A
            }
        }"#,
            3,
        );

        check_depth(
            r#"
        {
            obj { # 1
                ... on MyObj {
                    a b #2
                    ... on MyObj {
                        obj {
                            a #3
                        }
                    }
                }
            }
        }"#,
            3,
        );
    }
}
