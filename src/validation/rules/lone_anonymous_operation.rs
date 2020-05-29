use crate::parser::query::{Definition, Document, OperationDefinition};
use crate::validation::visitor::{Visitor, VisitorContext};
use crate::Positioned;

#[derive(Default)]
pub struct LoneAnonymousOperation {
    operation_count: Option<usize>,
}

impl<'a> Visitor<'a> for LoneAnonymousOperation {
    fn enter_document(&mut self, _ctx: &mut VisitorContext<'a>, doc: &'a Document) {
        self.operation_count = Some(
            doc.definitions()
                .iter()
                .filter(|d| match &d.node {
                    Definition::Operation(_) => true,
                    Definition::Fragment(_) => false,
                })
                .count(),
        );
    }

    fn enter_operation_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        operation_definition: &'a Positioned<OperationDefinition>,
    ) {
        if let Some(operation_count) = self.operation_count {
            let (err, pos) = match &operation_definition.node {
                OperationDefinition::SelectionSet(s) => (operation_count > 1, s.position()),
                OperationDefinition::Query(query) if query.name.is_none() => {
                    (operation_count > 1, query.position())
                }
                OperationDefinition::Mutation(mutation) if mutation.name.is_none() => {
                    (operation_count > 1, mutation.position())
                }
                OperationDefinition::Subscription(subscription) if subscription.name.is_none() => {
                    (operation_count > 1, subscription.position())
                }
                _ => {
                    return;
                }
            };

            if err {
                ctx.report_error(
                    vec![pos],
                    "This anonymous operation must be the only defined operation",
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expect_fails_rule, expect_passes_rule};

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
