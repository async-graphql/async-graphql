use async_graphql_parser::types::{ExecutableDocument, OperationDefinition, VariableDefinition};
use async_graphql_value::Name;

use crate::{
    parser::types::Field,
    registry::{MetaType, MetaTypeName},
    validation::visitor::{VisitMode, Visitor, VisitorContext},
    Positioned,
};

pub struct ComplexityCalculate<'ctx, 'a> {
    pub complexity: &'a mut usize,
    pub complexity_stack: Vec<usize>,
    pub variable_definition: Option<&'ctx [Positioned<VariableDefinition>]>,
}

impl<'a> ComplexityCalculate<'_, 'a> {
    pub fn new(complexity: &'a mut usize) -> Self {
        Self {
            complexity,
            complexity_stack: Default::default(),
            variable_definition: None,
        }
    }
}

impl<'ctx> Visitor<'ctx> for ComplexityCalculate<'ctx, '_> {
    fn mode(&self) -> VisitMode {
        VisitMode::Inline
    }

    fn enter_document(&mut self, _ctx: &mut VisitorContext<'ctx>, _doc: &'ctx ExecutableDocument) {
        self.complexity_stack.push(0);
    }

    fn exit_document(&mut self, _ctx: &mut VisitorContext<'ctx>, _doc: &'ctx ExecutableDocument) {
        *self.complexity = self.complexity_stack.pop().unwrap();
    }

    fn enter_operation_definition(
        &mut self,
        _ctx: &mut VisitorContext<'ctx>,
        _name: Option<&'ctx Name>,
        operation_definition: &'ctx Positioned<OperationDefinition>,
    ) {
        self.variable_definition = Some(&operation_definition.node.variable_definitions);
    }

    fn enter_field(&mut self, _ctx: &mut VisitorContext<'_>, _field: &Positioned<Field>) {
        self.complexity_stack.push(0);
    }

    fn exit_field(&mut self, ctx: &mut VisitorContext<'ctx>, field: &'ctx Positioned<Field>) {
        let children_complex = self.complexity_stack.pop().unwrap();

        if let Some(MetaType::Object { fields, .. }) = ctx.parent_type() {
            if let Some(meta_field) = fields.get(MetaTypeName::concrete_typename(
                field.node.name.node.as_str(),
            )) {
                if let Some(f) = &meta_field.compute_complexity {
                    match f(
                        ctx,
                        self.variable_definition.unwrap_or(&[]),
                        &field.node,
                        children_complex,
                    ) {
                        Ok(n) => {
                            *self.complexity_stack.last_mut().unwrap() += n;
                        }
                        Err(err) => ctx.report_error(vec![field.pos], err.to_string()),
                    }
                    return;
                }
            }
        }

        *self.complexity_stack.last_mut().unwrap() += 1 + children_complex;
    }
}

#[cfg(test)]
#[allow(clippy::diverging_sub_expression)]
mod tests {
    use async_graphql_derive::SimpleObject;
    use futures_util::stream::BoxStream;

    use super::*;
    use crate::{
        parser::parse_query, validation::visit, EmptyMutation, Object, Schema, Subscription,
    };

    struct Query;

    #[derive(SimpleObject)]
    #[graphql(internal)]
    struct MySimpleObj {
        #[graphql(complexity = 0)]
        a: i32,
        #[graphql(complexity = 0)]
        b: String,
        #[graphql(complexity = 5)]
        c: i32,
    }

    #[derive(Copy, Clone)]
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

        async fn simple_obj(&self) -> MySimpleObj {
            todo!()
        }

        #[graphql(complexity = "count * child_complexity + 2")]
        #[allow(unused_variables)]
        async fn simple_objs(
            &self,
            #[graphql(default_with = "5")] count: usize,
        ) -> Vec<MySimpleObj> {
            todo!()
        }

        async fn obj(&self) -> MyObj {
            todo!()
        }

        #[graphql(complexity = "5 * child_complexity")]
        async fn obj2(&self) -> MyObj {
            todo!()
        }

        #[graphql(complexity = "count * child_complexity")]
        #[allow(unused_variables)]
        async fn objs(&self, #[graphql(default_with = "5")] count: usize) -> Vec<MyObj> {
            todo!()
        }

        #[graphql(complexity = 3)]
        async fn d(&self) -> MyObj {
            todo!()
        }
    }

    struct Subscription;

    #[Subscription(internal)]
    impl Subscription {
        async fn value(&self) -> BoxStream<'static, i32> {
            todo!()
        }

        async fn obj(&self) -> BoxStream<'static, MyObj> {
            todo!()
        }

        #[graphql(complexity = "count * child_complexity")]
        #[allow(unused_variables)]
        async fn objs(
            &self,
            #[graphql(default_with = "5")] count: usize,
        ) -> BoxStream<'static, Vec<MyObj>> {
            todo!()
        }

        #[graphql(complexity = 3)]
        async fn d(&self) -> BoxStream<'static, MyObj> {
            todo!()
        }
    }

    #[track_caller]
    fn check_complexity(query: &str, expect_complexity: usize) {
        let registry =
            Schema::<Query, EmptyMutation, Subscription>::create_registry(Default::default());
        let doc = parse_query(query).unwrap();
        let mut ctx = VisitorContext::new(&registry, &doc, None);
        let mut complexity = 0;
        let mut complexity_calculate = ComplexityCalculate::new(&mut complexity);
        visit(&mut complexity_calculate, &mut ctx, &doc);
        assert_eq!(complexity, expect_complexity);
    }

    #[test]
    fn simple_object() {
        check_complexity(
            r#"{
                simpleObj { a b }
            }"#,
            1,
        );

        check_complexity(
            r#"{
                simpleObj { a b c }
            }"#,
            6,
        );

        check_complexity(
            r#"{
                simpleObjs(count: 7) { a b c }
            }"#,
            7 * 5 + 2,
        );
    }

    #[test]
    fn complex_object() {
        check_complexity(
            r#"
        {
            value #1
        }"#,
            1,
        );

        check_complexity(
            r#"
        {
            value #1
            d #3
        }"#,
            4,
        );

        check_complexity(
            r#"
        {
            value obj { #2
                a b #2
            }
        }"#,
            4,
        );

        check_complexity(
            r#"
        {
            value obj { #2
                a b obj { #3
                    a b obj { #3
                        a #1
                    }
                }
            }
        }"#,
            9,
        );

        check_complexity(
            r#"
        fragment A on MyObj {
            a b ... A2 #2
        }

        fragment A2 on MyObj {
            obj { # 1
                a # 1
            }
        }

        query {
            obj { # 1
                ... A
            }
        }"#,
            5,
        );

        check_complexity(
            r#"
        {
            obj { # 1
                ... on MyObj {
                    a b #2
                    ... on MyObj {
                        obj { #1
                            a #1
                        }
                    }
                }
            }
        }"#,
            5,
        );

        check_complexity(
            r#"
        {
            objs(count: 10) {
                a b
            }
        }"#,
            20,
        );

        check_complexity(
            r#"
        {
            objs {
                a b
            }
        }"#,
            10,
        );

        check_complexity(
            r#"
        fragment A on MyObj {
            a b
        }

        query {
            objs(count: 10) {
                ... A
            }
        }"#,
            20,
        );
    }

    #[test]
    fn complex_subscription() {
        check_complexity(
            r#"
        subscription {
            value #1
        }"#,
            1,
        );

        check_complexity(
            r#"
        subscription {
            value #1
            d #3
        }"#,
            4,
        );

        check_complexity(
            r#"
        subscription {
            value obj { #2
                a b #2
            }
        }"#,
            4,
        );

        check_complexity(
            r#"
        subscription {
            value obj { #2
                a b obj { #3
                    a b obj { #3
                        a #1
                    }
                }
            }
        }"#,
            9,
        );

        check_complexity(
            r#"
        fragment A on MyObj {
            a b ... A2 #2
        }

        fragment A2 on MyObj {
            obj { # 1
                a # 1
            }
        }

        subscription query {
            obj { # 1
                ... A
            }
        }"#,
            5,
        );

        check_complexity(
            r#"
        subscription {
            obj { # 1
                ... on MyObj {
                    a b #2
                    ... on MyObj {
                        obj { #1
                            a #1
                        }
                    }
                }
            }
        }"#,
            5,
        );

        check_complexity(
            r#"
        subscription {
            objs(count: 10) {
                a b
            }
        }"#,
            20,
        );

        check_complexity(
            r#"
        subscription {
            objs {
                a b
            }
        }"#,
            10,
        );

        check_complexity(
            r#"
        fragment A on MyObj {
            a b
        }

        subscription query {
            objs(count: 10) {
                ... A
            }
        }"#,
            20,
        );

        check_complexity(
            r#"
            query {
                obj2 { a b }
            }"#,
            10,
        );
    }
}
