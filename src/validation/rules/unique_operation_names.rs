use crate::validation::visitor::{Visitor, VisitorContext};
use graphql_parser::query::{Mutation, OperationDefinition, Query, Subscription};
use std::collections::HashSet;

#[derive(Default)]
pub struct UniqueOperationNames<'a> {
    names: HashSet<&'a str>,
}

impl<'a> Visitor<'a> for UniqueOperationNames<'a> {
    fn enter_operation_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        operation_definition: &'a OperationDefinition,
    ) {
        let name = match operation_definition {
            OperationDefinition::Query(Query { name, position, .. }) => {
                name.as_ref().map(|name| (name, position))
            }
            OperationDefinition::Mutation(Mutation { name, position, .. }) => {
                name.as_ref().map(|name| (name, position))
            }
            OperationDefinition::Subscription(Subscription { name, position, .. }) => {
                name.as_ref().map(|name| (name, position))
            }
            OperationDefinition::SelectionSet(_) => None,
        };

        if let Some((name, pos)) = name {
            if !self.names.insert(name.as_str()) {
                ctx.report_error(
                    vec![*pos],
                    format!("There can only be one operation named \"{}\"", name),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::test_harness::{expect_fails_rule, expect_passes_rule};

    pub fn factory<'a>() -> UniqueOperationNames<'a> {
        UniqueOperationNames::default()
    }

    #[test]
    fn no_operations() {
        expect_passes_rule(
            factory,
            r#"
          fragment fragA on Dog {
            name
          }
        "#,
        );
    }

    #[test]
    fn one_anon_operation() {
        expect_passes_rule(
            factory,
            r#"
          {
            field
          }
        "#,
        );
    }

    #[test]
    fn one_named_operation() {
        expect_passes_rule(
            factory,
            r#"
          query Foo {
            field
          }
        "#,
        );
    }

    #[test]
    fn multiple_operations() {
        expect_passes_rule(
            factory,
            r#"
          query Foo {
            dog {
              name
            }
          }
          query Bar {
            dog {
              name
            }
          }
        "#,
        );
    }

    #[test]
    fn multiple_operations_of_different_types() {
        expect_passes_rule(
            factory,
            r#"
          query Foo {
            field
          }
          mutation Bar {
            field
          }
        "#,
        );
    }

    #[test]
    fn fragment_and_operation_named_the_same() {
        expect_passes_rule(
            factory,
            r#"
          query Foo {
            dog {
              ...Foo
            }
          }
          fragment Foo on Dog {
            name
          }
        "#,
        );
    }

    #[test]
    fn multiple_operations_of_same_name() {
        expect_fails_rule(
            factory,
            r#"
          query Foo {
            dog {
              name
            }
          }
          query Foo {
            human {
              name
            }
          }
        "#,
        );
    }

    #[test]
    fn multiple_ops_of_same_name_of_different_types() {
        expect_fails_rule(
            factory,
            r#"
          query Foo {
            dog {
              name
            }
          }
          mutation Foo {
            testInput
          }
        "#,
        );
    }
}
