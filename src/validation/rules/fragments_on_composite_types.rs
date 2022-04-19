use crate::{
    parser::types::{FragmentDefinition, InlineFragment},
    validation::visitor::{Visitor, VisitorContext},
    Name, Positioned,
};

#[derive(Default)]
pub struct FragmentsOnCompositeTypes;

impl<'a> Visitor<'a> for FragmentsOnCompositeTypes {
    fn enter_fragment_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        name: &'a Name,
        fragment_definition: &'a Positioned<FragmentDefinition>,
    ) {
        if let Some(current_type) = ctx.current_type() {
            if !current_type.is_composite() {
                ctx.report_error(
                    vec![fragment_definition.pos],
                    format!(
                        "Fragment \"{}\" cannot condition non composite type \"{}\"",
                        name, fragment_definition.node.type_condition.node.on.node,
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
                    vec![inline_fragment.pos],
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

    fn factory() -> FragmentsOnCompositeTypes {
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
          { __typename }
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
          { __typename }
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
          { __typename }
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
          { __typename }
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
          { __typename }
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
          { __typename }
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
          { __typename }
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
          { __typename }
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
          { __typename }
        "#,
        );
    }
}
