use crate::parser::types::{ExecutableDefinition, ExecutableDocument, OperationDefinition};
use crate::validation::visitor::{Visitor, VisitorContext};
use crate::Positioned;

#[derive(Default)]
pub struct LoneAnonymousOperation {
    operation_count: Option<usize>,
}

impl<'a> Visitor<'a> for LoneAnonymousOperation {
    fn enter_document(&mut self, _ctx: &mut VisitorContext<'a>, doc: &'a ExecutableDocument) {
        self.operation_count = Some(
            doc.definitions
                .iter()
                .filter(|d| matches!(&d, ExecutableDefinition::Operation(_)))
                .count(),
        );
    }

    fn enter_operation_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        operation_definition: &'a Positioned<OperationDefinition>,
    ) {
        if let Some(operation_count) = self.operation_count {
            if operation_definition.node.name.is_none() && operation_count > 1 {
                ctx.report_error(
                    vec![operation_definition.pos],
                    "This anonymous operation must be the only defined operation",
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn factory() -> LoneAnonymousOperation {
        LoneAnonymousOperation::default()
    }

    #[test]
    fn no_operations() {
        expect_passes_rule!(
            factory,
            r#"
          fragment fragA on Type {
            field
          }
        "#,
        );
    }

    #[test]
    fn one_anon_operation() {
        expect_passes_rule!(
            factory,
            r#"
          {
            field
          }
        "#,
        );
    }

    #[test]
    fn multiple_named_operations() {
        expect_passes_rule!(
            factory,
            r#"
          query Foo {
            field
          }
          query Bar {
            field
          }
        "#,
        );
    }

    #[test]
    fn anon_operation_with_fragment() {
        expect_passes_rule!(
            factory,
            r#"
          {
            ...Foo
          }
          fragment Foo on Type {
            field
          }
        "#,
        );
    }

    #[test]
    fn multiple_anon_operations() {
        expect_fails_rule!(
            factory,
            r#"
          {
            fieldA
          }
          {
            fieldB
          }
        "#,
        );
    }

    #[test]
    fn anon_operation_with_a_mutation() {
        expect_fails_rule!(
            factory,
            r#"
          {
            fieldA
          }
          mutation Foo {
            fieldB
          }
        "#,
        );
    }
}
