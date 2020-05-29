use crate::parser::query::{FragmentDefinition, InlineFragment, TypeCondition};
use crate::validation::visitor::{Visitor, VisitorContext};
use crate::Positioned;

#[derive(Default)]
pub struct FragmentsOnCompositeTypes;

impl<'a> Visitor<'a> for FragmentsOnCompositeTypes {
    fn enter_fragment_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        fragment_definition: &'a Positioned<FragmentDefinition>,
    ) {
        if let Some(current_type) = ctx.current_type() {
            if !current_type.is_composite() {
                let TypeCondition::On(name) = &fragment_definition.type_condition.node;
                ctx.report_error(
                    vec![fragment_definition.position()],
                    format!(
                        "Fragment \"{}\" cannot condition non composite type \"{}\"",
                        fragment_definition.name, name
                    ),
                );
            }
        }
    }

    fn enter_inline_fragment(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        inline_fragment: &'a Positioned<InlineFragment>,
    ) {
        if let Some(current_type) = ctx.current_type() {
            if !current_type.is_composite() {
                ctx.report_error(
                    vec![inline_fragment.position()],
                    format!(
                        "Fragment cannot condition non composite type \"{}\"",
                        current_type.name()
                    ),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expect_fails_rule, expect_passes_rule};

    pub fn factory<'a>() -> FragmentsOnCompositeTypes {
        FragmentsOnCompositeTypes
    }

    #[test]
    fn on_object() {
        expect_passes_rule!(
            factory,
            r#"
          fragment validFragment on Dog {
            barks
          }
        "#,
        );
    }

    #[test]
    fn on_interface() {
        expect_passes_rule!(
            factory,
            r#"
          fragment validFragment on Pet {
            name
          }
        "#,
        );
    }

    #[test]
    fn on_object_inline() {
        expect_passes_rule!(
            factory,
            r#"
          fragment validFragment on Pet {
            ... on Dog {
              barks
            }
          }
        "#,
        );
    }

    #[test]
    fn on_inline_without_type_cond() {
        expect_passes_rule!(
            factory,
            r#"
          fragment validFragment on Pet {
            ... {
              name
            }
          }
        "#,
        );
    }

    #[test]
    fn on_union() {
        expect_passes_rule!(
            factory,
            r#"
          fragment validFragment on CatOrDog {
            __typename
          }
        "#,
        );
    }

    #[test]
    fn not_on_scalar() {
        expect_fails_rule!(
            factory,
            r#"
          fragment scalarFragment on Boolean {
            bad
          }
        "#,
        );
    }

    #[test]
    fn not_on_enum() {
        expect_fails_rule!(
            factory,
            r#"
          fragment scalarFragment on FurColor {
            bad
          }
        "#,
        );
    }

    #[test]
    fn not_on_input_object() {
        expect_fails_rule!(
            factory,
            r#"
          fragment inputFragment on ComplexInput {
            stringField
          }
        "#,
        );
    }

    #[test]
    fn not_on_scalar_inline() {
        expect_fails_rule!(
            factory,
            r#"
          fragment invalidFragment on Pet {
            ... on String {
              barks
            }
          }
        "#,
        );
    }
}
